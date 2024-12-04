use nannou::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::logging::*;

pub struct FrameController {
    #[allow(dead_code)]
    target_fps: f64,
    frame_duration: Duration,
    // Captured every loop regardless of skipped/rendered frames
    last_frame_time: Instant,
    last_render_time: Instant,
    accumulator: Duration,
    frame_count: u64,
    render_flag: bool,
    paused: bool,
}

impl FrameController {
    pub fn new(target_fps: f64) -> Self {
        let now = Instant::now();
        Self {
            target_fps,
            frame_duration: Duration::from_secs_f64(1.0 / target_fps),
            last_frame_time: now,
            last_render_time: now,
            accumulator: Duration::ZERO,
            frame_count: 0,
            render_flag: false,
            paused: false,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_frame_time;
        self.accumulator += elapsed;
        self.last_frame_time = now;
        self.render_flag = false;

        // Render frames for each interval the accumulator surpasses
        while self.accumulator >= self.frame_duration {
            self.accumulator -= self.frame_duration;
            self.frame_count += 1;
            self.render_flag = true;
        }

        // Adjust for small drifts (if the drift is negligible, round up to the next frame)
        if self.accumulator < Duration::from_millis(1) {
            self.accumulator = Duration::ZERO;
        }

        if self.render_flag {
            trace!(
                "Rendering. Time since last: {:?}",
                now - self.last_render_time
            );
            self.last_render_time = now;
        } else {
            trace!("Skipping render this frame.");
        }
    }

    pub fn should_render(&self) -> bool {
        self.render_flag && !self.paused
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn get_fps(&self) -> f64 {
        self.target_fps
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
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

pub fn get_frame_count() -> u64 {
    let controller = CONTROLLER.lock().unwrap();
    controller.as_ref().map_or(0, |c| c.get_frame_count())
}

pub fn get_fps() -> f64 {
    let controller = CONTROLLER.lock().unwrap();
    controller.as_ref().map_or(0.0, |c| c.get_fps())
}

pub fn is_paused() -> bool {
    let controller = CONTROLLER.lock().unwrap();
    controller.as_ref().map_or(false, |c| c.is_paused())
}

pub fn set_paused(paused: bool) {
    let mut controller = CONTROLLER.lock().unwrap();
    if let Some(controller) = controller.as_mut() {
        controller.set_paused(paused);
    } else {
        warn!("Unable to paused frame_controller");
    }
}
