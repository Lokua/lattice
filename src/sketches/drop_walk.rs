use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "drop_walk",
    display_name: "Drop Walk",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(250),
};

const MAX_DROPS: usize = 5000;

#[derive(SketchComponents)]
pub struct Model {
    animation: Animation,
    controls: Controls,
    max_drops: usize,
    drops: Vec<(Drop, Hsl)>,
    droppers: Vec<Dropper>,
}

type DropFn = fn(&Dropper, Vec2, &mut Vec<(Drop, Hsl)>, usize, Hsl, f32);
type ColorFn = fn(&Controls) -> Hsl;

struct Dropper {
    kind: String,
    trigger: Trigger,
    drop_fn: DropFn,
    min_radius: f32,
    max_radius: f32,
    color_fn: ColorFn,
    walker: Walker,
}

impl Dropper {
    pub fn new(
        kind: String,
        trigger: Trigger,
        drop_fn: DropFn,
        min_radius: f32,
        max_radius: f32,
        color_fn: ColorFn,
        walker: Walker,
    ) -> Self {
        Self {
            kind,
            trigger,
            drop_fn,
            min_radius,
            max_radius,
            color_fn,
            walker,
        }
    }
}

struct Walker {
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    step_size: f32,
}

impl Walker {
    pub fn step(&mut self) {
        match random_range(0.0, 4.0).floor() as u32 {
            0 => {
                self.x += self.step_size;
            }
            1 => {
                self.x -= self.step_size;
            }
            2 => {
                self.y += self.step_size;
            }
            _ => {
                self.y -= self.step_size;
            }
        }

        self.constrain();
    }

    fn constrain(&mut self) {
        let half_w = self.w / 2.0;
        let half_h = self.h / 2.0;

        if self.x < -half_w {
            self.x += (self.step_size * 2.0).min(half_w);
        }
        if self.x > half_w {
            self.x -= (self.step_size * 2.0).min(half_w);
        }
        if self.y < -half_h {
            self.y += (self.step_size * 2.0).min(half_h);
        }
        if self.y > half_h {
            self.y -= (self.step_size * 2.0).min(half_h);
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

pub fn init_model(_app: &App, _window_rect: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);
    let controls = Controls::new(vec![
        Control::Checkbox {
            name: "debug_walker".to_string(),
            value: false,
            disabled: None,
        },
        Control::Slider {
            name: "step_size".to_string(),
            value: 2.0,
            min: 1.0,
            max: 200.0,
            step: 1.0,
            disabled: None,
        },
        Control::Slider {
            name: "splatter_radius".to_string(),
            value: 2.0,
            min: 1.0,
            max: 50.0,
            step: 1.0,
            disabled: None,
        },
        Control::Slider {
            name: "drop_max_radius".to_string(),
            value: 20.0,
            min: 1.0,
            max: 50.0,
            step: 1.0,
            disabled: None,
        },
        Control::Slider {
            name: "color_ratio".to_string(),
            value: 0.5,
            min: 0.0,
            max: 1.0,
            step: 0.001,
            disabled: None,
        },
    ]);

    let droppers = vec![
        Dropper::new(
            "center".to_string(),
            animation.create_trigger(0.25, 0.0),
            drop_it,
            1.0,
            controls.float("drop_max_radius"),
            color_1,
            Walker {
                w: SKETCH_CONFIG.w as f32,
                h: SKETCH_CONFIG.h as f32,
                x: 0.0,
                y: 0.0,
                step_size: controls.float("step_size"),
            },
        ),
        Dropper::new(
            "center".to_string(),
            animation.create_trigger(0.25, 0.0),
            drop_it,
            1.0,
            controls.float("drop_max_radius"),
            color_2,
            Walker {
                w: SKETCH_CONFIG.w as f32,
                h: SKETCH_CONFIG.h as f32,
                x: 0.0,
                y: 0.0,
                step_size: controls.float("step_size"),
            },
        ),
        Dropper::new(
            "center".to_string(),
            animation.create_trigger(0.25, 0.0),
            drop_it,
            1.0,
            controls.float("drop_max_radius"),
            color_3,
            Walker {
                w: SKETCH_CONFIG.w as f32,
                h: SKETCH_CONFIG.h as f32,
                x: 0.0,
                y: 0.0,
                step_size: controls.float("step_size"),
            },
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
    model.droppers.iter_mut().for_each(|dropper| {
        if model.animation.should_trigger(&mut dropper.trigger) {
            dropper.walker.step_size = model.controls.float("step_size");
            dropper.walker.step();

            match dropper.kind.as_str() {
                "center" => {
                    dropper.min_radius = 0.1;
                    dropper.max_radius =
                        model.controls.float("drop_max_radius");
                }
                _ => {}
            }

            for _ in 0..6 {
                (dropper.drop_fn)(
                    dropper,
                    dropper.walker.to_vec2(),
                    &mut model.drops,
                    model.max_drops,
                    (dropper.color_fn)(&model.controls),
                    model.controls.float("splatter_radius"),
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

    if model.controls.bool("debug_walker") {
        for dropper in &model.droppers {
            draw.ellipse()
                .color(BLACK)
                .xy(dropper.walker.to_vec2())
                .radius(10.0);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn drop_it(
    dropper: &Dropper,
    point: Vec2,
    drops: &mut Vec<(Drop, Hsl)>,
    max_drops: usize,
    color: Hsl,
    splatter_radius: f32,
) {
    let point = nearby_point(point, splatter_radius);
    let drop = Drop::new(
        point,
        random_range(dropper.min_radius, dropper.max_radius),
        64,
    );

    for (other, _color) in drops.iter_mut() {
        drop.marble(other);
    }

    drops.push((drop, color));

    while drops.len() > max_drops {
        drops.remove(0);
    }
}

fn color_1(controls: &Controls) -> Hsl {
    if random_f32() > controls.float("color_ratio") {
        hsl(random_range(0.38, 0.47), 1.0, 0.5)
    } else {
        hsl(0.0, 0.0, 1.0)
    }
}

fn color_2(controls: &Controls) -> Hsl {
    if random_f32() > controls.float("color_ratio") {
        hsl(random_range(0.8, 0.95), 1.0, 0.5)
    } else {
        hsl(0.0, 0.0, 1.0)
    }
}

fn color_3(controls: &Controls) -> Hsl {
    if random_f32() > controls.float("color_ratio") {
        hsl(random_range(0.6, 0.65), 1.0, 0.5)
    } else {
        hsl(0.0, 0.0, 1.0)
    }
}
