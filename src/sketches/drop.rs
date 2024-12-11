use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "drop",
    display_name: "Drop",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(250),
};

const MAX_DROPS: usize = 2500;

pub struct Model {
    animation: Animation,
    controls: Controls,
    max_drops: usize,
    drops: Vec<(Drop, Hsl)>,
    droppers: Vec<Dropper>,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

type DropFn = fn(Vec2, &mut Vec<(Drop, Hsl)>, usize);

struct Dropper {
    trigger: Trigger,
    zone: DropZone,
    drop_fn: DropFn,
}

impl Dropper {
    pub fn new(trigger: Trigger, zone: DropZone, drop_fn: DropFn) -> Self {
        Self {
            trigger,
            zone,
            drop_fn,
        }
    }
}

pub fn init_model() -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::new(vec![]);

    let rect: Rect<f32> = Rect::from_x_y_w_h(
        0.0,
        0.0,
        SKETCH_CONFIG.w as f32 / 4.0,
        SKETCH_CONFIG.h as f32 / 4.0,
    );

    let duration = 0.5;
    let delay = duration / 9.0;

    let droppers = vec![
        Dropper::new(
            animation.create_trigger(duration, 0.0),
            DropZone::new(vec2(0.0, 0.0)),
            drop_it,
        ),
        // Top
        Dropper::new(
            animation.create_trigger(duration, delay),
            DropZone::new(vec2(0.0, rect.top())),
            drop_it,
        ),
        Dropper::new(
            animation.create_trigger(duration, delay * 2.0),
            DropZone::new(rect.top_right()),
            drop_it,
        ),
        // Right
        Dropper::new(
            animation.create_trigger(duration, delay * 3.0),
            DropZone::new(vec2(rect.top(), 0.0)),
            drop_it,
        ),
        Dropper::new(
            animation.create_trigger(duration, delay * 4.0),
            DropZone::new(rect.bottom_right()),
            drop_it,
        ),
        // Bottom
        Dropper::new(
            animation.create_trigger(duration, delay * 5.0),
            DropZone::new(vec2(0.0, rect.bottom())),
            drop_it,
        ),
        Dropper::new(
            animation.create_trigger(duration, delay * 6.0),
            DropZone::new(rect.bottom_left()),
            drop_it,
        ),
        // Left
        Dropper::new(
            animation.create_trigger(duration, delay * 7.0),
            DropZone::new(vec2(rect.bottom(), 0.0)),
            drop_it,
        ),
        Dropper::new(
            animation.create_trigger(duration, delay * 8.0),
            DropZone::new(rect.top_left()),
            drop_it,
        ),
    ];

    Model {
        animation,
        controls,
        max_drops: MAX_DROPS,
        drops: Vec::new(),
        droppers,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let offset = model.animation.animate(
        vec![KF::new(1.0, 4.0), KF::new(2.0, 4.0), KF::new(3.0, 4.0)],
        0.0,
    );
    model.droppers.iter_mut().for_each(|dropper| {
        if model.animation.should_trigger(&mut dropper.trigger) {
            for _ in 0..3 {
                (dropper.drop_fn)(
                    dropper.zone.center * offset,
                    &mut model.drops,
                    model.max_drops,
                );
            }
        }
    });
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let _window_rect = app
        .window(frame.window_id())
        .expect("Unable to get window")
        .rect();

    let draw = app.draw();

    draw.background().color(hsl(0.0, 0.0, 1.0));

    for (drop, color) in model.drops.iter() {
        draw.polygon()
            .color(*color)
            .points(drop.vertices().iter().cloned());
    }

    draw.to_frame(app, &frame).unwrap();
}

fn drop_it(point: Vec2, drops: &mut Vec<(Drop, Hsl)>, max_drops: usize) {
    let point = nearby_point(point, 50.0);
    let drop = Drop::new(point, random_range(2.0, 20.0), 64);
    for (other, _color) in drops.iter_mut() {
        drop.marble(other);
    }
    let lightness = if random_f32() < 0.5 { 0.0 } else { 1.0 };
    drops.push((drop, hsl(0.0, 0.0, lightness)));
    while drops.len() > max_drops {
        drops.remove(0);
    }
}

fn nearby_point(base_point: Vec2, radius: f32) -> Vec2 {
    let angle = random_range(0.0, TWO_PI);
    let distance = random_range(0.0, radius);
    return Vec2::new(
        base_point.x + distance * angle.cos(),
        base_point.y + distance * angle.sin(),
    );
}
