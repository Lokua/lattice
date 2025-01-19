use nannou::prelude::*;
use wgpu::util::DeviceExt;

use super::prelude::*;

// This code is a mess and an example of what happens when you
// try to abstract something you don't fully understand

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
}

pub const QUAD_COVER_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0],
    },
];

pub struct PipelineConfig {
    pub topology: wgpu::PrimitiveTopology,
    pub vertex_data: Option<&'static [Vertex]>,
    pub blend: Option<wgpu::BlendState>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            topology: wgpu::PrimitiveTopology::TriangleList,
            vertex_data: Some(QUAD_COVER_VERTICES),
            blend: None,
        }
    }
}

pub struct GpuState {
    render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: Option<wgpu::Buffer>,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
    n_vertices: u32,
}

static FLOW_FIELD_VERTEX_ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
    0 => Float32x2,
    1 => Float32x4,
    // 1 => Float32x2,
    // 2 => Float32,
    // 3 => Float32x4
];

impl GpuState {
    pub fn new<P: bytemuck::Pod + bytemuck::Zeroable>(
        app: &App,
        shader_source: wgpu::ShaderModuleDescriptor,
        initial_params: &P,
    ) -> Self {
        Self::new_with_config(
            app,
            shader_source,
            initial_params,
            Default::default(),
        )
    }

    pub fn new_with_config<P: bytemuck::Pod + bytemuck::Zeroable>(
        app: &App,
        shader_source: wgpu::ShaderModuleDescriptor,
        initial_params: &P,
        config: PipelineConfig,
    ) -> Self {
        let size = std::mem::size_of::<P>();
        info!("ShaderParams size: {} bytes", size);
        if size % 16 != 0 {
            warn!(
                "Param size {}, need {} pad bytes",
                size,
                (16 - (size % 16)) % 16
            );
        }

        let window = app.main_window();
        let device = window.device();
        let format = Frame::TEXTURE_FORMAT;
        let sample_count = window.msaa_samples();
        let shader_module = device.create_shader_module(shader_source);

        let (vertex_buffer, n_vertices) = if let Some(vertices) =
            config.vertex_data
        {
            let vertices_bytes = unsafe { wgpu::bytes::from_slice(vertices) };
            let buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: vertices_bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                });
            (Some(buffer), vertices.len() as u32)
        } else {
            (None, 0)
        };

        let params_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::bytes_of(initial_params),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        let params_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<P>() as _,
                        ),
                    },
                    count: None,
                }],
                label: Some("params_bind_group_layout"),
            });

        let params_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &params_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                }],
                label: Some("params_bind_group"),
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&params_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_builder = wgpu::RenderPipelineBuilder::from_layout(
            &pipeline_layout,
            &shader_module,
        )
        .primitive_topology(config.topology)
        .vertex_entry_point("vs_main")
        .fragment_shader(&shader_module)
        .fragment_entry_point("fs_main");

        // Add vertex buffer layout only if we have vertex data
        let render_pipeline_builder = if config.vertex_data.is_some() {
            render_pipeline_builder.add_vertex_buffer::<Vertex>(
                &wgpu::vertex_attr_array![0 => Float32x2],
            )
        } else {
            render_pipeline_builder
        };

        let target = if let Some(blend) = config.blend {
            wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            }
        } else {
            wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }
        };

        let render_pipeline = render_pipeline_builder
            .color_state(target)
            .sample_count(sample_count)
            .build(device);

        Self {
            render_pipeline,
            vertex_buffer,
            params_buffer,
            params_bind_group,
            n_vertices,
        }
    }

    pub fn new_test<V: bytemuck::Pod, P: bytemuck::Pod + bytemuck::Zeroable>(
        app: &App,
        shader_source: wgpu::ShaderModuleDescriptor,
        initial_params: &P,
        vertices: Option<&[V]>,
        config: PipelineConfig,
    ) -> Self {
        let size = std::mem::size_of::<P>();
        info!("ShaderParams size: {} bytes", size);
        if size % 16 != 0 {
            warn!(
                "Param size {}, need {} pad bytes",
                size,
                (16 - (size % 16)) % 16
            );
        }

        let window = app.main_window();
        let device = window.device();
        let format = Frame::TEXTURE_FORMAT;
        let sample_count = window.msaa_samples();
        let shader_module = device.create_shader_module(shader_source);

        let (vertex_buffer, n_vertices) = if let Some(verts) = vertices {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: std::mem::size_of::<V>() as u64 * 10_000 * 6,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            (Some(buffer), verts.len() as u32)
        } else {
            (None, 0)
        };

        let params_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::bytes_of(initial_params),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        let params_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<P>() as _,
                        ),
                    },
                    count: None,
                }],
                label: Some("params_bind_group_layout"),
            });

        let params_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &params_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                }],
                label: Some("params_bind_group"),
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&params_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_builder = wgpu::RenderPipelineBuilder::from_layout(
            &pipeline_layout,
            &shader_module,
        )
        .primitive_topology(config.topology)
        .vertex_entry_point("vs_main")
        .fragment_shader(&shader_module)
        .fragment_entry_point("fs_main");

        let render_pipeline_builder = if vertices.is_some() {
            render_pipeline_builder
                .add_vertex_buffer::<V>(FLOW_FIELD_VERTEX_ATTRS)
        } else {
            render_pipeline_builder
        };

        let target = if let Some(blend) = config.blend {
            wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            }
        } else {
            wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }
        };

        let render_pipeline = render_pipeline_builder
            .color_state(target)
            .sample_count(sample_count)
            .build(device);

        Self {
            render_pipeline,
            vertex_buffer,
            params_buffer,
            params_bind_group,
            n_vertices,
        }
    }

    pub fn update_params<P: bytemuck::Pod>(&self, app: &App, params: &P) {
        app.main_window().queue().write_buffer(
            &self.params_buffer,
            0,
            bytemuck::bytes_of(params),
        );
    }

    pub fn render(&self, frame: &Frame) {
        let mut encoder = frame.command_encoder();
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(frame.texture_view(), |color| {
                color.load_op(wgpu::LoadOp::Load)
            })
            .begin(&mut encoder);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.params_bind_group, &[]);

        if let Some(ref vertex_buffer) = self.vertex_buffer {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..self.n_vertices, 0..1);
        } else {
            // When no vertex buffer is provided, use vertex_index
            render_pass.draw(0..6000000, 0..1); // TODO: Make this configurable
        }
    }

    // Add new method for procedural rendering with custom vertex count
    pub fn render_procedural(&self, frame: &Frame, vertex_count: u32) {
        let mut encoder = frame.command_encoder();
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(frame.texture_view(), |color| {
                color.load_op(wgpu::LoadOp::Load)
            })
            .begin(&mut encoder);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.params_bind_group, &[]);
        render_pass.draw(0..vertex_count, 0..1);
    }
}
