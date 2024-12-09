use std::sync::Arc;
use std::sync::Mutex;

use nannou::color::named::*;
use nannou::color::*;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "audio_test",
    display_name: "Audio Test",
    fps: 30.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(300),
};

const SAMPLE_RATE: usize = 48_000;
const N_BANDS: usize = 8;

pub struct Model {
    controls: Controls,
    audio: Arc<Mutex<AudioProcessor>>,
    fft_bands: Vec<f32>,
    slew_state: SlewState,
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

    let controls = Controls::new(vec![
        Control::Select {
            name: "mode".to_string(),
            value: "log(ish)".to_string(),
            options: vec!["log(ish)".to_string(), "mel".to_string()],
        },
        Control::Slider {
            name: "pre_emphasis".to_string(),
            value: 0.88,
            min: 0.0,
            max: 1.0,
            step: 0.001,
        },
        Control::Slider {
            name: "rise_rate".to_string(),
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
    ]);

    let mut slew_state = SlewState::new(N_BANDS);
    slew_state.config = SlewConfig {
        rise_rate: controls.get_float("rise_rate"),
        fall_rate: controls.get_float("fall_rate"),
    };

    Model {
        controls,
        audio,
        fft_bands: Vec::new(),
        slew_state,
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let audio_processor = model.audio.lock().unwrap();

    let emphasized = audio_processor
        .apply_pre_emphasis(model.controls.get_float("pre_emphasis"));

    let cutoffs = if model.controls.get_string("mode") == "log(ish)" {
        audio_processor.generate_cutoffs(N_BANDS + 1, 30.0, 10_000.0)
    } else {
        audio_processor.generate_mel_cutoffs(N_BANDS + 1, 30.0, 10_000.0)
    };
    let bands = audio_processor.bands_from_buffer(&emphasized, &cutoffs);

    model.slew_state.config.rise_rate = model.controls.get_float("rise_rate");
    model.slew_state.config.fall_rate = model.controls.get_float("fall_rate");

    model.fft_bands = audio_processor.follow_band_envelopes(
        bands,
        model.slew_state.config,
        &model.slew_state.previous_values,
    );

    model.slew_state.update(model.fft_bands.clone());

    trace!("model.slew_state: {:?}", model.slew_state);
    trace!("bands: {:?}", model.fft_bands);
    trace!("cutoffs: {:?}", cutoffs);
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let w = SKETCH_CONFIG.w as f32;
    let h = SKETCH_CONFIG.h as f32;
    let draw = app.draw();

    frame.clear(BLACK);
    draw.background().color(rgb(0.2, 0.2, 0.2));

    let gradient: Gradient<LinSrgb> = Gradient::new(vec![
        PURPLE.into_lin_srgb(),
        GREEN.into_lin_srgb(),
        LIGHTBLUE.into_lin_srgb(),
    ]);

    let start_x = -w / 2.0;
    let cell_pad = 0.0;
    let cell_size = w / model.fft_bands.len() as f32;
    for (index, band) in model.fft_bands.iter().enumerate() {
        draw.rect()
            .x_y(
                start_x + index as f32 * cell_size + cell_size / 2.0,
                -h / 2.0 + (band * h) / 2.0,
            )
            .w_h(cell_size as f32 - cell_pad, band * h)
            .color(gradient.get(index as f32 / model.fft_bands.len() as f32));
    }

    draw.to_frame(app, &frame).unwrap();
}
