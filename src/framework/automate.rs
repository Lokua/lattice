//! Supporting types for the [`Animation::automate`] method, providing a
//! breakpoint animation system for creating complex automation curves, similar
//! to those found in DAWs, yet with some special tricks of its own.

use nannou::math::map_range;
use nannou::rand::rngs::StdRng;
use nannou::rand::{Rng, SeedableRng};
use std::f32::EPSILON;
use std::str::FromStr;

use super::prelude::*;

/// The core structure need to configure segments for the
/// [`Animation::automate`] method. See the various constructors such as
/// [`Breakpoint::step`], [`Breakpoint::ramp`], etc. for in depth details.
#[derive(Clone, Debug)]
pub struct Breakpoint {
    pub kind: Kind,
    pub position: f32,
    pub value: f32,
}

impl Breakpoint {
    pub fn new(kind: Kind, position: f32, value: f32) -> Self {
        Self {
            kind,
            position,
            value,
        }
    }

    /// Create a step that will be held at `value` until the next breakpoint.
    pub fn step(position: f32, value: f32) -> Self {
        Self::new(Kind::Step, position, value)
    }

    /// Create a step that will curve from this `value` to the next breakpoint's
    /// value with adjustable easing. `position` is expressed in beats.
    pub fn ramp(position: f32, value: f32, easing: Easing) -> Self {
        Self::new(Kind::Ramp { easing }, position, value)
    }

    /// Creates a linear ramp from this `value` to the next breakpoint's value
    /// with amplitude modulation applied over it and finalized by various
    /// clamping modes and easing algorithms that together can produce extremely
    /// complex curves. Like position, `frequency` is expressed in beats.
    /// `amplitude` represents how much above and below the base interpolated
    /// value the modulation will add or subtract depending on its phase.
    /// Negative amplitudes can be used to invert the modulation. For
    /// [`Shape::Sine`] and [`Shape::Triangle`], the modulation wave is phase
    /// shifted to always start and end at or very close to zero to ensure
    /// smooth transitions between segments (this is not the case for
    /// [`Shape::Square`] because discontinuities are unavoidable). The `width`
    /// parameter controls the [`Shape::Square`] duty cycle. For `Sine` and
    /// `Triangle` shapes, it will skew the peaks, for example when applied to a
    /// triangle a `width` of 0.0 will produce a downwards saw while 1.0 will
    /// produce an upwards one - applied to sine is a similarly skewed,
    /// asymmetric wave. For all shapes a value of 0.5 will produce the natural
    /// wave. Beware this method can produce values outside of the otherwise
    /// normalized \[0, 1\] range when the `constrain` parameter is set to
    /// [`Constrain::None`].
    pub fn wave(
        position: f32,
        value: f32,
        shape: Shape,
        frequency: f32,
        width: f32,
        amplitude: f32,
        easing: Easing,
        constrain: Constrain,
    ) -> Self {
        Self::new(
            Kind::Wave {
                shape,
                frequency,
                width,
                amplitude,
                easing,
                constrain,
            },
            position,
            value,
        )
    }

    /// Create a step chosen randomly from the passed in `amplitude` which
    /// specifies the range of possible deviation from `value`.
    ///
    /// > TIP: you can make this a smooth random by applying a
    /// [`SlewLimiter`] to the output.
    pub fn random(position: f32, value: f32, amplitude: f32) -> Self {
        Self::new(Kind::Random { amplitude }, position, value)
    }

    /// # ⚠️ Experimental
    /// Similar to [`Self::wave`], only uses Perlin noise to amplitude modulate
    /// the base curve. Useful for adding jitter when `frequency` is shorter
    /// than the duration of this point's `position` and the next; larger values
    /// equal to that duration or longer are much smoother.
    pub fn random_smooth(
        position: f32,
        value: f32,
        frequency: f32,
        amplitude: f32,
        easing: Easing,
        constrain: Constrain,
    ) -> Self {
        Self::new(
            Kind::RandomSmooth {
                frequency,
                amplitude,
                easing,
                constrain,
            },
            position,
            value,
        )
    }

    /// The last breakpoint in any sequence represents the final value and is
    /// never actually entered. Technically any kind of breakpoint can be used
    /// at the end and will be interpreted exactly the same way (only value and
    /// position will be used to mark the end of a sequence), but this is
    /// provided for code clarity as it reads better. If you are using
    /// [`Mode::Loop`] it's a good idea to make the value of this endpoint match
    /// the first value to avoid discontinuity (unless you want that).
    pub fn end(position: f32, value: f32) -> Self {
        Self::new(Kind::End, position, value)
    }
}

