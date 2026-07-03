use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::animation::playback::Animator;
use crate::core::ecs::MeshType;
use crate::core::math::Quaternion;
use crate::core::skeleton::Skeleton;
use crate::network::{EntityData, ServerMessage};
use crate::render::export::{export_asset, ExportFormat, ExportParams};
use crate::render::mesh::{create_cube, create_plane, create_sphere};
use crate::render::raycast::pick_entity;
use crate::render::{OrbitCamera, SkinRenderer, StaticRenderer, Vertex};
use crate::ui::{EditorState, LogLevel};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Muse — AI Character & Animation Studio")
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let surface = instance.create_surface(window.clone()).expect("Failed to create surface");
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to request adapter");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to request device");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0]);
    let size = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let mut depth_view = create_depth_view(&device, config.width, config.height);

    let mut egui_winit = egui_winit::State::new(
        egui::Context::default(),
        egui::viewport::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    let mut egui_renderer =
        egui_wgpu::Renderer::new(&device, config.format, None, 1);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let ws_tx = Arc::new(Mutex::new(Some(tx)));
    let mut editor = EditorState::new(ws_tx.clone());

    let log_queue = Arc::new(Mutex::new(Vec::<(LogLevel, String)>::new()));
    let log_queue_clone = log_queue.clone();

    let state_queue = Arc::new(Mutex::new(Vec::<ServerMessage>::new()));
    let state_queue_clone = state_queue.clone();

    let mut skin_renderer = SkinRenderer::new(&device, &queue, &config);
    let mut static_renderer = StaticRenderer::new(&device, &queue, &config);
    let mut camera = OrbitCamera::new(config.width as f32 / config.height as f32);
    let mut animator = Animator::new();

    {
        let cube = editor.scene.spawn_primitive(MeshType::Cube);
        if let Some(t) = editor.scene.world.get_mut::<crate::core::ecs::TransformComponent>(cube) {
            t.position = (-1.5, 0.5, 0.0);
        }
        if let Some(m) = editor.scene.world.get_mut::<crate::core::ecs::MaterialComponent>(cube) {
            m.albedo = (0.8, 0.2, 0.2);
        }

        let sphere = editor.scene.spawn_primitive(MeshType::Sphere(16));
        if let Some(t) = editor.scene.world.get_mut::<crate::core::ecs::TransformComponent>(sphere) {
            t.position = (0.0, 0.5, 0.0);
            t.scale = (0.8, 0.8, 0.8);
        }
        if let Some(m) = editor.scene.world.get_mut::<crate::core::ecs::MaterialComponent>(sphere) {
            m.albedo = (0.2, 0.6, 0.8);
        }

        let plane = editor.scene.spawn_primitive(MeshType::Plane);
        if let Some(t) = editor.scene.world.get_mut::<crate::core::ecs::TransformComponent>(plane) {
            t.position = (0.0, -0.5, 0.0);
            t.scale = (3.0, 1.0, 3.0);
        }
        if let Some(m) = editor.scene.world.get_mut::<crate::core::ecs::MaterialComponent>(plane) {
            m.albedo = (0.3, 0.3, 0.3);
        }

        editor.push_log(LogLevel::Ok, "Scene initialized with 3 entities");
    }

    let mut last_frame_time = std::time::Instant::now();
    let mut mouse_pressed = false;
    let mut prev_mouse_pos: Option<(f64, f64)> = None;
    let mut right_mouse_pressed = false;
    let mut viewport_click: Option<(f64, f64)> = None;
    let mut mouse_drag = false;

    std::thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            loop {
                match connect_async("ws://127.0.0.1:8081/ws").await {
                    Ok((ws_stream, _)) => {
                        {
                            let mut guard = log_queue_clone.lock().expect("log queue lock poisoned");
                            guard.push((LogLevel::Info, "[WS] connected".to_string()));
                        }
                        let (mut write, mut read) = ws_stream.split();

                        let log_queue_clone2 = log_queue_clone.clone();
                        let state_queue_clone2 = state_queue_clone.clone();
                        let read_task = async {
                            while let Some(msg) = read.next().await {
                                if let Ok(Message::Text(text)) = msg {
                                    match serde_json::from_str::<ServerMessage>(&text) {
                                        Ok(msg_enum) => {
                                            let mut q = state_queue_clone2.lock().expect("state queue lock poisoned");
                                            match &msg_enum {
                                                ServerMessage::Error { detail } => {
                                                    let mut guard = log_queue_clone2.lock().expect("log queue lock poisoned");
                                                    guard.push((
                                                        LogLevel::Err,
                                                        format!("Server: {}", detail),
                                                    ));
                                                }
                                                _ => {}
                                            }
                                            q.push(msg_enum);
                                        }
                                        Err(_) => {}
                                    }
                                }
                            }
                        };

                        let log_queue_clone3 = log_queue_clone.clone();
                        let write_task = async {
                            while let Some(to_send) = rx.recv().await {
                                if let Err(e) = write.send(Message::Text(to_send)).await {
                                    let mut guard = log_queue_clone3.lock().expect("log queue lock poisoned");
                                    guard.push((
                                        LogLevel::Err,
                                        format!("[WS] send error: {}", e),
                                    ));
                                    break;
                                }
                            }
                        };

                        tokio::select! {
                            _ = read_task => {},
                            _ = write_task => {},
                        }
                    }
                    Err(e) => {
                        {
                            let mut guard = log_queue_clone.lock().expect("log queue lock poisoned");
                            guard.push((LogLevel::Warn, format!("[WS] connection failed: {}", e)));
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
    });

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                let response = egui_winit.on_window_event(&window, event);
                let consumed = response.consumed;

                if !consumed {
                    match event {
                        WindowEvent::MouseInput { state, button, .. } => {
                            if *button == MouseButton::Left {
                                if *state == ElementState::Pressed {
                                    mouse_pressed = true;
                                    mouse_drag = false;
                                    viewport_click = prev_mouse_pos;
                                } else {
                                    mouse_pressed = false;
                                    if !mouse_drag {
                                        if let Some(click_pos) = viewport_click {
                                            let w = config.width as f64;
                                            let h = config.height as f64;
                                            let mx = click_pos.0;
                                            let my = click_pos.1;
                                            if mx >= 0.0 && my >= 0.0 && mx < w && my < h {
                                                if let Some((hit, _)) = pick_entity(mx as f32, my as f32, w as f32, h as f32, &camera, &editor.scene) {
                                                    editor.select_entity(hit);
                                                } else {
                                                    editor.clear_selection();
                                                }
                                            }
                                        }
                                    }
                                    prev_mouse_pos = None;
                                    viewport_click = None;
                                }
                            }
                            if *button == MouseButton::Right {
                                right_mouse_pressed = *state == ElementState::Pressed;
                                if !right_mouse_pressed {
                                    prev_mouse_pos = None;
                                }
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let pos = (position.x, position.y);
                            if mouse_pressed || right_mouse_pressed {
                                if let Some(prev) = prev_mouse_pos {
                                    mouse_drag = true;
                                    let dx = ((pos.0 - prev.0) * 0.005) as f32;
                                    let dy = ((pos.1 - prev.1) * 0.005) as f32;
                                    if mouse_pressed {
                                        camera.yaw += dx;
                                        camera.pitch = (camera.pitch + dy).clamp(0.05, std::f32::consts::PI - 0.05);
                                    }
                                    if right_mouse_pressed {
                                        let fwd_norm = (camera.yaw.sin() * camera.pitch.sin()).abs().max(0.01);
                                        let pan_speed = camera.distance * 0.002 / fwd_norm;
                                        let (sy, cy) = camera.yaw.sin_cos();
                                        camera.target[0] += (-cy * dx - sy * dy) * pan_speed;
                                        camera.target[2] += (-sy * dx + cy * dy) * pan_speed;
                                        camera.target[1] += dy * pan_speed * 0.5;
                                    }
                                }
                                prev_mouse_pos = Some(pos);
                            } else {
                                prev_mouse_pos = None;
                            }
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            let scroll = match delta {
                                MouseScrollDelta::LineDelta(_, y) => *y,
                                MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
                            };
                            camera.distance = (camera.distance - scroll * 0.3).clamp(0.5, 30.0);
                        }
                        _ => {}
                    }
                } else {
                    prev_mouse_pos = None;
                    mouse_pressed = false;
                    right_mouse_pressed = false;
                }

                if consumed {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            config.width = physical_size.width;
                            config.height = physical_size.height;
                            surface.configure(&device, &config);
                            depth_view = create_depth_view(&device, config.width, config.height);
                            camera.aspect = config.width as f32 / config.height as f32;
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = now.duration_since(last_frame_time).as_secs_f32().min(0.1);
                        last_frame_time = now;

                        {
                            let mut logs = log_queue.lock().expect("log queue lock poisoned");
                            for (lvl, l) in logs.drain(..) {
                                editor.push_log(lvl, &l);
                            }
                        }

                        {
                            let mut updates = state_queue.lock().expect("state queue lock poisoned");
                            for msg in updates.drain(..) {
                                match msg {
                                    ServerMessage::JobUpdate { job } => {
                                        match job.status.as_str() {
                                            "queued" => editor.gen_status.add_job(job.id.clone(), job.job_type.clone()),
                                            "running" => editor.gen_status.update_progress(&job.id, job.progress as f32),
                                            "completed" => editor.gen_status.complete(&job.id),
                                            "failed" => editor.gen_status.fail(&job.id, job.error.unwrap_or_default()),
                                            _ => {}
                                        }
                                    }
                                    ServerMessage::MeshGenerated { mesh, skeleton, clip } => {
                                        editor.loaded_character = true;
                                        editor.character_mesh = Some(mesh);
                                        editor.character_skeleton = Some(skeleton);
                                        if !clip.is_null() {
                                            editor.loaded_motion = true;
                                            editor.motion_clip = Some(clip);
                                        }
                                        editor.push_log(LogLevel::Ok, "Character loaded");
                                    }
                                    ServerMessage::MotionGenerated { clip } => {
                                        editor.loaded_motion = true;
                                        editor.motion_clip = Some(clip);
                                        editor.push_log(LogLevel::Ok, "Motion loaded");
                                    }
                                    ServerMessage::ChatReply { reply, entities, actions: _ } => {
                                        let ids: Vec<u64> = entities.iter().filter_map(|e: &EntityData| e.entity_id).collect();
                                        editor.chat.receive_response(&reply, &ids);
                                        editor.handle_entities(&entities);
                                    }
                                    ServerMessage::Progress { action, progress, message } => {
                                        editor.push_log(LogLevel::Info, &format!("[{}] {}% - {}", action, (progress*100.0) as u32, message));
                                    }
                                    ServerMessage::Error { .. } => {}
                                }
                            }
                        }

                        if let Some(mesh_val) = editor.character_mesh.as_ref().cloned() {
                            if let Some(skel_val) = editor.character_skeleton.as_ref() {
                                let (verts, idxs) = parse_mesh_data(&mesh_val);
                                if !verts.is_empty() && !idxs.is_empty() {
                                    skin_renderer.upload_mesh(&device, verts, idxs, skel_val);
                                    editor.push_log(LogLevel::Ok, "Mesh uploaded to GPU");
                                }
                            }
                        }

                        if let Some(clip_val) = editor.motion_clip.as_ref().cloned() {
                            let clip = parse_motion_clip(&clip_val);
                            if clip.frame_count() > 0 {
                                animator.play(clip);
                                editor.push_log(LogLevel::Ok, "Animation loaded");
                            }
                        }

                        animator.update(dt);

                        let input = egui_winit.take_egui_input(&window);
                        egui_winit.egui_ctx().begin_frame(input);
                        editor.draw(egui_winit.egui_ctx());

                        if let Some(msg_json) = editor.chat.pending_send.take() {
                            if let Some(tx) = editor.ws_tx.lock().unwrap().as_ref() {
                                let _ = tx.send(serde_json::json!({"type":"chat","messages":serde_json::from_str::<serde_json::Value>(&msg_json).unwrap()}).to_string());
                            }
                            editor.chat.processing = true;
                        }

                        if editor.export_triggered {
                            editor.export_triggered = false;
                            if let (Some(mesh), Some(skel)) = (&editor.character_mesh, &editor.character_skeleton) {
                                let format = match editor.export_format.as_str() {
                                    "fbx" => ExportFormat::Fbx,
                                    _ => ExportFormat::Glb,
                                };
                                let params = ExportParams {
                                    mesh,
                                    skeleton: skel,
                                    clip: editor.motion_clip.as_ref(),
                                    format,
                                    file_path: editor.export_path.clone(),
                                };
                                match export_asset(&params) {
                                    Ok(()) => editor.push_log(LogLevel::Ok, &format!("Exported to {}", editor.export_path)),
                                    Err(e) => editor.push_log(LogLevel::Err, &format!("Export failed: {}", e)),
                                }
                            } else {
                                editor.push_log(LogLevel::Err, "No character data to export");
                            }
                        }
                        let full_output = egui_winit.egui_ctx().end_frame();
                        let paint_jobs = egui_winit
                            .egui_ctx()
                            .tessellate(full_output.shapes, full_output.pixels_per_point);

                        let output = match surface.get_current_texture() {
                            Ok(output) => output,
                            Err(wgpu::SurfaceError::Outdated) => return,
                            Err(e) => {
                                eprintln!("Dropped frame: {:?}", e);
                                return;
                            }
                        };
                        let view = output
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder = device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            },
                        );

                        {
                            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("3D Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.03, g: 0.04, b: 0.06, a: 1.0 }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                    view: &depth_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(0.0),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                }),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });
                            let pose = animator.current_pose();
                            skin_renderer.update_pose(&queue, &pose, &camera);
                            skin_renderer.draw(&mut rpass);

                            static_renderer.clear();
                            for (mesh_type, world_mat, albedo, metallic, roughness) in editor.scene.collect_render_data(editor.selected_entity) {
                                let (verts, idxs) = match mesh_type {
                                    MeshType::Cube => create_cube(),
                                    MeshType::Sphere(seg) => create_sphere(seg),
                                    MeshType::Plane => create_plane(),
                                    MeshType::Quad => create_plane(),
                                    MeshType::Cylinder => create_cube(),
                                    MeshType::Custom => continue,
                                };
                                static_renderer.add_mesh(&device, verts, idxs, world_mat, [albedo.0, albedo.1, albedo.2], metallic, roughness);
                            }
                            static_renderer.update_camera(&queue, &camera);
                            static_renderer.draw(&mut rpass);
                        }

                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [config.width, config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };

                        for (id, delta) in &full_output.textures_delta.set {
                            egui_renderer
                                .update_texture(&device, &queue, *id, delta);
                        }
                        egui_renderer.update_buffers(
                            &device,
                            &queue,
                            &mut encoder,
                            &paint_jobs,
                            &screen_descriptor,
                        );

                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Egui Pass"),
                                    color_attachments: &[Some(
                                        wgpu::RenderPassColorAttachment {
                                            view: &view,
                                            resolve_target: None,
                                            ops: wgpu::Operations {
                                                load: wgpu::LoadOp::Load,
                                                store: wgpu::StoreOp::Store,
                                            },
                                        },
                                    )],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });

                            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                        }

                        for id in &full_output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        queue.submit(std::iter::once(encoder.finish()));
                        output.present();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn parse_mesh_data(val: &serde_json::Value) -> (Vec<Vertex>, Vec<u32>) {
    let verts = val["vertices"].as_array();
    let idxs = val["indices"].as_array();
    if verts.is_none() || idxs.is_none() {
        return (Vec::new(), Vec::new());
    }
    let verts = verts.unwrap();
    let idxs = idxs.unwrap();
    let vertices: Vec<Vertex> = verts.iter().map(|v| Vertex {
        position: arr3(&v["position"]),
        normal: arr3(&v["normal"]),
        uv: arr2(&v["uv"]),
        bone_weights: arr4_f32(&v["bone_weights"]),
        bone_indices: arr4_u32(&v["bone_indices"]),
    }).collect();
    let indices: Vec<u32> = idxs.iter().map(|i| i.as_u64().unwrap_or(0) as u32).collect();
    (vertices, indices)
}

fn arr3(v: &serde_json::Value) -> [f32; 3] {
    let a = v.as_array().unwrap();
    [a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32]
}

fn arr2(v: &serde_json::Value) -> [f32; 2] {
    let a = v.as_array().unwrap();
    [a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32]
}

fn arr4_f32(v: &serde_json::Value) -> [f32; 4] {
    let a = v.as_array().unwrap();
    [a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32, a[3].as_f64().unwrap_or(0.0) as f32]
}

fn arr4_u32(v: &serde_json::Value) -> [u32; 4] {
    let a = v.as_array().unwrap();
    [a[0].as_u64().unwrap_or(0) as u32, a[1].as_u64().unwrap_or(0) as u32, a[2].as_u64().unwrap_or(0) as u32, a[3].as_u64().unwrap_or(0) as u32]
}

fn parse_motion_clip(val: &serde_json::Value) -> crate::animation::playback::MotionClip {
    let skeleton: Skeleton = serde_json::from_value(val["skeleton"].clone()).unwrap_or(Skeleton { name: String::new(), joints: Vec::new() });
    let fps = val["fps"].as_f64().unwrap_or(30.0) as f32;
    let loop_ = val["loop"].as_bool().unwrap_or(false);
    let frames: Vec<Vec<Quaternion>> = val["frames"].as_array().map(|fa| {
        fa.iter().map(|frame| {
            frame.as_array().map(|qa| {
                qa.iter().map(|q| Quaternion {
                    w: q["w"].as_f64().unwrap_or(1.0) as f32,
                    x: q["x"].as_f64().unwrap_or(0.0) as f32,
                    y: q["y"].as_f64().unwrap_or(0.0) as f32,
                    z: q["z"].as_f64().unwrap_or(0.0) as f32,
                }).collect()
            }).unwrap_or_default()
        }).collect()
    }).unwrap_or_default();
    let root_positions: Vec<(f32, f32, f32)> = val["root_positions"].as_array().map(|rpa| {
        rpa.iter().map(|rp| {
            let a = rp.as_array().unwrap();
            (a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32)
        }).collect()
    }).unwrap_or_default();
    crate::animation::playback::MotionClip { skeleton, frames, root_positions, fps, loop_ }
}
