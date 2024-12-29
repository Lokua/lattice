use nannou::rand;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use crate::framework::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Keyframe {
    pub value: f32,
    pub duration: f32,
}

pub type KF = Keyframe;

impl Keyframe {
    pub const END: f32 = 0.0;

    pub fn new(value: f32, duration: f32) -> Self {
        Self { value, duration }
    }
}

pub fn kf(value: f32, duration: f32) -> KF {
    KF::new(value, duration)
}

pub fn kf_seq(kfs: &[(f32, f32)]) -> Vec<KF> {
    kfs.iter().map(|kf| KF::new(kf.0, kf.1)).collect()
}

#[derive(Clone, Debug)]
pub struct KeyframeRandom {
    pub range: (f32, f32),
    pub duration: f32,
}

impl KeyframeRandom {
    pub fn new(range: (f32, f32), duration: f32) -> Self {
        Self { range, duration }
    }

    // TODO: make private again after merging
    fn generate_value(&self, seed: u64) -> f32 {
        let mut rng = StdRng::seed_from_u64(seed);
        let random = rng.gen::<f32>();
        self.range.0 + (self.range.1 - self.range.0) * random
    }
}

pub type KFR = KeyframeRandom;

pub fn kfr(range: (f32, f32), duration: f32) -> KFR {
    KFR::new(range, duration)
}

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

    pub fn ping_pong(&self, duration: f32) -> f32 {
        self.ping_pong_loop_progress(duration)
    }

    pub fn ping_pong_loop_progress(&self, duration: f32) -> f32 {
        let progress = self.loop_progress(duration * 2.0);
        if progress < 0.5 {
            progress * 2.0
        } else {
            (1.0 - progress) * 2.0
        }
    }

    /// Creates a new trigger that when used in conjuction with `should_trigger`
    /// will fire at regular intervals with an optional delay.
    ///
    /// # Arguments
    /// * `every` - Number of beats between each trigger
    /// * `delay` - Offfset before trigger (in beats)
    ///
    /// # Panics
    /// * If delay is greater than or equal to the interval length (`every`)
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

    /// Checks if a trigger should fire based on the current beat position.
    /// Returns true if:
    /// 1. We're in a new interval (crossed an interval boundary)
    /// 2. We're past the delay point in the current interval
    /// 3. We haven't already triggered in this interval
    ///
    /// # Arguments
    /// * `config` - Mutable reference to a Trigger config
    ///
    /// # Returns
    /// * `true` if the trigger should fire, `false` otherwise
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

    /// Convenience version of #lerp that uses array of tuples instead of Keyframe ctor
    /// and also provides an ending Keyframe that mirrors the first Keyframe value
    /// for continuous wrapping.
    pub fn lrp(&self, kfs: &[(f32, f32)], delay: f32) -> f32 {
        let mut kfs: Vec<KF> = kfs.iter().map(|k| kf(k.0, k.1)).collect();
        kfs.push(kf(kfs[0].value, KF::END));
        self.lerp(kfs, delay)
    }

    /// Animates through keyframes with continuous linear interpolation.
    /// Each keyframe value smoothly transitions to the next over its duration.
    ///
    /// # Arguments
    /// * `keyframes` - Vector of keyframes defining values and their durations (in beats)
    /// * `delay` - Initial delay before animation starts in each keyframe (in beats)
    pub fn lerp(&self, keyframes: Vec<Keyframe>, delay: f32) -> f32 {
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

        trace!("--- lerp");
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

    /// Animates through keyframes with stepped transitions and configurable ramping.
    ///
    /// Each keyframe value is held for its full duration. After transitioning to the
    /// next keyframe, the value ramps to the next value over the specified ramp_time.
    ///
    /// # Arguments
    /// * `keyframes` - Vector of keyframes defining values and their durations
    /// * `delay` - Initial delay before ramp starts in each keyframe
    /// * `ramp_time` - Duration of transition between keyframe values (in beats)
    /// * `ramp` - Function to control the transition curve (0.0 to 1.0)
    pub fn ramp(
        &self,
        keyframes: Vec<Keyframe>,
        delay: f32,
        ramp_time: f32,
        ramp: fn(f32) -> f32,
    ) -> f32 {
        if keyframes.is_empty() {
            return 0.0;
        }

        let total_beats: f32 = keyframes.iter().map(|kf| kf.duration).sum();
        let total_frames = self.beats_to_frames(total_beats);
        let delay_frames = self.beats_to_frames(delay);
        let wrapped_frame = self.current_frame() % total_frames;

        // Handle delay period
        if wrapped_frame < delay_frames {
            return keyframes[0].value;
        }

        // No ramping at absolute start
        if self.current_frame() <= self.beats_to_frames(ramp_time) {
            return keyframes[0].value;
        }

        // Find current segment
        let mut current_segment_index = 0;
        let mut beats_elapsed = 0.0;

        for (index, kf) in keyframes.iter().enumerate() {
            beats_elapsed += kf.duration;
            let frames_elapsed = self.beats_to_frames(beats_elapsed);

            if wrapped_frame < frames_elapsed {
                current_segment_index = index;
                break;
            }
        }

        // Get the current keyframe
        let current_keyframe = &keyframes[current_segment_index];

        // Calculate position within current segment
        let segment_start_beats: f32 = keyframes
            .iter()
            .take(current_segment_index)
            .map(|kf| kf.duration)
            .sum();

        let frame_in_segment =
            wrapped_frame - self.beats_to_frames(segment_start_beats);
        let time_in_segment = frame_in_segment / self.beats_to_frames(1.0);

        let previous_index = if current_segment_index == 0 {
            keyframes.len() - 1
        } else {
            current_segment_index - 1
        };
        let previous_value = keyframes[previous_index].value;

        let ramp_progress = time_in_segment / ramp_time;
        let clamped_progress = ramp_progress.clamp(0.0, 1.0);
        let eased_progress = ramp(clamped_progress);

        let value = if time_in_segment <= ramp_time {
            lerp(previous_value, current_keyframe.value, eased_progress)
        } else {
            current_keyframe.value
        };

        trace!("--- ramp");
        trace!("wrapped_frame: {}", wrapped_frame);
        trace!("total_beats: {}", total_beats);
        trace!("total_frames: {}", total_frames);
        trace!("current_segment_index: {}", current_segment_index);
        trace!("current_keyframe.value: {}", current_keyframe.value);
        trace!("previous_value: {}", previous_value);
        trace!("segment_start_beats: {}", segment_start_beats);
        trace!("frame_in_segment: {}", frame_in_segment);
        trace!("time_in_segment: {}", time_in_segment);
        trace!("clamped_progress: {}", clamped_progress);
        trace!("eased_progress: {}", eased_progress);
        trace!("value: {}", value);
        trace!("---");

        value
    }

    pub fn r_ramp(
        &self,
        keyframes: &[KeyframeRandom],
        delay: f32,
        ramp_time: f32,
        ramp: fn(f32) -> f32,
    ) -> f32 {
        if keyframes.is_empty() {
            return 0.0;
        }

        let total_beats: f32 = keyframes.iter().map(|kf| kf.duration).sum();
        let total_frames = self.beats_to_frames(total_beats);
        let delay_frames = self.beats_to_frames(delay);
        let wrapped_frame = self.current_frame() % total_frames;
        let current_cycle = (self.current_frame() / total_frames) as u64;

        // Handle initial case - first frame of first cycle
        if self.current_frame() == 0.0 {
            return keyframes[0].generate_value(0);
        }

        // Find current segment
        let mut current_segment_index = 0;
        let mut beats_elapsed = 0.0;

        for (index, kf) in keyframes.iter().enumerate() {
            beats_elapsed += kf.duration;
            let frames_elapsed = self.beats_to_frames(beats_elapsed);

            if wrapped_frame < frames_elapsed {
                current_segment_index = index;
                break;
            }
        }

        // Calculate position within current segment
        let segment_start_beats: f32 = keyframes
            .iter()
            .take(current_segment_index)
            .map(|kf| kf.duration)
            .sum();

        let frame_in_segment =
            wrapped_frame - self.beats_to_frames(segment_start_beats);

        // Generate current value
        let current_value = if current_segment_index == 0 {
            keyframes[0].generate_value(current_cycle)
        } else {
            keyframes[current_segment_index]
                .generate_value(current_cycle * keyframes.len() as u64)
        };

        // Generate previous value
        let previous_value = if current_segment_index == 0 {
            if current_cycle == 0 {
                keyframes[0].generate_value(0)
            } else {
                keyframes[keyframes.len() - 1].generate_value(
                    (current_cycle - 1) * keyframes.len() as u64,
                )
            }
        } else {
            keyframes[current_segment_index - 1]
                .generate_value(current_cycle * keyframes.len() as u64)
        };

        // Handle delay period
        if frame_in_segment < delay_frames {
            return previous_value;
        }

        // Calculate ramp progress after delay
        let adjusted_frames = frame_in_segment - delay_frames;
        let ramp_frames = self.beats_to_frames(ramp_time);
        let ramp_progress = adjusted_frames / ramp_frames;
        let clamped_progress = ramp_progress.clamp(0.0, 1.0);
        let eased_progress = ramp(clamped_progress);

        // Return interpolated or final value
        if adjusted_frames <= ramp_frames {
            lerp(previous_value, current_value, eased_progress)
        } else {
            current_value
        }
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
    fn test_lerp_returns_initial_value() {
        init(0);
        let a = create_instance();
        let result = a
            .lerp(vec![Keyframe::new(99.0, 1.0), Keyframe::new(1.0, 0.0)], 0.0);
        assert_eq!(result, 99.0, "returns 0 at frame 0");
    }

    #[test]
    #[serial]
    fn test_lerp_returns_halfway_point() {
        init(2);
        let a = create_instance();
        let result =
            a.lerp(vec![Keyframe::new(0.0, 1.0), Keyframe::new(1.0, 0.0)], 0.0);
        assert_eq!(result, 0.5, "returns 0.5 when 1/2 between 0 and 1");
    }

    #[test]
    #[serial]
    fn test_lerp_consistently_returns_correct_value() {
        init(0);
        let a = create_instance();
        let times = vec![0.5, 1.5];

        for beats in times {
            let frame_count = a.beats_to_frames(beats) as u32;
            frame_controller::set_frame_count(frame_count);
            let result = a.lerp(
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
        let mut trigger = animation.create_trigger(1.0, 0.0);

        assert!(
            animation.should_trigger(&mut trigger),
            "should trigger at start"
        );

        init(1);
        assert!(
            !animation.should_trigger(&mut trigger),
            "should not trigger mid-beat"
        );

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
        let mut trigger = animation.create_trigger(2.0, 0.5);

        assert!(
            !animation.should_trigger(&mut trigger),
            "should not trigger at start due to delay"
        );

        init(2);
        assert!(
            animation.should_trigger(&mut trigger),
            "should trigger at delay point"
        );

        init(4);
        assert!(
            !animation.should_trigger(&mut trigger),
            "should not trigger before next interval"
        );

        init(10);
        assert!(
            animation.should_trigger(&mut trigger),
            "should trigger at next interval after delay"
        );
    }

    #[test]
    #[serial]
    fn test_ramp_basic() {
        init(0);
        let a = create_instance();

        // Test at start (frame 0)
        let result =
            a.ramp(vec![KF::new(0.0, 2.0), KF::new(1.0, 2.0)], 0.0, 0.5, |x| x);
        assert_eq!(result, 0.0, "should start at initial value");

        // Test just before end of first keyframe (frame 7)
        init(7);
        let result =
            a.ramp(vec![KF::new(0.0, 2.0), KF::new(1.0, 2.0)], 0.0, 0.5, |x| x);
        println!("Frame 7 result: {}", result);
        assert_eq!(
            result, 0.0,
            "should still be at initial value just before duration end"
        );

        // Test at exact end of first keyframe (frame 8)
        init(8);
        let result =
            a.ramp(vec![KF::new(0.0, 2.0), KF::new(1.0, 2.0)], 0.0, 0.5, |x| x);
        println!("Frame 8 result: {}", result);
        assert_eq!(result, 0.0, "should start ramping after first keyframe");

        // Test one frame into ramp (frame 9)
        init(9);
        let result =
            a.ramp(vec![KF::new(0.0, 2.0), KF::new(1.0, 2.0)], 0.0, 0.5, |x| x);
        println!("Frame 9 result: {}", result);
        let time_in_segment = 9.0 / a.beats_to_frames(1.0);
        println!("Time in segment at frame 9: {}", time_in_segment);
        println!("Beats to frames(1.0): {}", a.beats_to_frames(1.0));
        assert!(
            result > 0.45 && result < 0.55,
            "should be halfway through ramp"
        );

        // Test at end of ramp (frame 10)
        init(10);
        let result =
            a.ramp(vec![KF::new(0.0, 2.0), KF::new(1.0, 2.0)], 0.0, 0.5, |x| x);
        println!("Frame 10 result: {}", result);
        assert_eq!(result, 1.0, "should reach final value after ramp");
    }

    #[test]
    #[serial]
    fn test_ramp_with_delay() {
        init(0);
        let a = create_instance();
        let result =
            a.ramp(vec![KF::new(0.0, 2.0), KF::new(1.0, 2.0)], 1.0, 0.5, |x| x);
        assert_eq!(result, 0.0, "should return initial value during delay");
    }

    #[test]
    #[serial]
    fn test_ramp_first_vs_subsequent_cycles() {
        init(0);
        let a = create_instance();

        let result =
            a.ramp(vec![KF::new(1.0, 2.0), KF::new(0.0, 2.0)], 0.0, 0.5, |x| x);
        assert_eq!(
            result, 1.0,
            "first cycle should start at value without ramping"
        );

        init(17);
        let result =
            a.ramp(vec![KF::new(1.0, 2.0), KF::new(0.0, 2.0)], 0.0, 0.5, |x| x);
        assert!(
            result > 0.45 && result < 0.55,
            "subsequent cycles should ramp between values"
        );
    }

    #[test]
    #[serial]
    fn test_r_ramp_delay_at_keyframe_transition() {
        init(8); // Start at second keyframe transition
        let a = create_instance();

        let previous_value = a.r_ramp(
            &[KFR::new((0.0, 1.0), 2.0), KFR::new((2.0, 3.0), 2.0)],
            0.5, // 0.5 beat delay
            0.5,
            |x| x,
        );

        // Move just past keyframe boundary but still within delay period
        init(9);
        let delayed_value = a.r_ramp(
            &[KFR::new((0.0, 1.0), 2.0), KFR::new((2.0, 3.0), 2.0)],
            0.5,
            0.5,
            |x| x,
        );

        assert_eq!(
            previous_value, delayed_value,
            "Should maintain previous value"
        );
    }

    #[test]
    #[serial]
    fn test_r_ramp_single_keyframe() {
        init(0);
        let a = create_instance();

        let first_value =
            a.r_ramp(&[KFR::new((0.0, 1.0), 2.0)], 0.0, 0.5, |x| x);

        // Move to frame 8 (start of next cycle, still previous value)
        init(8);
        let previous_value =
            a.r_ramp(&[KFR::new((0.0, 1.0), 2.0)], 0.0, 0.5, |x| x);

        // Values should be the same at start of cycles
        assert_eq!(
            first_value, previous_value,
            "Should generate same value at start of cycle"
        );

        // Move to frame 9 (should be halfway through transition)
        init(9);
        let mid_value = a.r_ramp(&[KFR::new((0.0, 1.0), 2.0)], 0.0, 0.5, |x| x);

        // Move to frame 10 (should be at new value)
        init(10);
        let new_value = a.r_ramp(&[KFR::new((0.0, 1.0), 2.0)], 0.0, 0.5, |x| x);

        let expected_midpoint = (previous_value + new_value) / 2.0;
        let tolerance = 0.001;
        assert!(
            (mid_value - expected_midpoint).abs() < tolerance,
            "Value at frame 9 should be halfway between {} and {}. Got {}",
            previous_value,
            new_value,
            mid_value
        );
    }

    #[test]
    #[serial]
    fn test_r_ramp_post_delay_transition() {
        // Frame 8 would be the start of second keyframe
        // Frame 10 = frame 8 + delay (0.5 beats = 2 frames)
        // Frame 11 should be in the middle of the transition
        init(11);
        let a = create_instance();

        let value = a.r_ramp(
            &[KFR::new((0.0, 1.0), 2.0), KFR::new((2.0, 3.0), 2.0)],
            0.5, // 0.5 beats delay
            1.0, // 1.0 beats ramp time
            |x| x,
        );

        // At this point, we should be transitioning from first keyframe value (0.0-1.0)
        // to second keyframe value (2.0-3.0)
        let first_keyframe_value = KFR::new((0.0, 1.0), 2.0).generate_value(0);
        let second_keyframe_value = KFR::new((2.0, 3.0), 2.0).generate_value(1);

        let min_expected = first_keyframe_value.min(second_keyframe_value);
        let max_expected = first_keyframe_value.max(second_keyframe_value);

        assert!(
            value >= min_expected && value <= max_expected,
            "Value {} should be between {} and {} during transition",
            value,
            min_expected,
            max_expected
        );
    }

    #[test]
    #[serial]
    fn test_r_ramp_consistent_interpolation() {
        let a = create_instance();
        let keyframes = vec![KFR::new((0.0, 1.0), 1.0)];
        let ramp_frames = vec![1, 5, 9, 13, 17, 21, 25, 29, 33, 37];

        let mut results = vec![];

        for frame in 0..40 {
            init(frame);
            let value = a.r_ramp(&keyframes, 0.0, 0.5, linear);
            results.push((frame, value));
        }

        for frame in ramp_frames {
            // Skip the first frame
            if frame == 1 {
                continue;
            }

            // Find results with safe error handling
            let prev_result = results
                .iter()
                .find(|&&(f, _)| f == frame - 1)
                .expect(&format!("No result for previous frame {}", frame - 1));
            let curr_result = results
                .iter()
                .find(|&&(f, _)| f == frame)
                .expect(&format!("No result for current frame {}", frame));
            let next_result = results
                .iter()
                .find(|&&(f, _)| f == frame + 1)
                .expect(&format!("No result for next frame {}", frame + 1));

            let expected_midpoint = (prev_result.1 + next_result.1) / 2.0;

            assert!(
                (curr_result.1 - expected_midpoint).abs() < 0.001,
                "Frame {}: Interpolation incorrect. \
                Previous: {}, Current: {}, Next: {}, Expected Midpoint: {}",
                frame,
                prev_result.1,
                curr_result.1,
                next_result.1,
                expected_midpoint
            );
        }
    }
}
