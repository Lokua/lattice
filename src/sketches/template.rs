use nannou::prelude::*;

use crate::framework::animation::Animation;
use crate::framework::metadata::SketchMetadata;

pub const METADATA: SketchMetadata = SketchMetadata {
    name: "template",
    display_name: "Template",
    fps: 60.0,
    bpm: 134.0,
};

struct Object {
    position: Vec2,
    radius: f32,
    color: Rgb,
}
impl Default for Object {
    fn default() -> Self {
        Self {
            position: vec2(0.0, 0.0),
            radius: 50.0,
            color: rgb(1.0, 0.0, 0.0),
        }
    }
}

type AnimationFn<R> = Option<Box<dyn Fn(&Object, &Animation, &Settings) -> R>>;

struct ObjectConfig {
    object: Object,
    position_fn: AnimationFn<Vec2>,
    radius_fn: AnimationFn<f32>,
}
impl ObjectConfig {
    pub fn update(&mut self, animation: &Animation, settings: &Settings) {
        if let Some(radius_fn) = &self.radius_fn {
            self.object.radius = radius_fn(&self.object, animation, settings);
        }
        if let Some(position_fn) = &self.position_fn {
            self.object.position =
                position_fn(&self.object, animation, settings);
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

pub struct Settings {
    radius: f32,
}

pub struct Model {
    animation: Animation,
    object_configs: Vec<ObjectConfig>,
    settings: Settings,
}

pub fn init_model() -> Model {
    let animation = Animation::new(METADATA.bpm);

    let object_configs = vec![
        ObjectConfig {
            object: Object {
                position: vec2(-350.0 / 4.0, 0.0),
                radius: 50.0,
                ..Default::default()
            },
            radius_fn: Some(Box::new(|_object, animation, settings| {
                animation.get_ping_pong_loop_progress(2.0) * settings.radius
            })),
            ..Default::default()
        },
        ObjectConfig {
            object: Object {
                position: vec2(350.0 / 4.0, 0.0),
                radius: 50.0,
                color: rgb(0.0, 0.0, 1.0),
            },
            radius_fn: Some(Box::new(|_object, animation, settings| {
                animation.get_ping_pong_loop_progress(3.0) * settings.radius
            })),
            ..Default::default()
        },
    ];

    Model {
        animation,
        object_configs,
        settings: Settings { radius: 50.0 },
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    for config in &mut model.object_configs {
        config.update(&model.animation, &model.settings)
    }
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    frame.clear(BLACK);
    draw.background().color(rgb(0.1, 0.1, 0.1));

    for config in &model.object_configs {
        draw.ellipse()
            .radius(config.object.radius)
            .xy(config.object.position)
            .color(config.object.color);
    }

    draw.to_frame(app, &frame).unwrap();
}
