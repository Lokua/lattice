use nannou::prelude::*;
use nannou::winit::window::Window as WinitWindow;

use crate::framework::animation::Animation;
use crate::framework::metadata::SketchMetadata;

pub const METADATA: SketchMetadata = SketchMetadata {
    name: "template",
    display_name: "Template",
    fps: 30.0,
    bpm: 134.0,
};

struct Object {
    position: Vec2,
    radius: f32,
}

struct ObjectConfig {
    object: Object,
    position_fn: Option<Box<dyn Fn(&Animation) -> Vec2>>,
    radius_fn: Option<Box<dyn Fn(&Animation) -> f32>>,
}

impl ObjectConfig {
    pub fn new(
        object: Object,
        position_fn: Option<Box<dyn Fn(&Animation) -> Vec2>>,
        radius_fn: Option<Box<dyn Fn(&Animation) -> f32>>,
    ) -> Self {
        Self {
            object,
            position_fn,
            radius_fn,
        }
    }
    pub fn update(&mut self, animation: &Animation) {
        if let Some(radius_fn) = &self.radius_fn {
            self.object.radius = radius_fn(animation);
        }
        if let Some(position_fn) = &self.position_fn {
            self.object.position = position_fn(animation);
        }
    }
}

pub struct Model {
    _window: window::Id,
    animation: Animation,
    object_configs: Vec<ObjectConfig>,
}

pub fn model(app: &App) -> Model {
    let w: i32 = 700;
    let h: i32 = 700;

    let _window = app
        .new_window()
        .title(METADATA.display_name)
        .size(w as u32, h as u32)
        .build()
        .unwrap();

    let window = app.window(_window).unwrap();
    let winit_window: &WinitWindow = window.winit_window();
    winit_window
        .set_outer_position(nannou::winit::dpi::PhysicalPosition::new(0, 0));

    let animation = Animation::new(METADATA.bpm);

    let object_configs = vec![
        ObjectConfig::new(
            Object {
                position: vec2(-w as f32 / 4.0, 0.0),
                radius: 50.0,
            },
            None,
            Some(Box::new(|animation| {
                (animation.get_ping_pong_loop_progress(2.0) * 10.0) + 10.0
            })),
        ),
        ObjectConfig::new(
            Object {
                position: vec2(w as f32 / 4.0, 0.0),
                radius: 50.0,
            },
            None,
            Some(Box::new(|animation| {
                (animation.get_ping_pong_loop_progress(3.0) * 10.0) * 10.0
            })),
        ),
    ];

    Model {
        _window,
        animation,
        object_configs,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    for config in &mut model.object_configs {
        config.update(&model.animation)
    }
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(rgb(0.1, 0.1, 0.1));
    frame.clear(BLACK);

    for config in &model.object_configs {
        draw.ellipse()
            .radius(config.object.radius)
            .xy(config.object.position)
            .color(RED);
    }

    draw.to_frame(app, &frame).unwrap();
}
