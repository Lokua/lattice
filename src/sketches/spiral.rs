use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "sand_lines_wgpu",
    display_name: "Sand Lines WGPU",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(440),
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    // w, h, ..unused
    resolution: [f32; 4],

    // start_x, start_y, end_x, end_y
    a: [f32; 4],

    // points_per_segment, noise_scale, angle_variation, n_lines
    b: [f32; 4],

    // point_size, circle_r_min, circle_r_max, offset_mult
    c: [f32; 4],
}

#[derive(SketchComponents)]
pub struct Model {
    controls: Controls,
    wr: WindowRect,
    gpu: gpu::GpuState,
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let controls = Controls::with_previous(vec![
        Control::slider("v_count_millions", 6.0, (1.0, 100.0), 1.0),
        Control::slider("n_lines", 64.0, (1.0, 256.0), 1.0),
        Control::slider("points_per_segment", 100.0, (10.0, 10_000.0), 10.0),
        Control::slider("noise_scale", 0.001, (0.0, 0.1), 0.0001),
        Control::slider("angle_variation", 0.2, (0.0, TWO_PI), 0.1),
        Control::slider("point_size", 0.001, (0.0005, 0.01), 0.0001),
        Control::slider_norm("circle_r_min", 0.5),
        Control::slider_norm("circle_r_max", 0.9),
        Control::slider("offset_mult", 0.9, (0.0, 3.0), 0.001),
    ]);

    let params = ShaderParams {
        resolution: [0.0; 4],
        a: [0.0; 4],
        b: [0.0; 4],
        c: [0.0; 4],
    };

    let shader = wgpu::include_wgsl!("./spiral.wgsl");
    let gpu = gpu::GpuState::new_with_config(
        app,
        shader,
        &params,
        gpu::PipelineConfig {
            vertex_data: None,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            ..Default::default()
        },
    );

    Model { controls, wr, gpu }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    if m.controls.changed() {
        let points_per_segment = m.controls.float("points_per_segment") as u32;

        let params = ShaderParams {
            resolution: [m.wr.w(), m.wr.h(), 0.0, 0.0],
            a: [-0.9, 0.0, 0.9, 0.0],
            b: [
                points_per_segment as f32,
                m.controls.float("noise_scale"),
                m.controls.float("angle_variation"),
                m.controls.float("n_lines"),
            ],
            c: [
                m.controls.float("point_size"),
                m.controls.float("circle_r_min"),
                m.controls.float("circle_r_max"),
                m.controls.float("offset_mult"),
            ],
        };

        m.gpu.update_params(app, &params);
        m.controls.mark_unchanged();
    }
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(WHITE);

    let points_per_line = m.controls.float("points_per_segment") as u32;
    let n_lines = m.controls.float("n_lines") as u32;
    let total_points = points_per_line * n_lines;
    let density = m.controls.float("v_count_millions") as u32;
    let spiral_vertices = total_points * 6 * density;
    let background_vertices = 3;
    let total_vertices = background_vertices + spiral_vertices;

    m.gpu.render_procedural(&frame, total_vertices);
}
