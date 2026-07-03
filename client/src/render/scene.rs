use std::mem;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::math::{forward_kinematics, multiply_mat4, Transform};
use crate::core::skeleton::Pose;
use crate::render::camera::OrbitCamera;
use crate::render::mesh::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SkinUniformRaw {
    joint_count: u32,
    padding: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraRaw {
    view_proj: [f32; 16],
    camera_pos: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct LightRaw {
    direction: [f32; 3],
    padding: f32,
    color: [f32; 3],
    ambient: [f32; 3],
}

const MAX_JOINTS: usize = 128;

pub struct SkinRenderer {
    pipeline: wgpu::RenderPipeline,
    skin_uniform_buf: wgpu::Buffer,
    joint_matrix_buf: wgpu::Buffer,
    camera_buf: wgpu::Buffer,
    #[allow(dead_code)]
    #[allow(dead_code)]
    light_buf: wgpu::Buffer,
    bind_group_0: wgpu::BindGroup,
    bind_group_1: wgpu::BindGroup,
    #[allow(dead_code)]
    fallback_texture: wgpu::Texture,
    #[allow(dead_code)]
    fallback_view: wgpu::TextureView,
    #[allow(dead_code)]
    fallback_sampler: wgpu::Sampler,
    mesh_buffers: Option<SkinMeshBuffers>,
    rest_inv_bind: Vec<[f32; 16]>,
    joint_count: u32,
}

struct SkinMeshBuffers {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,
}

impl SkinRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("skin.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("skin.wgsl").into()),
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 3 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32x4, offset: 48, shader_location: 4 },
            ],
        };

        let skin_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("skin_uniform"),
            contents: bytemuck::bytes_of(&SkinUniformRaw { joint_count: 1, padding: 0 }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let joint_matrix_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("joint_matrices"),
            size: (MAX_JOINTS * 64) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::bytes_of(&CameraRaw {
                view_proj: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
                camera_pos: [0.0, 0.0, 0.0, 1.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light = LightRaw {
            direction: [-0.5, -0.8, -0.3],
            padding: 0.0,
            color: [1.0, 0.95, 0.9],
            ambient: [0.06, 0.06, 0.08],
        };
        let light_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light"),
            contents: bytemuck::bytes_of(&light),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let fallback_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fallback"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            fallback_texture.as_image_copy(),
            &[255u8, 255, 255, 255],
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let fallback_view = fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let fallback_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout_0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skin_bg0_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None,
                    }, count: None,
                },
            ],
        });
        let bind_group_layout_1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skin_bg1_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture {
                        multisampled: false, sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2,
                    }, count: None,
                },
            ],
        });

        let bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("skin_bg0"),
            layout: &bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: skin_uniform_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: joint_matrix_buf.as_entire_binding() },
            ],
        });
        let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("skin_bg1"),
            layout: &bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&fallback_sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&fallback_view) },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skin_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0, &bind_group_layout_1],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skin_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
        });

        Self {
            pipeline,
            skin_uniform_buf,
            joint_matrix_buf,
            camera_buf,
            light_buf,
            bind_group_0,
            bind_group_1,
            fallback_texture,
            fallback_view,
            fallback_sampler,
            mesh_buffers: None,
            rest_inv_bind: Vec::new(),
            joint_count: 0,
        }
    }

    pub fn upload_mesh(&mut self, device: &wgpu::Device, vertices: Vec<Vertex>, indices: Vec<u32>, skeleton_json: &serde_json::Value) {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_index"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.mesh_buffers = Some(SkinMeshBuffers {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
        });

        self.compute_rest_inv_bind(skeleton_json);
    }

    fn compute_rest_inv_bind(&mut self, skeleton_json: &serde_json::Value) {
        let joints = skeleton_json["joints"].as_array().unwrap();
        self.joint_count = joints.len() as u32;

        let parent_indices: Vec<i32> = joints.iter().map(|j| j["parent_index"].as_i64().unwrap_or(-1) as i32).collect();

        let rest_local: Vec<Transform> = joints.iter().map(|j| {
            let t = &j["local_transform"];
            let r = &t["rotation"];
            Transform {
                translation: (
                    t["translation"][0].as_f64().unwrap_or(0.0) as f32,
                    t["translation"][1].as_f64().unwrap_or(0.0) as f32,
                    t["translation"][2].as_f64().unwrap_or(0.0) as f32,
                ),
                rotation: crate::core::math::Quaternion {
                    w: r["w"].as_f64().unwrap_or(1.0) as f32,
                    x: r["x"].as_f64().unwrap_or(0.0) as f32,
                    y: r["y"].as_f64().unwrap_or(0.0) as f32,
                    z: r["z"].as_f64().unwrap_or(0.0) as f32,
                },
                scale: (1.0, 1.0, 1.0),
            }
        }).collect();

        let rest_global = forward_kinematics(&rest_local, &parent_indices);
        self.rest_inv_bind = rest_global.iter().map(|t| invert_affine(&t.to_matrix())).collect();
    }

    pub fn update_pose(&self, queue: &wgpu::Queue, pose: &Pose, camera: &OrbitCamera) {
        let parent_indices: Vec<i32> = pose.skeleton.joints.iter().map(|j| j.parent_index).collect();
        let local = pose.local_transforms();
        let global = forward_kinematics(&local, &parent_indices);

        let mut bone_mats = [[0.0f32; 16]; MAX_JOINTS];
        for (i, g) in global.iter().enumerate() {
            if i < MAX_JOINTS && i < self.rest_inv_bind.len() {
                bone_mats[i] = multiply_mat4(&g.to_matrix(), &self.rest_inv_bind[i]);
            }
        }

        if self.joint_count > 0 {
            queue.write_buffer(&self.skin_uniform_buf, 0, bytemuck::bytes_of(&SkinUniformRaw {
                joint_count: self.joint_count,
                padding: 0,
            }));
        }

        queue.write_buffer(&self.joint_matrix_buf, 0, bytemuck::cast_slice(&bone_mats[..]));

        let vp = camera.view_proj();
        let eye = camera.eye();
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&CameraRaw {
            view_proj: vp,
            camera_pos: [eye[0], eye[1], eye[2], 1.0],
        }));
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        self.bind(rpass);
        if let Some(mesh) = &self.mesh_buffers {
            rpass.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
            rpass.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }

    fn bind<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group_0, &[]);
        rpass.set_bind_group(1, &self.bind_group_1, &[]);
    }
}

fn invert_affine(m: &[f32; 16]) -> [f32; 16] {
    let r00 = m[0]; let r01 = m[4]; let r02 = m[8];
    let r10 = m[1]; let r11 = m[5]; let r12 = m[9];
    let r20 = m[2]; let r21 = m[6]; let r22 = m[10];
    let t0 = m[3]; let t1 = m[7]; let t2 = m[11];
    [
        r00, r01, r02, 0.0,
        r10, r11, r12, 0.0,
        r20, r21, r22, 0.0,
        -(r00 * t0 + r01 * t1 + r02 * t2),
        -(r10 * t0 + r11 * t1 + r12 * t2),
        -(r20 * t0 + r21 * t1 + r22 * t2),
        1.0,
    ]
}
