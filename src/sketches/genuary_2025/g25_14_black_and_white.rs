use bytemuck::{Pod, Zeroable};
use gpu::GpuState;
use nannou::prelude::*;

use crate::framework::prelude::*;

// ~/Documents/Live/2025/2025.01.15 - 2020.01.28 F7 - Lattice Auto

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "g25_14_black_and_white",
    display_name: "Genuary 14: Interference",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 127.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(580),
};

#[derive(SketchComponents)]
pub struct Template {
    controls: ControlHub<OscTransportTiming>,
    gpu: gpu::GpuState<gpu::BasicPositionVertex>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ShaderParams {
    // w, h, ..unused
    resolution: [f32; 4],

    // wave1_freq, wave1_angle, wave2_freq, wave2_angle
    a: [f32; 4],

    // wave1_phase, wave2_phase, wave1_y, wave2_y
    b: [f32; 4],

    // unused, type_mix, unused, checker
    c: [f32; 4],

    // curve_x, curve_y, wave_distort, smoothing
    d: [f32; 4],

    // wave1_amp, wave2_amp, ..unused
    e: [f32; 4],
}

pub fn init(app: &App, ctx: &LatticeContext) -> Template {
    let controls = ControlHub::from_path(
        to_absolute_path(file!(), "./g25_14_black_and_white.yaml"),
        OscTransportTiming::new(ctx.bpm()),
    );

    let params = ShaderParams {
        resolution: [0.0; 4],
        a: [0.0; 4],
        b: [0.0; 4],
        c: [0.0; 4],
        d: [0.0; 4],
        e: [0.0; 4],
    };

    let gpu = GpuState::new_fullscreen(
        app,
        ctx.window_rect().resolution_u32(),
        to_absolute_path(file!(), "./g25_14_black_and_white.wgsl"),
        &params,
        true,
    );

    Template { controls, gpu }
}

impl Sketch for Template {
    fn update(&mut self, app: &App, _update: Update, ctx: &LatticeContext) {
        self.controls.update();

        let wr = ctx.window_rect();
        let phase_mod = self.controls.get("phase_mod");

        let params = ShaderParams {
            resolution: [wr.w(), wr.h(), 0.0, 0.0],
            a: [
                self.controls.get("wave1_freq"),
                0.0, // wave1_angle
                self.controls.get("wave2_freq"),
                0.25, // wave2_angle
            ],
            b: [
                self.controls.animation.r_ramp(
                    &[kfr((0.0, phase_mod), 2.0)],
                    0.0,
                    1.0,
                    Easing::Linear,
                ),
                self.controls.animation.r_ramp(
                    &[kfr((0.0, phase_mod), 2.0)],
                    1.0,
                    1.0,
                    Easing::Linear,
                ),
                self.controls.get("wave1_y"),
                self.controls.get("wave2_y"),
            ],
            c: [
                0.0,
                self.controls.get("type_mix"),
                0.0,
                self.controls.get("checker"),
            ],
            d: [
                self.controls.get("curve_x"),
                self.controls.get("curve_y"),
                self.controls.get("wave_distort"),
                0.0, // smoothing
            ],
            e: [
                self.controls.get("wave1_amp"),
                self.controls.get("wave2_amp"),
                0.0,
                0.0,
            ],
        };

        self.gpu.update_params(app, wr.resolution_u32(), &params);
    }

    fn view(&self, _app: &App, frame: Frame, _ctx: &LatticeContext) {
        frame.clear(BLACK);
        self.gpu.render(&frame);
    }
}
