//! Animation module providing timing-synchronized animation and transition
//! controls.
//!
//! This module offers a flexible system for creating time-based animations that
//! are synchronized to musical time (beats) through various timing sources. The
//! core animation system supports:
//!
//! - Beat-synchronized timing with support for various clock sources (MIDI,
//!   OSC, manual)
//! - Simple oscillations and phase-based animations
//! - Linear and eased transitions between keyframes
//! - Complex automation curves with configurable breakpoints
//! - Randomized transitions with constraints
//!
//! # Musical Timing
//!
//! All animations are synchronized to musical time expressed in beats, where
//! one beat equals one quarter note. The animation system can be driven by
//! different timing sources (Frame, MIDI, OSC) that provide the current beat
//! position, allowing animations to stay in sync with external music software
//! or hardware.
//!
//! # Basic Usage
//!
//! ```rust
//! let animation = Animation::new(Timing::new(ctx.bpm()));
//!
//! // Simple oscillation between 0-1 over 4 beats
//! let phase = animation.loop_phase(4.0); // Returns 0.0 to 1.0
//!
//! // Triangle wave oscillation between ranges
//! let value = animation.triangle(
//!     // Duration in beats
//!     4.0,
//!     // Min/max range
//!     (0.0, 100.0),  
//!     // Phase offset
//!     0.0,           
//! );
//! ```
//!
//! # Advanced Automation
//!
//! The [`Animation::automate`] method provides DAW-style automation curves with
//! multiple breakpoint types and transition modes:
//!
//! ```rust
//! let value = animation.automate(
//!     &[
//!         // Start with a step change
//!         Breakpoint::step(0.0, 0.0),
//!         // Ramp with exponential easing
//!         Breakpoint::ramp(1.0, 1.0, Easing::EaseInExpo),
//!         // Add amplitude modulation
//!         Breakpoint::wave(
//!             // Position in beats
//!             2.0,
//!             // Base value
//!             0.5,
//!             Shape::Sine,
//!             // Frequency in beats
//!             0.25,
//!             // Width
//!             0.5,
//!             // Amplitude
//!             0.25,
//!             Easing::Linear,
//!             Constrain::None,
//!         ),
//!         // Mark end of sequence
//!         Breakpoint::end(4.0, 0.0),
//!     ],
//!     Mode::Loop
//! );
//! ```
//!
//! The module supports both one-shot and looping animations, with precise
//! control over timing, transitions, and modulation. All animations
//! automatically sync to the provided timing source's beat position.

use std::cell::RefCell;
use std::collections::HashMap;

use nannou::math::map_range;
use nannou::rand;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use serde::Deserialize;

use super::frame_controller;
use super::prelude::*;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Keyframe {
    pub value: f32,
    pub duration: f32,
}

impl Keyframe {
    pub fn new(value: f32, duration: f32) -> Self {
        Self { value, duration }
    }
}

/// Convenience method to create keyframes without
/// the verbosity of [`Keyframe::new`]
pub fn kf(value: f32, duration: f32) -> Keyframe {
    Keyframe::new(value, duration)
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

    pub fn generate_value(&self, seed: u64) -> f32 {
        let mut rng = StdRng::seed_from_u64(seed);
        let random = rng.gen::<f32>();
        self.range.0 + (self.range.1 - self.range.0) * random
    }
}

/// Convenience method to create keyframes without
/// the verbosity of [`KeyframeRandom::new`]
pub fn kfr(range: (f32, f32), duration: f32) -> KeyframeRandom {
    KeyframeRandom::new(range, duration)
}

/// Data structure used in conjunction with
/// [`Animation::create_trigger`] and [`Animation::should_trigger`]
pub struct Trigger {
    every: f32,
    delay: f32,
    last_trigger_count: f32,
}

#[derive(Clone, Debug)]
pub struct Animation<T: TimingSource> {
    pub timing: T,
    random_smooth_previous_values: RefCell<HashMap<u64, f32>>,
}

impl<T: TimingSource> Animation<T> {
    pub fn new(timing: T) -> Self {
        Self {
            timing,
            random_smooth_previous_values: RefCell::new(HashMap::new()),
        }
    }

