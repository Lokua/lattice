use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "wave_fract",
    display_name: "Wave Fract",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(500),
};

#[derive(SketchComponents)]
pub struct Model {
    #[allow(dead_code)]
    animation: Animation<MidiSongTiming>,
    controls: Controls,
    wr: WindowRect,
    gpu: gpu::GpuState,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    // w, h, ..unused
    resolution: [f32; 4],

    // wave_phase, wave_radial_freq, wave_horiz_freq, wave_vert_freq
    a: [f32; 4],

    // fract_count, fract_scale, fract_color_scale, wave_power
    b: [f32; 4],

    // reduce_mix, map_mix, wave_bands, wave_threshold
    c: [f32; 4],

    // fract_contrast, fract_steps, ..unused
    d: [f32; 4],
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let animation = Animation::new(MidiSongTiming::new(SKETCH_CONFIG.bpm));

    let controls = Controls::with_previous(vec![
        Control::checkbox("animate_wave_phase", false),
        Control::slider_x(
            "wave_phase",
            0.0,
            (0.0, TAU),
            0.001,
            |controls: &Controls| controls.bool("animate_wave_phase"),
        ),
        Control::slider_norm("wave_radial_freq", 0.5),
        Control::checkbox("link_axes", false),
        Control::slider_norm("wave_horiz_freq", 0.5),
        Control::slider_x(
            "wave_vert_freq",
            0.5,
            (0.0, 1.0),
            0.001,
            |controls: &Controls| controls.bool("link_axes"),
        ),
        Control::slider_norm("wave_power", 0.5),
        Control::slider("wave_bands", 0.0, (2.0, 10.0), 1.0),
        Control::slider("wave_threshold", 0.0, (-1.0, 1.0), 0.001),
        Control::Separator {}, // -------------------
        Control::slider_norm("fract_count", 0.5),
        Control::slider_norm("fract_scale", 0.5),
        Control::slider_norm("fract_color_scale", 0.5),
        Control::slider("fract_contrast", 0.5, (0.0, 5.0), 0.001),
        Control::slider("fract_steps", 0.5, (0.0, 15.0), 0.01),
        Control::Separator {}, // -------------------
        Control::slider_norm("reduce_mix", 0.5),
        Control::slider_norm("map_mix", 0.5),
    ]);

    let params = ShaderParams {
        resolution: [0.0; 4],
        a: [0.0; 4],
        b: [0.0; 4],
        c: [0.0; 4],
        d: [0.0; 4],
    };

    let shader = wgpu::include_wgsl!("./wave_fract.wgsl");
    let gpu = gpu::GpuState::new(app, shader, &params);

    Model {
        animation,
        controls,
        wr,
        gpu,
    }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    let params = ShaderParams {
        resolution: [m.wr.w(), m.wr.h(), 0.0, 0.0],
        a: [
            if m.controls.bool("animate_wave_phase") {
                m.animation.loop_progress(4.0)
            } else {
                m.controls.float("wave_phase")
            },
            m.controls.float("wave_radial_freq"),
            m.controls.float("wave_horiz_freq"),
            if m.controls.bool("link_axes") {
                m.controls.float("wave_horiz_freq")
            } else {
                m.controls.float("wave_vert_freq")
            },
        ],
        b: [
            m.controls.float("fract_count"),
            m.controls.float("fract_scale"),
            m.controls.float("fract_color_scale"),
            m.controls.float("wave_power"),
        ],
        c: [
            m.controls.float("reduce_mix"),
            m.controls.float("map_mix"),
            m.controls.float("wave_bands"),
            m.controls.float("wave_threshold"),
        ],
        d: [
            m.controls.float("fract_contrast"),
            m.controls.float("fract_steps"),
            0.0,
            0.0,
        ],
    };

    m.gpu.update_params(app, &params);
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(BLACK);
    m.gpu.render(&frame);
}
