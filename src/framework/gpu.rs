use nannou::prelude::*;
use wgpu::util::DeviceExt;

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

pub struct GpuState {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
}

impl GpuState {
    pub fn new<P: bytemuck::Pod + bytemuck::Zeroable>(
        app: &App,
        shader_source: wgpu::ShaderModuleDescriptor,
        initial_params: &P,
    ) -> Self {
        let window = app.main_window();
        let device = window.device();
        let format = Frame::TEXTURE_FORMAT;
        let sample_count = window.msaa_samples();
        let shader_module = device.create_shader_module(shader_source);

        let vertices_bytes =
            unsafe { wgpu::bytes::from_slice(QUAD_COVER_VERTICES) };
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: vertices_bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });

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
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                label: Some("Basic Pipeline Layout"),
                bind_group_layouts: &[&params_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_builder = wgpu::RenderPipelineBuilder::from_layout(
            &pipeline_layout,
            &shader_module,
        );

        let render_pipeline = render_pipeline_builder
            .vertex_entry_point("vs_main")
            .fragment_shader(&shader_module)
            .fragment_entry_point("fs_main")
            .color_format(format)
            .add_vertex_buffer::<Vertex>(
                &wgpu::vertex_attr_array![0 => Float32x2],
            )
            .sample_count(sample_count)
            .build(device);

        Self {
            render_pipeline,
            vertex_buffer,
            params_buffer,
            params_bind_group,
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
