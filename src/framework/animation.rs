use crate::framework::prelude::*;

pub struct Keyframe {
    pub value: f32,
    pub duration: f32,
}
impl Keyframe {
    pub fn new(value: f32, duration: f32) -> Self {
        Self { value, duration }
    }
}

pub struct Animation {
    bpm: f32,
}
impl Animation {
    pub fn new(bpm: f32) -> Self {
        Self { bpm }
    }

    /// Retrieves the current frame count from the frame controller.
    /// returns f32 for math convenience (frame_controller uses u32)
    fn current_frame(&self) -> f32 {
        frame_controller::frame_count() as f32
    }

    /// Retrieves the current frame count from the frame controller.
    /// returns f32 for math convenience (frame_controller uses u32)
    fn fps(&self) -> f32 {
        frame_controller::fps()
    }

    /// Converts beats to frames based on BPM and FPS.
    pub fn beats_to_frames(&self, beats: f32) -> f32 {
        let seconds_per_beat = 60.0 / self.bpm;
        let total_seconds = beats * seconds_per_beat;
        total_seconds * self.fps()
    }

    pub fn loop_progress(&self, duration: f32) -> f32 {
        let frame_count = self.current_frame();
        if frame_count == 0.0 || frame_count == 1.0 {
            debug!("frame_count init {}", frame_count);
        }
        let fps = frame_controller::fps();
        let beat_duration = 60.0 / self.bpm;
        let total_frames = beat_duration * duration * fps;
        let current_frame = frame_count % total_frames;
        let progress = current_frame / total_frames as f32;
        progress
    }

    pub fn ping_pong_loop_progress(&self, duration: f32) -> f32 {
        let progress = self.loop_progress(duration * 2.0);
        if progress < 0.5 {
            progress * 2.0
        } else {
            (1.0 - progress) * 2.0
        }
    }

    pub fn animate(&self, keyframes: Vec<Keyframe>, delay: f32) -> f32 {
        let current_frame = self.current_frame();

        let total_beats: f32 = keyframes
            .iter()
            .take(keyframes.len() - 1)
            .map(|kf| kf.duration)
            .sum();

        let total_frames = self.beats_to_frames(total_beats);
        let delay_frames = self.beats_to_frames(delay);

        if current_frame < delay_frames {
            return keyframes[0].value;
        }

        if current_frame >= total_frames {
            return keyframes[keyframes.len() - 1].value;
        }

        let mut current_segment_index = 0;
        for (index, _kf) in keyframes.iter().enumerate() {
            let duration_to_here: f32 =
                keyframes.iter().take(index + 1).map(|kf| kf.duration).sum();

            if current_frame < duration_to_here {
                current_segment_index = index;
                break;
            }
        }

        let current_keyframe = &keyframes[current_segment_index];
        let next_keyframe = &keyframes[current_segment_index + 1];

        let segment_start_beats: f32 = keyframes
            .iter()
            .take(current_segment_index)
            .map(|kf| kf.duration)
            .sum();

        let frame_in_segment =
            current_frame - self.beats_to_frames(segment_start_beats);
        let segment_progress =
            frame_in_segment / self.beats_to_frames(current_keyframe.duration);
        let value = lerp(
            current_keyframe.value,
            next_keyframe.value,
            segment_progress,
        );

        trace!("---");
        trace!("current_frame: {}", current_frame);
        trace!("total_beats: {}", total_beats);
        trace!("total_frames: {}", total_frames);
        trace!("current_segment_index: {}", current_segment_index);
        trace!("current_keyframe.value: {}", current_keyframe.value);
        trace!("next_keyframe.value: {}", next_keyframe.value);
        trace!("segment_start_beats: {}", segment_start_beats);
        trace!("frame_in_segment: {}", frame_in_segment);
        trace!("segment_progress: {}", segment_progress);
        trace!("value: {}", value);
        trace!("---");

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    // this way each 1/16 = 1 frame, 4 frames per beat,
    // less likely to deal with precision issues.
    const FPS: f32 = 24.0;
    const BPM: f32 = 360.0;

    // uncomment to ensure animate works the same with different rates
    // const FRAME_RATE: f32 = 60.0;
    // const BPM: f32 = 120.0;

    static INIT: Once = Once::new();

    fn init(frame_count: u32) {
        INIT.call_once(|| {
            env_logger::builder().is_test(true).init();
            frame_controller::ensure_controller(FPS);
        });
        frame_controller::set_frame_count(frame_count);
    }

    fn create_instance() -> Animation {
        Animation::new(BPM)
    }

    #[test]
    fn test_animate_returns_initial_value() {
        init(0);
        let a = create_instance();
        let result = a.animate(
            vec![Keyframe::new(99.0, 1.0), Keyframe::new(1.0, 0.0)],
            0.0,
        );
        assert_eq!(result, 99.0, "returns 0 at frame 0");
    }

    #[test]
    fn test_animate_returns_halfway_point() {
        init(2);
        let a = create_instance();
        let result = a.animate(
            vec![Keyframe::new(0.0, 1.0), Keyframe::new(1.0, 0.0)],
            0.0,
        );
        assert_eq!(result, 0.5, "returns 0.5 when 1/2 between 0 and 1");
    }

    #[test]
    fn test_animate_consistently_returns_correct_value() {
        let times = vec![1.5];
        let a = create_instance();
        init(0);

        for beats in times {
            frame_controller::set_frame_count(a.beats_to_frames(beats) as u32);
            let result = a.animate(
                vec![Keyframe::new(1.0, 3.0), Keyframe::new(3.0, 0.0)],
                0.0,
            );
            assert_eq!(result, 2.0, "returns the last keyframe value");
        }
    }
}
