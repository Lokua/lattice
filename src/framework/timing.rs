use midir::MidiInput;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::framework::prelude::*;

// Core trait that defines what we need from any timing source
pub trait TimingSource: Clone {
    /// Get the current beat position (fractional beats from start)
    fn beat_position(&self) -> f32;

    /// Get total number of beats elapsed
    fn total_beats(&self) -> f32;

    /// Convert beats to frames (for compatibility with frame-based code)
    fn beats_to_frames(&self, beats: f32) -> f32;
}

// The original frame-based implementation
#[derive(Clone)]
pub struct FrameTiming {
    bpm: f32,
    fps: f32,
}

impl FrameTiming {
    pub fn new(bpm: f32, fps: f32) -> Self {
        Self { bpm, fps }
    }

    fn current_frame(&self) -> f32 {
        frame_controller::frame_count() as f32
    }
}

impl TimingSource for FrameTiming {
    fn beat_position(&self) -> f32 {
        self.current_frame() / self.beats_to_frames(1.0)
    }

    fn total_beats(&self) -> f32 {
        self.beat_position()
    }

    fn beats_to_frames(&self, beats: f32) -> f32 {
        let seconds_per_beat = 60.0 / self.bpm;
        let total_seconds = beats * seconds_per_beat;
        total_seconds * self.fps
    }
}

// Constants for MIDI timing
const PULSES_PER_QUARTER_NOTE: u32 = 24;
const TICKS_PER_QUARTER_NOTE: u32 = 960; // Common MIDI resolution

// MIDI Song Position timing implementation
#[derive(Clone)]
pub struct MidiSongTiming {
    // Atomic counters for thread safety
    clock_count: Arc<AtomicU32>,
    song_position: Arc<AtomicU32>, // In MIDI ticks (1 tick = 1/960th of a quarter note)
    bpm: f32,                      // For conversion calculations
}

impl MidiSongTiming {
    pub fn new(bpm: f32) -> Self {
        let timing = Self {
            clock_count: Arc::new(AtomicU32::default()),
            song_position: Arc::new(AtomicU32::default()),
            bpm,
        };

        timing.setup_midi_listener();
        timing
    }

    fn setup_midi_listener(&self) {
        let _clock_count = self.clock_count.clone();
        let _song_position = self.song_position.clone();

        match MidiInput::new("Timing Input") {
            Ok(_midi_in) => {
                // TODO: Connect to proper MIDI input port
                // Handle incoming MIDI messages:
                // - 0xF8: Timing Clock (increment clock_count)
                // - 0xF2: Song Position Pointer (update song_position)
            }
            Err(err) => {
                warn!("Failed to initialize MIDI input: {}", err);
            }
        }
    }

    fn get_position_in_beats(&self) -> f32 {
        // Convert MIDI ticks to beats (quarter notes)
        let ticks = self.song_position.load(Ordering::Relaxed);
        let beats = ticks as f32 / TICKS_PER_QUARTER_NOTE as f32;

        // Add fractional position from clock count
        let clock_offset = self.clock_count.load(Ordering::Relaxed) as f32
            / PULSES_PER_QUARTER_NOTE as f32;

        beats + clock_offset
    }
}

impl TimingSource for MidiSongTiming {
    fn beat_position(&self) -> f32 {
        self.get_position_in_beats()
    }

    fn total_beats(&self) -> f32 {
        self.get_position_in_beats()
    }

    fn beats_to_frames(&self, beats: f32) -> f32 {
        let bpm = self.bpm;
        let seconds_per_beat = 60.0 / bpm;
        beats * seconds_per_beat * frame_controller::fps()
    }
}

// Helper functions for testing
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_frame_timing_conversion() {
        let timing = FrameTiming::new(120.0, 60.0);

        // At 120 BPM, one beat = 0.5 seconds
        // At 60 FPS, 0.5 seconds = 30 frames
        assert_eq!(timing.beats_to_frames(1.0), 30.0);
    }

    #[test]
    #[serial]
    fn test_midi_timing_beats() {
        let timing = MidiSongTiming::new(120.0);

        // Simulate receiving SPP message for bar 44
        timing
            .song_position
            .store(44 * 4 * TICKS_PER_QUARTER_NOTE, Ordering::Relaxed);

        // Each bar is 4 beats, so bar 44 starts at beat 176
        assert_eq!(timing.beat_position(), 176.0);
    }
}
