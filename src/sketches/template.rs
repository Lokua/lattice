use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "template",
    display_name: "Template",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(150),
};

#[derive(SketchComponents)]
pub struct Model {
    animation: Animation,
    controls: Controls,
    window_rect: WindowRect,
    radius: f32,
    hue: f32,
}

pub fn init_model(_app: &App, window_rect: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::new(vec![Control::slider(
        "radius",
        100.0,
        (10.0, 500.0),
        1.0,
    )]);

    Model {
        animation,
        controls,
        window_rect,
        radius: 0.0,
        hue: 0.0,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let radius_max = model.controls.float("radius");

    model.radius = model.animation.lerp(
        vec![
            KF::new(20.0, 2.0),
            KF::new(radius_max, 1.0),
            KF::new(radius_max / 2.0, 0.5),
            KF::new(radius_max, 0.5),
            KF::new(20.0, KF::END),
        ],
        0.0,
    );

    model.hue = model.animation.ping_pong_loop_progress(12.0)
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(model.window_rect.w(), model.window_rect.h())
        .hsla(0.0, 0.0, 0.02, 0.1);

    draw.ellipse()
        .color(hsl(model.hue, 0.5, 0.5))
        .radius(model.radius)
        .x_y(0.0, 0.0);

    draw.to_frame(app, &frame).unwrap();
}
