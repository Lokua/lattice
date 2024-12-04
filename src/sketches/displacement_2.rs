use nannou::color::{Gradient, Mix};
use nannou::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::framework::{
    animation::Animation,
    controls::{Control, Controls},
    displacer::{CustomDistanceFn, Displacer},
    logging::*,
    sketch::{SketchConfig, SketchModel},
    util::{create_grid, IntoLinSrgb, TrigonometricExt},
};

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "displacement_2",
    display_name: "Displacement v2",
    fps: 30.0,
    bpm: 134.0,
    w: 1000,
    h: 1000,
    gui_w: None,
    gui_h: Some(400),
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
    grid_size: usize,
    grid_w: f32,
    grid_h: f32,
    displacer_configs: Vec<DisplacerConfig>,
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
        Control::Select {
            name: "pattern".into(),
            value: "cos,sin".into(),
            options: generate_pattern_options(),
        },
        Control::Checkbox {
            name: "clamp_circle_radii".into(),
            value: false,
        },
        Control::Checkbox {
            name: "animate_frequency".into(),
            value: false,
        },
        Control::Slider {
            name: "gradient_spread".into(),
            value: 0.99,
            min: 0.0,
            max: 1.0,
            step: 0.0001,
        },
        Control::Slider {
            name: "circle_radius_min".into(),
            value: 1.0,
            min: 0.1,
            max: 12.0,
            step: 0.1,
        },
        Control::Slider {
            name: "circle_radius_max".into(),
            value: 5.0,
            min: 0.1,
            max: 12.0,
            step: 0.1,
        },
        Control::Slider {
            name: "displacer_radius".into(),
            value: 0.001,
            min: 0.0001,
            max: 0.01,
            step: 0.0001,
        },
        Control::Slider {
            name: "displacer_strength".into(),
            value: 34.0,
            min: 0.5,
            max: 100.0,
            step: 0.5,
        },
        Control::Slider {
            name: "weave_frequency".into(),
            value: 0.01,
            min: 0.01,
            max: 0.2,
            step: 0.001,
        },
        Control::Slider {
            name: "weave_scale".into(),
            value: 0.05,
            min: 0.001,
            max: 0.1,
            step: 0.001,
        },
        Control::Slider {
            name: "weave_amplitude".into(),
            value: 0.001,
            min: 0.0001,
            max: 0.01,
            step: 0.0001,
        },
        Control::Checkbox {
            name: "center".into(),
            value: true,
        },
        Control::Checkbox {
            name: "quad_1".into(),
            value: false,
        },
        Control::Checkbox {
            name: "quad_2".into(),
            value: false,
        },
        Control::Checkbox {
            name: "quad_3".into(),
            value: false,
        },
        Control::Checkbox {
            name: "quad_4".into(),
            value: false,
        },
    ]);

    let displacer_configs = vec![
        DisplacerConfig::new(
            "center",
            Displacer::new(vec2(0.0, 0.0), 20.0, 10.0, None),
            None,
            None,
        ),
        DisplacerConfig::new(
            "quad_1",
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
            "quad_2",
            Displacer::new(
                vec2(w as f32 / 4.0, -h as f32 / 4.0),
                20.0,
                10.0,
                None,
            ),
            None,
            None,
        ),
        DisplacerConfig::new(
            "quad_3",
            Displacer::new(
                vec2(-w as f32 / 4.0, -h as f32 / 4.0),
                20.0,
                10.0,
                None,
            ),
            None,
            None,
        ),
        DisplacerConfig::new(
            "quad_4",
            Displacer::new(
                vec2(-w as f32 / 4.0, h as f32 / 4.0),
                20.0,
                10.0,
                None,
            ),
            None,
            None,
        ),
    ];

    Model {
        grid_size: 128,
        grid_w,
        grid_h,
        displacer_configs,
        animation,
        controls,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    for config in &mut model.displacer_configs {
        let displacer_radius = model.controls.get_float("displacer_radius");
        let displacer_strength = model.controls.get_float("displacer_strength");
        let weave_scale = model.controls.get_float("weave_scale");
        let weave_amplitude = model.controls.get_float("weave_amplitude");
        let pattern = model.controls.get_string("pattern");
        let weave_frequency = if model.controls.get_bool("animate_frequency") {
            map_range(
                model.animation.get_ping_pong_loop_progress(32.0),
                0.0,
                1.0,
                0.01,
                model.controls.get_float("weave_frequency"),
            )
        } else {
            model.controls.get_float("weave_frequency")
        };

        let distance_fn: CustomDistanceFn =
            Some(Arc::new(move |grid_point, position| {
                weave(
                    grid_point.x,
                    grid_point.y,
                    position.x,
                    position.y,
                    weave_frequency,
                    weave_scale,
                    weave_amplitude,
                    pattern.clone(),
                )
            }));

        config.update(&model.animation, &model.controls);
        config.displacer.set_custom_distance_fn(distance_fn);
        config.displacer.set_radius(displacer_radius);
        config.displacer.set_strength(displacer_strength);
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
        * model.controls.get_float("displacer_strength");

    let enabled_displacer_configs: Vec<&DisplacerConfig> = model
        .displacer_configs
        .iter()
        .filter(|x| model.controls.get_bool(x.kind))
        .collect();

    for point in grid {
        let mut total_displacement = vec2(0.0, 0.0);
        let mut total_influence = 0.0;
        let mut colors: Vec<(LinSrgb, f32)> = Vec::new();

        let displacements: Vec<(Vec2, f32)> = enabled_displacer_configs
            .iter()
            .map(|config| {
                let displacement = config.displacer.influence(point);
                let influence = displacement.length();
                total_displacement += displacement;
                total_influence += influence;
                (displacement, influence)
            })
            .collect();

        for (index, config) in enabled_displacer_configs.iter().enumerate() {
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

        let magnitude = if model.controls.get_bool("clamp_circle_radii") {
            total_displacement.length().clamp(0.0, max_mag)
        } else {
            total_displacement.length()
        };
        let radius = map_range(
            magnitude,
            0.0,
            max_mag,
            model.controls.get_float("circle_radius_min"),
            model.controls.get_float("circle_radius_max"),
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

pub fn weave(
    grid_x: f32,
    grid_y: f32,
    center_x: f32,
    center_y: f32,
    frequency: f32,
    distance_scale: f32,
    amplitude: f32,
    pattern: String,
) -> f32 {
    let x = grid_x * frequency;
    let y = grid_y * frequency;

    let position_pattern = match pattern.as_str() {
        "Comp1" => (x.sin() * y.cos()) + (x - y).tanh() * (x + y).tan(),
        _ => parse_dynamic_pattern(&pattern, x, y),
    };

    let distance =
        ((center_x - grid_x).powi(2) + (center_y - grid_y).powi(2)).sqrt();

    let distance_pattern = (distance * distance_scale).sin();

    position_pattern * distance_pattern * amplitude
}

fn generate_pattern_options() -> Vec<String> {
    let functions = vec![
        "cos", "sin", "tan", "tanh", "sec", "csc", "cot", "sech", "csch",
        "coth",
    ];

    let mut options: Vec<String> = functions
        .iter()
        .flat_map(|a| functions.iter().map(move |b| format!("{},{}", a, b)))
        .collect();

    let custom_algs = vec!["Comp1".into()];

    // Append custom algorithms
    options.extend(custom_algs);

    options
}

fn trig_fn_lookup() -> HashMap<&'static str, fn(f32) -> f32> {
    let mut map = HashMap::new();
    map.insert("cos", f32::cos as fn(f32) -> f32);
    map.insert("sin", f32::sin as fn(f32) -> f32);
    map.insert("tan", f32::tan as fn(f32) -> f32);
    map.insert("tanh", f32::tanh as fn(f32) -> f32);
    map.insert("sec", f32::sec as fn(f32) -> f32);
    map.insert("csc", f32::csc as fn(f32) -> f32);
    map.insert("cot", f32::cot as fn(f32) -> f32);
    map.insert("sech", f32::sech as fn(f32) -> f32);
    map.insert("csch", f32::csch as fn(f32) -> f32);
    map.insert("coth", f32::coth as fn(f32) -> f32);
    map
}

pub fn parse_dynamic_pattern(pattern: &str, x: f32, y: f32) -> f32 {
    let lookup = trig_fn_lookup();
    let parts: Vec<&str> = pattern.split(',').collect();

    if parts.len() != 2 {
        error!("Invalid pattern: {}", pattern);
        return 0.0;
    }

    let (a, b) = (parts[0], parts[1]);

    let func_a = lookup.get(a);
    let func_b = lookup.get(b);

    match (func_a, func_b) {
        (Some(&f_a), Some(&f_b)) => f_a(x) + f_b(y),
        _ => {
            error!("Unknown function(s) in pattern: {}", pattern);
            0.0
        }
    }
}
