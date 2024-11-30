use nannou::prelude::*;
use nannou::winit::window::Window as WinitWindow;

mod displacer;
use displacer::Displacer;

fn main() {
    nannou::app(model).update(update).run();
}

struct DisplacerConfig {
    displacer: Displacer,
    radius: f32,
    strength: f32,
}

struct Model {
    _window: window::Id,
    circle_radius: f32,
    grid_size: usize,
    displacer_configs: Vec<DisplacerConfig>,
}

fn model(app: &App) -> Model {
    let w: i32 = 700;
    let h: i32 = 700;

    let _window = app
        .new_window()
        .size(w as u32, h as u32)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(_window).unwrap();
    let winit_window: &WinitWindow = window.winit_window();
    winit_window
        .set_outer_position(nannou::winit::dpi::PhysicalPosition::new(0, 0));

    let displacer_configs = vec![
        DisplacerConfig {
            displacer: Displacer::new(vec2(0.0, 0.0), 100.0, 50.0),
            radius: 100.0,
            strength: 50.0,
        },
        DisplacerConfig {
            displacer: Displacer::new(
                vec2(w as f32 / 4.0, h as f32 / 4.0),
                100.0,
                50.0,
            ),
            radius: 100.0,
            strength: 50.0,
        },
    ];

    Model {
        _window,
        circle_radius: 2.0,
        grid_size: 64,
        displacer_configs,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    for config in &mut model.displacer_configs {
        config.displacer.update(Some(displacer::DisplacerState {
            position: None,
            radius: Some(map_range(
                (app.time * 16.0).sin(),
                -1.0,
                1.0,
                config.radius,
                config.radius * 1.5,
            )),
            strength: Some(map_range(
                (app.time * 24.0).sin(),
                -1.0,
                1.0,
                config.strength,
                config.strength * 1.5,
            )),
        }))
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    let draw = app.draw();
    draw.background().color(rgb(0.1, 0.1, 0.1));
    let window = app.window_rect();

    let grid =
        create_grid(window.w(), window.h(), model.grid_size, |x, y| vec2(x, y));

    for point in grid {
        let total_displacement = model
            .displacer_configs
            .iter()
            .fold(vec2(0.0, 0.0), |acc, config| {
                acc + config.displacer.influence(point)
            });
        draw.ellipse()
            .radius(model.circle_radius)
            .xy(point + total_displacement)
            .color(BEIGE);
    }

    draw.to_frame(app, &frame).unwrap()
}

fn create_grid<F>(
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
