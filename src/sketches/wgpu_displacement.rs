use gpu::GpuState;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "wgpu_displacement",
    display_name: "WGPU Displacement",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(360),
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
    // displacer "instance" params
    radius: f32,
    strength: f32,
    scaling_power: f32,
    // "global" params
    r: f32,
    g: f32,
    b: f32,
    offset: f32,
    ring_strength: f32,
    angular_variation: f32,
    threshold: f32,
    mix: f32,
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::with_previous(vec![
        Control::slider("radius", 0.5, (0.0, 10.0), 0.01),
        Control::slider("strength", 0.5, (0.0, 5.0), 0.001),
        Control::slider("scaling_power", 1.0, (0.01, 20.0), 0.01),
        Control::Separator {},
        Control::slider_norm("r", 0.5),
        Control::slider_norm("g", 0.0),
        Control::slider_norm("b", 1.0),
        Control::Separator {},
        Control::slider_norm("offset", 0.2),
        Control::slider("ring_strength", 20.0, (1.0, 100.0), 0.01),
        Control::slider("angular_variation", 4.0, (1.0, 45.0), 1.0),
        Control::slider_norm("threshold", 0.5),
        Control::slider_norm("mix", 0.5),
    ]);

    let params = ShaderParams {
        radius: 0.0,
        strength: 0.0,
        scaling_power: 0.0,
        r: 0.0,
        g: 0.0,
        b: 0.0,
        offset: 0.0,
        ring_strength: 0.0,
        angular_variation: 0.0,
        threshold: 0.0,
        mix: 0.0,
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
    let strength = m.controls.float("strength");
    let strength_range = m.controls.slider_range("strength");
    let strength_swing = 0.2;

    let params = ShaderParams {
        radius: m.controls.float("radius"),
        strength: m.animation.lrp(
            &[
                (
                    clamp(
                        strength - strength_swing,
                        strength_range.0,
                        strength_range.1,
                    ),
                    1.0,
                ),
                (
                    clamp(
                        strength + strength_swing,
                        strength_range.0,
                        strength_range.1,
                    ),
                    1.0,
                ),
            ],
            0.0,
        ),
        scaling_power: m.controls.float("scaling_power"),
        r: m.controls.float("r"),
        g: m.controls.float("g"),
        b: m.controls.float("b"),
        offset: m.controls.float("offset"),
        ring_strength: m.controls.float("ring_strength"),
        angular_variation: m.controls.float("angular_variation"),
        threshold: m.controls.float("threshold"),
        mix: m.controls.float("mix"),
    };

    m.gpu.update_params(app, &params);
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    m.gpu.render(&frame);
}
