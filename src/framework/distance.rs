use std::f32::consts::PI;

/// Calculates Euclidean distance between two points (Circular Symmetry)
pub fn euclidean(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

/// Calculates Manhattan distance between two points (Diamond Symmetry)
pub fn manhattan(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    (x2 - x1).abs() + (y2 - y1).abs()
}

/// Calculates Chebyshev distance between two points (Square Symmetry)
pub fn chebyshev(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    (x2 - x1).abs().max((y2 - y1).abs())
}

/// Calculates Minkowski distance between two points (Blend of Euclidean and Manhattan)
pub fn minkowski(x1: f32, y1: f32, x2: f32, y2: f32, p: f32) -> f32 {
    ((x2 - x1).abs().powf(p) + (y2 - y1).abs().powf(p)).powf(1.0 / p)
}

/// Generates a radial sinusoidal pattern based on distance
pub fn radial_sinusoidal(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    (distance / 50.0).sin().abs() * 100.0
}

/// Returns angle between two points in radians
pub fn polar(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    (y2 - y1).atan2(x2 - x1)
}

/// Creates a spiral pattern based on distance and angle
pub fn spiral(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let angle = (y2 - y1).atan2(x2 - x1);
    distance + angle * 100.0
}

/// Generates a harmonic pattern combining sine and cosine waves
pub fn harmonic(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    (distance / 50.0).sin() * 50.0 + (distance / 75.0).cos() * 30.0
}

/// Generates wave-like patterns that emanate from points
pub fn concentric_waves(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    frequency: f32,
) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    (distance * frequency).sin().abs() * (-distance * 0.01).exp()
}

/// Creates interference patterns between two points
pub fn wave_interference(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    frequency: f32,
) -> f32 {
    let d1 = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let d2 = ((x2 - x1 - 50.0).powi(2) + (y2 - y1 - 50.0).powi(2)).sqrt();
    (d1 * frequency).sin() + (d2 * frequency).sin() * 50.0
}

/// Creates a ripple effect with customizable decay
pub fn ripple(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    frequency: f32,
    decay: f32,
) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    (distance * frequency).sin() * (-distance * decay).exp() * 100.0
}

/// Creates a moiré pattern effect
pub fn moire(x1: f32, y1: f32, x2: f32, y2: f32, scale: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx / scale).sin() * (dy / scale).sin() * 100.0
}

/// Creates fractal-like noise patterns
pub fn fractal_noise(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let mut value = 0.0;
    for i in 1..=4 {
        value += (distance * 0.05 * i as f32).sin() * (1.0 / i as f32);
    }
    value * 50.0
}

/// Creates a vortex-like pattern
pub fn vortex(x1: f32, y1: f32, x2: f32, y2: f32, spiral_factor: f32) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let angle = (y2 - y1).atan2(x2 - x1);
    (distance + angle / spiral_factor) % 100.0
}

/// Creates a cellular-like pattern
pub fn cellular(x1: f32, y1: f32, x2: f32, y2: f32, scale: f32) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    ((distance % scale) - scale / 2.0).abs() * 2.0
}

pub fn wood_grain(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    grain_size: f32,
    angle_mult: f32,
) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let angle = (y2 - y1).atan2(x2 - x1);
    (distance / grain_size).sin() * distance * 0.5
        + (angle * 5.0).sin() * angle_mult
}

pub fn wood_grain_advanced(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    grain_size: f32,
    angle_mult: f32,
    distance_power: f32,    // 2
    distance_strength: f32, // 0.5
    angle_frequency: f32,   // 5.0
) -> f32 {
    let distance = ((x2 - x1).powf(distance_power)
        + (y2 - y1).powf(distance_power))
    .sqrt();
    let angle = (y2 - y1).atan2(x2 - x1);
    (distance / grain_size).sin() * distance * distance_strength
        + (angle * angle_frequency).sin() * angle_mult
}

/// Creates patterns similar to topographic maps
pub fn topographic(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    contour_interval: f32,
) -> f32 {
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    ((distance + x2 * 0.5 + y2 * 0.5) % contour_interval
        - contour_interval / 2.0)
        .abs()
        * 2.0
}

/// Creates a pattern that looks like fabric weave
pub fn weave(x1: f32, y1: f32, x2: f32, y2: f32, frequency: f32) -> f32 {
    ((x2 * frequency).sin() + (y2 * frequency).sin())
        * (((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt() * 0.05).sin()
        * 50.0
}

/// Creates a kaleidoscope-like effect
pub fn kaleidoscope(x1: f32, y1: f32, x2: f32, y2: f32, segments: f32) -> f32 {
    let angle = (y2 - y1).atan2(x2 - x1);
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let segment_angle = (angle + PI) % (PI * 2.0 / segments);
    (segment_angle * segments + distance * 0.05).sin() * 100.0
}
