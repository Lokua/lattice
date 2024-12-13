use nannou::color::*;
use nannou::noise::NoiseFn;
use nannou::noise::Perlin;
use nannou::prelude::*;

use crate::framework::prelude::*;

// https://www.youtube.com/watch?v=0YvPgYDR1oM&list=PLeCiJGCSl7jc5UWvIeyQAvmCNc47IuwkM&index=6
// https://github.com/Lokua/p5/blob/main/src/sketches/perlinNoiseLoop.mjs

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "perlin_loop",
    display_name: "Perlin Loop",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(200),
};

pub struct Model {
    #[allow(dead_code)]
    animation: Animation,
    controls: Controls,
    perlin: Perlin,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

pub fn init_model() -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);
    let controls = Controls::new(vec![
        Control::slider("radius", 200.0, (1.0, 500.0), 1.0),
        Control::slider("height", 64.0, (1.0, 50.0), 0.1),
        Control::slider("thinness", 1.5, (0.5, 5.0), 0.25),
    ]);

    Model {
        animation,
        controls,
        perlin: Perlin::new(),
    }
}

pub fn update(_app: &App, _model: &mut Model, _update: Update) {}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let _window_rect = app
        .window(frame.window_id())
        .expect("Unable to get window")
        .rect();

    let draw = app.draw();
    draw.background().hsl(0.0, 0.0, 0.03);

    let space = PI / 45.0;
    let radius = model.controls.float("radius");
    let height = model.controls.float("height");
    let thinness = model.controls.float("thinness");

    let steps = (360.0 / space) as i32;
    for i in 0..steps {
        let angle = deg_to_rad(i as f32);
        let x_off = map_range(angle.cos(), -1.0, 1.0, 0.0, 3.0);
        let y_off = map_range(angle.sin(), -1.0, 1.0, 0.0, 3.0);
        let n = model.perlin.get([x_off, y_off]);
        let h = map_range(n, 0.0, 0.1, 0.0, height);
        draw.rect()
            .color(BEIGE)
            .x_y(radius * angle.cos(), radius * angle.sin())
            .w_h(h, thinness)
            .rotate(angle);
    }

    draw.to_frame(app, &frame).unwrap();
}
