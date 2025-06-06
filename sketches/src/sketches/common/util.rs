#![allow(unused)]
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use geom::Ellipse;
use xtal::prelude::*;
use nannou::color::{LinSrgb, Srgb};
use nannou::prelude::*;
use nannou::rand::Rng;
use nannou::rand::rand;
use nannou::rand::thread_rng;

pub const PHI_F32: f32 = 1.618_033_9;

pub const QUAD_POSITIONS: [[f32; 3]; 6] = [
    // Bottom-left
    [-1.0, -1.0, 0.0],
    // Bottom-right
    [1.0, -1.0, 0.0],
    // Top-right
    [1.0, 1.0, 0.0],
    // Bottom-left
    [-1.0, -1.0, 0.0],
    // Top-right
    [1.0, 1.0, 0.0],
    // Top-left
    [-1.0, 1.0, 0.0],
];

pub const CUBE_POSITIONS: [[f32; 3]; 36] = [
    // Front face
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
    // Back face
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, -0.5, -0.5],
    // Top face
    [-0.5, 0.5, -0.5],
    [-0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, 0.5],
    [0.5, 0.5, -0.5],
    // Bottom face
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5],
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5],
    [-0.5, -0.5, 0.5],
    // Right face
    [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, 0.5, 0.5],
    [0.5, -0.5, -0.5],
    [0.5, 0.5, 0.5],
    [0.5, -0.5, 0.5],
    // Left face
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, 0.5, 0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, 0.5],
    [-0.5, 0.5, -0.5],
];

pub trait IntoLinSrgb {
    fn into_lin_srgb(self) -> LinSrgb;
}

impl IntoLinSrgb for Srgb<u8> {
    fn into_lin_srgb(self) -> LinSrgb {
        LinSrgb::new(
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
        )
    }
}

pub trait IntoLinSrgba {
    fn into_lin_srgba(self, alpha: f32) -> LinSrgba;
}

impl IntoLinSrgba for Srgb<u8> {
    fn into_lin_srgba(self, alpha: f32) -> LinSrgba {
        LinSrgba::new(
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
            alpha,
        )
    }
}

pub fn lin_srgb_to_lin_srgba(color: LinSrgb, alpha: f32) -> LinSrgba {
    LinSrgba::from_components((color.red, color.green, color.blue, alpha))
}

// Numbers from Rec. 709 color space standard
pub fn luminance(color: &LinSrgb) -> f32 {
    0.2126 * color.red + 0.7152 * color.green + 0.0722 * color.blue
}

pub fn create_grid<F>(
    w: f32,
    h: f32,
    divisions: usize,
    transform_xy: F,
) -> (Vec<Vec2>, f32)
where
    F: Fn(f32, f32) -> Vec2,
{
    let mut grid = Vec::new();
    let cell_size = (f32::min(w, h) / divisions as f32).floor();
    let cols = (w / cell_size).floor() as usize;
    let rows = (h / cell_size).floor() as usize;

    let grid_width = cols as f32 * cell_size;
    let grid_height = rows as f32 * cell_size;

    let start_x = -grid_width / 2.0;
    let start_y = grid_height / 2.0;

    for col in 0..cols {
        for row in 0..rows {
            let x = start_x + col as f32 * cell_size + cell_size / 2.0;
            let y = start_y - row as f32 * cell_size - cell_size / 2.0;
            grid.push(transform_xy(x, y));
        }
    }

    (grid, cell_size)
}

pub trait TrigonometricExt {
    fn sec(self) -> Self;
    fn csc(self) -> Self;
    fn cot(self) -> Self;
    fn sech(self) -> Self;
    fn csch(self) -> Self;
    fn coth(self) -> Self;
}

impl TrigonometricExt for f32 {
    fn sec(self) -> Self {
        1.0 / self.cos()
    }

    fn csc(self) -> Self {
        1.0 / self.sin()
    }

    fn cot(self) -> Self {
        1.0 / self.tan()
    }

    fn sech(self) -> Self {
        1.0 / self.cosh()
    }

    fn csch(self) -> Self {
        1.0 / self.sinh()
    }

    fn coth(self) -> Self {
        1.0 / self.tanh()
    }
}

