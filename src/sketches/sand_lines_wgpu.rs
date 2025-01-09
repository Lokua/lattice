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
    gui_h: Some(240),
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    // w, h, ..unused
    resolution: [f32; 4],

    // [start_x, start_y, end_x, end_y]
    ref_points: [f32; 4],

    // [points_per_segment, noise_scale, angle_variation, n_lines]
    settings: [f32; 4],

    // [point_size, passes, unused]
    settings2: [f32; 4],
}

#[derive(SketchComponents)]
pub struct Model {
    controls: Controls,
    wr: WindowRect,
    gpu: gpu::GpuState,
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let controls = Controls::with_previous(vec![
        Control::slider("passes", 3.0, (1.0, 10.0), 1.0),
        Control::slider("n_lines", 64.0, (1.0, 128.0), 1.0),
        Control::slider("points_per_segment", 100.0, (10.0, 500.0), 10.0),
        Control::slider("noise_scale", 0.001, (0.0, 0.1), 0.0001),
        Control::slider("angle_variation", 0.2, (0.0, TWO_PI), 0.1),
        Control::slider("point_size", 0.001, (0.0005, 0.01), 0.0001),
    ]);

    let points_per_segment = controls.float("points_per_segment") as u32;

    let params = ShaderParams {
        resolution: [wr.w(), wr.h(), 0.0, 0.0],
        ref_points: [-0.9, 0.0, 0.9, 0.0],
        settings: [
            points_per_segment as f32,
            controls.float("noise_scale"),
            controls.float("angle_variation"),
            controls.float("n_lines"),
        ],
        settings2: [
            controls.float("point_size"),
            controls.float("passes"),
            0.0,
            0.0,
        ],
    };

    let shader = wgpu::include_wgsl!("./sand_lines_wgpu.wgsl");
    let gpu = gpu::GpuState::new_with_config(
        app,
        shader,
        &params,
        gpu::PipelineConfig {
            vertex_data: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            ref_points: [-0.9, 0.0, 0.9, 0.0],
            settings: [
                points_per_segment as f32,
                m.controls.float("noise_scale"),
                m.controls.float("angle_variation"),
                m.controls.float("n_lines"),
            ],
            settings2: [
                m.controls.float("point_size"),
                m.controls.float("passes"),
                0.0,
                0.0,
            ],
        };

        m.gpu.update_params(app, &params);
        m.controls.mark_unchanged();
    }
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(WHITE);
    m.gpu.render(&frame);
}
