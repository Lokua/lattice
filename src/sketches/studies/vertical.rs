use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "vertical",
    display_name: "Vertical",
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
    lines: Vec<Vec<Point2>>,
    patterns: Vec<XPosFn>,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

type XPosFn = fn(f32, f32, f32, f32, f32, f32) -> f32;

pub fn init_model() -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::new(vec![
        Control::slider("scale", 1.0, (0.1, 4.0), 0.1),
        Control::slider("n_lines", 64.0, (16.0, 256.0), 2.0),
        Control::slider("amplitude", 20.0, (0.0, 300.0), 1.0),
        Control::slider("frequency", 0.1, (0.0, 0.1), 0.00001),
    ]);

    let lines = Vec::with_capacity(controls.float("n_lines") as usize);

    Model {
        animation,
        controls,
        lines,
        patterns: vec![
            per_line,      // v
            serrated,      // u
            ripples,       // v
            zipper,        // u
            pockets,       // v
            morse,         // u
            line_phase,    // v
            double_helix,  // u
            pattern_sine,  // v
            pinch,         // u
            pattern_helix, // v
            breathing,     // u
        ],
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let w = SKETCH_CONFIG.w as f32;
    let h = SKETCH_CONFIG.h as f32;
    let n_lines = model.controls.float("n_lines") as usize;
    let a = model.controls.float("amplitude");
    let f = model.controls.float("frequency");

    model.lines = Vec::new();

    let step = w / n_lines as f32;
    let start_x = -(w / 2.0) + (step / 2.0);

    for i in 0..n_lines {
        let x = start_x + i as f32 * step;
        let mut points = Vec::new();

        for j in 0..n_lines {
            let y = map_range(j, 0, n_lines - 1, -h / 2.0, h / 2.0);

            let values = model
                .patterns
                .iter()
                .map(|func| func(x, y, i as f32, a, f, n_lines as f32))
                .collect::<Vec<f32>>();

            let x = multi_lerp(&values, model.animation.ping_pong(36.0));

            points.push(pt2(x, y));
        }

        model.lines.push(points);
    }
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
        .hsla(0.0, 0.0, 0.02, 0.7);

    let zoomed_draw = draw.scale(model.controls.float("scale"));

    for (index, line) in model.lines.iter().enumerate() {
        zoomed_draw
            .polyline()
            .weight(2.0)
            .points(line.iter().cloned())
            .color(hsla(
                0.44,
                index as f32 / model.lines.len() as f32,
                0.45,
                0.3,
            ));
    }

    draw.to_frame(app, &frame).unwrap();
}

// Varied from line to line (v)
// ----------------------------

fn per_line(x: f32, y: f32, i: f32, a: f32, f: f32, _n: f32) -> f32 {
    let line_f = f * (1.0 + i * 0.1);
    x + a * (y * line_f).sin()
}
fn ripples(x: f32, y: f32, i: f32, a: f32, f: f32, n: f32) -> f32 {
    let distance_from_center = (i as f32 - n as f32 / 2.0).abs();
    let line_f = f * (1.0 + distance_from_center * 0.05);
    x + a * (y * line_f).sin()
}
fn pockets(x: f32, y: f32, i: f32, a: f32, f: f32, _n: f32) -> f32 {
    let group = (i as f32 * 0.2).sin();
    x + a * (y * f).sin() * group
}
fn line_phase(x: f32, y: f32, i: f32, a: f32, f: f32, _n: f32) -> f32 {
    let line_phase = i as f32 * 0.1;
    x + a * (y * f + line_phase).sin() * (y * f * 2.0 + line_phase).cos()
}
fn pattern_sine(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    x + a * (y * f).sin()
}
fn pattern_helix(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    x + a * (y * f).sin() * (1.0 + (y * f * 0.5).cos())
}

// Unified aross all lines (u)
// ---------------------------
fn serrated(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    x + a * ((y * f).sin() * (y * f * 5.0).cos())
}
fn zipper(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    x + a * (y * f).sin() * ((y * f * 3.0).cos()).abs()
}
fn morse(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    let fast = (y * f * 10.0).sin().abs();
    let slow = (y * f).sin();
    x + a * fast * slow
}
fn double_helix(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    x + a * (y * f).sin() * (1.0 + (y * f * 0.5).cos())
}
fn pinch(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    let pinch = 1.0 / (1.0 + (y * f).cos().abs());
    x + a * pinch * (y * f).sin()
}
fn breathing(x: f32, y: f32, _i: f32, a: f32, f: f32, _n: f32) -> f32 {
    x + a * (y * f).sin() * (y * 0.01).cos()
}
