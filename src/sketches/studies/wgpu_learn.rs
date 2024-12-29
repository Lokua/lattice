use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "wgpu_learn",
    display_name: "WGPU Learn",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(220),
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
}

const VERTICES: &[Vertex] = &[
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

#[derive(SketchComponents)]
pub struct Model {
    #[allow(dead_code)]
    animation: Animation,
    controls: Controls,
    #[allow(dead_code)]
    wr: WindowRect,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::with_previous(vec![
        Control::slider_norm("a", 0.5),
        Control::slider_norm("b", 0.5),
        Control::slider_norm("c", 0.5),
        Control::slider_norm("d", 0.5),
    ]);

    let params = ShaderParams {
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    };

    let window = app.main_window();
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    let shader = wgpu::include_wgsl!("./wgpu_learn.wgsl");
    let shader_module = device.create_shader_module(shader);

    let vertices_bytes = unsafe { wgpu::bytes::from_slice(VERTICES) };
    let vertex_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: vertices_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

    let params_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
                        std::mem::size_of::<ShaderParams>() as _,
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
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(sample_count)
        .build(device);

    Model {
        animation,
        controls,
        wr,
        render_pipeline,
        vertex_buffer,
        params_buffer,
        params_bind_group,
    }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    let params = ShaderParams {
        a: m.controls.float("a"),
        b: m.controls.float("b"),
        c: m.controls.float("c"),
        d: m.controls.float("d"),
    };

    app.main_window().queue().write_buffer(
        &m.params_buffer,
        0,
        bytemuck::bytes_of(&params),
    );
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| {
            color.load_op(wgpu::LoadOp::Load)
        })
        .begin(&mut encoder);
    render_pass.set_pipeline(&m.render_pipeline);
    render_pass.set_bind_group(0, &m.params_bind_group, &[]);
    render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
    render_pass.draw(0..6, 0..1);
}
