use nannou::prelude::*;
use nannou_osc as osc;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "genuary_14",
    display_name: "Genuary 14: Interference",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(580),
};

#[derive(SketchComponents)]
pub struct Model {
    #[allow(dead_code)]
    animation: Animation,
    controls: Controls,
    wr: WindowRect,
    gpu: gpu::GpuState,
    midi: MidiControls,
    osc: osc::Receiver,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    // w, h, ..unused
    resolution: [f32; 4],

    // wave1_frequency, wave1_angle, wave2_frequency, wave2_angle
    a: [f32; 4],

    // wave1_phase, wave2_phase, wave1_y_influence, wave2_y_influence
    b: [f32; 4],

    // unused, type_mix, unused, checkerboard
    c: [f32; 4],

    // curve_freq_x, curve_freq_y, wave_distort, smoothing
    d: [f32; 4],

    // wave1_amp, wave2_amp, ..unused
    e: [f32; 4],
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::with_previous(vec![
        // Control::checkbox("animate_wave1_phase", false),
        // Control::slider("wave1_amp", 1.0, (0.0, 2.0), 0.001),
        // Control::slider_norm("wave1_frequency", 0.02),
        // Control::slider("wave1_angle", 0.0, (0.0, 1.0), 0.125),
        // Control::slider_x(
        //     "wave1_phase",
        //     0.0,
        //     (0.0, 1.0),
        //     0.0001,
        //     |controls: &Controls| controls.bool("animate_wave1_phase"),
        // ),
        // Control::slider_norm("wave1_y_influence", 0.5),
        // Control::Separator {}, // ------------------------------------------
        // Control::checkbox("animate_wave2_phase", false),
        // Control::slider("wave2_amp", 1.0, (0.0, 2.0), 0.001),
        // Control::slider_norm("wave2_frequency", 0.02),
        // Control::slider("wave2_angle", 0.0, (0.0, 1.0), 0.125),
        // Control::slider_x(
        //     "wave2_phase",
        //     0.0,
        //     (0.0, 1.0),
        //     0.0001,
        //     |controls: &Controls| controls.bool("animate_wave2_phase"),
        // ),
        // Control::slider_norm("wave2_y_influence", 0.5),
        // Control::Separator {}, // ------------------------------------------
        // Control::checkbox("checkerboard", false),
        // Control::slider_norm("type_mix", 0.0),
        // Control::slider("curve_freq_x", 0.3, (0.0, 2.0), 0.001),
        // Control::slider("curve_freq_y", 0.3, (0.0, 2.0), 0.001),
        // Control::slider_norm("wave_distort", 0.4),
        // Control::slider_norm("smoothing", 0.5),
    ]);

    let params = ShaderParams {
        resolution: [0.0; 4],
        a: [0.0; 4],
        b: [0.0; 4],
        c: [0.0; 4],
        d: [0.0; 4],
        e: [0.0; 4],
    };

    let shader = wgpu::include_wgsl!("./genuary_14.wgsl");
    let gpu = gpu::GpuState::new(app, shader, &params);

    let midi = MidiControlBuilder::new()
        .control_mapped("wave1_amp", (0, 1), (0.0, 2.0), 0.0)
        .control("wave1_frequency", (0, 2), 0.0)
        .control("wave1_y_influence", (0, 3), 0.0)
        .control_mapped("wave2_amp", (0, 4), (0.0, 2.0), 0.0)
        .control("wave2_frequency", (0, 5), 0.0)
        .control("wave2_y_influence", (0, 6), 0.0)
        .control("checkerboard", (0, 7), 0.0)
        .control("type_mix", (0, 8), 0.0)
        .control_mapped("curve_freq_x", (0, 9), (0.0, 2.0), 0.3)
        .control_mapped("curve_freq_y", (0, 10), (0.0, 2.0), 0.3)
        .control("wave_distort", (0, 11), 0.0)
        .control("phase_mod", (0, 12), 0.0)
        .build();

    let osc = osc::Receiver::bind_to("127.0.0.1:4000")
        .expect("Failed to bind to port");

    Model {
        animation,
        controls,
        wr,
        gpu,
        midi,
        osc,
    }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    let phase_mod = m.midi.get("phase_mod");

    // FIX: this resets to zero because it's not on the model
    let mut curve_x = 0.0;
    let packets = m.osc.try_iter().collect::<Vec<_>>();
    for (packet, _addr) in packets {
        match packet {
            osc::Packet::Message(msg) => match msg.addr.as_str() {
                "/curve_freq_x" => {
                    if let Some(value) = msg.args.get(0) {
                        if let osc::Type::Float(v) = value {
                            curve_x = *v;
                            debug!("value: {}", v);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    let params = ShaderParams {
        resolution: [m.wr.w(), m.wr.h(), 0.0, 0.0],
        a: [
            m.midi.get("wave1_frequency"),
            // wave1_angle
            0.0,
            m.midi.get("wave2_frequency"),
            // wave2_angle
            0.25,
        ],
        b: [
            m.animation.r_rmp(&[((0.0, phase_mod), 2.0)], 0.0, 1.0),
            m.animation.r_rmp(&[((0.0, phase_mod), 2.0)], 1.0, 1.0),
            m.midi.get("wave1_y_influence"),
            m.midi.get("wave2_y_influence"),
        ],
        c: [0.0, m.midi.get("type_mix"), 0.0, m.midi.get("checkerboard")],
        d: [
            // m.midi.get("curve_freq_x"),
            curve_x,
            m.midi.get("curve_freq_y"),
            m.midi.get("wave_distort"),
            // smoothing
            0.0,
        ],
        e: [m.midi.get("wave1_amp"), m.midi.get("wave2_amp"), 0.0, 0.0],
    };

    m.gpu.update_params(app, &params);
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(BLACK);
    m.gpu.render(&frame);
}
