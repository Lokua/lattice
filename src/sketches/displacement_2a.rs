use geom::Ellipse;
use nannou::color::{Gradient, Mix};
use nannou::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "displacement_2a",
    display_name: "Displacement 2a",
    fps: 30.0,
    bpm: 134.0,
    w: 1000,
    h: 1000,
    gui_w: None,
    gui_h: Some(680),
};

const DEBUG_QUADS: bool = false;
const GRID_SIZE: usize = 128;
const SAMPLE_RATE: usize = 48_000;
const N_BANDS: usize = 8;

pub struct Model {
    grid: Vec<Vec2>,
    displacer_configs: Vec<DisplacerConfig>,
    animation: Animation,
    controls: Controls,
    cached_pattern: String,
    cached_trig_fns: Option<(fn(f32) -> f32, fn(f32) -> f32)>,
    gradient: Gradient<LinSrgb>,
    ellipses: Vec<(Vec2, f32, LinSrgb)>,
    audio: Audio,
    fft_bands: Vec<f32>,
}

impl Model {
    fn update_trig_fns(&mut self) {
        let pattern = self.controls.string("pattern");
        let lookup = trig_fn_lookup();
        let parts: Vec<&str> = pattern.split(',').collect();

        self.cached_trig_fns = if parts.len() == 2 {
            match (lookup.get(parts[0]), lookup.get(parts[1])) {
                (Some(&f_a), Some(&f_b)) => {
                    self.cached_pattern = pattern;
                    Some((f_a, f_b))
                }
                _ => {
                    error!("Unknown function(s) in pattern: {}", pattern);
                    None
                }
            }
        } else if pattern == "Comp1" {
            self.cached_pattern = pattern;
            None
        } else {
            error!("Invalid pattern: {}", pattern);
            None
        };
    }
    fn weave_frequency(&self) -> f32 {
        let value = self.controls.float("weave_frequency");
        if self.controls.bool("animate_frequency") {
            map_range(
                self.animation.animate(
                    vec![
                        KF::new(0.0, 16.0),
                        KF::new(1.0, 16.0),
                        KF::new(0.0, KF::END),
                    ],
                    0.0,
                ),
                0.0,
                1.0,
                0.01,
                value,
            )
        } else {
            value
        }
    }
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

type AnimationFn<R> =
    Option<Arc<dyn Fn(&Displacer, &Animation, &Controls) -> R + Send + Sync>>;

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

pub fn init_model() -> Model {
    let w = SKETCH_CONFIG.w;
    let h = SKETCH_CONFIG.h;
    let animation = Animation::new(SKETCH_CONFIG.bpm);
    let audio = Audio::new(SAMPLE_RATE, SKETCH_CONFIG.fps);

    let controls = Controls::new(vec![
        Control::Checkbox {
            name: "audio_enabled".into(),
            value: false,
        },
        Control::Slider {
            name: "rise_rate".into(),
            value: 0.96,
            min: 0.001,
            max: 1.0,
            step: 0.001,
        },
        Control::Slider {
            name: "fall_rate".to_string(),
            value: 0.9,
            min: 0.0,
            max: 1.0,
            step: 0.001,
        },
        Control::Select {
            name: "pattern".into(),
            value: "cos,sin".into(),
            options: generate_pattern_options(),
        },
        Control::Slider {
            name: "scale".into(),
            value: 1.0,
            min: 0.1,
            max: 4.0,
            step: 0.1,
        },
        Control::Checkbox {
            name: "clamp_circle_radii".into(),
            value: false,
        },
        Control::Checkbox {
            name: "animate_frequency".into(),
            value: false,
        },
        Control::Checkbox {
            name: "quad_restraint".into(),
            value: false,
        },
        Control::Checkbox {
            name: "qr_uses_circle".into(),
            value: false,
        },
        Control::Slider {
            name: "qr_lerp".into(),
            value: 0.5,
            min: 0.0,
            max: 1.0,
            step: 0.0001,
        },
        Control::Slider {
            name: "qr_divisor".into(),
            value: 2.0,
            min: 0.5,
            max: 16.0,
            step: 0.125,
        },
        Control::Slider {
            name: "qr_pos".into(),
            value: 1.0,
            min: 0.125,
            max: 1.0,
            step: 0.125,
        },
        Control::Slider {
            name: "qr_size".into(),
            value: 1.0,
            min: 0.125,
            max: 1.0,
            step: 0.125,
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
            value: 0.03,
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
    ]);

    const BEATS: f32 = 8.0;

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
            Some(Arc::new(move |_displacer, ax, _controls| {
                let w = SKETCH_CONFIG.w as f32;
                let h = SKETCH_CONFIG.h as f32;
                let xp = w / 4.0;
                let yp = h / 4.0;
                let x = ax.animate(
                    vec![
                        KF::new(xp, BEATS),   // Start at right
                        KF::new(-xp, BEATS),  // Move to left
                        KF::new(-xp, BEATS),  // Stay at left
                        KF::new(xp, BEATS),   // Move to right
                        KF::new(xp, KF::END), // Complete the cycle
                    ],
                    0.0,
                );
                let y = ax.animate(
                    vec![
                        KF::new(yp, BEATS),   // Start at top
                        KF::new(yp, BEATS),   // Stay at top
                        KF::new(-yp, BEATS),  // Move to bottom
                        KF::new(-yp, BEATS),  // Stay at bottom
                        KF::new(yp, KF::END), // Move back to top
                    ],
                    0.0,
                );
                vec2(x, y)
            })),
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
            Some(Arc::new(move |_displacer, ax, _controls| {
                let w = SKETCH_CONFIG.w as f32;
                let h = SKETCH_CONFIG.h as f32;
                let xp = w / 4.0;
                let yp = -h / 4.0;
                let x = ax.animate(
                    vec![
                        KF::new(xp, BEATS),   // Start at right
                        KF::new(xp, BEATS),   // Stay at right
                        KF::new(-xp, BEATS),  // Move to left
                        KF::new(-xp, BEATS),  // Stay at left
                        KF::new(xp, KF::END), // Complete the cycle
                    ],
                    0.0,
                );
                let y = ax.animate(
                    vec![
                        KF::new(yp, BEATS),   // Start at bottom
                        KF::new(-yp, BEATS),  // Move to top
                        KF::new(-yp, BEATS),  // Stay at top
                        KF::new(yp, BEATS),   // Move at bottom
                        KF::new(yp, KF::END), // Complete the cycle
                    ],
                    0.0,
                );
                vec2(x, y)
            })),
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
            Some(Arc::new(move |_displacer, ax, _controls| {
                let w = SKETCH_CONFIG.w as f32;
                let h = SKETCH_CONFIG.h as f32;
                let xp = -w / 4.0;
                let yp = -h / 4.0;
                let x = ax.animate(
                    vec![
                        KF::new(xp, BEATS),   // Start at left
                        KF::new(-xp, BEATS),  // Move to right
                        KF::new(-xp, BEATS),  // Stay to right
                        KF::new(xp, BEATS),   // Move to left
                        KF::new(xp, KF::END), // Complete the cycle
                    ],
                    0.0,
                );
                let y = ax.animate(
                    vec![
                        KF::new(yp, BEATS),   // Start at bottom
                        KF::new(yp, BEATS),   // Move to top
                        KF::new(-yp, BEATS),  // Stay at top
                        KF::new(-yp, BEATS),  // Move to bottom
                        KF::new(yp, KF::END), // Complete the cycle
                    ],
                    0.0,
                );
                vec2(x, y)
            })),
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
            Some(Arc::new(move |_displacer, ax, _controls| {
                let w = SKETCH_CONFIG.w as f32;
                let h = SKETCH_CONFIG.h as f32;
                let xp = -w / 4.0;
                let yp = h / 4.0;
                let x = ax.animate(
                    vec![
                        KF::new(xp, BEATS),   // Start at left
                        KF::new(xp, BEATS),   // Stay at left
                        KF::new(-xp, BEATS),  // Move to right
                        KF::new(-xp, BEATS),  // Stay at right
                        KF::new(xp, KF::END), // Complete the cycle
                    ],
                    0.0,
                );
                let y = ax.animate(
                    vec![
                        KF::new(yp, BEATS),   // Start at top
                        KF::new(-yp, BEATS),  // Move to bottom
                        KF::new(-yp, BEATS),  // Stay at bottom
                        KF::new(yp, BEATS),   // Move to top
                        KF::new(yp, KF::END), // Complete the cycle
                    ],
                    0.0,
                );
                vec2(x, y)
            })),
            None,
        ),
    ];

