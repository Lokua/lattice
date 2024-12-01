use nannou::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct FrameController {
    #[allow(dead_code)]
    target_fps: f64,
    frame_duration: Duration,
    last_frame_time: Instant,
    accumulator: Duration,
    frame_count: u64,
    render_flag: bool,
}

impl FrameController {
    pub fn new(target_fps: f64) -> Self {
        Self {
            target_fps,
            frame_duration: Duration::from_secs_f64(1.0 / target_fps),
            last_frame_time: Instant::now(),
            accumulator: Duration::ZERO,
            frame_count: 0,
            render_flag: false,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_frame_time;

        if elapsed >= Duration::from_millis(1) {
            println!("\nFrame Controller Debug:");
            println!("  Elapsed since last check: {:?}", elapsed);
            println!("  Current accumulator: {:?}", self.accumulator);

            self.accumulator += elapsed;
            println!(
                "  Accumulator after adding elapsed: {:?}",
                self.accumulator
            );
            println!("  Target frame duration: {:?}", self.frame_duration);

            self.last_frame_time = now;

            self.render_flag = self.accumulator >= self.frame_duration;
            if self.render_flag {
                self.accumulator -= self.frame_duration;
                self.frame_count += 1;
                println!("  WILL RENDER - Frame {}", self.frame_count);
                println!("  Remaining accumulator: {:?}", self.accumulator);
            } else {
                println!("  Skipping render");
            }
        } else {
            self.render_flag = false;
        }
    }

    pub fn should_render(&self) -> bool {
        self.render_flag
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }
}

static CONTROLLER: Lazy<Mutex<Option<FrameController>>> =
    Lazy::new(|| Mutex::new(None));

pub fn init_controller(fps: f64) {
    let mut controller = CONTROLLER.lock().unwrap();
    *controller = Some(FrameController::new(fps));
}

pub fn wrapped_update<M, F>(
    app: &App,
    model: &mut M,
    update: Update,
    update_fn: F,
) where
    F: FnOnce(&App, &mut M, Update),
{
    let should_update = {
        let mut controller = CONTROLLER.lock().unwrap();
        if let Some(controller) = controller.as_mut() {
            controller.update();
            controller.should_render()
        } else {
            false
        }
    };

    if should_update {
        update_fn(app, model, update);
    }
}

pub fn wrapped_view<M, F>(app: &App, model: &M, frame: Frame, view_fn: F)
where
    F: FnOnce(&App, &M, Frame),
{
    let should_render = {
        let controller = CONTROLLER.lock().unwrap();
        controller.as_ref().map_or(false, |c| c.should_render())
    };

    if should_render {
        view_fn(app, model, frame);
    }
}
