use nannou::color::{Gradient, Mix};
use nannou::prelude::*;
use nannou::winit::window::Window as WinitWindow;
use nannou_egui::{self, egui, Egui};

use crate::framework::animation::Animation;
use crate::framework::displacer::Displacer;
use crate::framework::metadata::SketchMetadata;
use crate::framework::util::create_grid;

pub const METADATA: SketchMetadata = SketchMetadata {
    name: "displacement_1",
    display_name: "Displacement v1",
    fps: 30.0,
    bpm: 134.0,
};

struct Settings {
    strength: f32,
    max_radius: f32,
}

struct DisplacerConfig {
    displacer: Displacer,
    position_animation: Option<Vec2>,
    radius_animation: Option<(f32, f32, f32)>,
}

impl DisplacerConfig {
    pub fn new(
        displacer: Displacer,
        position_animation: Option<Vec2>,
        radius_animation: Option<(f32, f32, f32)>,
    ) -> Self {
        Self {
            displacer,
            position_animation,
            radius_animation,
        }
    }
}

pub struct Model {
    _window: window::Id,
    egui: Egui,
    settings: Settings,
    circle_radius: f32,
    grid_size: usize,
    displacer_configs: [DisplacerConfig; 6],
    animation: Animation,
}

pub fn model(app: &App) -> Model {
    let w: i32 = 700;
    let h: i32 = 700;

    let _window = app
        .new_window()
        .title(METADATA.display_name)
        .size(w as u32, h as u32)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(_window).unwrap();
    let winit_window: &WinitWindow = window.winit_window();
    winit_window
        .set_outer_position(nannou::winit::dpi::PhysicalPosition::new(0, 0));

    let egui = Egui::from_window(&window);

    let animation = Animation::new(METADATA.bpm);

    let pad = 40.0;
    let corner_radius_animation = Some((25.0, 50.0, 1.0));
    let corner_strength = 25.0;

    let displacer_configs = [
        DisplacerConfig::new(
            Displacer::new(vec2(0.0, 0.0), 10.0, 50.0, None),
            None,
            Some((50.0, 300.0, 0.5)),
        ),
        DisplacerConfig::new(
            Displacer::new(vec2(200.0, 200.0), 150.0, 50.0, None),
            Some(vec2(200.0, 200.0)),
            Some((25.0, 250.0, 1.5)),
        ),
        DisplacerConfig::new(
            Displacer::new(
                vec2((w as f32 / 2.0) - pad, (h as f32 / 2.0) - pad),
                10.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_animation,
        ),
        DisplacerConfig::new(
            Displacer::new(
                vec2((-w as f32 / 2.0) + pad, (h as f32 / 2.0) - pad),
                10.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_animation,
        ),
        DisplacerConfig::new(
            Displacer::new(
                vec2((-w as f32 / 2.0) + pad, (-h as f32 / 2.0) + pad),
                10.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_animation,
        ),
        DisplacerConfig::new(
            Displacer::new(
                vec2((w as f32 / 2.0) - pad, (-h as f32 / 2.0) + pad),
                10.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_animation,
        ),
    ];

    Model {
        _window,
        egui,
        settings: Settings {
            strength: 50.0,
            max_radius: 200.0,
        },
        circle_radius: 2.0,
        grid_size: 64,
        displacer_configs,
        animation,
    }
}

pub fn update(app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("Strength:");
        ui.add(egui::Slider::new(&mut settings.strength, 1.0..=200.0));
        ui.label("Max Radius:");
        ui.add(egui::Slider::new(&mut settings.max_radius, 1.0..=500.0));
    });

    for config in &mut model.displacer_configs {
        config.displacer.set_strength(settings.strength);

        if let Some((min, _max, duration)) = config.radius_animation {
            let value = model.animation.get_ping_pong_loop_progress(duration);
            let radius = map_range(value, 0.0, 1.0, min, settings.max_radius);
            config.displacer.set_radius(radius);
        }

        if let Some(v) = config.position_animation {
            config
                .displacer
                .set_position(vec2(v.x * app.time.cos(), v.y * app.time.sin()));
        }
    }
}

fn raw_window_event(
    _app: &App,
    model: &mut Model,
    event: &nannou::winit::event::WindowEvent,
) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let window = app.window_rect();
    let grid =
        create_grid(window.w(), window.h(), model.grid_size, |x, y| vec2(x, y));
    let gradient = Gradient::new(vec![
        LinSrgb::new(
            BEIGE.red as f32 / 255.0,
            BEIGE.green as f32 / 255.0,
            BEIGE.blue as f32 / 255.0,
        ),
        LinSrgb::new(
            PURPLE.red as f32 / 255.0,
            PURPLE.green as f32 / 255.0,
            PURPLE.blue as f32 / 255.0,
        ),
    ]);

    draw.background().color(rgb(0.1, 0.1, 0.1));
    frame.clear(BLACK);

    for point in grid {
        let mut total_displacement = vec2(0.0, 0.0);
        let mut total_influence = 0.0;

        let displacements: Vec<(Vec2, f32)> = model
            .displacer_configs
            .iter()
            .map(|config| {
                let displacement = config.displacer.influence(point);
                let influence = displacement.length();
                total_displacement += displacement;
                total_influence += influence;
                (displacement, influence)
            })
            .collect();

        let mut colors: Vec<(LinSrgb, f32)> = Vec::new();
        for (index, config) in model.displacer_configs.iter().enumerate() {
            let (_displacement, influence) = displacements[index];
            let color_position = influence / config.displacer.strength;
            let color = gradient.get(color_position.clamp(0.0, 1.0));
            let weight = influence / total_influence.max(1.0);
            colors.push((color, weight));
        }

        let blended_color = colors
            .iter()
            .fold(gradient.get(0.0), |acc, (color, weight)| {
                acc.mix(color, *weight)
            });

        draw.ellipse()
            .radius(model.circle_radius)
            .xy(point + total_displacement)
            .color(blended_color);
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
