use crate::framework::frame_controller::{get_fps, get_frame_count};

pub struct Animation {
    bpm: f32,
}

impl Animation {
    pub fn new(bpm: f32) -> Self {
        Self { bpm }
    }

    pub fn get_loop_progress(&self, duration: f32) -> f32 {
        let beat_duration = 60.0 / self.bpm;
        let total_frames =
            (beat_duration * duration * get_fps() as f32).round() as u64;
        let current_frame = get_frame_count() % total_frames;
        let progress = current_frame as f32 / total_frames as f32;
        progress as f32
    }

    pub fn get_ping_pong_loop_progress(&self, duration: f32) -> f32 {
        let progress = self.get_loop_progress(duration * 2.0);
        if progress < 0.5 {
            progress * 2.0
        } else {
            (1.0 - progress) * 2.0
        }
    }
}
