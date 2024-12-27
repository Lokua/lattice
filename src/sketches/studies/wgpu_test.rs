use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "wgpu_test",
    display_name: "WGPU Test",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(150),
};

// #[repr(C)] tells Rust to lay out the struct's memory the same way C would
// This ensures consistent memory layout between CPU and GPU and prevents Rust
// from reordering fields for optimization.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
}

// Defines a fullscreen quad using two triangles:
//      (-1,1)      (1,1)
//         ┌─────────┐
//         │       ╱ │
//         │    ╱    │
//         │ ╱       │
//         └─────────┘
//      (-1,-1)     (1,-1)
// This gives us a "canvas" for our fragment shader to draw on
// NOTE: this is also known as "normalized device coordinates" (NDC)
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
// bytemuck::Pod and bytemuck::Zeroable are safety traits for type casting.
// Pod: "Plain Old Data" - safe to treat as a bunch of bytes
// Zeroable means it's safe to create an instance filled with 0s
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    time: f32,
    mix_factor: f32,
}

#[derive(SketchComponents)]
pub struct Model {
    controls: Controls,
    #[allow(dead_code)]
    wr: WindowRect,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let window = app.main_window();
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    // Create shader module
    let shader = wgpu::include_wgsl!("./wgpu_test.wgsl");
    let shader_module = device.create_shader_module(shader);

    // Create vertex buffer
    let vertices_bytes = unsafe { wgpu::bytes::from_slice(VERTICES) };
    let vertex_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: vertices_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

    // Create params buffer and bind group
    let params = ShaderParams {
        time: 0.0,
        mix_factor: 0.5,
    };

    // Create time uniform buffer and bind group
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

    // Create pipeline layout and render pipeline
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

    let controls = Controls::with_previous(vec![Control::slider(
        "mix",
        0.5,
        (0.0, 1.0),
        0.01,
    )]);

    Model {
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
        time: app.time,
        mix_factor: m.controls.float("mix"),
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
        .color_attachment(frame.texture_view(), |color| color)
        .begin(&mut encoder);

    render_pass.set_pipeline(&m.render_pipeline);
    render_pass.set_bind_group(0, &m.params_bind_group, &[]);
    render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));

    // This is drawing our VERTICES const.
    // 0..6 says "use 6 vertices" (our two triangles)
    // 0..1 says "draw one instance"
    render_pass.draw(0..6, 0..1);
}
