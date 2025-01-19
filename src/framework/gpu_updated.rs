use nannou::prelude::*;
use wgpu::util::DeviceExt;

static FLOW_FIELD_VERTEX_ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
    0 => Float32x2,
    1 => Float32x4,
    // 1 => Float32x2,
    // 2 => Float32,
    // 3 => Float32x4
];

pub struct GpuState<V: bytemuck::Pod + bytemuck::Zeroable> {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: Option<wgpu::Buffer>,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
    n_vertices: u32,
    _marker: std::marker::PhantomData<V>,
}

impl<V: bytemuck::Pod + bytemuck::Zeroable> GpuState<V> {
    pub fn new<P: bytemuck::Pod + bytemuck::Zeroable>(
        app: &App,
        shader: wgpu::ShaderModuleDescriptor,
        initial_params: &P,
        vertices: Option<&[V]>,
        vertex_attributes: &[wgpu::VertexAttribute],
        // vertex_layout: &[wgpu::VertexAttribute],
        topology: wgpu::PrimitiveTopology,
        blend: Option<wgpu::BlendState>,
    ) -> Self {
        let window = app.main_window();
        let device = window.device();
        let sample_count = window.msaa_samples();
        let format = Frame::TEXTURE_FORMAT;
        let shader_module = device.create_shader_module(shader);

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
                label: Some("Params Bind Group Layout"),
            });

        let params_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &params_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                }],
                label: Some("Params Bind Group"),
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&params_bind_group_layout],
                push_constant_ranges: &[],
            });

        let (vertex_buffer, n_vertices) = if let Some(verts) = vertices {
            let buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(verts),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST,
                });
            (Some(buffer), verts.len() as u32)
        } else {
            (None, 0)
        };

        let vertex_attributes =
            &Self::infer_vertex_attributes(&vertices.unwrap());

        let vertex_buffers = if vertices.is_some() {
            vec![wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<V>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                // attributes: FLOW_FIELD_VERTEX_ATTRS,
                attributes: vertex_attributes,
                // attributes: vertex_layout,
            }]
        } else {
            vec![]
        };

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: sample_count,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Self {
            render_pipeline,
            vertex_buffer,
            params_buffer,
            params_bind_group,
            n_vertices,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn update_params<P: bytemuck::Pod>(&self, app: &App, params: &P) {
        app.main_window().queue().write_buffer(
            &self.params_buffer,
            0,
            bytemuck::bytes_of(params),
        );
    }

    pub fn update_vertex_buffer(&self, app: &App, vertices: &[V]) {
        app.main_window().queue().write_buffer(
            self.vertex_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&vertices),
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
            // Procedural rendering
            render_pass.draw(0..3, 0..1);
        }
    }

    // pub fn render(&self, frame: &Frame) {
    //     let mut encoder = frame.command_encoder();
    //     let mut render_pass = wgpu::RenderPassBuilder::new()
    //         .color_attachment(frame.texture_view(), |color| {
    //             color.load_op(wgpu::LoadOp::Load)
    //         })
    //         .begin(&mut encoder);
    //     render_pass.set_pipeline(&self.render_pipeline);
    //     render_pass.set_bind_group(0, &self.params_bind_group, &[]);

    //     if let Some(ref vertex_buffer) = self.vertex_buffer {
    //         render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    //         render_pass.draw(0..self.n_vertices, 0..1);
    //     } else {
    //         // When no vertex buffer is provided, use vertex_index
    //         render_pass.draw(0..6000000, 0..1); // TODO: Make this configurable
    //     }
    // }

    // // Add new method for procedural rendering with custom vertex count
    // pub fn render_procedural(&self, frame: &Frame, vertex_count: u32) {
    //     let mut encoder = frame.command_encoder();
    //     let mut render_pass = wgpu::RenderPassBuilder::new()
    //         .color_attachment(frame.texture_view(), |color| {
    //             color.load_op(wgpu::LoadOp::Load)
    //         })
    //         .begin(&mut encoder);
    //     render_pass.set_pipeline(&self.render_pipeline);
    //     render_pass.set_bind_group(0, &self.params_bind_group, &[]);
    //     render_pass.draw(0..vertex_count, 0..1);
    // }

    fn infer_vertex_attributes(vertices: &[V]) -> Vec<wgpu::VertexAttribute> {
        if vertices.is_empty() {
            panic!("Vertex data cannot be empty when inferring attributes");
        }

        let mut attributes = Vec::new();
        let mut offset = 0;
        let vertex_size = std::mem::size_of::<V>();

        // // Examine chunks of the vertex size that could represent f32 arrays
        // while offset < vertex_size {
        //     let remaining_size = vertex_size - offset;

        //     // Determine the largest possible f32 array at this offset
        //     let format = if remaining_size >= 16 {
        //         (16, wgpu::VertexFormat::Float32x4)
        //     } else if remaining_size >= 12 {
        //         (12, wgpu::VertexFormat::Float32x3)
        //     } else if remaining_size >= 8 {
        //         (8, wgpu::VertexFormat::Float32x2)
        //     } else if remaining_size >= 4 {
        //         (4, wgpu::VertexFormat::Float32)
        //     } else {
        //         panic!("Unexpected remaining size: {}", remaining_size);
        //     };

        //     attributes.push(wgpu::VertexAttribute {
        //         offset: offset as u64,
        //         shader_location: attributes.len() as u32,
        //         format: format.1,
        //     });

        //     offset += format.0;
        // }

        attributes
    }
}
