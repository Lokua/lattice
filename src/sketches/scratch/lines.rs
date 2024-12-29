use nannou::prelude::*;

use crate::framework::prelude::*;

// https://www.generativehut.com/post/how-to-make-generative-art-feel-natural

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "lines",
    display_name: "Lines",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(230),
    play_mode: PlayMode::Loop,
};

const N_LINES: i32 = 4;
const STROKE_WEIGHT: f32 = 4.0;
const SPACING: f32 = 32.0;

#[derive(SketchComponents)]
pub struct Model {
    controls: Controls,
    window_rect: WindowRect,
    slant_points: Vec<(Vec2, Vec2)>,
    jerky_points: Vec<Vec<Vec2>>,
    chaikin_points: Vec<Vec<Vec2>>,
    kernel_points: Vec<Vec<Vec2>>,
    pad: f32,
}

pub fn init_model(_app: &App, wr: WindowRect) -> Model {
    let controls = Controls::new(vec![
        Control::slider("deviation", 5.0, (1.0, 10.0), 0.1),
        Control::slider("n_points", 16.0, (3.0, 64.0), 1.0),
        Control::slider("chaikin_passes", 4.0, (1.0, 16.0), 1.0),
        Control::slider("kernel_passes", 2.0, (1.0, 16.0), 1.0),
    ]);

    let pad = wr.w_(20.0);

    Model {
        controls,
        window_rect: wr,
        slant_points: vec![],
        jerky_points: vec![],
        chaikin_points: vec![],
        kernel_points: vec![],
        pad,
    }
}

pub fn update(_app: &App, m: &mut Model, _update: Update) {
    if m.controls.changed() {
        let deviation = m.controls.float("deviation");
        let n_points = m.controls.float("n_points") as usize;
        let chaikin_passes = m.controls.float("chaikin_passes") as usize;
        let kernel_passes = m.controls.float("kernel_passes") as usize;
        let wr = &m.window_rect;
        let params = &LineParams {
            pad: m.pad,
            deviation,
            n_points,
            chaikin_passes,
            kernel_passes,
        };
        m.slant_points = generate_slant_points(wr, params);
        m.jerky_points = generate_jerky_points(wr, params);
        m.chaikin_points = generate_points_using_chaikin_smoothing(wr, params);
        m.kernel_points = generate_points_using_kernel_smoothing(wr, params);

        m.controls.mark_unchanged();
    }
}

pub fn view(app: &App, m: &Model, frame: Frame) {
    let draw = app.draw();
    let wr = &m.window_rect;

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(wr.w(), wr.h())
        .hsla(0.0, 0.0, 1.0, 1.0);

    let start_x = -wr.hw() + m.pad;
    let end_x = wr.hw() - m.pad;
    let n_demos = 5;

    for demo in 0..n_demos {
        let y = wr.top() - (wr.h() / n_demos as f32) * (demo as f32 + 0.5);
        let draw_shited = draw.translate(vec3(0.0, y, 0.0));

        match demo {
            0 => {
                draw_shited
                    .line()
                    .start(vec2(start_x, 0.0))
                    .end(vec2(end_x, 0.0))
                    .color(BLACK)
                    .stroke_weight(STROKE_WEIGHT);
            }
            1 => {
                for (start, end) in m.slant_points.iter() {
                    draw_shited
                        .line()
                        .start(*start)
                        .end(*end)
                        .color(BLACK)
                        .stroke_weight(STROKE_WEIGHT);
                }
            }
            2 => {
                for line in m.jerky_points.iter() {
                    draw_shited
                        .polyline()
                        .weight(STROKE_WEIGHT)
                        .points(line.iter().cloned())
                        .color(BLACK);
                }
            }
            3 => {
                for line in m.chaikin_points.iter() {
                    draw_shited
                        .polyline()
                        .weight(STROKE_WEIGHT)
                        .points(line.iter().cloned())
                        .color(BLACK);
                }
            }
            4 => {
                for line in m.kernel_points.iter() {
                    draw_shited
                        .polyline()
                        .weight(STROKE_WEIGHT)
                        .points(line.iter().cloned())
                        .color(BLACK);
                }
            }
            _ => unreachable!(),
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

struct LineParams {
    pad: f32,
    deviation: f32,
    n_points: usize,
    chaikin_passes: usize,
    kernel_passes: usize,
}

fn generate_slant_points(
    wr: &WindowRect,
    params: &LineParams,
) -> Vec<(Vec2, Vec2)> {
    let start_x = -wr.hw() + params.pad;
    let end_x = wr.hw() - params.pad;
    let mut points = vec![];

    for i in 0..N_LINES {
        let base_y = i as f32 * wr.h_(SPACING);
        let offset_start_y = random_normal(params.deviation);
        let offset_end_y = random_normal(params.deviation);
        let start = vec2(start_x, base_y + offset_start_y);
        let end = vec2(end_x, base_y + offset_end_y);
        points.push((start, end));
    }

    points
}

fn generate_jerky_points(
    wr: &WindowRect,
    params: &LineParams,
) -> Vec<Vec<Vec2>> {
    let start_x = -wr.hw() + params.pad;
    let end_x = wr.hw() - params.pad;
    let mut lines = vec![];
    let segment_length = (end_x - start_x) / params.n_points as f32;

    for i in 0..N_LINES {
        let mut points = vec![];
        let base_y = i as f32 * wr.h_(SPACING);

        for j in 0..=params.n_points {
            let x = start_x + (j as f32 * segment_length);
            let offset_start_y = random_normal(params.deviation);
            let y = base_y + offset_start_y;
            points.push(pt2(x, y));
        }

        lines.push(points);
    }

    lines
}

fn generate_points_using_chaikin_smoothing(
    wr: &WindowRect,
    params: &LineParams,
) -> Vec<Vec<Vec2>> {
    let start_x = -wr.hw() + params.pad;
    let end_x = wr.hw() - params.pad;
    let mut lines = vec![];
    let segment_length = (end_x - start_x) / params.n_points as f32;

    for i in 0..N_LINES {
        let mut points = vec![];
        let base_y = i as f32 * wr.h_(SPACING);

        for j in 0..=params.n_points {
            let x = start_x + (j as f32 * segment_length);
            let offset_start_y = random_normal(params.deviation);
            let y = base_y + offset_start_y;
            points.push(pt2(x, y));
        }

        let smoothed = chaikin(points, params.chaikin_passes, false);
        lines.push(smoothed);
    }

    lines
}

fn generate_points_using_kernel_smoothing(
    wr: &WindowRect,
    params: &LineParams,
) -> Vec<Vec<Vec2>> {
    let start_x = -wr.hw() + params.pad;
    let end_x = wr.hw() - params.pad;
    let mut lines = vec![];
    let segment_length = (end_x - start_x) / params.n_points as f32;

    for i in 0..N_LINES {
        let mut points = vec![];
        let base_y = i as f32 * wr.h_(SPACING);

        for j in 0..=params.n_points {
            let x = start_x + (j as f32 * segment_length);
            let offset_start_y = random_normal(params.deviation);
            let y = base_y + offset_start_y;
            points.push(pt2(x, y));
        }

        points = average_neighbors(points, params.kernel_passes);

        lines.push(points);
    }

    lines
}
