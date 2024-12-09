use nannou::color::*;
use nannou::noise::NoiseFn;
use nannou::noise::Perlin;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "untitled",
    display_name: "Untitled",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(200),
};

pub struct Model {
    animation: Animation,
    controls: Controls,
    perlin: Perlin,
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

    let controls = Controls::new(vec![
        Control::Slider {
            name: "point_size".to_string(),
            value: 2.0,
            min: 0.5,
            max: 20.0,
            step: 0.5,
        },
        Control::Slider {
            name: "count".to_string(),
            value: 100.0,
            min: 1.0,
            max: 1_000.0,
            step: 10.0,
        },
        Control::Slider {
            name: "offset".to_string(),
            value: 50.0,
            min: 1.0,
            max: 300.0,
            step: 1.0,
        },
    ]);

    Model {
        animation,
        controls,
        perlin: Perlin::new(),
        radius: 100.0,
        hue: 0.0,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    model.hue = model.animation.ping_pong_loop_progress(12.0)
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let window_rect = app
        .window(frame.window_id())
        .expect("Unable to get window")
        .rect();

    let draw = app.draw();
    let count = model.controls.float("count");
    let point_size = model.controls.float("point_size");
    let offset = model.controls.float("offset");

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(window_rect.w(), window_rect.h())
        .hsla(0.0, 0.0, 0.00, 0.3);

    let center = Vec2::new(0.0, 0.0);
    let zone = DropZone::new(center);
    let rect = Rect::from_xy_wh(center, center + model.radius);
    let inner_radius = rect.len();
    let outer_radius = rect.pad(-10.0).len();

    for _i in 0..count as i32 {
        draw.ellipse()
            .color(hsl(model.hue, 0.62, 0.62))
            .xy(zone.point_within_circular_zone(
                inner_radius - 20.0,
                outer_radius - 20.0,
            ))
            .radius(point_size);
    }

    let configs: Vec<(&DropZone, fn(&DropZone, f32, f32) -> Vec2, Vec2)> = vec![
        (
            &zone,
            DropZone::point_within_rectangular_zone_top,
            Vec2::new(0.0, offset),
        ),
        (
            &zone,
            DropZone::point_within_rectangular_zone_bottom,
            Vec2::new(0.0, -offset),
        ),
        (
            &zone,
            DropZone::point_within_rectangular_zone_right,
            Vec2::new(offset, 0.0),
        ),
        (
            &zone,
            DropZone::point_within_rectangular_zone_left,
            Vec2::new(-offset, 0.0),
        ),
    ];

    for (zone_instance, method, offs) in configs {
        for _i in 0..count as i32 {
            let mut zoned = method(zone_instance, inner_radius, outer_radius);
            let t = app.time * 0.0001;

            let noise_x = model.perlin.get([
                (zoned.x + offs.x * 0.001) as f64,
                (zoned.y + offs.y * 0.001) as f64,
                t as f64,
            ]) as f32
                * 10.0;

            let noise_y = model.perlin.get([
                (zoned.y + offs.y * 0.001) as f64,
                (zoned.x + offs.x * 0.001) as f64,
                t as f64 + 0.2,
            ]) as f32
                * 10.0;

            zoned.x = random_range(zoned.x, zoned.x + offs.x + 0.001) + noise_x;
            zoned.y = random_range(zoned.y, zoned.y + offs.y + 0.001) + noise_y;

            let normalized_distance = map_range(
                zoned.length(),
                inner_radius,
                outer_radius + offs.length(),
                1.0,
                0.0,
            );
            draw.ellipse()
                .color(hsl(1.0 - model.hue, 0.62, normalized_distance))
                .xy(zoned)
                .radius(point_size);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
