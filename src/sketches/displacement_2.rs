use nannou::color::{Gradient, Mix};
use nannou::prelude::*;
use std::sync::Arc;

use crate::framework::displacer::CustomDistanceFn;
use crate::framework::{
    animation::Animation,
    controls::{Control, Controls},
    displacer::Displacer,
    distance::weave,
    sketch::{SketchConfig, SketchModel},
    util::{create_grid, IntoLinSrgb},
};

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "displacement_2",
    display_name: "Displacement v2",
    fps: 30.0,
    bpm: 134.0,
    w: 700,
    h: 700,
};

type AnimationFn<R> =
    Option<Arc<dyn Fn(&Displacer, &Animation, &Controls) -> R>>;

struct DisplacerConfig {
    #[allow(dead_code)]
    kind: &'static str,
    displacer: Displacer,
    position_animation: AnimationFn<Vec2>,
    radius_animation: AnimationFn<f32>,
}

impl DisplacerConfig {
    pub fn new(
        kind: &'static str,
        displacer: Displacer,
        position_animation: AnimationFn<Vec2>,
        radius_animation: AnimationFn<f32>,
    ) -> Self {
        Self {
            kind,
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

pub struct Model {
    circle_radius: f32,
    grid_size: usize,
    grid_w: f32,
    grid_h: f32,
    displacer_configs: [DisplacerConfig; 2],
    animation: Animation,
    controls: Controls,
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
        Control::Slider {
            name: "gradient_spread".to_string(),
            value: 0.99,
            min: 0.0,
            max: 1.0,
            step: 0.0001,
        },
        Control::Slider {
            name: "radius".to_string(),
            value: 210.0,
            min: 0.0001,
            max: 500.0,
            step: 1.0,
        },
        Control::Slider {
            name: "strength".to_string(),
            value: 34.0,
            min: 0.0001,
            max: 100.0,
            step: 1.0,
        },
        Control::Slider {
            name: "frequency".to_string(),
            value: 0.05,
            min: 0.0,
            max: 0.1,
            step: 0.001,
        },
    ]);

    let displacer_configs = [
        DisplacerConfig::new(
            "center",
            Displacer::new(
                vec2(w as f32 / 4.0, h as f32 / 4.0),
                20.0,
                10.0,
                None,
            ),
            None,
            None,
        ),
        DisplacerConfig::new(
            "center",
            Displacer::new(
                vec2(-w as f32 / 4.0, -h as f32 / 4.0),
                20.0,
                10.0,
                None,
            ),
            None,
            None,
        ),
    ];

    Model {
        circle_radius: 12.0,
        grid_size: 64,
        grid_w,
        grid_h,
        displacer_configs,
        animation,
        controls,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    for config in &mut model.displacer_configs {
        let radius = model.controls.get_float("radius");
        let strength = model.controls.get_float("strength");
        let frequency = model.controls.get_float("frequency");

        let distance_fn: CustomDistanceFn =
            Some(Arc::new(move |grid_point, position| {
                weave(
                    grid_point.x,
                    grid_point.y,
                    position.x,
                    position.y,
                    frequency,
                )
            }));

        config.update(&model.animation, &model.controls);
        config.displacer.set_custom_distance_fn(distance_fn);
        config.displacer.set_strength(strength);
        config.displacer.set_radius(radius);
    }
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // TODO: opt move to model (unless we want dynamic grid)
    let grid = create_grid(model.grid_w, model.grid_h, model.grid_size, vec2);

    // TODO: opt move to model
    let gradient =
        Gradient::new(vec![LIGHTCORAL.into_lin_srgb(), AZURE.into_lin_srgb()]);

    let gradient_spread = model.controls.get_float("gradient_spread");

    frame.clear(BLACK);
    draw.background().color(rgb(0.1, 0.1, 0.1));

    let max_mag = model.displacer_configs.len() as f32
        * model.controls.get_float("strength");

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
            let color_position = (influence / config.displacer.strength)
                .powf(gradient_spread)
                .clamp(0.0, 1.0);
            let color = gradient.get(color_position);
            let weight = influence / total_influence.max(1.0);
            colors.push((color, weight));
        }

        let blended_color = colors
            .iter()
            .fold(gradient.get(0.0), |acc, (color, weight)| {
                acc.mix(color, *weight)
            });

        let radius = map_range(
            total_displacement.length(),
            0.0,
            max_mag,
            model.circle_radius / 3.0,
            model.circle_radius,
        );

        draw.ellipse()
            .no_fill()
            .stroke(blended_color)
            .stroke_weight(0.5)
            .radius(radius)
            .xy(point + total_displacement);
    }

    draw.to_frame(app, &frame).unwrap();
}
