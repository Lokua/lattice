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
    bpm: 135.0,
    w: 1000,
    h: 1000,
    gui_w: None,
    gui_h: Some(700),
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
    last_position_animation: String,
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
        } else {
            self.cached_pattern = pattern;
            None
        };
    }
    fn weave_frequency(&self) -> f32 {
        let value = self.controls.float("weave_frequency");
        if self.controls.bool("animate_frequency") {
            map_range(
                self.animation.lerp(
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

pub fn init_model() -> Model {
    let w = SKETCH_CONFIG.w;
    let h = SKETCH_CONFIG.h;
    let animation = Animation::new(SKETCH_CONFIG.bpm);
    let audio = Audio::new(SAMPLE_RATE, SKETCH_CONFIG.fps);

    let controls = Controls::new(vec![
        Control::checkbox("audio_enabled", false),
        Control::slider("rise_rate", 0.96, (0.001, 1.0), 0.001),
        Control::slider("fall_rate", 0.9, (0.0, 1.0), 0.001),
        Control::select("pattern", "cos,sin", generate_pattern_options()),
        Control::slider("scale", 1.0, (0.1, 4.0), 0.1),
        Control::checkbox("clamp_circle_radii", false),
        Control::checkbox("animate_frequency", false),
        Control::checkbox("quad_restraint", false),
        Control::slider("qr_lerp", 0.5, (0.0, 1.0), 0.0001),
        Control::slider("qr_divisor", 2.0, (0.5, 16.0), 0.125),
        Control::slider("qr_pos", 1.0, (0.125, 1.0), 0.125),
        Control::slider("qr_size", 1.0, (0.125, 1.0), 0.125),
        Control::select(
            "position_animation",
            "Counter Clockwise",
            string_vec!["None", "Counter Clockwise"],
        ),
        Control::select(
            "qr_shape",
            "Rectangle",
            string_vec!["Rectangle", "Circle", "Ripple", "Spiral"],
        ),
        Control::checkbox("center", true),
        Control::checkbox("quad_1", false),
        Control::checkbox("quad_2", false),
        Control::checkbox("quad_3", false),
        Control::checkbox("quad_4", false),
        Control::slider("gradient_spread", 0.99, (0.0, 1.0), 0.0001),
        Control::slider("circle_radius_min", 1.0, (0.1, 12.0), 0.1),
        Control::slider("circle_radius_max", 5.0, (0.1, 12.0), 0.1),
        Control::slider("displacer_radius", 0.001, (0.0001, 0.01), 0.0001),
        Control::slider("displacer_strength", 34.0, (0.5, 100.0), 0.5),
        Control::slider("weave_frequency", 0.03, (0.01, 0.2), 0.001),
        Control::slider("weave_scale", 0.05, (0.001, 0.1), 0.001),
        Control::slider("weave_amplitude", 0.001, (0.0001, 0.01), 0.0001),
    ]);

    let mut displacer_configs = vec![
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

    let last_position_animation = controls.string("position_animation");
    let position_animations = animation_fns(&last_position_animation);
    for i in 0..displacer_configs.len() {
        displacer_configs[i].position_animation =
            position_animations[i].clone();
    }

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
        last_position_animation,
    }
}

pub fn update(app: &App, model: &mut Model, _update: Update) {
    if model.cached_trig_fns == None
        || (model.cached_pattern != model.controls.string("pattern"))
    {
        model.update_trig_fns();
    }

    if model.last_position_animation
        != model.controls.string("position_animation")
    {
        debug!("position animation changed");
        model.last_position_animation =
            model.controls.string("position_animation");
        let position_animations = animation_fns(&model.last_position_animation);
        for i in 0..model.displacer_configs.len() {
            model.displacer_configs[i].position_animation =
                position_animations[i].clone();
        }
    }

    let audio_enabled = model.controls.bool("audio_enabled");
    let clamp_circle_radii = model.controls.bool("clamp_circle_radii");
    let quad_restraint = model.controls.bool("quad_restraint");
    let qr_shape = model.controls.string("qr_shape");
    let qr_lerp = model.controls.float("qr_lerp");
    let qr_divisor = model.controls.float("qr_divisor");
    let qr_pos = model.controls.float("qr_pos");
    let qr_size = model.controls.float("qr_size");
    let gradient_spread = model.controls.float("gradient_spread");
    let displacer_radius = model.controls.float("displacer_radius");
    let displacer_strength = model.controls.float("displacer_strength");
    let weave_scale = model.controls.float("weave_scale");
    let weave_amplitude = model.controls.float("weave_amplitude");
    let pattern = model.controls.string("pattern");
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
            map_range(model.fft_bands[3], 0.0, 1.0, 0.5, displacer_strength)
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
    let time = app.time;

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
                    if QuadShape::from_str(&qr_shape).contains_point(
                        config.displacer.position * qr_pos,
                        vec2(
                            SKETCH_CONFIG.w as f32 / 3.0,
                            SKETCH_CONFIG.h as f32 / 3.0,
                        ) * qr_size,
                        *point,
                        time,
                    ) {
                        current_qr_kind = config.kind;
                        quad_contains = true;
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

            // let blended_color = gradient.get(
            //     (magnitude / max_mag).powf(gradient_spread).clamp(0.0, 1.0),
            // );

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
                        model.fft_bands[4],
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
            .resolution(12.0)
            .xy(*position);
    }

    draw.to_frame(app, &frame).unwrap();
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
        "Custo" => (x.sin() * y.cos()) + (x - y).tanh() * (x + y).tan(),
        "Sprial" => {
            (x * y).sin() * (x - y).cos() + ((x * x + y * y).sqrt()).tanh()
        }
        "Ripple" => (x * x + y * y).sin() * (x - y).cos() + (x * y).tanh(),
        "VortxFld" => {
            let radius = (x * x + y * y).sqrt();
            let angle = x.atan2(y);
            (angle * radius.sin()).cos() * radius.sin() + (x * y).cos()
        }
        "VortxFld2" => {
            let radius = (x * x + y * y).sqrt();
            let angle = (-x).atan2(-y);
            (angle * radius.sin()).cos() * radius.sin() + (x * y).cos()
        }
        "FracRipl" => (x * x - y * y).sin() + (2.0 * x * y).cos(),
        "Quantum" => x.sin() * y.cos() * (y.sin() * x.cos()).tanh(),
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

fn generate_pattern_options() -> Vec<String> {
    let functions = vec![
        "cos", "sin", "tan", "tanh", "sec", "csc", "cot", "sech", "csch",
        "coth",
    ];

    let options: Vec<String> = functions
        .iter()
        .flat_map(|a| functions.iter().map(move |b| format!("{},{}", a, b)))
        .collect();

    // Start with custom patterns
    let mut all_patterns = vec![
        "Custo".into(),
        "Sprial".into(),
        "Ripple".into(),
        "VortxFld".into(),
        "VortxFld2".into(),
        "FracRipl".into(),
        "Quantum".into(),
    ];

    // Extend with the function combinations
    all_patterns.extend(options);
    all_patterns
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

fn animation_fns(position_animations_kind: &str) -> Vec<AnimationFn<Vec2>> {
    match position_animations_kind {
        "Counter Clockwise" => animations_counter_clockwise(),
        _ => animations_none(),
    }
}

fn animations_none() -> Vec<AnimationFn<Vec2>> {
    let w = SKETCH_CONFIG.w as f32;
    let h = SKETCH_CONFIG.h as f32;

    vec![
        None,
        Some(Arc::new(move |_, _, _| vec2(w / 4.0, h / 4.0))),
        Some(Arc::new(move |_, _, _| vec2(w / 4.0, -h / 4.0))),
        Some(Arc::new(move |_, _, _| vec2(-w / 4.0, -h / 4.0))),
        Some(Arc::new(move |_, _, _| vec2(-w / 4.0, h / 4.0))),
    ]
}

fn animations_counter_clockwise() -> Vec<AnimationFn<Vec2>> {
    const BEATS: f32 = 8.0;
    vec![
        None,
        Some(Arc::new(move |_displacer, ax, _controls| {
            let w = SKETCH_CONFIG.w as f32;
            let h = SKETCH_CONFIG.h as f32;
            let xp = w / 4.0;
            let yp = h / 4.0;
            let x = ax.lerp(
                vec![
                    KF::new(xp, BEATS),   // Start at right
                    KF::new(-xp, BEATS),  // Move to left
                    KF::new(-xp, BEATS),  // Stay at left
                    KF::new(xp, BEATS),   // Move to right
                    KF::new(xp, KF::END), // Complete the cycle
                ],
                0.0,
            );
            let y = ax.lerp(
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
        Some(Arc::new(move |_displacer, ax, _controls| {
            let w = SKETCH_CONFIG.w as f32;
            let h = SKETCH_CONFIG.h as f32;
            let xp = w / 4.0;
            let yp = -h / 4.0;
            let x = ax.lerp(
                vec![
                    KF::new(xp, BEATS),   // Start at right
                    KF::new(xp, BEATS),   // Stay at right
                    KF::new(-xp, BEATS),  // Move to left
                    KF::new(-xp, BEATS),  // Stay at left
                    KF::new(xp, KF::END), // Complete the cycle
                ],
                0.0,
            );
            let y = ax.lerp(
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
        Some(Arc::new(move |_displacer, ax, _controls| {
            let w = SKETCH_CONFIG.w as f32;
            let h = SKETCH_CONFIG.h as f32;
            let xp = -w / 4.0;
            let yp = -h / 4.0;
            let x = ax.lerp(
                vec![
                    KF::new(xp, BEATS),   // Start at left
                    KF::new(-xp, BEATS),  // Move to right
                    KF::new(-xp, BEATS),  // Stay to right
                    KF::new(xp, BEATS),   // Move to left
                    KF::new(xp, KF::END), // Complete the cycle
                ],
                0.0,
            );
            let y = ax.lerp(
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
        Some(Arc::new(move |_displacer, ax, _controls| {
            let w = SKETCH_CONFIG.w as f32;
            let h = SKETCH_CONFIG.h as f32;
            let xp = -w / 4.0;
            let yp = h / 4.0;
            let x = ax.lerp(
                vec![
                    KF::new(xp, BEATS),   // Start at left
                    KF::new(xp, BEATS),   // Stay at left
                    KF::new(-xp, BEATS),  // Move to right
                    KF::new(-xp, BEATS),  // Stay at right
                    KF::new(xp, KF::END), // Complete the cycle
                ],
                0.0,
            );
            let y = ax.lerp(
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
    ]
}

fn _split_into_nonet(
    ellipses: &[(Vec2, f32, LinSrgb)],
) -> Vec<Vec<(Vec2, f32, LinSrgb)>> {
    let (min_x, max_x, min_y, max_y) = ellipses.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |(min_x, max_x, min_y, max_y), (pos, _, _)| {
            (
                min_x.min(pos.x),
                max_x.max(pos.x),
                min_y.min(pos.y),
                max_y.max(pos.y),
            )
        },
    );

    let width = max_x - min_x;
    let height = max_y - min_y;
    let section_width = width / 3.0;
    let section_height = height / 3.0;
    let mut sections = vec![Vec::new(); 9];

    // For each point
    for point in ellipses.iter() {
        let (pos, radius, color) = point;
        let col = ((pos.x - min_x) / section_width).floor() as usize;
        let row = ((pos.y - min_y) / section_height).floor() as usize;
        let section = row * 3 + col;

        if section < 9 {
            if DEBUG_QUADS {
                let debug_color = match section {
                    0 => RED.into_lin_srgb(),
                    1 => GREEN.into_lin_srgb(),
                    2 => BLUE.into_lin_srgb(),
                    3 => YELLOW.into_lin_srgb(),
                    4 => PURPLE.into_lin_srgb(),
                    5 => CYAN.into_lin_srgb(),
                    6 => ORANGE.into_lin_srgb(),
                    7 => PINK.into_lin_srgb(),
                    _ => WHITE.into_lin_srgb(),
                };

                sections[section].push((*pos, *radius, debug_color));
            } else {
                sections[section].push((*pos, *radius, *color));
            }
        }
    }

    sections
}

#[derive(Clone, Copy, PartialEq)]
pub enum QuadShape {
    Rectangle,
    Circle,
    Ripple,
    Spiral,
}

impl QuadShape {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Rectangle" => QuadShape::Rectangle,
            "Circle" => QuadShape::Circle,
            "Ripple" => QuadShape::Ripple,
            "Spiral" => QuadShape::Spiral,
            _ => QuadShape::Rectangle,
        }
    }

    pub fn contains_point(
        &self,
        center: Vec2,
        size: Vec2,
        point: Vec2,
        time: f32,
    ) -> bool {
        match self {
            QuadShape::Rectangle => {
                let rect = Rect::from_xy_wh(center, size);
                rect_contains_point(&rect, &point)
            }
            QuadShape::Circle => {
                let rect = Rect::from_xy_wh(center, size);
                circle_contains_point(&Ellipse::new(rect, 24.0), &point)
            }
            QuadShape::Ripple => {
                let dx = (point.x - center.x) / size.x;
                let dy = (point.y - center.y) / size.y;
                let distance = (dx * dx + dy * dy).sqrt();
                let wave = (distance * 10.0 + time).sin() * 0.2;
                distance < 1.0 + wave
            }
            QuadShape::Spiral => {
                let dx = (point.x - center.x) / size.x;
                let dy = (point.y - center.y) / size.y;
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);
                let spiral = (angle * 3.0 + time + distance * 4.0).sin() * 0.2;
                distance < 1.0 + spiral
            }
        }
    }
}
