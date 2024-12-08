use std::sync::Arc;
use std::sync::Mutex;

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::*;
use cpal::BuildStreamError;
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
    radius1: f32,
    radius2: f32,
    fft_radii: Vec<f32>,
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
        name: "radius_max".to_string(),
        value: 333.0,
        min: 10.0,
        max: 500.0,
        step: 1.0,
    }]);

    Model {
        controls,
        audio,
        radius1: 0.0,
        radius2: 0.0,
        fft_radii: Vec::new(),
    }
}

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    let audio_processor = model.audio.lock().unwrap();
    let radius_max = model.controls.get_float("radius_max");

    model.radius1 =
        map_range(audio_processor.peak(), -1.0, 1.0, 0.0, radius_max);

    model.radius2 =
        map_range(audio_processor.rms(), -1.0, 1.0, 0.0, radius_max);

    let bands = audio_processor.bands(3);
    debug!("bands: {:?}", bands);
    model.fft_radii = bands
        .iter()
        .map(|&x| map_range(x, 0.0, 1.0, 0.0, radius_max))
        .collect();
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let radius_max = model.controls.get_float("radius_max");

    frame.clear(BLACK);
    draw.background().color(rgb(0.2, 0.2, 0.2));

    draw.ellipse()
        .no_fill()
        .stroke(RED)
        .stroke_weight(10.0)
        .radius(model.radius1)
        .x_y(0.0, 0.0);

    draw.ellipse()
        .no_fill()
        .stroke_weight(10.0)
        .stroke(LIGHTSALMON)
        .radius(model.radius2)
        .x_y(0.0, 0.0);

    for radius in &model.fft_radii {
        draw.ellipse()
            .no_fill()
            .stroke_weight(5.0)
            .stroke(rgba(
                1.0,
                1.0,
                1.0,
                map_range(*radius, 0.0, radius_max, 0.0, 1.0),
            ))
            .radius(*radius)
            .x_y(0.0, 0.0);
    }

    draw.to_frame(app, &frame).unwrap();
}

fn init_audio(
    shared_audio: Arc<Mutex<AudioProcessor>>,
) -> Result<(), BuildStreamError> {
    let audio_host = cpal::default_host();
    let devices: Vec<_> = audio_host.output_devices().unwrap().collect();
    let target_device_name = "BlackHole 2ch";

    let device = devices
        .into_iter()
        .find(|device| {
            let name = device.name().unwrap();
            debug!("Enumerating devices. Device name: {}", name);
            device.name().unwrap() == target_device_name
        })
        .expect(
            format!("No device named {} found", target_device_name).as_str(),
        );

    let output_config = match device.default_output_config() {
        Ok(config) => {
            debug!("Default output stream config: {:?}", config);
            config
        }
        Err(err) => {
            panic!("Failed to get default output config: {:?}", err);
        }
    };

    let stream = match output_config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &output_config.into(),
            move |source_data: &[f32], _| {
                // Left = even, Right = odd
                // `data.iter().skip(1).step_by(2)` for right
                let data: Vec<f32> =
                    source_data.iter().step_by(2).cloned().collect();
                let mut audio_processor = shared_audio.lock().unwrap();
                audio_processor.add_samples(&data);
            },
            move |err| error!("An error occured on steam: {}", err),
            None,
        )?,
        sample_format => {
            panic!("Unsupported sample format {}", sample_format);
        }
    };

    let _ = stream.play();

    Ok(())
}
