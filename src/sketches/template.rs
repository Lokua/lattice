use nannou::color::named::*;
use nannou::color::*;
use nannou::prelude::*;

use crate::framework::{
    animation::Animation,
    controls::{Control, Controls},
    sketch::{SketchConfig, SketchModel},
    util::IntoLinSrgb,
};

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "template",
    display_name: "Template",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: None,
};

struct Object {
    position: Vec2,
    radius: f32,
    color: LinSrgb,
}
impl Default for Object {
    fn default() -> Self {
        Self {
            position: vec2(0.0, 0.0),
            radius: 50.0,
            color: MAGENTA.into_lin_srgb(),
        }
    }
}

type AnimationFn<R> = Option<Box<dyn Fn(&Object, &Animation, &Controls) -> R>>;

struct ObjectConfig {
    object: Object,
    position_fn: AnimationFn<Vec2>,
    radius_fn: AnimationFn<f32>,
}
impl ObjectConfig {
    pub fn update(&mut self, animation: &Animation, controls: &Controls) {
        if let Some(radius_fn) = &self.radius_fn {
            self.object.radius = radius_fn(&self.object, animation, controls);
        }
        if let Some(position_fn) = &self.position_fn {
            self.object.position =
                position_fn(&self.object, animation, controls);
        }
    }
}
impl Default for ObjectConfig {
    fn default() -> Self {
        Self {
            object: Default::default(),
            position_fn: None,
            radius_fn: None,
        }
    }
}

pub struct Model {
    animation: Animation,
    object_configs: Vec<ObjectConfig>,
    controls: Controls,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

pub fn init_model() -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::new(vec![Control::Slider {
        name: "radius".to_string(),
        value: 150.0,
        min: 10.0,
        max: 500.0,
        step: 1.0,
    }]);

    let mut object_configs = vec![
        ObjectConfig {
            object: Object {
                radius: 50.0,
                ..Default::default()
            },
            radius_fn: Some(Box::new(|_object, ah, controls| {
                ah.get_ping_pong_loop_progress(2.0)
                    * controls.get_float("radius")
            })),
            ..Default::default()
        },
        ObjectConfig {
            object: Object {
                position: vec2(0.0, 0.0),
                radius: 50.0,
                color: CYAN.into_lin_srgb(),
            },
            radius_fn: Some(Box::new(|_object, ah, controls| {
                ah.get_ping_pong_loop_progress(3.0)
                    * controls.get_float("radius")
            })),
            ..Default::default()
        },
    ];

    let count = 10;
    let angle_step = (PI * 2.0) / count as f32;
    for i in 0..10 {
        object_configs.push(ObjectConfig {
            object: Object {
                color: BEIGE.into_lin_srgb(),
                radius: 10.0,
                ..Default::default()
            },
            position_fn: Some(Box::new(move |_object, ah, controls| {
                let global_angle = ah.get_loop_progress(8.0) * (PI * 2.0);
                let angle = i as f32 * angle_step + global_angle;
                let movement_radius = controls.get_float("radius") * 2.0;
                vec2(
                    movement_radius * angle.cos(),
                    movement_radius * angle.sin(),
                )
            })),
            ..Default::default()
        })
    }

    Model {
        animation,
        object_configs,
        controls,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    for config in &mut model.object_configs {
        config.update(&model.animation, &model.controls)
    }
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    frame.clear(BLACK);
    draw.background().color(rgb(0.1, 0.1, 0.1));

    for config in &model.object_configs {
        draw.ellipse()
            .color(config.object.color)
            .radius(config.object.radius)
            .xy(config.object.position);
    }

    draw.to_frame(app, &frame).unwrap();
}
