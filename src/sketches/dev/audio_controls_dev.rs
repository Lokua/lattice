use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "audio_controls_dev",
    display_name: "Audio Controls Test",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(300),
    play_mode: PlayMode::Loop,
};

#[derive(SketchComponents)]
pub struct Model {
    controls: Controls,
    audio: AudioControls,
    wr: WindowRect,
}

pub fn init_model(_app: &App, wr: WindowRect) -> Model {
    let controls = Controls::new(vec![
        Control::slider_norm("preemphasis", 0.0),
        Control::slider_norm("detect", 0.0),
        Control::slider_norm("rise", 0.0),
        Control::slider_norm("fall", 0.0),
    ]);

    let audio = AudioControlBuilder::new()
        .control_from_config(
            "bd",
            AudioControlConfig {
                channel: 0,
                slew_config: SlewConfig::default(),
                preemphasis: 0.0,
                detect: 0.0,
                range: (0.0, 700.0),
                default: 0.0,
            },
        )
        .control_from_config(
            "hh",
            AudioControlConfig {
                channel: 1,
                slew_config: SlewConfig::default(),
                preemphasis: 0.0,
                detect: 0.0,
                range: (0.0, 700.0),
                default: 0.0,
            },
        )
        .control_from_config(
            "chord",
            AudioControlConfig {
                channel: 2,
                slew_config: SlewConfig::default(),
                preemphasis: 0.0,
                detect: 0.0,
                range: (0.0, 700.0),
                default: 0.0,
            },
        )
        .build();

    Model {
        audio,
        controls,
        wr,
    }
}

pub fn update(_app: &App, m: &mut Model, _update: Update) {
    // debug_throttled!(500, "a: {}, b: {}", m.audio.get("bd"), m.audio.get("hh"));

    if m.controls.changed() {
        let preemphasis = m.controls.float("preemphasis");
        let detect = m.controls.float("detect");
        let rise = m.controls.float("rise");
        let fall = m.controls.float("fall");

        m.audio.update_controls(|control| {
            control.preemphasis = preemphasis;
            control.detect = detect;
            control.slew_config.rise = rise;
            control.slew_config.fall = fall;
        });

        m.controls.mark_unchanged();
    }
}

pub fn view(app: &App, m: &Model, frame: Frame) {
    let draw = app.draw();

    draw.rect()
        .color(WHITE)
        .x_y(0.0, 0.0)
        .w_h(m.wr.w(), m.wr.h());

    let bd = m.audio.get("bd");
    let hh = m.audio.get("hh");
    let chord = m.audio.get("chord");

    draw.ellipse()
        .color(rgba(0.02, 0.02, 0.02, 0.9))
        .radius(bd)
        .x_y(-m.wr.w() / 4.0, 0.0);

    draw.ellipse()
        .color(rgba(0.5, 0.5, 0.5, 0.9))
        .radius(hh)
        .x_y(m.wr.w() / 4.0, 0.0);

    draw.ellipse()
        .color(rgba(0.8, 0.8, 0.8, 0.9))
        .radius(chord)
        .x_y(0.0, 0.0);

    draw.to_frame(app, &frame).unwrap();
}
