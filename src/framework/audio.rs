use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

pub struct AudioProcessor {
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

    pub fn bands(&self, n_bands: usize) -> Vec<f32> {
        let mut buffer: Vec<Complex<f32>> =
            self.buffer.iter().map(|&x| Complex::new(x, 0.0)).collect();

        self.fft.process(&mut buffer);

        let magnitudes: Vec<f32> = buffer.iter().map(|c| c.norm()).collect();

        let band_magnitudes: Vec<f32> = magnitudes
            .chunks(magnitudes.len() / (n_bands + 1))
            .map(|band| band.iter().sum::<f32>())
            .collect();

        let max_band_magnitude =
            band_magnitudes.iter().cloned().fold(0.0, f32::max);

        if max_band_magnitude == 0.0 {
            // Avoid division by zero
            return vec![0.0; 4];
        }

        magnitudes
            .chunks(magnitudes.len() / (n_bands + 1))
            // normalize 0-1
            .map(|band| band.iter().sum::<f32>() / max_band_magnitude)
            .skip(1)
            .collect()
    }
}
