use std::mem;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::math::multiply_mat4;
use crate::render::camera::OrbitCamera;
use crate::render::mesh::StaticVertex;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraRaw {
    view_proj: [f32; 16],
    camera_pos: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MaterialRaw {
    albedo: [f32; 4],
    metallic: f32,
    roughness: f32,
    ambient_occlusion: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct LightRaw {
    direction: [f32; 3],
    padding: f32,
    color: [f32; 3],
    _gap: f32,
    ambient: [f32; 3],
    _padding: f32,
}

pub struct StaticMeshGpu {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,
    model_buf: wgpu::Buffer,
    material_buf: wgpu::Buffer,
    bind_group_0: wgpu::BindGroup,
}

pub struct StaticRenderer {
    pipeline: wgpu::RenderPipeline,
    camera_buf: wgpu::Buffer,
    light_buf: wgpu::Buffer,
    bind_group_layout_0: wgpu::BindGroupLayout,
    bind_group_1: wgpu::BindGroup,
    meshes: Vec<StaticMeshGpu>,
    pool: Vec<StaticMeshGpu>,
}

impl StaticRenderer {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("static_mesh.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("static_mesh.wgsl").into()),
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<StaticVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 },
            ],
        };

        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("static_camera"),
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
            _gap: 0.0,
            ambient: [0.06, 0.06, 0.08],
            _padding: 0.0,
        };
        let light_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("static_light"),
            contents: bytemuck::bytes_of(&light),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout_0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("static_bg0_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group_layout_1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("static_bg1_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("static_bg1"),
            layout: &bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: light_buf.as_entire_binding() },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("static_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0, &bind_group_layout_1],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("static_pipeline"),
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
            camera_buf,
            light_buf,
            bind_group_layout_0,
            bind_group_1,
            meshes: Vec::new(),
            pool: Vec::new(),
        }
    }

    pub fn add_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: Vec<StaticVertex>,
        indices: Vec<u32>,
        model_matrix: [f32; 16],
        albedo: [f32; 3],
        metallic: f32,
        roughness: f32,
    ) {
        let material = MaterialRaw {
            albedo: [albedo[0], albedo[1], albedo[2], 1.0],
            metallic,
            roughness,
            ambient_occlusion: 1.0,
            _padding: 0.0,
        };

        if let Some(mut entry) = self.pool.pop() {
            queue.write_buffer(&entry.model_buf, 0, bytemuck::bytes_of(&model_matrix));
            queue.write_buffer(&entry.material_buf, 0, bytemuck::bytes_of(&material));
            if entry.index_count == indices.len() as u32 {
                entry.index_count = indices.len() as u32;
                self.meshes.push(entry);
                return;
            }
        }

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("static_mesh_vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("static_mesh_index"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let model_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("static_model"),
            contents: bytemuck::bytes_of(&model_matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let material_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("static_mat"),
            contents: bytemuck::bytes_of(&material),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("static_bg0"),
            layout: &self.bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: model_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: material_buf.as_entire_binding() },
            ],
        });

        self.meshes.push(StaticMeshGpu {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
            model_buf,
            material_buf,
            bind_group_0,
        });
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &OrbitCamera) {
        let vp = camera.view_proj();
        let eye = camera.eye();
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&CameraRaw {
            view_proj: vp,
            camera_pos: [eye[0], eye[1], eye[2], 1.0],
        }));
    }

    pub fn clear(&mut self) {
        self.pool.append(&mut self.meshes);
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        for mesh in &self.meshes {
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &mesh.bind_group_0, &[]);
            rpass.set_bind_group(1, &self.bind_group_1, &[]);
            rpass.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
            rpass.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}

pub fn identity_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn translation_matrix(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        x,   y,   z,   1.0,
    ]
}

pub fn scale_matrix(sx: f32, sy: f32, sz: f32) -> [f32; 16] {
    [
        sx,  0.0, 0.0, 0.0,
        0.0, sy,  0.0, 0.0,
        0.0, 0.0, sz,  0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn multiply_matrices(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    multiply_mat4(a, b)
}
