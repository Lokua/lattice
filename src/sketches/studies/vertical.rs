use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "vertical",
    display_name: "Vertical",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(500),
};

pub struct Model {
    #[allow(dead_code)]
    animation: Animation,
    controls: Controls,
    lines: Vec<Vec<Point2>>,
    patterns: Vec<XModFn>,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

pub fn init_model() -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let mode_options = [string_vec!["multi_lerp"], XMods::to_names()].concat();

    let controls = Controls::new(vec![
        Control::slider("scale", 1.0, (0.1, 4.0), 0.1),
        Control::select("mode", "per_line", mode_options),
        Control::slider("n_lines", 64.0, (16.0, 256.0), 2.0),
        Control::slider("amplitude", 20.0, (0.0, 300.0), 1.0),
        Control::slider("frequency", 0.1, (0.0, 0.1), 0.00001),
        Control::slider("weight", 1.0, (0.1, 4.0), 0.1),
        Control::slider_x(
            "x_line_scaling",
            0.1,
            (0.0, 0.5),
            0.01,
            disabled_unless_modes(&[]),
        ),
        Control::slider("x_phase_shift", 0.1, (0.0, 1.0), 0.01),
        Control::slider("x_harmonic_ratio", 2.0, (1.0, 4.0), 0.1),
        Control::slider("x_distance_scaling", 0.05, (0.0, 0.2), 0.01),
        Control::slider("x_complexity", 1.0, (0.1, 3.0), 0.1),
    ]);

    let lines = Vec::with_capacity(controls.float("n_lines") as usize);

    Model {
        animation,
        controls,
        lines,
        patterns: XMods::to_vec(),
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let w = SKETCH_CONFIG.w as f32;
    let h = SKETCH_CONFIG.h as f32;
    let n_lines = model.controls.float("n_lines") as usize;
    let a = model.controls.float("amplitude");
    let f = model.controls.float("frequency");

    let params = XModParams {
        line_scaling: model.controls.float("x_line_scaling"),
        phase_shift: model.controls.float("x_phase_shift"),
        harmonic_ratio: model.controls.float("x_harmonic_ratio"),
        distance_scaling: model.controls.float("x_distance_scaling"),
        complexity: model.controls.float("x_complexity"),
    };

    model.lines = Vec::new();
    let step = w / n_lines as f32;
    let start_x = -(w / 2.0) + (step / 2.0);

    for i in 0..n_lines {
        let x = start_x + i as f32 * step;
        let mut points = Vec::new();

        for j in 0..n_lines {
            let y = map_range(j, 0, n_lines - 1, -h / 2.0, h / 2.0);

            let x = match model.controls.string("mode").as_str() {
                "multi_lerp" => {
                    let values = model
                        .patterns
                        .iter()
                        .map(|func| {
                            func(x, y, i as f32, a, f, n_lines as f32, &params)
                        })
                        .collect::<Vec<f32>>();

                    multi_lerp(&values, model.animation.ping_pong(24.0))
                }
                _ => {
                    let func =
                        XMods::func_by_name(model.controls.string("mode"));
                    func(x, y, i as f32, a, f, n_lines as f32, &params)
                }
            };

            points.push(pt2(x, y));
        }

        model.lines.push(points);
    }
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let window_rect = app
        .window(frame.window_id())
        .expect("Unable to get window")
        .rect();

    let draw = app.draw();

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(window_rect.w(), window_rect.h())
        .hsla(0.0, 0.0, 0.02, 0.7);

    let zoomed_draw = draw.scale(model.controls.float("scale"));

    for (index, line) in model.lines.iter().enumerate() {
        zoomed_draw
            .polyline()
            .weight(model.controls.float("weight"))
            .points(line.iter().cloned())
            .color(hsla(
                0.44,
                index as f32 / model.lines.len() as f32,
                0.45,
                0.3,
            ));
    }

    draw.to_frame(app, &frame).unwrap();
}

pub fn disabled_unless_modes(
    modes: &[&str],
) -> impl Fn(&ControlValues) -> bool {
    let modes: Vec<String> = modes.iter().map(|s| s.to_string()).collect();
    move |controls| {
        if let Some(ControlValue::String(mode)) = controls.get("mode") {
            !modes.contains(mode)
        } else {
            true // Disable if mode control doesn't exist or isn't a string
        }
    }
}

type XModFn = fn(f32, f32, f32, f32, f32, f32, &XModParams) -> f32;

struct XModParams {
    line_scaling: f32,
    phase_shift: f32,
    harmonic_ratio: f32,
    distance_scaling: f32,
    complexity: f32,
}

impl XModParams {
    #[allow(dead_code)]
    fn default() -> Self {
        Self {
            line_scaling: 0.1,
            phase_shift: 0.1,
            harmonic_ratio: 2.0,
            distance_scaling: 0.05,
            complexity: 1.0,
        }
    }
}

struct XMods {}

impl XMods {
    fn to_names() -> Vec<String> {
        string_vec![
            "per_line",
            "ripples",
            "line_phase",
            "spiral",
            "wave_interference",
            "harmonic_cascade",
            "fractal_waves",
            "moire",
            "standing_waves",
            "quantum_ripples"
        ]
    }