pub fn trig_fn_lookup() -> HashMap<&'static str, fn(f32) -> f32> {
    let mut map = HashMap::default();
    map.insert("cos", f32::cos as fn(f32) -> f32);
    map.insert("sin", f32::sin as fn(f32) -> f32);
    map.insert("tan", f32::tan as fn(f32) -> f32);
    map.insert("tanh", f32::tanh as fn(f32) -> f32);
    map.insert("sec", f32::sec as fn(f32) -> f32);
    map.insert("csc", f32::csc as fn(f32) -> f32);
    map.insert("cot", f32::cot as fn(f32) -> f32);
    map.insert("sech", f32::sech as fn(f32) -> f32);
    map.insert("csch", f32::csch as fn(f32) -> f32);
    map.insert("coth", f32::coth as fn(f32) -> f32);
    map
}

pub fn rect_contains_point(rect: &Rect, point: &Vec2) -> bool {
    rect.left() <= point.x
        && point.x <= rect.right()
        && rect.bottom() <= point.y
        && point.y <= rect.top()
}

pub fn circle_contains_point(circle: &Ellipse, point: &Vec2) -> bool {
    let rect = circle.rect;
    let center = rect.xy();
    let radius = rect.w() / 2.0;

    let dx = point.x - center.x;
    let dy = point.y - center.y;
    dx * dx + dy * dy <= radius * radius
}

pub fn nearby_point(base_point: Vec2, radius: f32) -> Vec2 {
    let angle = random_range(0.0, TWO_PI);
    let distance = random_range(0.0, radius);
    Vec2::new(
        base_point.x + distance * angle.cos(),
        base_point.y + distance * angle.sin(),
    )
}

pub fn multi_lerp(values: &[f32], t: f32) -> f32 {
    let num_segments = values.len() - 1;
    let scaled_t = t * num_segments as f32;
    let index = scaled_t.floor() as usize;
    let segment_t = scaled_t - index as f32;

    // Handle edge case where t = 1.0
    if index >= num_segments {
        return values[num_segments];
    }

    lerp(values[index], values[index + 1], segment_t)
}

pub fn map_clamp(
    value: f32,
    in_min: f32,
    in_max: f32,
    out_min: f32,
    out_max: f32,
    ease: impl Fn(f32) -> f32,
) -> f32 {
    let normalized = (value - in_min) / (in_max - in_min);
    let eased = ease(normalized);
    let clamped = eased.clamp(0.0, 1.0);
    out_min + (clamped * (out_max - out_min))
}

/// triangle_map(0.0, 0.0, 1.0, 0.0, 100.0)); // 0
/// triangle_map(0.25, 0.0, 1.0, 0.0, 100.0)); // 50
/// triangle_map(0.5, 0.0, 1.0, 0.0, 100.0)); // 100
/// triangle_map(0.75, 0.0, 1.0, 0.0, 100.0)); // 50
/// triangle_map(1.0, 0.0, 1.0, 0.0, 100.0)); // 0
pub fn triangle_map(
    n: f32,
    in_min: f32,
    in_max: f32,
    out_min: f32,
    out_max: f32,
) -> f32 {
    // Normalize input to [0, 1]
    let normalized = (n - in_min) / (in_max - in_min);

    // Create triangle wave (no need for modulo since we're handling one cycle)
    let triangle = if normalized <= 0.5 {
        // Rising part: 0.0 -> 1.0
        normalized * 2.0
    } else {
        // Falling part: 1.0 -> 0.0
        2.0 * (1.0 - normalized)
    };

    // Map to output range
    triangle * (out_max - out_min) + out_min
}

pub fn rotate_point(point: Vec2, center: Vec2, angle: f32) -> Vec2 {
    let translated = point - center;
    let rotated = vec2(
        translated.x * angle.cos() - translated.y * angle.sin(),
        translated.x * angle.sin() + translated.y * angle.cos(),
    );
    rotated + center
}

pub fn random_normal(std_dev: f32) -> f32 {
    let u1: f32 = random();
    let u2: f32 = random();

    // Use the Box-Muller transform to create a normal distribution
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
    z0 * std_dev
}

