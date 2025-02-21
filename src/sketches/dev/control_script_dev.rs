use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

// Live/2025/Lattice - ControlScript Test

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "control_script_dev",
    display_name: "ControlScript Test",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(300),
};

#[derive(SketchComponents)]
pub struct Model {
    controls: ControlScript<Timing>,
    wr: WindowRect,
}

pub fn init_model(_app: &App, wr: WindowRect) -> Model {
    let controls = ControlScript::from_path(
        to_absolute_path(file!(), "control_script_dev.yaml"),
        Timing::new(SKETCH_CONFIG.bpm),
    );

    Model { controls, wr }
}

pub fn update(_app: &App, m: &mut Model, _update: Update) {
    debug_throttled!(500, "test_anim: {}", m.controls.get("test_anim"));
    m.controls.update();
}

pub fn view(app: &App, m: &Model, frame: Frame) {
    let draw = app.draw();

    // background
    draw.rect().x_y(0.0, 0.0).w_h(m.wr.w(), m.wr.h()).hsla(
        0.0,
        0.0,
        0.02,
        m.controls.get("bg_alpha"),
    );

    let hue = m.controls.get("hue");
    let radius = m.controls.get("radius");
    let pos_x = m.controls.get("pos_x");
    let pos_y = m.controls.get("pos_y");
    let radius_small = m.controls.get("radius_small");
    let pos_x2 = m.controls.get("pos_x2");
    let pos_x3 = m.controls.get("pos_x3");
    let rect_y = m.controls.get("rect_y");
    let line = m.controls.get("line");
    let red_ball_radius = m.controls.get("red_ball_radius");

    draw.ellipse()
        .color(hsl(hue, 0.5, 0.5))
        .radius(radius)
        .x_y(0.0, 0.0);

    draw.ellipse()
        .color(WHITE)
        .radius(radius_small)
        .x_y(pos_x * m.wr.hw(), pos_y * m.wr.hh());

    draw.ellipse()
        .color(RED)
        .radius(red_ball_radius)
        .x_y(pos_x2 * m.wr.hw(), -m.wr.h() / 4.0);

    draw.ellipse()
        .color(BLUE)
        .radius(20.0)
        .x_y(pos_x3 * m.wr.hw(), -m.wr.h() / 4.0);

    draw.rect()
        .color(CYAN)
        .x_y(0.0, map_range(rect_y, 0.0, 1.0, -m.wr.hh(), m.wr.hh()))
        .w_h(m.wr.w() - 100.0, 30.0);

    draw.rect()
        .color(MAGENTA)
        .x_y(0.0, map_range(line, 0.0, 1.0, -m.wr.hh(), m.wr.hh()))
        .w_h(m.wr.w(), 20.0);

    draw.to_frame(app, &frame).unwrap();
}
