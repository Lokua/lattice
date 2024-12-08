use std::sync::Arc;
use std::sync::Mutex;

use nannou::color::named::*;
use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "audio_test",
    display_name: "Audio Test",
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(150),
};

const SAMPLE_RATE: usize = 48_000;

pub struct Model {
    controls: Controls,
    audio: Arc<Mutex<AudioProcessor>>,
    fft_bands: Vec<f32>,
}

impl SketchModel for Model {
    fn controls(&mut self) -> Option<&mut Controls> {
        Some(&mut self.controls)
    }
}

pub fn init_model() -> Model {
    let audio = Arc::new(Mutex::new(AudioProcessor::new(
        SAMPLE_RATE,
        SKETCH_CONFIG.fps,
    )));
    init_audio(audio.clone()).expect("Unable to init audio");

    let controls = Controls::new(vec![Control::Slider {
        name: "pre_emphasis".to_string(),
        value: 1.0,
        min: 0.001,
        max: 0.0,
        step: 0.001,
    }]);

    Model {
        controls,
        audio,
        fft_bands: Vec::new(),
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let audio_processor = model.audio.lock().unwrap();

    model.fft_bands =
        audio_processor.bands(vec![30, 100, 200, 500, 1000, 4000, 10_000]);
    debug!("bands: {:?}", model.fft_bands);
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let w = SKETCH_CONFIG.w as f32;
    let h = SKETCH_CONFIG.h as f32;
    let draw = app.draw();

    frame.clear(BLACK);
    draw.background().color(rgb(0.2, 0.2, 0.2));

    let gradient: Gradient<LinSrgb> = Gradient::new(vec![
        BLUE.into_lin_srgb(),
        PURPLE.into_lin_srgb(),
        RED.into_lin_srgb(),
        GREEN.into_lin_srgb(),
        YELLOW.into_lin_srgb(),
    ]);

    let start_x = -w / 2.0;
    let cell_size = w / model.fft_bands.len() as f32;
    for (index, band) in model.fft_bands.iter().enumerate() {
        draw.rect()
            .x_y(
                start_x + index as f32 * cell_size + cell_size / 2.0,
                -h / 2.0 + (band * h) / 2.0,
            )
            .w_h(cell_size as f32, band * h)
            .color(gradient.get(index as f32 / model.fft_bands.len() as f32));
    }

    draw.to_frame(app, &frame).unwrap();
}
