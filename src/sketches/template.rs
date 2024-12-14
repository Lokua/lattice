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

pub struct Model {
    animation: Animation,
    controls: Controls,
    radius: f32,
    hue: f32,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

pub fn init_model() -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::new(vec![Control::Slider {
        name: "radius".to_string(),
        value: 100.0,
        min: 10.0,
        max: 500.0,
        step: 1.0,
        disabled: None,
    }]);

    let radius = controls.float("radius");

    Model {
        animation,
        controls,
        radius,
        hue: 0.0,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let radius_max = model.controls.float("radius");

    model.radius = model.animation.animate(
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
    let window_rect = app
        .window(frame.window_id())
        .expect("Unable to get window")
        .rect();

    let draw = app.draw();

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(window_rect.w(), window_rect.h())
        .hsla(0.0, 0.0, 0.02, 0.1);

    draw.ellipse()
        .color(hsl(model.hue, 0.5, 0.5))
        .radius(model.radius)
        .x_y(0.0, 0.0);

    draw.to_frame(app, &frame).unwrap();
}
