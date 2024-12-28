use crate::framework::prelude::*;
use nannou::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "sand_lines_points",
    display_name: "Sand Lines Points",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(240),
};

const N_LINES: u32 = 64;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    // [ax, ay, bx, by]
    ref_points: [f32; 4],
    // [points_per_segment, noise_scale, angle_variation, n_lines]
    settings: [f32; 4],
    // [point_size, ...unused]
    settings2: [f32; 4],
}

#[derive(SketchComponents)]
pub struct Model {
    controls: Controls,
    #[allow(dead_code)]
    wr: WindowRect,
    render_pipeline: wgpu::RenderPipeline,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    vertex_buffer: wgpu::Buffer,
    n_points: u32,
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let controls = Controls::with_previous(vec![
        Control::slider("points_per_segment", 100.0, (10.0, 500.0), 10.0),
        Control::slider("noise_scale", 0.001, (0.0, 0.1), 0.0001),
        Control::slider("angle_variation", 0.2, (0.0, TWO_PI), 0.1),
        Control::slider("point_size", 0.001, (0.0005, 0.02), 0.0001),
    ]);

    let window = app.main_window();
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    // Create shader module
    let shader = wgpu::include_wgsl!("./sand_lines_wgpu.wgsl");
    let shader_module = device.create_shader_module(shader);

    // Create empty vertex buffer (required but not used)
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Empty Vertex Buffer"),
        size: 1,
        usage: wgpu::BufferUsages::VERTEX,
        mapped_at_creation: false,
    });

    // Initial parameters
    let points_per_segment = controls.float("points_per_segment") as u32;
    // lines * segments_per_line * points_per_segment
    let n_points = N_LINES * 1 * points_per_segment;

    let params = ShaderParams {
        ref_points: [-1.0, 0.0, 1.0, 0.0],
        settings: [points_per_segment as f32, 0.1, 0.2, N_LINES as f32],
        settings2: [
            controls.float("point_size"),
            // ...padding
            0.0,
            0.0,
            0.0,
        ],
    };

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
                visibility: wgpu::ShaderStages::VERTEX,
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
            label: Some("Point Pipeline Layout"),
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
        .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
        .sample_count(sample_count)
        .build(device);

    Model {
        controls,
        wr,
        render_pipeline,
        params_buffer,
        params_bind_group,
        vertex_buffer,
        n_points,
    }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    if m.controls.changed() {
        let points_per_segment = m.controls.float("points_per_segment") as u32;
        m.n_points = N_LINES * 1 * points_per_segment;

        let params = ShaderParams {
            ref_points: [-0.9, 0.0, 0.9, 0.0],
            settings: [
                points_per_segment as f32,
                m.controls.float("noise_scale"),
                m.controls.float("angle_variation"),
                N_LINES as f32,
            ],
            settings2: [
                m.controls.float("point_size"),
                // ...padding
                0.0,
                0.0,
                0.0,
            ],
        };

        app.main_window().queue().write_buffer(
            &m.params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );

        m.controls.mark_unchanged();
    }
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(WHITE);
    {
        let mut encoder = frame.command_encoder();
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(frame.texture_view(), |color| {
                color.load_op(wgpu::LoadOp::Load)
            })
            .begin(&mut encoder);

        render_pass.set_pipeline(&m.render_pipeline);
        render_pass.set_bind_group(0, &m.params_bind_group, &[]);
        render_pass.draw(0..(6 * m.n_points), 0..1);
    }
}
