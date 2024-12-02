use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

use crate::framework::animation::Animation;
use crate::framework::metadata::SketchMetadata;
use crate::framework::util::{set_window_position, uuid_5};

pub const METADATA: SketchMetadata = SketchMetadata {
    name: "egui_poc",
    display_name: "EGUI POC",
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
    _main_window_id: window::Id,
    _gui_window_id: window::Id,
    egui: Egui,
    animation: Animation,
    object_configs: Vec<ObjectConfig>,
    settings: Settings,
}

pub fn model(app: &App) -> Model {
    // let primary_monitor = app.primary_monitor().unwrap();
    // let screen_size = primary_monitor.size();
    // let screen_width = screen_size.width;
    // let w: i32 = (screen_width / 4) as i32;
    // let h: i32 = (screen_width / 4) as i32;
    let w: i32 = 700;
    let h: i32 = 700;

    let main_window_id = app
        .new_window()
        .title(METADATA.display_name)
        .size(w as u32, h as u32)
        .build()
        .unwrap();
    set_window_position(app, main_window_id, 0, 0);

    let gui_window_id = app
        .new_window()
        .title(METADATA.display_name.to_owned() + " Controls")
        .size(w as u32 / 2, h as u32 / 2)
        .view(view_gui)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    set_window_position(app, gui_window_id, w * 2, 0);

    let egui = Egui::from_window(&app.window(gui_window_id).unwrap());

    let animation = Animation::new(METADATA.bpm);

    let object_configs = vec![
        ObjectConfig {
            object: Object {
                position: vec2(-w as f32 / 4.0, 0.0),
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
                position: vec2(w as f32 / 4.0, 0.0),
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
        _main_window_id: main_window_id,
        _gui_window_id: unsafe { WindowId::dummy() },
        egui,
        animation,
        object_configs,
        settings: Settings { radius: 50.0 },
    }
}

pub fn update(app: &App, model: &mut Model, update: Update) {
    for config in &mut model.object_configs {
        config.update(&model.animation, &model.settings)
    }

    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::CentralPanel::default()
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(3, 3, 3))
                .inner_margin(egui::Margin::same(16.0)),
        )
        .show(&ctx, |ui| {
            let mut style = (*ctx.style()).clone();
            style.visuals.override_text_color =
                Some(egui::Color32::from_rgb(200, 200, 200));
            ctx.set_style(style);

            if ui.button("Capture Frame").clicked() {
                if let Some(window) = app.window(model._main_window_id) {
                    let filename =
                        format!("{}-{}.png", METADATA.name, uuid_5());
                    let file_path = app
                        .project_path()
                        .unwrap()
                        .join("images")
                        .join(filename);
                    window.capture_frame(file_path);
                }
            }

            ui.label("Radius:");
            ui.add(egui::Slider::new(&mut model.settings.radius, 1.0..=500.0));
        });
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

pub fn view_gui(_app: &App, model: &Model, frame: Frame) {
    model.egui.draw_to_frame(&frame).unwrap();
}

fn raw_window_event(
    _app: &App,
    model: &mut Model,
    event: &nannou::winit::event::WindowEvent,
) {
    model.egui.handle_raw_event(event);
}