// https://www.generativehut.com/post/how-to-make-generative-art-feel-natural
pub fn chaikin(
    points: Vec<Vec2>,
    iterations: usize,
    closed: bool,
) -> Vec<Vec2> {
    if iterations == 0 || points.len() < 2 {
        return points;
    }

    let n = points.len();
    let capacity = if closed { n * 2 } else { (n - 1) * 2 + 1 };
    let mut smooth = Vec::with_capacity(capacity);

    // For open curves, keep the first point
    if !closed {
        smooth.push(points[0]);
    }

    // Process points
    let points_to_process = if closed { n } else { n - 1 };
    for i in 0..points_to_process {
        let current = points[i];
        let next = if closed {
            points[(i + 1) % n]
        } else {
            points[i + 1]
        };

        let q = pt2(
            0.75 * current.x + 0.25 * next.x,
            0.75 * current.y + 0.25 * next.y,
        );

        let r = pt2(
            0.25 * current.x + 0.75 * next.x,
            0.25 * current.y + 0.75 * next.y,
        );

        smooth.push(q);
        smooth.push(r);
    }

    // For open curves, keep the last point
    if !closed {
        smooth.push(*points.last().unwrap());
    }

    if iterations == 1 {
        smooth
    } else {
        chaikin(smooth, iterations - 1, closed)
    }
}

/// Apply kernel smoothing
pub fn average_neighbors(points: Vec<Vec2>, iterations: usize) -> Vec<Vec2> {
    if iterations == 0 || points.len() < 3 {
        return points;
    }

    let smoothed = points
        .iter()
        .enumerate()
        .map(|(i, point)| {
            if i == 0 || i == points.len() - 1 {
                return *point;
            }

            let prev = points[i - 1];
            let next = points[i + 1];
            pt2(point.x, (point.y + prev.y + next.y) / 3.0)
        })
        .collect();

    if iterations == 1 {
        smoothed
    } else {
        average_neighbors(smoothed, iterations - 1)
    }
}

pub fn on_screen(v: Vec2, wr: &WindowRect) -> bool {
    v.x >= -wr.hw() && v.x <= wr.hw() && v.y >= -wr.hh() && v.y <= wr.hh()
}

pub fn parse_bar_beat_16th(time_str: &str) -> Result<f32, Box<dyn Error>> {
    let parts: Vec<f32> = time_str
        .split('.')
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()?;

    if parts.len() != 3 {
        return Err("Time string must be in format bar.beat.16th".into());
    }

    let [bars, beats, sixteenths] = [parts[0], parts[1], parts[2]];
    let total_beats = (bars * 4.0) + beats + (sixteenths * 0.25);

    Ok(total_beats)
}

pub fn str_to_f32_seed(id: &str) -> f32 {
    let mut hash: u32 = 0;

    for byte in id.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    hash as f32
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[macro_export]
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {
            assert!(
                ($a - $b).abs() < 0.001,
                "Values not approximately equal: {} and {}, difference: {}",
                $a, $b, ($a - $b).abs()
            );
        };
        ($a:expr, $b:expr, $epsilon:expr) => {
            assert!(
                ($a - $b).abs() < $epsilon,
                "Values not approximately equal: 
                    {} and {}, difference: {}, tolerance: {}",
                $a, $b, ($a - $b).abs(), $epsilon
            );
        };
    }

    #[test]
    fn test_triangle_map() {
        // Test the key points specified in the original examples
        assert_approx_eq!(triangle_map(0.0, 0.0, 1.0, 0.0, 100.0), 0.0);
        assert_approx_eq!(triangle_map(0.25, 0.0, 1.0, 0.0, 100.0), 50.0);
        assert_approx_eq!(triangle_map(0.5, 0.0, 1.0, 0.0, 100.0), 100.0);
        assert_approx_eq!(triangle_map(0.75, 0.0, 1.0, 0.0, 100.0), 50.0);
        assert_approx_eq!(triangle_map(1.0, 0.0, 1.0, 0.0, 100.0), 0.0);

        // Test with different input/output ranges
        assert_approx_eq!(triangle_map(5.0, 0.0, 10.0, 0.0, 1.0), 1.0);

        // Test negative ranges
        assert_approx_eq!(triangle_map(-1.0, -1.0, 1.0, -100.0, 100.0), -100.0);
        assert_approx_eq!(triangle_map(0.0, -1.0, 1.0, -100.0, 100.0), 100.0);
        assert_approx_eq!(triangle_map(1.0, -1.0, 1.0, -100.0, 100.0), -100.0);
    }
}
