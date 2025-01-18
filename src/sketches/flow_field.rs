use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "flow_field",
    display_name: "Flow Field",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(220),
};

#[derive(SketchComponents)]
#[sketch(clear_color = "hsl(0.0, 0.0, 0.03, 0.5)")]
pub struct Model {
    #[allow(dead_code)]
    animation: Animation<FrameTiming>,
    controls: Controls,
    wr: WindowRect,
    points: Vec<Vec2>,
}

pub fn init_model(_app: &App, wr: WindowRect) -> Model {
    let animation = Animation::new(FrameTiming::new(SKETCH_CONFIG.bpm));

    let controls = Controls::with_previous(vec![
        Control::slider("particle_count", 1_000.0, (10.0, 10_000.0), 1.0),
        Control::slider("noise_scale", 0.000_01, (0.000_01, 0.002), 0.000_01),
        Control::slider("velocity", 0.01, (0.01, 3.0), 0.01),
        Control::slider_norm("bg_alpha", 0.02),
    ]);

    Model {
        animation,
        controls,
        wr,
        points: vec![],
    }
}

pub fn update(_app: &App, m: &mut Model, _update: Update) {
    if m.controls.any_changed_in(&["particle_count"]) {
        let particle_count = m.controls.float("particle_count") as usize;

        // TODO: add if short, remove if over
        m.points.clear();

        for _ in 0..particle_count {
            m.points.push(random_point(&m.wr));
        }

        m.controls.mark_unchanged();
    }

    let ns = m.controls.float("noise_scale");
    let vel = m.controls.float("velocity");

    m.points = m
        .points
        .iter()
        .map(|&p| {
            let mut p = p;
            let n = (p.x * ns).sin() + (p.y * ns).cos();
            let a = TWO_PI * n;
            p.x += a.cos() * vel;
            p.y += a.sin() * vel;

            if p.x < -m.wr.hw() {
                p.x += m.wr.w();
            } else if p.x > m.wr.hw() {
                p.x -= m.wr.w();
            }

            // if p.y < -m.wr.hh() {
            //     p.y += m.wr.h();
            // } else if p.y > m.wr.hh() {
            //     p.y -= m.wr.h();
            // }

            if !on_screen(p, &m.wr) {
                p = random_point(&m.wr);
            }
            p
        })
        .collect();
}

pub fn view(app: &App, m: &Model, frame: Frame) {
    let draw = app.draw();

    draw.rect().wh(m.wr.vec2()).color(hsla(
        0.0,
        0.0,
        0.03,
        m.controls.float("bg_alpha"),
    ));

    m.points.iter().for_each(|p| {
        draw.ellipse().radius(1.0).xy(*p).color(MAGENTA);
    });

    draw.to_frame(app, &frame).unwrap();
}

fn random_point(wr: &WindowRect) -> Vec2 {
    vec2(
        random_range(-wr.hw(), wr.hw()),
        random_range(-wr.hh(), wr.hh()),
    )
}
