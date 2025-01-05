use gpu::GpuState;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "genuary_2",
    display_name: "Genuary 2: Layers Upon Layers",
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
    wr: WindowRect,
    gpu: gpu::GpuState,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    // w, h ..unused
    resolution: [f32; 4],

    // constrast, smooth_mix, time, time_2
    a: [f32; 4],

    // t1, t2, t3, post_mix
    b: [f32; 4],

    // r1, r2, r3, unused
    c: [f32; 4],

    // g1, g2, g3, unused
    d: [f32; 4],
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::with_previous(vec![
        Control::slider_norm("smooth_mix", 0.5),
        Control::slider("contrast", 1.5, (0.1, 5.0), 0.1),
        Control::Separator {},
        Control::slider("g1", 2.0, (2.0, 32.0), 1.0),
        Control::slider("g2", 4.0, (2.0, 32.0), 1.0),
        Control::slider("g3", 8.0, (2.0, 32.0), 1.0),
        Control::Separator {},
        Control::slider_norm("post_mix", 0.5),
    ]);

    let params = ShaderParams {
        resolution: [0.0, 0.0, 0.0, 0.0],
        a: [0.0, 0.0, 0.0, 0.0],
        b: [0.0, 0.0, 0.0, 0.0],
        c: [0.0, 0.0, 0.0, 0.0],
        d: [0.0, 0.0, 0.0, 0.0],
    };

    let shader = wgpu::include_wgsl!("./genuary_2.wgsl");
    let gpu = GpuState::new(app, shader, &params);

    Model {
        animation,
        controls,
        wr,
        gpu,
    }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    let time = 4.0;
    let kfs = [kfr((0.0, 1.0), time)];

    let params = ShaderParams {
        resolution: [m.wr.w(), m.wr.h(), 0.0, 0.0],
        a: [
            m.controls.float("contrast"),
            m.controls.float("smooth_mix"),
            m.animation.ping_pong(8.0),
            m.animation.ping_pong(12.0),
        ],
        b: [
            m.animation.r_ramp(&kfs, 0.0, time * 0.5, ease_in_out),
            m.animation.r_ramp(&kfs, 0.5, time * 0.5, ease_in_out),
            m.animation.r_ramp(&kfs, 1.0, time * 0.5, ease_in_out),
            m.controls.float("post_mix"),
        ],
        c: [
            m.animation.r_ramp(&kfs, 0.0, time * 0.5, ease_in_out),
            m.animation.r_ramp(&kfs, 0.5, time * 0.5, ease_in_out),
            m.animation.r_ramp(&kfs, 1.0, time * 0.5, ease_in_out),
            0.0,
        ],
        d: [
            m.controls.float("g1"),
            m.controls.float("g2"),
            m.controls.float("g3"),
            0.0,
        ],
    };

    m.gpu.update_params(app, &params);
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(BLACK);
    m.gpu.render(&frame);
}
