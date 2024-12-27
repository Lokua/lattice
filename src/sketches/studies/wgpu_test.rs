use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "wgpu_test",
    display_name: "WGPU Test",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(150),
};

#[derive(SketchComponents)]
pub struct Model {
    controls: Controls,
    wr: WindowRect,
    line: Vec<Vec2>,
}

pub fn init_model(_app: &App, wr: WindowRect) -> Model {
    let controls = Controls::with_previous(vec![
        Control::slider("segments", 8.0, (2.0, 64.0), 1.0),
        Control::slider("deviation", 1.0, (1.0, 100.0), 1.0),
    ]);

    Model {
        controls,
        wr,
        line: vec![],
    }
}

pub fn update(_app: &App, m: &mut Model, _update: Update) {
    if m.controls.any_changed_in(&["segments", "deviation"]) {
        let segments = m.controls.float("segments");
        let deviation = m.controls.float("deviation");
        let pad = 24.0;

        m.line = reference_line(
            vec2(-m.wr.hw() + pad, 0.0),
            vec2(m.wr.hw() - pad, 0.0),
            segments as usize,
            deviation,
            2,
        );

        m.controls.mark_unchanged();
    }
}

pub fn view(app: &App, m: &Model, frame: Frame) {
    let draw = app.draw();

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(m.wr.w(), m.wr.h())
        .hsla(0.0, 0.0, 1.0, 1.0);

    draw.polyline()
        .weight(2.0)
        .points(m.line.iter().cloned())
        .color(hsla(0.0, 0.0, 0.0, 1.0));

    draw.to_frame(app, &frame).unwrap();
}

fn reference_line(
    start: Vec2,
    end: Vec2,
    segments: usize,
    deviation: f32,
    smoothing_passes: usize,
) -> Vec<Vec2> {
    let length = end.x - start.x;

    let reference_points: Vec<Vec2> = (0..=segments as u32)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let x = start.x + length * t;
            if i == 0 || i == segments as u32 {
                vec2(x, 0.0)
            } else {
                vec2(x, random_normal(deviation))
            }
        })
        .collect();

    average_neighbors(reference_points, smoothing_passes)
}
