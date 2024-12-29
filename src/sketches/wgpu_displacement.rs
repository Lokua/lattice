use gpu::GpuState;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "wgpu_displacement",
    display_name: "WGPU Displacement Pattern",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(340),
};

#[derive(SketchComponents)]
pub struct Model {
    #[allow(dead_code)]
    animation: Animation,
    controls: Controls,
    #[allow(dead_code)]
    wr: WindowRect,
    gpu: gpu::GpuState,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    radius: f32,
    strength: f32,
    scaling_power: f32,
    mix: f32,
    r: f32,
    g: f32,
    b: f32,
    ring_strength: f32,
    angular_variation: f32,
    threshold: f32,
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::with_previous(vec![
        Control::slider("radius", 0.5, (0.0, 10.0), 0.01),
        Control::slider("strength", 0.5, (0.0, 5.0), 0.001),
        Control::slider("scaling_power", 1.0, (0.01, 10.0), 0.01),
        Control::slider_norm("mix", 0.5),
        Control::Separator {},
        Control::slider_norm("r", 0.5),
        Control::slider_norm("g", 0.0),
        Control::slider_norm("b", 1.0),
        Control::Separator {},
        Control::slider("ring_strength", 20.0, (1.0, 100.0), 0.01),
        Control::slider("angular_variation", 4.0, (1.0, 360.0), 1.0),
        Control::slider_norm("threshold", 0.5),
    ]);

    let params = ShaderParams {
        radius: 0.0,
        strength: 0.0,
        scaling_power: 0.0,
        mix: 0.0,
        r: 0.0,
        g: 0.0,
        b: 0.0,
        ring_strength: 0.0,
        angular_variation: 0.0,
        threshold: 0.0,
    };

    let shader = wgpu::include_wgsl!("./wgpu_displacement.wgsl");
    let gpu = GpuState::new(app, shader, &params);

    Model {
        animation,
        controls,
        wr,
        gpu,
    }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    let params = ShaderParams {
        radius: m.controls.float("radius"),
        strength: m.controls.float("strength"),
        scaling_power: m.controls.float("scaling_power"),
        mix: m.controls.float("mix"),
        r: m.controls.float("r"),
        g: m.controls.float("g"),
        b: m.controls.float("b"),
        ring_strength: m.controls.float("ring_strength"),
        angular_variation: m.controls.float("angular_variation"),
        threshold: m.controls.float("threshold"),
    };

    m.gpu.update_params(app, &params);
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    m.gpu.render(&frame);
}
