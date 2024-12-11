use geom::Ellipse;
use nannou::color::{LinSrgb, Srgb};
use nannou::{
    prelude::*,
    rand::{thread_rng, Rng},
};

pub const PHI_F32: f32 = 1.618033988749894_f32;
pub const TWO_PI: f32 = PI * 2.0;

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

pub fn create_grid<F>(
    w: f32,
    h: f32,
    divisions: usize,
    transform_xy: F,
) -> Vec<Vec2>
where
    F: Fn(f32, f32) -> Vec2,
{
    let mut grid = Vec::new();
    let cell_size = f32::min(w, h) / divisions as f32;
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

    grid
}

pub fn set_window_position(app: &App, window_id: window::Id, x: i32, y: i32) {
    app.window(window_id)
        .unwrap()
        .winit_window()
        .set_outer_position(nannou::winit::dpi::PhysicalPosition::new(x, y));
}

pub fn uuid_5() -> String {
    uuid(5)
}

/// Generate a random string of the specified length consisting of
/// lowercase letters and numbers.
pub fn uuid(length: usize) -> String {
    const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
    const NUMBERS: &str = "0123456789";

    let mut rng = thread_rng();
    (0..length)
        .map(|_| {
            if rng.gen_bool(0.5) {
                LETTERS
                    .chars()
                    .nth(rng.gen_range(0..LETTERS.len()))
                    .unwrap()
            } else {
                NUMBERS
                    .chars()
                    .nth(rng.gen_range(0..NUMBERS.len()))
                    .unwrap()
            }
        })
        .collect()
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

pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
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

pub fn safe_range(min: f32, max: f32) -> (f32, f32) {
    let a = if max < min { max } else { min };
    let mut b = if min > max { min } else { max };
    if a == b {
        b += 0.0001;
    }
    (a, b)
}