    /// Return the number of beats that have elapsed
    /// since (re)start of this Animation's Timing source
    pub fn beats(&self) -> f32 {
        self.timing.beats()
    }

    /// Convert `beats` to frame count
    pub fn beats_to_frames(&self, beats: f32) -> f32 {
        let seconds_per_beat = 60.0 / self.timing.bpm();
        let total_seconds = beats * seconds_per_beat;
        total_seconds * frame_controller::fps()
    }

    /// Return a relative phase position from [0, 1] within
    /// the passed in duration (specified in beats)
    pub fn loop_phase(&self, duration: f32) -> f32 {
        let total_beats = self.beats();
        (total_beats / duration) % 1.0
    }

    /// Cycle from 0 to 1 and back to 0 over the passed in duration
    /// See [`Self::triangle`] for an advanced version with more options
    pub fn tri(&self, duration: f32) -> f32 {
        let x = (self.beats() / duration) % 1.0;
        ternary!(x < 0.5, x, 1.0 - x) * 2.0
    }

    /// Cycle from `min` to `max` and back to `min` in exactly `duration`
    /// beats. `phase_offset` in [0.0..1.0] shifts our position in that cycle.
    /// Only positive offsets are supported.
    pub fn triangle(
        &self,
        duration: f32,
        (min, max): (f32, f32),
        phase_offset: f32,
    ) -> f32 {
        let mut x = (self.beats() / duration + phase_offset.abs() * 0.5) % 1.0;
        x = ternary!(x < 0.5, x, 1.0 - x) * 2.0;
        map_range(x, 0.0, 1.0, min, max)
    }

    /// Generate a randomized value once during every cycle of `duration`. The
    /// function is completely deterministic given the same parameters in
    /// relation to the current beat.
    pub fn random(
        &self,
        duration: f32,
        (min, max): (f32, f32),
        stem: u64,
    ) -> f32 {
        let loop_count = (self.beats() / duration).floor();
        let seed = stem + ((duration + (max - min) + loop_count) as u64);
        let mut rng = StdRng::seed_from_u64(seed);
        rng.gen_range(min..=max)
    }

    /// Generate a randomized value once during every cycle of `duration`. The
    /// function is completely deterministic given the same parameters in
    /// relation to the current beat. The `stem` - which serves as the root of
    /// an internal seed generator - is also a unique ID for internal slew state
    /// and for that reason you should make sure all animations in your sketch
    /// have unique stems. `slew` controls smoothing when the value changes with
    /// 0.0 being instant and 1.0 being essentially frozen.
    pub fn random_slewed(
        &self,
        duration: f32,
        (min, max): (f32, f32),
        slew: f32,
        stem: u64,
    ) -> f32 {
        let loop_count = (self.beats() / duration).floor();

        let seed = stem
            + duration.to_bits() as u64
            + (max - min).to_bits() as u64
            + loop_count as u64;

        let mut rng = StdRng::seed_from_u64(seed);
        let value = rng.gen_range(min..=max);

        let mut prev_values = self.random_smooth_previous_values.borrow_mut();
        let value = prev_values.get(&stem).map_or(value, |prev| {
            SlewLimiter::slew_pure(*prev, value, slew, slew)
        });

        prev_values.insert(stem, value);

        value
    }

    /// Creates a new [`Trigger`] with specified interval and delay;
    /// Use with [`Self::should_trigger`].
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

    /// Checks if a trigger should fire based on current beat position.
    /// When used with [`Self::create_trigger`], provides a means
    /// of executing arbitrary code at specific intervals
    ///
    /// ```rust
    /// // Do something once every 4 bars
    /// if animation.should_trigger(animation.create_trigger(16.0, 0.0)) {
    ///   // do stuff
    /// }
    /// ```
    pub fn should_trigger(&self, config: &mut Trigger) -> bool {
        let total_beats = self.beats();
        let current_interval = (total_beats / config.every).floor();
        let position_in_interval = total_beats % config.every;

        let should_trigger = current_interval != config.last_trigger_count
            && position_in_interval >= config.delay;

        if should_trigger {
            config.last_trigger_count = current_interval;
        }

        should_trigger
    }

