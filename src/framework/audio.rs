pub struct AudioProcessor {
    buffer: Vec<f32>,
    // Calculated as audio_frames_per_visual_frame
    buffer_size: usize,
}

impl AudioProcessor {
    pub fn new(sample_rate: usize, frame_rate: f32) -> Self {
        let buffer_size = (sample_rate as f32 / frame_rate).ceil() as usize;
        Self {
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }

    pub fn add_samples(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);
        if self.buffer.len() > self.buffer_size {
            self.buffer.drain(0..(self.buffer.len() - self.buffer_size));
        }
    }

    pub fn peak(&self) -> f32 {
        self.buffer.iter().cloned().fold(f32::MIN, f32::max)
    }

    pub fn rms(&self) -> f32 {
        (self.buffer.iter().map(|&x| x * x).sum::<f32>()
            / self.buffer.len() as f32)
            .sqrt()
    }
}
