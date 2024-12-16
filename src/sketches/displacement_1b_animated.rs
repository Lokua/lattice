use nannou::color::Gradient;
use nannou::prelude::*;
use rayon::prelude::*;
use std::sync::Arc;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "displacement_1b_animated",
    display_name: "Displacement 1b Animated",
    fps: 60.0,
    bpm: 134.0,
    w: 1000,
    h: 1000,
    gui_w: None,
    gui_h: Some(320),
};

const GRID_SIZE: usize = 256;

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
        Control::checkbox("show_center", true),
        Control::checkbox("show_corner", false),
        Control::slider("gradient_spread", 0.5, (0.0, 1.0), 0.0001),
        Control::slider("size_min", 0.1, (0.1, 4.0), 0.1),
        Control::slider("size_max", 2.5, (0.1, 4.0), 0.1),
        Control::slider("scaling_power", 3.0, (0.5, 11.0), 0.25),
        Control::slider("displacer_radius", 210.0, (1.0, 500.0), 1.0),
        Control::slider("displacer_strength", 95.0, (1.0, 500.0), 1.0),
        Control::slider("corner_radius", 210.0, (1.0, 500.0), 1.0),
        Control::slider("corner_strength", 95.0, (1.0, 500.0), 1.0),
    ]);

    let w4th = w as f32 / 4.0;
    let h4th = h as f32 / 4.0;

    let displacer_configs = vec![
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Center,
            Displacer::new_with_position(vec2(0.0, 0.0)),
        ),
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Corner,
            Displacer::new_with_position(vec2(w4th, h4th)),
        ),
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Corner,
            Displacer::new_with_position(vec2(w4th, -h4th)),
        ),
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Corner,
            Displacer::new_with_position(vec2(-w4th, -h4th)),
        ),
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Corner,
            Displacer::new_with_position(vec2(-w4th, h4th)),
        ),
    ];

    Model {
        grid: create_grid(grid_w, grid_h, GRID_SIZE, vec2),
        displacer_configs,
        animation,
        controls,
        gradient: Gradient::new(vec![
            CYAN.into_lin_srgb(),
            MAGENTA.into_lin_srgb(),
            YELLOW.into_lin_srgb(),
        ]),
        ellipses: Vec::with_capacity(GRID_SIZE * GRID_SIZE),
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let size_min = model.controls.float("size_min");
    let size_max = model.controls.float("size_max");
    let radius = model.controls.float("displacer_radius");
    let strength = model.controls.float("displacer_strength");
    let corner_radius = model.controls.float("corner_radius");
    let corner_strength = model.controls.float("corner_strength");
    let gradient_spread = model.controls.float("gradient_spread");
    let scaling_power = model.controls.float("scaling_power");
    let show_center = model.controls.bool("show_center");
    let show_corner = model.controls.bool("show_corner");

    for config in &mut model.displacer_configs {
        config.update(&model.animation, &model.controls);
        match config.kind {
            DisplacerConfigKind::Center => {
                config.displacer.set_strength(strength);
                config.displacer.set_radius(radius);
            }
            DisplacerConfigKind::Corner => {
                config.displacer.set_strength(corner_radius);
                config.displacer.set_radius(corner_strength);
            }
        }
    }

    let max_mag = model.displacer_configs.len() as f32 * strength;
    let gradient = &model.gradient;

    let configs: Vec<&DisplacerConfig> = model
        .displacer_configs
        .iter()
        .filter(|config| match config.kind {
            DisplacerConfigKind::Center => show_center,
            DisplacerConfigKind::Corner => show_corner,
        })
        .collect();

    model.ellipses = model
        .grid
        .par_iter()
        .map(|point| {
            let total_displacement =
                configs.iter().fold(vec2(0.0, 0.0), |acc, config| {
                    acc + config.displacer.attract(*point, scaling_power)
                });

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
                size_min,
                size_max,
                ease_out,
            );

            (*point + total_displacement, radius, color)
        })
        .collect();
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(hsl(0.0, 0.0, 0.02));

    for (position, radius, color) in &model.ellipses {
        draw.rect()
            .no_fill()
            .stroke(lin_srgb_to_lin_srgba(*color, 1.0))
            .stroke_weight(0.5)
            .w_h(*radius, *radius)
            .xy(*position);
    }

    draw.to_frame(app, &frame).unwrap();
}

type AnimationFn<R> =
    Option<Arc<dyn Fn(&Displacer, &Animation, &Controls) -> R + Send + Sync>>;

enum DisplacerConfigKind {
    Center,
    Corner,
}

struct DisplacerConfig {
    #[allow(dead_code)]
    kind: DisplacerConfigKind,
    displacer: Displacer,
    position_animation: AnimationFn<Vec2>,
    radius_animation: AnimationFn<f32>,
}

impl DisplacerConfig {
    pub fn new(
        kind: DisplacerConfigKind,
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

    pub fn new_no_anim(
        kind: DisplacerConfigKind,
        displacer: Displacer,
    ) -> Self {
        Self::new(kind, displacer, None, None)
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
