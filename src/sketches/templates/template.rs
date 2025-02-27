use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "template",
    display_name: "Template",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(150),
};

pub struct Template {
    animation: Animation<Timing>,
    controls: Controls,
    wr: WindowRect,
    radius: f32,
    hue: f32,
}

pub fn init(_app: &App, wr: WindowRect) -> Template {
    let animation = Animation::new(Timing::new(SKETCH_CONFIG.bpm));

    let controls = Controls::new(vec![Control::slider(
        "radius",
        100.0,
        (10.0, 500.0),
        1.0,
    )]);

    Template {
        animation,
        controls,
        wr,
        radius: 0.0,
        hue: 0.0,
    }
}

impl Sketch for Template {
    fn update(&mut self, _app: &App, _update: Update) {
        let radius_max = self.controls.float("radius");

        self.radius = self.animation.lerp(
            &[
                kf(20.0, 2.0),
                kf(radius_max, 1.0),
                kf(radius_max / 2.0, 0.5),
                kf(radius_max, 0.5),
                kf(20.0, 0.0),
            ],
            0.0,
        );

        self.hue = self.animation.tri(12.0)
    }

    fn view(&self, app: &App, frame: Frame) {
        let draw = app.draw();

        draw.rect()
            .x_y(0.0, 0.0)
            .w_h(self.wr.w(), self.wr.h())
            .hsla(0.0, 0.0, 0.02, 0.1);

        draw.ellipse()
            .color(hsl(self.hue, 0.5, 0.5))
            .radius(self.radius)
            .x_y(0.0, 0.0);

        draw.to_frame(app, &frame).unwrap();
    }
}
