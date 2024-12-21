use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

// https://github.com/inconvergent/sand-spline/blob/master/main-hlines.py

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "lines_texture",
    display_name: "Lines Texture",
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
    points: Vec<Vec2>,
}

pub fn init_model(_app: &App, wr: WindowRect) -> Model {
    let controls = Controls::new(vec![]);

    Model {
        controls,
        wr,
        points: vec![],
    }
}

pub fn update(_app: &App, m: &mut Model, _update: Update) {
    if m.controls.changed() {
        let pad = m.wr.w_(32.0);
        let start = vec2(-m.wr.hw() + pad, 0.0);
        let end = vec2(m.wr.hw() - pad, 0.0);
        let length = end.x - start.x;
        let n_segments = 16.0;

        let mut reference_points: Vec<Vec2> = (0..=n_segments as u32)
            .map(|i| {
                let t = i as f32 / n_segments as f32;
                let x = start.x + length * t;
                if i == 0 || i == n_segments as u32 {
                    vec2(x, 0.0)
                } else {
                    vec2(x, random_normal(2.0))
                }
            })
            .collect();

        reference_points = average_neighbors(reference_points, 0);

        let step = 0.1;
        let mut noise: Vec<f32> =
            (0..reference_points.len()).map(|_i| 0.0).collect();

        let do_pass = |noise: &mut Vec<f32>| {
            for n in noise.iter_mut() {
                *n += random_normal(1.0) * step * 8.0;
            }

            let mut output_points: Vec<Vec2> = vec![];

            for (index, point) in reference_points.iter().enumerate() {
                if index < reference_points.len() - 1 {
                    let next_point = reference_points[index + 1];
                    let points_per_segment = 64;

                    for _ in 0..points_per_segment {
                        let t = random::<f32>();
                        let base_point = *point * (1.0 - t) + next_point * t;

                        let base_angle = PI / 2.0;
                        let angle = base_angle + random_normal(0.5);
                        let noise_amount =
                            noise[index] * (1.0 - t) + noise[index + 1] * t;
                        let offset = vec2(
                            noise_amount * angle.cos(),
                            noise_amount * angle.sin(),
                        );

                        output_points.push(base_point + offset);
                    }
                }
            }

            output_points
        };

        m.points = vec![];
        for _ in 0..50 {
            let points = do_pass(&mut noise);
            m.points.extend(points);
        }

        m.controls.mark_unchanged();
    }
}

pub fn view(app: &App, m: &Model, frame: Frame) {
    let draw = app.draw();

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(m.wr.w(), m.wr.h())
        .hsla(0.0, 0.0, 1.0, 1.0);

    for point in &m.points {
        draw.rect()
            .xy(*point)
            .w_h(1.0, 1.0)
            .color(hsla(0.0, 0.0, 0.0, 0.1));
    }

    draw.to_frame(app, &frame).unwrap();
}