#[derive(Clone, Debug)]
pub enum Kind {
    Step,
    Ramp {
        easing: Easing,
    },
    Random {
        amplitude: f32,
    },
    RandomSmooth {
        frequency: f32,
        amplitude: f32,
        easing: Easing,
        constrain: Constrain,
    },
    Wave {
        shape: Shape,
        amplitude: f32,
        width: f32,
        frequency: f32,
        easing: Easing,
        constrain: Constrain,
    },
    End,
}

impl FromStr for Kind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "step" => Ok(Kind::Step),
            "ramp" => Ok(Kind::Ramp {
                easing: Easing::Linear,
            }),
            "random" => Ok(Kind::Random { amplitude: 0.25 }),
            "randomsmooth" => Ok(Kind::RandomSmooth {
                frequency: 0.25,
                amplitude: 0.25,
                easing: Easing::Linear,
                constrain: Constrain::None,
            }),
            "wave" => Ok(Kind::Wave {
                shape: Shape::Sine,
                frequency: 0.25,
                width: 0.5,
                amplitude: 0.25,
                easing: Easing::Linear,
                constrain: Constrain::None,
            }),
            "end" => Ok(Kind::End),
            _ => Err(format!("Unknown breakpoint kind variant: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Sine,
    Triangle,
    Square,
}
impl FromStr for Shape {
    type Err = String;

    fn from_str(shape: &str) -> Result<Self, Self::Err> {
        match shape.to_lowercase().as_str() {
            "sine" => Ok(Shape::Sine),
            "triangle" => Ok(Shape::Triangle),
            "square" => Ok(Shape::Square),
            _ => Err(format!("No shape {} exists.", shape)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Loop,
    Once,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode.to_lowercase().as_str() {
            "loop" => Ok(Mode::Loop),
            "once" => Ok(Mode::Once),
            _ => Err(format!("No mode {} exists.", mode)),
        }
    }
}

impl<T: TimingSource> Animation<T> {
    /// An advanced animation method modelled on DAW automation lanes. It is
    /// capable of producing the same results as just about every other
    /// animation method yet is more powerful, but as such requires a bit more
    /// configuration. While other animation methods are focused on one style of
    /// keyframe/transition, `automate` allows many different types of
    /// transitions defined by a list of [`Breakpoint`], each with its own
    /// configurable [`Kind`]. See [breakpoints] for a static visualization of
    /// the kinds of curves `automate` can produce.
    ///
    /// [breakpoints]:
    ///     https://github.com/Lokua/lattice/blob/main/src/sketches/breakpoints.rs
    pub fn automate(&self, breakpoints: &[Breakpoint], mode: Mode) -> f32 {
        assert!(breakpoints.len() >= 1, "At least 1 breakpoint is required");
        assert!(
            breakpoints[0].position == 0.0,
            "First breakpoint must be 0.0"
        );

        if breakpoints.len() == 1 {
            return breakpoints[0].value;
        }

        let total_beats = breakpoints.last().unwrap().position;

        let beats_elapsed = ternary!(
            mode == Mode::Loop,
            self.beats() % total_beats,
            self.beats()
        );

        let mut breakpoint: Option<&Breakpoint> = None;
        let mut next_point: Option<&Breakpoint> = None;

        for (index, point) in breakpoints.iter().enumerate() {
            if index == breakpoints.len() - 1 && mode != Mode::Loop {
                return point.value;
            }

            let next = &breakpoints[(index + 1) % breakpoints.len()];

            if next.position < point.position {
                breakpoint = Some(next);
                next_point = Some(point);
                break;
            }

            if point.position <= beats_elapsed && next.position > beats_elapsed
            {
                breakpoint = Some(point);
                next_point = Some(next);
                break;
            }
        }

        match (breakpoint, next_point) {
            (Some(p1), None) => p1.value,
            (Some(p1), Some(p2)) => match &p1.kind {
                Kind::Step => p1.value,
                Kind::Ramp { easing } => {
                    Self::create_ramp(p1, p2, beats_elapsed, easing.clone())
                }
                Kind::Wave {
                    shape,
                    frequency,
                    width,
                    amplitude,
                    easing,
                    constrain,
                } => match shape {
                    Shape::Sine => {
                        let value = Self::create_ramp(
                            p1,
                            p2,
                            beats_elapsed,
                            easing.clone(),
                        );

                        let phase_in_cycle = beats_elapsed / frequency;

                        let t = phase_in_cycle % 1.0;
                        let m = 2.0 * (width - 0.5);
                        let mod_wave =
                            ((TWO_PI * t) + m * (TWO_PI * t).sin()).sin();

                        constrain.apply(value + (mod_wave * amplitude))
                    }
                    Shape::Triangle => {
                        let value = Self::create_ramp(
                            p1,
                            p2,
                            beats_elapsed,
                            easing.clone(),
                        );

                        let phase_offset = 0.25;
                        let phase_in_cycle = beats_elapsed / frequency;
                        let mut mod_wave =
                            (phase_in_cycle + phase_offset) % 1.0;

                        mod_wave = if mod_wave < *width {
                            4.0 * mod_wave - 1.0
                        } else {
                            3.0 - 4.0 * mod_wave
                        };

                        constrain.apply(value + (mod_wave * amplitude))
                    }
                    Shape::Square => {
                        let value = Self::create_ramp(
                            p1,
                            p2,
                            beats_elapsed,
                            easing.clone(),
                        );
                        let phase_in_cycle = beats_elapsed / frequency;

                        let mod_wave = if (phase_in_cycle % 1.0) < *width {
                            1.0
                        } else {
                            -1.0
                        };

                        constrain.apply(value + (mod_wave * amplitude))
                    }
                },
                Kind::Random { amplitude } => {
                    let loop_count = (self.beats() / p2.position).floor();
                    let seed = (p1.position
                        + p2.position
                        + p1.value
                        + amplitude
                        + loop_count) as u64;
                    let mut rng = StdRng::seed_from_u64(seed);
                    let y = p1.value;
                    let value = rng.gen_range(y - amplitude..=y + amplitude);
                    value
                }
                Kind::RandomSmooth {
                    frequency,
                    amplitude,
                    easing,
                    constrain,
                } => {
                    let value = Self::create_ramp(
                        p1,
                        p2,
                        beats_elapsed,
                        easing.clone(),
                    );

                    let x = (beats_elapsed / frequency) % 1.0;
                    let y = value;
                    let loop_count = (self.beats() / p2.position).floor();
                    let seed = (p1.position
                        + p2.position
                        + p1.value
                        + amplitude
                        + loop_count) as u64;
                    let noise_scale = 2.5;
                    let random_value = PerlinNoise::new(seed as u32)
                        .get([x * noise_scale, y * noise_scale]);

                    let random_mapped = map_range(
                        random_value,
                        -1.0,
                        1.0,
                        -amplitude,
                        amplitude + EPSILON,
                    );

                    constrain.apply(value + random_mapped)
                }
                Kind::End => {
                    loud_panic!("Somehow we've moved beyond the end")
                }
            },
            _ => {
                warn_once!("Could not match breakpoint {:?}", breakpoint);
                0.0
            }
        }
    }

    fn create_ramp(
        p1: &Breakpoint,
        p2: &Breakpoint,
        beats_elapsed: f32,
        easing: Easing,
    ) -> f32 {
        let duration = p2.position - p1.position;
        let t = easing.apply(((beats_elapsed - p1.position) / duration) % 1.0);
        let value = lerp(p1.value, p2.value, t);
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // 1 frame = 1/16; 4 frames per beat; 16 frames per bar
    use crate::framework::animation::animation_tests::{create_instance, init};

    #[test]
    #[serial]
    fn test_breakpoint_step_init() {
        init(0);
        let a = create_instance();
        let x = a.automate(&[Breakpoint::step(0.0, 44.0)], Mode::Once);
        assert_eq!(x, 44.0, "Returns initial value");
    }

    #[test]
    #[serial]
    fn test_breakpoint_step_2nd() {
        init(4);
        let a = create_instance();
        let x = a.automate(
            &[Breakpoint::step(0.0, 10.0), Breakpoint::step(1.0, 20.0)],
            Mode::Once,
        );
        assert_eq!(x, 20.0, "Returns 2nd stage");
    }

    #[test]
    #[serial]
    fn test_breakpoint_step_last() {
        init(100);
        let a = create_instance();
        let x = a.automate(
            &[Breakpoint::step(0.0, 10.0), Breakpoint::step(1.0, 20.0)],
            Mode::Once,
        );
        assert_eq!(x, 20.0, "Returns last stage");
    }

    #[test]
    #[serial]

    fn test_breakpoint_step_loop_mode() {
        init(4);
        let breakpoints = &[
            Breakpoint::step(0.0, 10.0),
            Breakpoint::step(1.0, 20.0),
            Breakpoint::end(2.0, 0.0),
        ];
        let a = create_instance();
        let x = a.automate(breakpoints, Mode::Loop);
        assert_eq!(x, 20.0, "Returns 2nd stage");
        init(8);
        let x = a.automate(breakpoints, Mode::Loop);
        assert_eq!(x, 10.0, "Returns 1st stage when looping back around");
    }

    #[test]
    #[serial]
    fn test_breakpoint_step_midway() {
        init(2);
        let a = create_instance();
        let x = a.automate(
            &[
                Breakpoint::ramp(0.0, 0.0, Easing::Linear),
                Breakpoint::end(1.0, 1.0),
            ],
            Mode::Once,
        );
        assert_eq!(x, 0.5, "Returns midway point");
    }

    #[test]
    #[serial]
    fn test_breakpoint_step_last_16th() {
        init(3);
        let a = create_instance();
        let x = a.automate(
            &[
                Breakpoint::ramp(0.0, 0.0, Easing::Linear),
                Breakpoint::end(1.0, 1.0),
            ],
            Mode::Once,
        );
        assert_eq!(x, 0.75, "Returns midway point");
    }

    #[test]
    #[serial]
    fn test_breakpoint_step_last_16th_loop() {
        init(7);
        let a = create_instance();
        let x = a.automate(
            &[
                Breakpoint::ramp(0.0, 0.0, Easing::Linear),
                Breakpoint::end(1.0, 1.0),
            ],
            Mode::Loop,
        );
        assert_eq!(x, 0.75, "Returns midway point");
    }

    #[test]
    #[serial]
    fn test_step_then_ramp() {
        let a = create_instance();
        let x = || {
            a.automate(
                &[
                    Breakpoint::step(0.0, 10.0),
                    Breakpoint::ramp(1.0, 20.0, Easing::Linear),
                    Breakpoint::end(2.0, 10.0),
                ],
                Mode::Loop,
            )
        };

        init(0);
        assert_eq!(x(), 10.0);
        init(1);
        assert_eq!(x(), 10.0);
        init(2);
        assert_eq!(x(), 10.0);
        init(3);
        assert_eq!(x(), 10.0);

        init(4);
        assert_eq!(x(), 20.0);
        init(5);
        assert_eq!(x(), 17.5);
        init(6);
        assert_eq!(x(), 15.0);
        init(7);
        assert_eq!(x(), 12.5);

        init(8);
        assert_eq!(x(), 10.0);
    }

    #[test]
    #[serial]
    fn test_wave_triangle_simple() {
        let a = create_instance();
        let x = || {
            a.automate(
                &[
                    Breakpoint::wave(
                        0.0,
                        0.0,
                        Shape::Triangle,
                        1.0,
                        0.5,
                        0.5,
                        Easing::Linear,
                        Constrain::None,
                    ),
                    Breakpoint::end(1.0, 1.0),
                ],
                Mode::Loop,
            )
        };

        // 0 beats
        init(0);
        assert_eq!(x(), 0.0);

        // 0.25 beats, base 0.25 + wave 0.5 = 0.75
        init(1);
        assert_eq!(x(), 0.75);

        // 0.5 beats, base 0.5 + wave 0.0 = 0.5
        init(2);
        assert_eq!(x(), 0.5);

        // 0.75 beats, base 0.75 + wave -0.5 = 0.25
        init(3);
        assert_eq!(x(), 0.25);

        // And back around
        init(4);
        assert_eq!(x(), 0.0);
    }

    #[test]
    #[serial]
    fn test_step_to_ramp_edge_case() {
        let a = create_instance();
        let x = || {
            a.automate(
                &[
                    Breakpoint::step(0.0, 0.0),
                    Breakpoint::step(0.5, 1.0),
                    Breakpoint::ramp(1.0, 0.5, Easing::EaseInExpo),
                    Breakpoint::wave(
                        1.5,
                        1.0,
                        Shape::Triangle,
                        0.25,
                        0.5,
                        0.25,
                        Easing::Linear,
                        Constrain::None,
                    ),
                    Breakpoint::end(2.0, 0.0),
                ],
                Mode::Once,
            )
        };

        init(4);
        assert_eq!(x(), 0.5);
    }

    #[test]
    #[serial]
    fn test_ramp_bug_2025_02_23() {
        let a = create_instance();
        let x = || {
            a.automate(
                &[
                    Breakpoint::ramp(0.0, 0.0, Easing::Linear),
                    // BUG: jumps from ~0.5 to 0.75, should be exactly 0.5
                    // ^ Fixed :)!
                    Breakpoint::ramp(32.0, 0.5, Easing::Linear),
                    Breakpoint::ramp(96.0, 1.0, Easing::Linear),
                    Breakpoint::ramp(128.0, 0.75, Easing::Linear),
                    Breakpoint::end(192.0, 0.25),
                ],
                Mode::Once,
            )
        };

        init(128);
        assert_eq!(x(), 0.5);
    }
}
