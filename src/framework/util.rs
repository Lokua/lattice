use nannou::prelude::*;

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

/// Determines whether the current frame should be processed based on the desired FPS.
/// Returns `true` if the frame should be processed, `false` otherwise.
pub fn should_render_frame(app: &App, target_fps: f64) -> bool {
    let app_fps = app.fps() as f64;
    let target_fps = target_fps;
    let desired_frame_interval =
        ((app_fps / target_fps).round()).max(1.0) as u64;
    let elapsed_frames = app.elapsed_frames();
    elapsed_frames % desired_frame_interval == 0
}

pub fn set_window_position(app: &App, window_id: window::Id, x: i32, y: i32) {
    app.window(window_id)
        .unwrap()
        .winit_window()
        .set_outer_position(nannou::winit::dpi::PhysicalPosition::new(x, y));
}