    let pad = w as f32 * (1.0 / 3.0);
    let cached_pattern = controls.string("pattern");

    Model {
        grid: create_grid(w as f32 - pad, h as f32 - pad, GRID_SIZE, vec2),
        displacer_configs,
        animation,
        controls,
        cached_pattern,
        cached_trig_fns: None,
        gradient: Gradient::new(vec![
            // LIGHTGREEN.into_lin_srgb(),
            LIGHTCORAL.into_lin_srgb(),
            AZURE.into_lin_srgb(),
        ]),
        ellipses: Vec::with_capacity(GRID_SIZE),
        audio,
        fft_bands: Vec::new(),
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.cached_trig_fns == None
        || (model.cached_pattern != model.controls.string("pattern"))
    {
        model.update_trig_fns();
    }

    let audio_enabled = model.controls.bool("audio_enabled");
    let clamp_circle_radii = model.controls.bool("clamp_circle_radii");
    let quad_restraint = model.controls.bool("quad_restraint");
    let qr_uses_circle = model.controls.bool("qr_uses_circle");
    let qr_lerp = model.controls.float("qr_lerp");
    let qr_divisor = model.controls.float("qr_divisor");
    let qr_pos = model.controls.float("qr_pos");
    let qr_size = model.controls.float("qr_size");
    let displacer_radius = model.controls.float("displacer_radius");
    let displacer_strength = model.controls.float("displacer_strength");
    let weave_scale = model.controls.float("weave_scale");
    let weave_amplitude = model.controls.float("weave_amplitude");
    let pattern = model.controls.string("pattern");
    let gradient_spread = model.controls.float("gradient_spread");
    let circle_radius_min = model.controls.float("circle_radius_min");
    let circle_radius_max = model.controls.float("circle_radius_max");
    let animation = &model.animation;
    let controls = &model.controls;
    let weave_frequency = model.weave_frequency();

    model.fft_bands = model.audio.bands(
        N_BANDS,
        30.0,
        10_000.0,
        0.0,
        model.controls.float("rise_rate"),
        model.controls.float("fall_rate"),
    );

    let cached_trig_fns = model.cached_trig_fns.clone();

    let band_for_freq = model.fft_bands[0];
    let distance_fn: CustomDistanceFn =
        Some(Arc::new(move |grid_point, position| {
            weave(
                grid_point.x,
                grid_point.y,
                position.x,
                position.y,
                if audio_enabled {
                    map_range(band_for_freq, 0.0, 1.0, weave_frequency, 0.01)
                } else {
                    weave_frequency
                },
                weave_scale,
                weave_amplitude,
                pattern.clone(),
                cached_trig_fns,
            )
        }));

    for config in model.displacer_configs.iter_mut() {
        config.update(animation, controls);
        config.displacer.set_custom_distance_fn(distance_fn.clone());
        config.displacer.set_radius(displacer_radius);
        config.displacer.set_strength(if audio_enabled {
            map_range(model.fft_bands[7], 0.0, 1.0, 0.5, displacer_strength)
        } else {
            displacer_strength
        });
    }

    let enabled_displacer_configs: Vec<&DisplacerConfig> = model
        .displacer_configs
        .iter()
        .filter(|x| model.controls.bool(x.kind))
        .collect();

    let max_mag = model.displacer_configs.len() as f32 * displacer_strength;
    let gradient = &model.gradient;

    model.ellipses = model
        .grid
        .par_iter()
        .map(|point| {
            let mut total_displacement = vec2(0.0, 0.0);
            let mut total_influence = 0.0;
            let mut quad_contains = false;
            let mut current_qr_kind = "center";

            for config in &enabled_displacer_configs {
                if quad_restraint {
                    let displacer_rect = Rect::from_xy_wh(
                        config.displacer.position * qr_pos,
                        vec2(
                            SKETCH_CONFIG.w as f32 / 3.0,
                            SKETCH_CONFIG.h as f32 / 3.0,
                        ) * qr_size,
                    );

                    if qr_uses_circle {
                        let displacer_circle =
                            Ellipse::new(displacer_rect, 24.0);
                        if circle_contains_point(&displacer_circle, point) {
                            current_qr_kind = config.kind;
                            quad_contains = true;
                        }
                    } else {
                        if rect_contains_point(&displacer_rect, point) {
                            current_qr_kind = config.kind;
                            quad_contains = true;
                        }
                    }
                }
                let displacement = config.displacer.influence(*point);
                let influence = displacement.length();
                total_displacement += displacement;
                total_influence += influence;
            }

            let mut blended_color = gradient.get(0.0);
            let inv_total = 1.0 / total_influence.max(1.0);

            for config in &enabled_displacer_configs {
                let displacement = config.displacer.influence(*point);
                let influence = displacement.length();
                let color_position = (influence / config.displacer.strength)
                    .powf(gradient_spread)
                    .clamp(0.0, 1.0);
                let color = gradient.get(color_position);
                let weight = influence * inv_total;
                blended_color = blended_color.mix(&color, weight);
            }

            let magnitude = if clamp_circle_radii {
                total_displacement.length().clamp(0.0, max_mag)
            } else {
                total_displacement.length()
            };
            let radius = map_range(
                magnitude,
                0.0,
                max_mag,
                circle_radius_min,
                circle_radius_max,
            );

            if quad_restraint && quad_contains {
                let color = if DEBUG_QUADS {
                    match current_qr_kind {
                        "quad_1" => RED,
                        "quad_2" => GREEN,
                        "quad_3" => BLUE,
                        "quad_4" => YELLOW,
                        _ => WHITE,
                    }
                    .into_lin_srgb()
                } else {
                    blended_color
                };
                let displaced_point = if audio_enabled {
                    let bass_to_qr_divisor = map_range(
                        model.fft_bands[5],
                        0.0,
                        1.0,
                        qr_divisor,
                        0.0,
                    );
                    *point + (total_displacement / bass_to_qr_divisor)
                } else {
                    *point + (total_displacement / qr_divisor)
                };
                (
                    displaced_point,
                    radius,
                    gradient.get(0.0).mix(&color, qr_lerp),
                )
            } else {
                (*point + total_displacement, radius, blended_color)
            }
        })
        .collect();
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    frame.clear(BLACK);
    draw.background().color(rgba(0.1, 0.1, 0.1, 0.01));

    let scaled_draw = draw.scale(model.controls.float("scale"));

    for (position, radius, color) in &model.ellipses {
        scaled_draw
            .ellipse()
            .no_fill()
            .stroke(*color)
            .stroke_weight(0.5)
            .radius(*radius)
            .xy(*position);
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
    trig_fns: Option<(fn(f32) -> f32, fn(f32) -> f32)>,
) -> f32 {
    let x = grid_x * frequency;
    let y = grid_y * frequency;

    let position_pattern = match pattern.as_str() {
        "Comp1" => (x.sin() * y.cos()) + (x - y).tanh() * (x + y).tan(),
        _ => match trig_fns {
            Some((f_a, f_b)) => f_a(x) + f_b(y),
            None => 0.0,
        },
    };

    let distance =
        ((center_x - grid_x).powi(2) + (center_y - grid_y).powi(2)).sqrt();

    let distance_pattern = (distance * distance_scale).sin();

    position_pattern * distance_pattern * amplitude
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

    options.extend(custom_algs);

    options
}
