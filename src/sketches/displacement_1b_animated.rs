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
    gui_h: Some(490),
};

const GRID_SIZE: usize = 128;
const USE_TRIANGLES: bool = true;

#[derive(SketchComponents)]
#[sketch(clear_color = "hsl(0.0, 0.0, 0.02, 0.5)")]
pub struct Model {
    window_rect: WindowRect,
    grid: Vec<Vec2>,
    displacer_configs: Vec<DisplacerConfig>,
    animation: Animation,
    controls: Controls,
    gradient: Gradient<LinSrgb>,
    objects: Vec<(Vec2, f32, f32, LinSrgb)>,
}

pub fn init_model(window_rect: WindowRect) -> Model {
    let w = SKETCH_CONFIG.w;
    let h = SKETCH_CONFIG.h;
    let grid_w = w as f32 - 80.0;
    let grid_h = h as f32 - 80.0;
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let modes = string_vec!["attract", "influence"];

    let controls = Controls::new(vec![
        Control::checkbox("show_center", true),
        Control::select("center_mode", "attract", modes.clone()),
        Control::Separator {},
        Control::checkbox("show_corner", false),
        Control::select("corner_mode", "attract", modes.clone()),
        Control::Separator {},
        Control::checkbox("show_trbl", false),
        Control::select("trbl_mode", "attract", modes.clone()),
        Control::Separator {},
        Control::checkbox("flip", false),
        Control::select(
            "sort",
            "luminance",
            string_vec!["luminance", "radius"],
        ),
        Control::checkbox("stroke", false),
        Control::slider_x(
            "stroke_weight",
            1.0,
            (0.25, 3.0),
            0.25,
            |controls| !controls.bool("stroke"),
        ),
        Control::slider("gradient_spread", 0.5, (0.0, 1.0), 0.0001),
        Control::slider("background_alpha", 1.0, (0.000, 1.0), 0.001),
        Control::slider("alpha", 1.0, (0.001, 1.0), 0.001),
        Control::slider("size_max", 2.5, (0.1, 20.0), 0.1),
        Control::slider("t_scale", 1.0, (1.0, 200.0), 1.0),
        Control::slider("scaling_power", 3.0, (0.5, 11.0), 0.25),
        Control::slider("mag_mult", 1.0, (1.0, 500.0), 1.0),
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
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Trbl,
            Displacer::new_with_position(vec2(0.0, h4th)),
        ),
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Trbl,
            Displacer::new_with_position(vec2(w4th, 0.0)),
        ),
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Trbl,
            Displacer::new_with_position(vec2(0.0, -h4th)),
        ),
        DisplacerConfig::new_no_anim(
            DisplacerConfigKind::Trbl,
            Displacer::new_with_position(vec2(-w4th, 0.0)),
        ),
    ];

    Model {
        window_rect,
        grid: create_grid(grid_w, grid_h, GRID_SIZE, vec2).0,
        displacer_configs,
        animation,
        controls,
        gradient: Gradient::new(vec![
            CYAN.into_lin_srgb(),
            ORANGE.into_lin_srgb(),
            MAGENTA.into_lin_srgb(),
            GREEN.into_lin_srgb(),
        ]),
        objects: Vec::with_capacity(GRID_SIZE * GRID_SIZE),
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let show_center = model.controls.bool("show_center");
    let center_mode = model.controls.string("center_mode");
    let show_corner = model.controls.bool("show_corner");
    let corner_mode = model.controls.string("corner_mode");
    let show_trbl = model.controls.bool("show_trbl");
    let trbl_mode = model.controls.string("trbl_mode");
    let sort = model.controls.string("sort");
    let size_max = model.controls.float("size_max");
    let gradient_spread = model.controls.float("gradient_spread");
    let scaling_power = model.controls.float("scaling_power");
    let t_scale = model.controls.float("t_scale");

    let max_mag =
        model.displacer_configs.len() as f32 * model.controls.float("mag_mult");
    let gradient = &model.gradient;

    if model.window_rect.changed() {
        (model.grid, _) = create_grid(
            model.window_rect.w(),
            model.window_rect.h(),
            GRID_SIZE,
            vec2,
        );
        model.window_rect.commit();
    }

    let configs: Vec<&mut DisplacerConfig> = model
        .displacer_configs
        .iter_mut()
        .filter(|config| match config.kind {
            DisplacerConfigKind::Center => show_center,
            DisplacerConfigKind::Corner => show_corner,
            DisplacerConfigKind::Trbl => show_trbl,
        })
        .map(|config| {
            config.update(&model.animation, &model.controls);
            match config.kind {
                DisplacerConfigKind::Center => {
                    config.displacer.set_strength(model.animation.r_ramp(
                        vec![KFR::new((1.0, 500.0), 8.0)],
                        0.0,
                        4.0,
                        linear,
                    ));
                    config.displacer.set_radius(model.animation.r_ramp(
                        vec![KFR::new((1.0, 500.0), 12.0)],
                        1.0,
                        3.0,
                        linear,
                    ));
                }
                DisplacerConfigKind::Corner => {
                    config.displacer.set_strength(model.animation.r_ramp(
                        vec![KFR::new((1.0, 500.0), 4.0)],
                        0.0,
                        4.0,
                        linear,
                    ));
                    config.displacer.set_radius(model.animation.r_ramp(
                        vec![KFR::new((1.0, 500.0), 8.0)],
                        1.0,
                        3.0,
                        linear,
                    ));
                }
                DisplacerConfigKind::Trbl => {
                    config.displacer.set_strength(model.animation.r_ramp(
                        vec![KFR::new((1.0, 500.0), 16.0)],
                        0.0,
                        6.0,
                        linear,
                    ));
                    config.displacer.set_radius(model.animation.r_ramp(
                        vec![KFR::new((1.0, 500.0), 24.0)],
                        2.0,
                        18.0,
                        linear,
                    ));
                }
            }
            config
        })
        .collect();

    model.objects = model
        .grid
        .par_iter()
        .map(|point| {
            let total_displacement =
                configs.iter().fold(vec2(0.0, 0.0), |acc, config| {
                    let mode = match config.kind {
                        DisplacerConfigKind::Center => &center_mode,
                        DisplacerConfigKind::Corner => &corner_mode,
                        DisplacerConfigKind::Trbl => &trbl_mode,
                    };

                    acc + if mode == "attract" {
                        config.displacer.attract(*point, scaling_power)
                    } else {
                        config.displacer.core_influence(*point, scaling_power)
                    }
                });

            let displacement_magnitude = total_displacement.length();
            let triangle_height =
                map_range(displacement_magnitude, 0.0, max_mag, 1.0, t_scale);

            let color = gradient.get(
                1.0 - (displacement_magnitude / max_mag)
                    .powf(gradient_spread)
                    .clamp(0.0, 1.0),
            );

            let radius = map_clamp(
                displacement_magnitude,
                0.0,
                max_mag,
                0.1,
                size_max,
                ease_out,
            );

            (*point + total_displacement, radius, triangle_height, color)
        })
        .collect();

    model.objects.sort_by(
        |(_position_a, radius_a, _triangle_height_a, color_a),
         (_position_b, radius_b, _triangle_height_b, color_b)| {
            match sort.as_str() {
                "radius" => radius_a.partial_cmp(radius_b).unwrap(),
                _ => {
                    luminance(color_a).partial_cmp(&luminance(color_b)).unwrap()
                }
            }
        },
    );
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.rect()
        .w_h(model.window_rect.w(), model.window_rect.h())
        .color(hsla(
            0.0,
            0.0,
            0.02,
            model.controls.float("background_alpha"),
        ));

    let alpha = model.controls.float("alpha");
    let stroke = model.controls.bool("stroke");
    let stroke_weight = model.controls.float("stroke_weight");
    let flip = model.controls.bool("flip");

    for (position, radius, triangle_height, color) in &model.objects {
        if USE_TRIANGLES {
            // Calculate outward direction from center to position
            let outward_dir = if position.length_squared() == 0.0 {
                vec2(1.0, 0.0)
            } else {
                (*position - vec2(0.0, 0.0)).normalize()
            };

            let radius = radius.max(0.01);

            // Triangle dimensions
            let height = radius; // Distance from the center to the tip
            let base_width = radius; // Width of the base

            // Calculate vertices
            let tip = *position
                + outward_dir
                    * height
                    * *triangle_height
                    * if flip { -1.0 } else { 1.0 };

            // Perpendicular vector for the base
            let perpendicular = vec2(-outward_dir.y, outward_dir.x);

            let base_left = *position - perpendicular * (base_width / 2.0);
            let base_right = *position + perpendicular * (base_width / 2.0);

            draw.polygon()
                .stroke(if stroke {
                    hsla(0.0, 0.0, 0.0, 1.0)
                } else {
                    hsla(0.0, 0.0, 0.0, 0.0)
                })
                .stroke_weight(stroke_weight)
                .points(vec![tip, base_left, base_right])
                .color(lin_srgb_to_lin_srgba(*color, alpha));
        } else {
            let rect = draw
                .rect()
                .color(lin_srgb_to_lin_srgba(*color, alpha))
                .w_h(*radius, *radius)
                .xy(*position);

            if stroke {
                rect.stroke(BLACK).stroke_weight(stroke_weight);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

type AnimationFn<R> =
    Option<Arc<dyn Fn(&Displacer, &Animation, &Controls) -> R + Send + Sync>>;

enum DisplacerConfigKind {
    Center,
    Corner,
    Trbl,
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
