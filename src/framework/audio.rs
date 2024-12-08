use cpal::traits::*;
use cpal::BuildStreamError;
use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::sync::{Arc, Mutex};

use super::prelude::*;

pub struct AudioProcessor {
    sample_rate: usize,
    buffer: Vec<f32>,
    buffer_size: usize,
    fft: Arc<dyn Fft<f32>>,
}

impl AudioProcessor {
    pub fn new(sample_rate: usize, frame_rate: f32) -> Self {
        let buffer_size = (sample_rate as f32 / frame_rate).ceil() as usize;
        let mut planner: FftPlanner<f32> = FftPlanner::new();
        let fft = planner.plan_fft_forward(buffer_size);

        Self {
            sample_rate,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
            fft,
        }
    }

    pub fn add_samples(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);

        if self.buffer.len() > self.buffer_size {
            self.buffer.drain(0..(self.buffer.len() - self.buffer_size));
        }
    }

    pub fn peak(&self) -> f32 {
        self.buffer.iter().fold(f32::MIN, |a, &b| f32::max(a, b))
    }

    pub fn rms(&self) -> f32 {
        (self.buffer.iter().map(|&x| x * x).sum::<f32>()
            / self.buffer.len() as f32)
            .sqrt()
    }

    /// Standard pre-emphasis filter: y[n] = x[n] - α * x[n-1]
    /// 0.97 is common is it gives about +20dB emphasis starting around 1kHz
    pub fn apply_pre_emphasis(&self, coefficient: f32) -> Vec<f32> {
        let mut filtered = Vec::with_capacity(self.buffer.len());
        filtered.push(self.buffer[0]);

        for i in 1..self.buffer.len() {
            filtered.push(self.buffer[i] - coefficient * self.buffer[i - 1]);
        }

        filtered
    }

    pub fn bands(&self, cutoffs: Vec<usize>) -> Vec<f32> {
        self.bands_from_buffer(&self.buffer, cutoffs)
    }

    // TODO: call this bands_from_buffer, use that in bands so we
    // can compose with apply_pre_emphasis etc.
    pub fn bands_from_buffer(
        &self,
        buffer: &Vec<f32>,
        cutoffs: Vec<usize>,
    ) -> Vec<f32> {
        let mut complex_input: Vec<Complex<f32>> =
            buffer.iter().map(|&x| Complex::new(x, 0.0)).collect();

        self.fft.process(&mut complex_input);

        let freq_resolution = (self.sample_rate / complex_input.len()) as f32;

        let stops: Vec<usize> = cutoffs
            .iter()
            .map(|cutoff| (*cutoff as f32 / freq_resolution).round() as usize)
            .collect();

        // Calculate magnitude and convert to dB for each bin
        let magnitudes: Vec<f32> = complex_input
            .iter()
            .map(|c| {
                let magnitude = c.norm() / complex_input.len() as f32;
                // Reduced noise floor to -80dB for better high frequency sensitivity
                20.0 * (magnitude.max(1e-8)).log10()
            })
            .collect();

        let get_band_magnitude = |start: usize, end: usize| -> f32 {
            // Ensure we don't go above Nyquist frequency
            let slice = &magnitudes[start..end.min(magnitudes.len())];
            if slice.is_empty() {
                return -80.0;
            }
            // Use peak detection instead of RMS
            *slice
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        };

        // Normalize to 0-1 range, considering typical dB ranges
        let normalize = |db: f32| {
            // Map -80dB to 0.0 and -20dB to 1.0 (increased sensitivity)
            ((db + 80.0) / 60.0).max(0.0).min(1.0)
        };

        let bands: Vec<f32> = stops
            .iter()
            .take(stops.len() - 1)
            .enumerate()
            .map(|(index, &stop)| get_band_magnitude(stop, stops[index + 1]))
            .map(normalize)
            .collect();

        bands
    }

    // TODO: make variable bands
    pub fn bands_3_lin(&self) -> (f32, f32, f32) {
        let mut complex_input: Vec<Complex<f32>> =
            self.buffer.iter().map(|&x| Complex::new(x, 0.0)).collect();

        self.fft.process(&mut complex_input);
        let freq_resolution = (self.sample_rate / complex_input.len()) as f32;

        let low_band_start = (20.0 / freq_resolution).round() as usize;
        let low_band_end = (200.0 / freq_resolution).round() as usize;
        let mid_band_end = (2000.0 / freq_resolution).round() as usize;
        let high_band_end = (20000.0 / freq_resolution).round() as usize;

        let magnitudes: Vec<f32> = complex_input
            .iter()
            .map(|c| (c.norm() / complex_input.len() as f32))
            .collect();

        let get_band_magnitude = |start: usize, end: usize| -> f32 {
            // Ensure we don't go above Nyquist frequency
            magnitudes[start..end.min(magnitudes.len())]
                .iter()
                .fold(0.0f32, |acc, &x| acc.max(x))
        };

        let low_band = get_band_magnitude(low_band_start, low_band_end);
        let mid_band = get_band_magnitude(low_band_end, mid_band_end);
        let high_band = get_band_magnitude(mid_band_end, high_band_end);

        let max_magnitude = low_band.max(mid_band).max(high_band);

        let normalize = |x: f32| {
            if max_magnitude > 0.0 {
                x / max_magnitude
            } else {
                0.0
            }
        };

        (
            normalize(low_band),
            normalize(mid_band),
            normalize(high_band),
        )
    }
}

pub fn init_audio(
    shared_audio: Arc<Mutex<AudioProcessor>>,
) -> Result<(), BuildStreamError> {
    let audio_host = cpal::default_host();
    let devices: Vec<_> = audio_host.input_devices().unwrap().collect();
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

    let input_config = match device.default_input_config() {
        Ok(config) => {
            debug!("Default output stream config: {:?}", config);
            config
        }
        Err(err) => {
            panic!("Failed to get default output config: {:?}", err);
        }
    };

    let stream = match input_config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &input_config.into(),
            move |source_data: &[f32], _| {
                // Left = even, Right = odd;
                // Do `data.iter().skip(1).step_by(2)` for right
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
