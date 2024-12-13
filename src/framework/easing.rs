use std::f32::consts::PI;

pub fn linear(t: f32) -> f32 {
    t
}

pub fn ease_in(t: f32) -> f32 {
    t * t
}

pub fn ease_out(t: f32) -> f32 {
    t * (2.0 - t)
}

pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

pub fn cubic_ease_in(t: f32) -> f32 {
    t * t * t
}

pub fn cubic_ease_out(t: f32) -> f32 {
    let u = t - 1.0;
    u * u * u + 1.0
}

pub fn cubic_ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let adjusted_t = t - 1.0;
        adjusted_t * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
    }
}

pub fn exponential(t: f32, exponent: f32) -> f32 {
    t.powf(exponent)
}

pub fn sine_ease_in(t: f32) -> f32 {
    1.0 - ((t * PI / 2.0).cos())
}

pub fn sine_ease_out(t: f32) -> f32 {
    (t * PI / 2.0).sin()
}

pub fn sine_ease_in_out(t: f32) -> f32 {
    0.5 * (1.0 - (PI * t).cos())
}

/// Suggested range [1, 10]
/// 1-5 Smooth
/// 5-15 = Balanced curves, noticable transition but not overly sharp
/// 15-20 = Very step curves, almost like a step function
pub fn sigmoid(t: f32, steepness: f32) -> f32 {
    1.0 / (1.0 + (-steepness * (t - 0.5)).exp())
}

pub fn logarithmic(t: f32) -> f32 {
    (1.0 + t * 9.0).ln() / 10.0f32.ln()
}