    fn func_by_name(name: String) -> XModFn {
        match name.as_str() {
            "per_line" => XMods::per_line,
            "ripples" => XMods::ripples,
            "line_phase" => XMods::line_phase,
            "spiral" => XMods::spiral,
            "wave_interference" => XMods::wave_interference,
            "harmonic_cascade" => XMods::harmonic_cascade,
            "fractal_waves" => XMods::fractal_waves,
            "moire" => XMods::moire,
            "standing_waves" => XMods::standing_waves,
            "quantum_ripples" => XMods::quantum_ripples,
            _ => panic!("No function named '{name}'"),
        }
    }

    fn to_vec() -> Vec<XModFn> {
        XMods::to_names()
            .into_iter()
            .map(XMods::func_by_name)
            .collect()
    }

    fn per_line(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        _n: f32,
        p: &XModParams,
    ) -> f32 {
        let freq = f * (1.0 + i * p.line_scaling);
        x + a * (y * freq).sin()
    }

    fn ripples(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        n: f32,
        p: &XModParams,
    ) -> f32 {
        let distance_from_center = (i - n / 2.0).abs();
        let freq = f * (1.0 + distance_from_center * p.distance_scaling);
        x + a * (y * freq).sin()
    }

    fn line_phase(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        _n: f32,
        p: &XModParams,
    ) -> f32 {
        let phase = i * p.phase_shift;
        x + a * (y * f + phase).sin() * (y * f * p.harmonic_ratio + phase).cos()
    }

    fn spiral(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        n: f32,
        p: &XModParams,
    ) -> f32 {
        let angle = (i / n) * TAU;
        let radius = ((x * x + y * y).sqrt() * f).sin();
        x + a * (radius * angle * p.complexity).sin()
    }

    fn wave_interference(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        _n: f32,
        p: &XModParams,
    ) -> f32 {
        let wave1 = (x * f + i * p.phase_shift).sin();
        let wave2 = (y * f * p.harmonic_ratio + i * p.phase_shift).sin();
        x + a * wave1 * wave2 * p.complexity
    }

    fn harmonic_cascade(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        _n: f32,
        p: &XModParams,
    ) -> f32 {
        let base_wave = (y * f).sin();
        let harmonic1 = (y * f * p.harmonic_ratio).sin() * 0.5;
        let harmonic2 = (y * f * p.harmonic_ratio * 2.0).sin() * 0.25;
        x + a * (base_wave + harmonic1 + harmonic2) * (1.0 + i * p.line_scaling)
    }

    fn fractal_waves(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        _n: f32,
        p: &XModParams,
    ) -> f32 {
        let mut sum = 0.0;
        let mut amplitude = a;
        let mut frequency = f;

        for _ in 0..3 {
            sum += (y * frequency + i * p.phase_shift).sin() * amplitude;
            amplitude *= 0.5;
            frequency *= p.harmonic_ratio;
        }
        x + sum * p.complexity
    }

    fn moire(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        n: f32,
        p: &XModParams,
    ) -> f32 {
        let pattern1 = (x * f + i * p.phase_shift).sin();
        let pattern2 =
            (y * f * p.harmonic_ratio + (n - i) * p.phase_shift).sin();
        x + a * pattern1 * pattern2 * p.complexity
    }

    fn standing_waves(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        _n: f32,
        p: &XModParams,
    ) -> f32 {
        let spatial = (x * f).sin() * (y * f).cos();
        let temporal = (i * p.phase_shift).cos();
        x + a * spatial * temporal * p.complexity
    }

    fn quantum_ripples(
        x: f32,
        y: f32,
        i: f32,
        a: f32,
        f: f32,
        n: f32,
        p: &XModParams,
    ) -> f32 {
        let distance = ((x * x + y * y).sqrt() + i * p.line_scaling) * f;
        let interference =
            (distance * p.harmonic_ratio).sin() * (distance / n).cos();
        x + a * interference * p.complexity
    }
}