    /// Convenience version of lerp that automatically adds a final keyframe
    /// to ensure a continuous loop back to the first keyframe, since this
    /// is such a common pattern.
    ///
    /// ## Example
    /// ```rust
    /// // from 0 to 1 over 4 beats
    /// // then 1 to 0 over 4 beats
    /// animation.lrp(&[kf(0.0, 4.0), kf(1.0, 4.0)], 0.0);
    ///
    /// // Here is the `lerp` equivalent:
    /// animation.lerp(
    ///     &[
    ///         kf(0.0, 4.0),
    ///         kf(1.0, 4.0),
    ///         kf(0.0, 0.0),
    ///     ],
    ///     0.0
    /// );
    /// ```
    pub fn lrp(&self, kfs: &[Keyframe], delay: f32) -> f32 {
        let mut kfs = Vec::from(kfs);
        kfs.push(kf(kfs[0].value, 0.0));
        self.lerp(&kfs, delay)
    }

    /// Animates through keyframes with continuous linear interpolation
    ///
    /// ## Example
    /// ```rust
    /// animation.lerp(
    ///     &[
    ///         // start at 0 and ramp to the next value (1)
    ///         // over 4 beats
    ///         keyframe::new(0.0, 4.0),
    ///         // start at 1 and ramp down to the next value (0)
    ///         // over 4 beats
    ///         keyframe::new(1.0, 4.0),
    ///         // the final keyframe is only to inform the 2nd
    ///         // to last keyframe where it should end; the duration
    ///         // argument is simply ignored so we use 0.0
    ///         // to be explicit about that
    ///         keyframe::new(0.0, 0.0),
    ///     ],
    ///     // no delay
    ///     0.0
    /// );
    /// ```
    pub fn lerp(&self, keyframes: &[Keyframe], delay: f32) -> f32 {
        let total_beats: f32 = keyframes
            .iter()
            .take(keyframes.len() - 1)
            .map(|kf| kf.duration)
            .sum();

        let current_beat = self.beats();
        let delay_beats = delay;
        let wrapped_beat = current_beat % total_beats;

        if wrapped_beat < delay_beats {
            return keyframes[0].value;
        }
        if wrapped_beat >= total_beats {
            return keyframes[keyframes.len() - 1].value;
        }

        let mut current_segment_index = 0;
        for (index, _kf) in keyframes.iter().enumerate() {
            let duration_to_here: f32 =
                keyframes.iter().take(index + 1).map(|kf| kf.duration).sum();

            if wrapped_beat < duration_to_here {
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

        let beat_in_segment = wrapped_beat - segment_start_beats;
        let segment_progress = beat_in_segment / current_keyframe.duration;

        let value = lerp(
            current_keyframe.value,
            next_keyframe.value,
            segment_progress,
        );

        trace!("current_beat: {}, value: {}", current_beat, value);

        value
    }

    /// Animates through keyframes with stepped transitions and configurable
    /// ramping. Ramps happen at the start of a segment. If duration is 1.0 and
    /// ramp_time is 0.5, the first 1/2 beat will be a ramp from the previous
    /// value to this value, then another 1/2 beat of the value held. The first
    /// time the keyframe sequence is played, there is no initial ramp, because
    /// there is no previous value to ramp from.
    ///
    /// ## Example
    /// ```rust
    /// animation.ramp(
    ///     &[
    ///         // The first time this animation executes:
    ///         // stay at 0.0 for 1 beat
    ///         Keyframe::new(0.0, 1.0),
    ///         // Ramp from 0.0 to 1.0 over 1/2 beat...
    ///         // then stay at 1.0 for the next 1/2 beat
    ///         Keyframe::new(1.0, 1.0),
    ///         // Now loop back to 1st keyframe, but this
    ///         // time ramp from 1.0 to 0.0 over 1/2 beat
    ///         // then stay there for the remaining 1/2 beat
    ///     ],
    ///     0.5,
    ///     linear  
    /// );
    /// ```
    pub fn ramp(
        &self,
        keyframes: &[Keyframe],
        delay: f32,
        ramp_time: f32,
        ramp: Easing,
    ) -> f32 {
        if keyframes.is_empty() {
            return 0.0;
        }

        let total_beats: f32 = keyframes.iter().map(|kf| kf.duration).sum();
        let wrapped_beat = self.beats() % total_beats;
        let delay_beats = delay;

        if wrapped_beat < delay_beats {
            return keyframes[0].value;
        }

        // No ramping at absolute start
        if self.beats() <= ramp_time {
            return keyframes[0].value;
        }

        let mut current_segment_index = 0;
        let mut beats_elapsed = 0.0;

        for (index, kf) in keyframes.iter().enumerate() {
            beats_elapsed += kf.duration;
            if wrapped_beat < beats_elapsed {
                current_segment_index = index;
                break;
            }
        }

        let current_keyframe = &keyframes[current_segment_index];

        let segment_start_beats: f32 = keyframes
            .iter()
            .take(current_segment_index)
            .map(|kf| kf.duration)
            .sum();

        let beat_in_segment = wrapped_beat - segment_start_beats;
        let time_in_segment = beat_in_segment;

        let previous_index = if current_segment_index == 0 {
            keyframes.len() - 1
        } else {
            current_segment_index - 1
        };
        let previous_value = keyframes[previous_index].value;

        let ramp_progress = time_in_segment / ramp_time;
        let clamped_progress = ramp_progress.clamp(0.0, 1.0);
        let eased_progress = ramp.apply(clamped_progress);

        if time_in_segment <= ramp_time {
            lerp(previous_value, current_keyframe.value, eased_progress)
        } else {
            current_keyframe.value
        }
    }

    /// Same as `ramp` only chooses random values from specified ranges instead
    /// of constant values. See [`Self::ramp`] for details about how/when
    /// ramping occurs.
    pub fn r_ramp(
        &self,
        keyframes: &[KeyframeRandom],
        delay: f32,
        ramp_time: f32,
        ramp: Easing,
        // ramp: fn(f32) -> f32,
    ) -> f32 {
        if keyframes.is_empty() {
            return 0.0;
        }

        let total_beats: f32 = keyframes.iter().map(|kf| kf.duration).sum();
        let beats_elapsed = self.beats();
        let wrapped_beat = beats_elapsed % total_beats;
        let delay_beats = delay;

        let cycle_float = (beats_elapsed / total_beats) + 1e-9;
        let current_cycle = cycle_float.floor() as u64;

        if beats_elapsed == 0.0 {
            return keyframes[0].generate_value(0);
        }

        let mut current_segment_index = 0;
        let mut beats_elapsed = 0.0;

        for (index, kf) in keyframes.iter().enumerate() {
            beats_elapsed += kf.duration;
            if wrapped_beat < beats_elapsed {
                current_segment_index = index;
                break;
            }
        }

        let segment_start_beats: f32 = keyframes
            .iter()
            .take(current_segment_index)
            .map(|kf| kf.duration)
            .sum();

        let beat_in_segment = wrapped_beat - segment_start_beats;

        let current_value = if current_segment_index == 0 {
            keyframes[0].generate_value(current_cycle * keyframes.len() as u64)
        } else {
            keyframes[current_segment_index].generate_value(
                current_cycle * keyframes.len() as u64
                    + current_segment_index as u64,
            )
        };

        let previous_value = if current_segment_index == 0 {
            if current_cycle == 0 {
                keyframes[keyframes.len() - 1].generate_value(0)
            } else {
                keyframes[keyframes.len() - 1].generate_value(
                    (current_cycle - 1) * keyframes.len() as u64
                        + (keyframes.len() - 1) as u64,
                )
            }
        } else {
            keyframes[current_segment_index - 1].generate_value(
                current_cycle * keyframes.len() as u64
                    + (current_segment_index - 1) as u64,
            )
        };

        if beat_in_segment < delay_beats {
            return previous_value;
        }

        let adjusted_beats = beat_in_segment - delay_beats;
        let ramp_progress = adjusted_beats / ramp_time;
        let clamped_progress = ramp_progress.clamp(0.0, 1.0);
        let eased_progress = ramp.apply(clamped_progress);

        let value = if adjusted_beats <= ramp_time {
            lerp(previous_value, current_value, eased_progress)
        } else {
            current_value
        };

        trace!("adjusted_beats: {}, ramp_progress: {}, clamped_progress: {}, eased_progress: {}, value: {}", 
            adjusted_beats, ramp_progress, clamped_progress, eased_progress, value
        );

        value
    }
}

#[cfg(test)]
pub mod animation_tests {
    use super::*;
    use serial_test::serial;
    use std::sync::Once;

    // this way each 1/16 = 1 frame, 4 frames per beat,
    // less likely to deal with precision issues.
    pub const FPS: f32 = 24.0;
    pub const BPM: f32 = 360.0;

    static INIT: Once = Once::new();

    pub fn init(frame_count: u32) {
        INIT.call_once(|| {
            env_logger::builder().is_test(true).init();
            frame_controller::ensure_controller(FPS);
        });
        frame_controller::set_frame_count(frame_count);
    }

    pub fn create_instance() -> Animation<FrameTiming> {
        Animation::new(FrameTiming::new(Bpm::new(BPM)))
    }

    #[test]
    #[serial]
    fn test_tri() {
        init(0);
        let a = create_instance();

        let val = a.tri(2.0);
        assert_eq!(val, 0.0, "1/16");

        init(1);
        let val = a.tri(2.0);
        assert_eq!(val, 0.25, "2/16");

        init(2);
        let val = a.tri(2.0);
        assert_eq!(val, 0.5, "3/16");

        init(3);
        let val = a.tri(2.0);
        assert_eq!(val, 0.75, "4/16");

        init(4);
        let val = a.tri(2.0);
        assert_eq!(val, 1.0, "5/16");

        init(5);
        let val = a.tri(2.0);
        assert_eq!(val, 0.75, "6/16");

        init(6);
        let val = a.tri(2.0);
        assert_eq!(val, 0.5, "7/16");

        init(7);
        let val = a.tri(2.0);
        assert_eq!(val, 0.25, "8/16");

        init(8);
        let val = a.tri(2.0);
        assert_eq!(val, 0.0, "9/16");
    }

    #[test]
    #[serial]
    fn test_triangle_8beats_positive_offset() {
        init(0);
        let a = create_instance();

        let val = a.triangle(4.0, (-1.0, 1.0), 0.125);
        assert_eq!(val, -0.75, "1st beat");

        init(15);
        let val = a.triangle(4.0, (-1.0, 1.0), 0.125);
        assert_eq!(val, -1.0, "last beat");

        init(16);
        let val = a.triangle(4.0, (-1.0, 1.0), 0.125);
        assert_eq!(val, -0.75, "1st beat - 2nd cycle");
    }

    #[test]
    #[serial]
    fn test_lerp_returns_initial_value() {
        init(0);
        let a = create_instance();
        let result = a.lerp(&[kf(99.0, 1.0), kf(1.0, 0.0)], 0.0);
        assert_eq!(result, 99.0, "returns 0 at frame 0");
    }

    #[test]
    #[serial]
    fn test_lerp_returns_halfway_point() {
        init(2);
        let a = create_instance();
        let result = a.lerp(&[kf(0.0, 1.0), kf(1.0, 0.0)], 0.0);
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
            let result =
                a.lerp(&[kf(0.0, 1.0), kf(1.0, 1.0), kf(0.0, 0.0)], 0.0);
            assert_eq!(result, 0.5, "returns the last keyframe value");
        }
    }

    #[test]
    #[serial]
    fn test_trigger_on_beat() {
        init(0);
        let animation = create_instance();
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
        let animation = create_instance();
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
            a.ramp(&[kf(0.0, 2.0), kf(1.0, 2.0)], 0.0, 0.5, Easing::Linear);
        assert_eq!(result, 0.0, "should start at initial value");

        // Test just before end of first keyframe (frame 7)
        init(7);
        let result =
            a.ramp(&[kf(0.0, 2.0), kf(1.0, 2.0)], 0.0, 0.5, Easing::Linear);
        println!("Frame 7 result: {}", result);
        assert_eq!(
            result, 0.0,
            "should still be at initial value just before duration end"
        );

        // Test at exact end of first keyframe (frame 8)
        init(8);
        let result =
            a.ramp(&[kf(0.0, 2.0), kf(1.0, 2.0)], 0.0, 0.5, Easing::Linear);
        println!("Frame 8 result: {}", result);
        assert_eq!(result, 0.0, "should start ramping after first keyframe");

        // Test one frame into ramp (frame 9)
        init(9);
        let result =
            a.ramp(&[kf(0.0, 2.0), kf(1.0, 2.0)], 0.0, 0.5, Easing::Linear);
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
            a.ramp(&[kf(0.0, 2.0), kf(1.0, 2.0)], 0.0, 0.5, Easing::Linear);
        println!("Frame 10 result: {}", result);
        assert_eq!(result, 1.0, "should reach final value after ramp");
    }

    #[test]
    #[serial]
    fn test_ramp_with_delay() {
        init(0);
        let a = create_instance();
        let result =
            a.ramp(&[kf(0.0, 2.0), kf(1.0, 2.0)], 1.0, 0.5, Easing::Linear);
        assert_eq!(result, 0.0, "should return initial value during delay");
    }

    #[test]
    #[serial]
    fn test_ramp_first_vs_subsequent_cycles() {
        init(0);
        let a = create_instance();

        let result =
            a.ramp(&[kf(1.0, 2.0), kf(0.0, 2.0)], 0.0, 0.5, Easing::Linear);
        assert_eq!(
            result, 1.0,
            "first cycle should start at value without ramping"
        );

        init(17);
        let result =
            a.ramp(&[kf(1.0, 2.0), kf(0.0, 2.0)], 0.0, 0.5, Easing::Linear);
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
            &[kfr((0.0, 1.0), 2.0), kfr((2.0, 3.0), 2.0)],
            0.5, // 0.5 beat delay
            0.5,
            Easing::Linear,
        );

        // Move just past keyframe boundary but still within delay period
        init(9);
        let delayed_value = a.r_ramp(
            &[kfr((0.0, 1.0), 2.0), kfr((2.0, 3.0), 2.0)],
            0.5,
            0.5,
            Easing::Linear,
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
            a.r_ramp(&[kfr((0.0, 1.0), 2.0)], 0.0, 0.5, Easing::Linear);

        // Move to frame 8 (start of next cycle, still previous value)
        init(8);
        let previous_value =
            a.r_ramp(&[kfr((0.0, 1.0), 2.0)], 0.0, 0.5, Easing::Linear);

        // Values should be the same at start of cycles
        assert_eq!(
            first_value, previous_value,
            "Should generate same value at start of cycle"
        );

        // Move to frame 9 (should be halfway through transition)
        init(9);
        let mid_value =
            a.r_ramp(&[kfr((0.0, 1.0), 2.0)], 0.0, 0.5, Easing::Linear);

        // Move to frame 10 (should be at new value)
        init(10);
        let new_value =
            a.r_ramp(&[kfr((0.0, 1.0), 2.0)], 0.0, 0.5, Easing::Linear);

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
            &[kfr((0.0, 1.0), 2.0), kfr((2.0, 3.0), 2.0)],
            0.5, // 0.5 beats delay
            1.0, // 1.0 beats ramp time
            Easing::Linear,
        );

        // At this point, we should be transitioning from first keyframe value (0.0-1.0)
        // to second keyframe value (2.0-3.0)
        let first_keyframe_value = kfr((0.0, 1.0), 2.0).generate_value(0);
        let second_keyframe_value = kfr((2.0, 3.0), 2.0).generate_value(1);

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
        let keyframes = vec![kfr((0.0, 1.0), 1.0)];
        let ramp_frames = vec![1, 5, 9, 13, 17, 21, 25, 29, 33, 37];

        let mut results = vec![];

        for frame in 0..40 {
            init(frame);
            let value = a.r_ramp(&keyframes, 0.0, 0.5, Easing::Linear);
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

    #[test]
    #[serial]
    fn test_r_ramp_osc_transport_timing() {
        let keyframes = &[
            kfr((0.0, 1.0), 1.5),
            kfr((0.0, 1.0), 3.0),
            kfr((0.0, 1.0), 2.5),
            kfr((0.0, 1.0), 1.0),
        ];

        let mut timing = OscTransportTiming::new(Bpm::new(BPM));
        let animation = Animation::new(timing.clone());

        // Test transition points directly
        let transition_points = vec![1.5, 4.5, 7.0, 8.0];

        for &beat in &transition_points {
            timing.set_beats(beat);
            let total_beats =
                keyframes.iter().map(|kf| kf.duration).sum::<f32>();
            let beats_elapsed = timing.beats();
            let wrapped_beat = beats_elapsed % total_beats;
            let cycle_float = (beats_elapsed / total_beats) + 1e-9;
            let current_cycle = cycle_float.floor() as u64;

            println!("\nAt beat: {}", beat);
            println!("total_beats: {}", total_beats);
            println!("beats_elapsed: {}", beats_elapsed);
            println!("wrapped_beat: {}", wrapped_beat);
            println!("cycle_float: {}", cycle_float);
            println!("current_cycle: {}", current_cycle);

            let before = animation.r_ramp(keyframes, 0.0, 0.25, Easing::Linear);

            timing.set_beats(beat + 0.1);
            let after = animation.r_ramp(keyframes, 0.0, 0.25, Easing::Linear);

            println!("Values: {} -> {}", before, after);

            assert!(
                (before - after).abs() > 0.0001,
                "Value should change at transition {}. Got {} -> {}",
                beat,
                before,
                after
            );
        }
    }

    #[test]
    #[serial]
    fn test_random() {
        let a = create_instance();

        init(0);
        let n = a.random(1.0, (0.0, 1.0), 999);

        init(1);
        let n2 = a.random(1.0, (0.0, 1.0), 999);
        assert_eq!(n, n2, "should return same N for full cycle");

        init(2);
        let n3 = a.random(1.0, (0.0, 1.0), 999);
        assert_eq!(n, n3, "should return same N for full cycle");

        init(3);
        let n4 = a.random(1.0, (0.0, 1.0), 999);
        assert_eq!(n, n4, "should return same N for full cycle");

        init(4);
        let n5 = a.random(1.0, (0.0, 1.0), 999);
        assert_ne!(n, n5, "should return new number on next cycle");
    }

    #[test]
    #[serial]
    fn test_random_stem() {
        let a = create_instance();

        init(0);
        let n1 = a.random(1.0, (0.0, 1.0), 99);
        let n2 = a.random(1.0, (0.0, 1.0), 99);

        assert_eq!(n1, n2, "should return same N for same args");

        let n3 = a.random(1.0, (0.0, 1.0), 100);
        assert_ne!(n1, n3, "should return different N for diff stems");
    }

    #[test]
    #[serial]
    fn test_random_smooth() {
        let a = create_instance();

        init(0);
        let n = a.random_slewed(1.0, (0.0, 1.0), 0.0, 9);

        init(1);
        let n2 = a.random_slewed(1.0, (0.0, 1.0), 0.0, 9);
        assert_eq!(n, n2, "should return same N for full cycle");

        init(2);
        let n3 = a.random_slewed(1.0, (0.0, 1.0), 0.0, 9);
        assert_eq!(n, n3, "should return same N for full cycle");

        init(3);
        let n4 = a.random_slewed(1.0, (0.0, 1.0), 0.0, 9);
        assert_eq!(n, n4, "should return same N for full cycle");

        init(4);
        let n5 = a.random_slewed(1.0, (0.0, 1.0), 0.0, 9);
        assert_ne!(n, n5, "should return new number on next cycle");
    }
}
