use nannou::color::{Gradient, Mix};
use nannou::prelude::*;
use std::f32::consts::PI;
use std::sync::Arc;

use crate::framework::{
    animation::Animation,
    controls::{Control, Controls},
    displacer::Displacer,
    sketch::{SketchConfig, SketchModel},
    util::{create_grid, IntoLinSrgb},
};

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "displacement_1",
    display_name: "Displacement v1",
    fps: 30.0,
    bpm: 134.0,
    w: 500,
    h: 500,
    gui_w: None,
    gui_h: None,
};

type AnimationFn<R> =
    Option<Arc<dyn Fn(&Displacer, &Animation, &Controls) -> R>>;

struct DisplacerConfig {
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
    displacer_configs: [DisplacerConfig; 6],
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
            value: 0.5,
            min: 0.0,
            max: 1.0,
            step: 0.0001,
        },
        Control::Slider {
            name: "max_radius".to_string(),
            value: 210.0,
            min: 0.0,
            max: 500.0,
            step: 1.0,
        },
        Control::Slider {
            name: "strength".to_string(),
            value: 95.0,
            min: 0.0,
            max: 500.0,
            step: 1.0,
        },
        Control::Slider {
            name: "corner_radius".to_string(),
            value: 54.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
        },
        Control::Slider {
            name: "corner_strength".to_string(),
            value: 25.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
        },
    ]);

    let pad = 40.0;
    let corner_strength = controls.get_float("corner_strength");

    let corner_radius_fn: AnimationFn<f32> =
        Some(Arc::new(|_displacer, ax, controls| {
            ax.get_ping_pong_loop_progress(1.0)
                * controls.get_float("corner_radius")
        }));

    let displacer_configs = [
        DisplacerConfig::new(
            "center",
            Displacer::new(vec2(0.0, 0.0), 10.0, 50.0, None),
            None,
            Some(Arc::new(|_displacer, ax, controls| {
                let value = ax.get_ping_pong_loop_progress(0.5);
                let radius = map_range(
                    value,
                    0.0,
                    1.0,
                    50.0,
                    controls.get_float("max_radius"),
                );
                radius
            })),
        ),
        DisplacerConfig::new(
            "roaming",
            Displacer::new(vec2(200.0, 200.0), 150.0, 50.0, None),
            Some(Arc::new(|_displacer, ax, _controls| {
                let movement_radius = 175.0;
                let angle = ax.get_loop_progress(8.0) * PI * 2.0;
                let x = angle.cos() * movement_radius;
                let y = angle.sin() * movement_radius;
                vec2(x, y)
            })),
            Some(Arc::new(|_displacer, ax, controls| {
                let progress = ax.get_ping_pong_loop_progress(1.5);
                let radius = map_range(
                    progress,
                    0.0,
                    1.0,
                    50.0,
                    controls.get_float("max_radius"),
                );
                radius
            })),
        ),
        DisplacerConfig::new(
            "corner_quad_1",
            Displacer::new(
                vec2((grid_w / 2.0) - pad, (grid_h / 2.0) - pad),
                1000.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_fn.clone(),
        ),
        DisplacerConfig::new(
            "corner_quad_2",
            Displacer::new(
                vec2((-grid_w / 2.0) + pad, (grid_h / 2.0) - pad),
                0.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_fn.clone(),
        ),
        DisplacerConfig::new(
            "corner_quad_3  ",
            Displacer::new(
                vec2((-grid_w / 2.0) + pad, (-grid_h / 2.0) + pad),
                0.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_fn.clone(),
        ),
        DisplacerConfig::new(
            "corner_quad_4",
            Displacer::new(
                vec2((grid_w / 2.0) - pad, (-grid_h / 2.0) + pad),
                0.0,
                corner_strength,
                None,
            ),
            None,
            corner_radius_fn.clone(),
        ),
    ];

    Model {
        circle_radius: 2.0,
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
        config.update(&model.animation, &model.controls);
        match config.kind {
            "center" | "roaming" => {
                let strength = model.controls.get_float("strength");
                config.displacer.set_strength(strength);
            }
            kind if kind.starts_with("corner") => {
                let strength = model.controls.get_float("corner_strength");
                config.displacer.set_strength(strength);
            }
            _ => (),
        }
    }
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // TODO: opt move to model (unless we want dynamic grid)
    let grid =
        create_grid(model.grid_w, model.grid_h, model.grid_size, |x, y| {
            vec2(x, y)
        });

    // TODO: opt move to model (better if per-displacer)
    let gradient =
        Gradient::new(vec![BEIGE.into_lin_srgb(), PURPLE.into_lin_srgb()]);

    let gradient_spread = model.controls.get_float("gradient_spread");

    frame.clear(BLACK);
    draw.background().color(rgb(0.1, 0.1, 0.1));

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

        draw.ellipse()
            .no_fill()
            .stroke(blended_color)
            .stroke_weight(0.5)
            .radius(model.circle_radius)
            .xy(point + total_displacement);
    }

    draw.to_frame(app, &frame).unwrap();
}
