use crate::framework::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Keyframe {
    pub value: f32,
    pub duration: f32,
}
impl Keyframe {
    pub const END: f32 = 0.0;
    pub fn new(value: f32, duration: f32) -> Self {
        Self { value, duration }
    }
}

pub type KF = Keyframe;

pub struct Trigger {
    every: f32,
    delay: f32,
    last_trigger_count: f32,
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

    /// Returns total beats elapsed since start
    pub fn get_total_beats_elapsed(&self) -> f32 {
        self.current_frame() / self.beats_to_frames(1.0)
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
        let total_beats: f32 = keyframes
            .iter()
            .take(keyframes.len() - 1)
            .map(|kf| kf.duration)
            .sum();

        let total_frames = self.beats_to_frames(total_beats);
        let delay_frames = self.beats_to_frames(delay);
        let wrapped_frame = self.current_frame() % total_frames;

        if wrapped_frame < delay_frames {
            return keyframes[0].value;
        }
        if wrapped_frame >= total_frames {
            return keyframes[keyframes.len() - 1].value;
        }

        let mut current_segment_index = 0;
        for (index, _kf) in keyframes.iter().enumerate() {
            let duration_to_here: f32 =
                keyframes.iter().take(index + 1).map(|kf| kf.duration).sum();

            let frames_to_here = self.beats_to_frames(duration_to_here);

            if wrapped_frame < frames_to_here {
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
            wrapped_frame - self.beats_to_frames(segment_start_beats);
        let segment_progress =
            frame_in_segment / self.beats_to_frames(current_keyframe.duration);
        let value = lerp(
            current_keyframe.value,
            next_keyframe.value,
            segment_progress,
        );

        trace!("---");
        trace!("wrapped_frame: {}", wrapped_frame);
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

    pub fn create_trigger(&self, every: f32, delay: f32) -> Trigger {
        if delay >= every {
            panic!("Delay must be less than interval length");
        }

        Trigger {
            every,
            delay,
            last_trigger_count: -1.0,
        }
    }

    pub fn should_trigger(&self, config: &mut Trigger) -> bool {
        let total_beats_elapsed = self.get_total_beats_elapsed();
        let current_interval = (total_beats_elapsed / config.every).floor();
        let position_in_interval = total_beats_elapsed % config.every;

        let should_trigger = current_interval != config.last_trigger_count
            && position_in_interval >= config.delay;

        if should_trigger {
            config.last_trigger_count = current_interval;
        }

        should_trigger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::Once;

    // this way each 1/16 = 1 frame, 4 frames per beat,
    // less likely to deal with precision issues.
    const FPS: f32 = 24.0;
    const BPM: f32 = 360.0;

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
    #[serial]
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
    #[serial]
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
    #[serial]
    fn test_animate_consistently_returns_correct_value() {
        init(0);
        let a = create_instance();
        let times = vec![0.5, 1.5];

        for beats in times {
            let frame_count = a.beats_to_frames(beats) as u32;
            frame_controller::set_frame_count(frame_count);
            let result = a.animate(
                vec![
                    Keyframe::new(0.0, 1.0),
                    Keyframe::new(1.0, 1.0),
                    Keyframe::new(0.0, 0.0),
                ],
                0.0,
            );
            assert_eq!(result, 0.5, "returns the last keyframe value");
        }
    }

    #[test]
    #[serial]
    fn test_trigger_on_beat() {
        init(0);
        let animation = Animation::new(BPM);
        let mut trigger = animation.create_trigger(1.0, 0.0); // Trigger every beat with no delay

        // At frame 0 (start of first beat)
        assert!(
            animation.should_trigger(&mut trigger),
            "should trigger at start"
        );

        // At frame 1 (still in first beat)
        init(1);
        assert!(
            !animation.should_trigger(&mut trigger),
            "should not trigger mid-beat"
        );

        // At frame 4 (start of second beat)
        init(4);
        assert!(
            animation.should_trigger(&mut trigger),
            "should trigger at next beat"
        );
    }

    #[test]
    #[serial]
    fn test_trigger_with_delay() {
        init(0);
        let animation = Animation::new(BPM);
        let mut trigger = animation.create_trigger(2.0, 0.5); // Trigger every 2 beats with 0.5 beat delay

        // At frame 0 (start)
        assert!(
            !animation.should_trigger(&mut trigger),
            "should not trigger at start due to delay"
        );

        // At frame 2 (0.5 beats in - delay point)
        init(2);
        assert!(
            animation.should_trigger(&mut trigger),
            "should trigger at delay point"
        );

        // At frame 4 (1.0 beats in)
        init(4);
        assert!(
            !animation.should_trigger(&mut trigger),
            "should not trigger before next interval"
        );

        // At frame 10 (2.5 beats in - next trigger point after delay)
        init(10);
        assert!(
            animation.should_trigger(&mut trigger),
            "should trigger at next interval after delay"
        );
    }
}
