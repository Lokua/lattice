use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "osc_test",
    display_name: "OSC Test",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(150),
    play_mode: PlayMode::Loop,
};

#[derive(SketchComponents)]
pub struct Model {
    osc: OscControls,
}

pub fn init_model(_app: &App, _wr: WindowRect) -> Model {
    let osc = OscControlBuilder::new()
        .control("/a", 0.0)
        .control("/b", 0.0)
        .build();

    Model { osc }
}

pub fn update(_app: &App, m: &mut Model, _update: Update) {
    debug!("/a: {}, /b: {}", m.osc.get("/a"), m.osc.get("/b"));
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let window_rect = app
        .window(frame.window_id())
        .expect("Unable to get window")
        .rect();

    let draw = app.draw();

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(window_rect.w(), window_rect.h())
        .hsla(0.0, 0.0, 0.02, 0.1);

    let b = model.osc.get("/b");
    draw.ellipse()
        .no_fill()
        .stroke(hsl(map_range(b, 0.0, 1.0, 0.7, 1.0), 0.5, 1.0 - (b * 0.5)))
        .stroke_weight(5.0)
        .radius(map_range(sigmoid(b, 12.0), 0.0, 1.0, 1.0, 400.0))
        .x_y(0.0, 0.0);

    let a = model.osc.get("/a");
    draw.ellipse()
        .no_fill()
        .stroke(hsl(0.5, 0.3, a * 0.5))
        .stroke_weight(10.0)
        .radius(map_range(sigmoid(a, 12.0), 0.0, 1.0, 1.0, 200.0))
        .x_y(0.0, 0.0);

    draw.to_frame(app, &frame).unwrap();
}
