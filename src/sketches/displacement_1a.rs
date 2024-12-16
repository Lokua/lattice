use nannou::color::{Gradient, IntoLinSrgba};
use nannou::prelude::*;
use rayon::prelude::*;
use std::sync::Arc;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "displacement_1a",
    display_name: "Displacement 1a",
    fps: 60.0,
    bpm: 134.0,
    w: 1000,
    h: 1000,
    gui_w: None,
    gui_h: Some(240),
};

const GRID_SIZE: usize = 128;

pub struct Model {
    grid: Vec<Vec2>,
    displacer_configs: Vec<DisplacerConfig>,
    animation: Animation,
    controls: Controls,
    gradient: Gradient<LinSrgb>,
    ellipses: Vec<(Vec2, f32, LinSrgb)>,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

pub fn init_model() -> Model {
    let w = SKETCH_CONFIG.w;
    let h = SKETCH_CONFIG.h;
    let grid_w = w as f32 - 80.0;
    let grid_h = h as f32 - 80.0;
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::new(vec![
        Control::slider("gradient_spread", 0.5, (0.0, 1.0), 0.0001),
        Control::slider("circle_radius_min", 0.1, (0.1, 4.0), 0.1),
        Control::slider("circle_radius_max", 2.5, (0.1, 4.0), 0.1),
        Control::slider("displacer_radius", 210.0, (0.0, 500.0), 1.0),
        Control::slider("displacer_strength", 95.0, (0.0, 500.0), 1.0),
        Control::slider("scaling_power", 3.0, (0.5, 7.0), 0.25),
    ]);

    let displacer_configs = vec![DisplacerConfig::new(
        Displacer::new(vec2(0.0, 0.0), 10.0, 50.0, None),
        None,
        None,
    )];

    Model {
        grid: create_grid(grid_w, grid_h, GRID_SIZE, vec2),
        displacer_configs,
        animation,
        controls,
        gradient: Gradient::new(vec![
            BEIGE.into_lin_srgb(),
            hsl(0.6, 0.8, 0.7).into_lin_srgba().into(),
        ]),
        ellipses: Vec::with_capacity(GRID_SIZE * GRID_SIZE),
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let circle_radius_min = model.controls.float("circle_radius_min");
    let circle_radius_max = model.controls.float("circle_radius_max");
    let radius = model.controls.float("displacer_radius");
    let strength = model.controls.float("displacer_strength");
    let gradient_spread = model.controls.float("gradient_spread");
    let scaling_power = model.controls.float("scaling_power");

    for config in &mut model.displacer_configs {
        config.update(&model.animation, &model.controls);
        config.displacer.set_strength(strength);
        config.displacer.set_radius(radius);
    }

    let max_mag = model.displacer_configs.len() as f32 * strength;
    let gradient = &model.gradient;

    model.ellipses = model
        .grid
        .par_iter()
        .map(|point| {
            let total_displacement = model.displacer_configs.iter().fold(
                vec2(0.0, 0.0),
                |acc, config| {
                    acc + config.displacer.attract(*point, scaling_power)
                },
            );

            let displacement_magnitude = total_displacement.length();

            let color = gradient.get(
                1.0 - (displacement_magnitude / max_mag)
                    .powf(gradient_spread)
                    .clamp(0.0, 1.0),
            );

            let radius = map_clamp(
                displacement_magnitude,
                0.0,
                max_mag,
                circle_radius_min,
                circle_radius_max,
                |x| x,
            );

            (*point + total_displacement, radius, color)
        })
        .collect();
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(hsl(0.0, 0.0, 0.02));

    for (position, radius, color) in &model.ellipses {
        draw.ellipse()
            .no_fill()
            .stroke(*color)
            .stroke_weight(0.5)
            .radius(*radius)
            .xy(*position);
    }

    draw.to_frame(app, &frame).unwrap();
}

type AnimationFn<R> =
    Option<Arc<dyn Fn(&Displacer, &Animation, &Controls) -> R + Send + Sync>>;

struct DisplacerConfig {
    displacer: Displacer,
    position_animation: AnimationFn<Vec2>,
    radius_animation: AnimationFn<f32>,
}

impl DisplacerConfig {
    pub fn new(
        displacer: Displacer,
        position_animation: AnimationFn<Vec2>,
        radius_animation: AnimationFn<f32>,
    ) -> Self {
        Self {
            displacer,
            position_animation,
            radius_animation,
        }
    }

    pub fn update(&mut self, animation: &Animation, controls: &Controls) {
        if let Some(position_fn) = &self.position_animation {
            self.displacer.position =
                position_fn(&self.displacer, animation, controls);
        }
        if let Some(radius_fn) = &self.radius_animation {
            self.displacer.radius =
                radius_fn(&self.displacer, animation, controls);
        }
    }
}