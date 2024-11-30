use nannou::color::{Gradient, Mix};
use nannou::prelude::*;
use nannou::winit::window::Window as WinitWindow;

use crate::framework::displacer::{Displacer, DisplacerState};
use crate::framework::util::create_grid;

struct DisplacerConfig {
    displacer: Displacer,
    animate_position: Box<dyn Fn(f32, &Displacer) -> Vec2>,
    animate_radius: Box<dyn Fn(f32, &Displacer) -> f32>,
    animate_strength: Box<dyn Fn(f32, &Displacer) -> f32>,
}

impl DisplacerConfig {
    pub fn new(
        displacer: Displacer,
        animate_position: Box<dyn Fn(f32, &Displacer) -> Vec2>,
        animate_radius: Box<dyn Fn(f32, &Displacer) -> f32>,
        animate_strength: Box<dyn Fn(f32, &Displacer) -> f32>,
    ) -> Self {
        Self {
            displacer,
            animate_position,
            animate_radius,
            animate_strength,
        }
    }

    pub fn update(&mut self, time: f32) {
        self.displacer.update(Some(DisplacerState {
            position: Some((self.animate_position)(time, &self.displacer)),
            radius: Some((self.animate_radius)(time, &self.displacer)),
            strength: Some((self.animate_strength)(time, &self.displacer)),
        }));
    }
}

pub struct Model {
    _window: window::Id,
    circle_radius: f32,
    grid_size: usize,
    displacer_configs: Vec<DisplacerConfig>,
}

pub fn model(app: &App) -> Model {
    let w: i32 = 700;
    let h: i32 = 700;

    let _window = app
        .new_window()
        .size(w as u32, h as u32)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(_window).unwrap();
    let winit_window: &WinitWindow = window.winit_window();
    winit_window
        .set_outer_position(nannou::winit::dpi::PhysicalPosition::new(0, 0));

    let mut displacer_configs = vec![
        DisplacerConfig::new(
            Displacer::new(vec2(0.0, 0.0), 100.0, 50.0),
            Box::new(|_time, displacer| displacer.position),
            Box::new(|time, _d| 50.0 + 40.0 * (time * 16.0).sin()),
            Box::new(|time, _d| 50.0 + 40.0 * (time * 8.0).sin()),
        ),
        DisplacerConfig::new(
            Displacer::new(vec2(0.0, 0.0), 100.0, 50.0),
            Box::new(|time, _d| vec2(200.0 * time.cos(), 200.0 * time.sin())),
            Box::new(|time, _d| 200.0 + 100.0 * (time * 24.0).sin()),
            Box::new(|time, _d| 60.0 + 50.0 * (time * 12.0).cos()),
        ),
    ];

    let pad = 40.0;
    let corner_placements = vec![
        vec2((w as f32 / 2.0) - pad, (h as f32 / 2.0) - pad),
        vec2((-w as f32 / 2.0) + pad, (h as f32 / 2.0) - pad),
        vec2((-w as f32 / 2.0) + pad, (-h as f32 / 2.0) + pad),
        vec2((w as f32 / 2.0) - pad, (-h as f32 / 2.0) + pad),
    ];
    for corner in corner_placements {
        displacer_configs.push(DisplacerConfig::new(
            Displacer::new(corner, 10.0, 20.0),
            Box::new(|_time, displacer| displacer.position),
            Box::new(|time, _d| 20.0 + 20.0 * ((time * 16.0).sin())),
            Box::new(|_time, displacer| displacer.strength),
        ));
    }

    Model {
        _window,
        circle_radius: 2.0,
        grid_size: 64,
        displacer_configs,
    }
}

pub fn update(app: &App, model: &mut Model, _update: Update) {
    for config in &mut model.displacer_configs {
        config.update(app.time);
    }
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
            let color_position = influence / (config.displacer.strength * 1.5);
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

    draw.to_frame(app, &frame).unwrap()
}
