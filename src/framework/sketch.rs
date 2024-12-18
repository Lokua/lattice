use nannou::prelude::*;

use super::prelude::*;

pub struct SketchConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub fps: f32,
    pub bpm: f32,
    pub w: i32,
    pub h: i32,
    pub gui_w: Option<u32>,
    pub gui_h: Option<u32>,
}

pub trait SketchModel {
    fn controls(&mut self) -> Option<&mut Controls> {
        None
    }

    fn clear_color(&self) -> Rgba {
        Rgba::new(0.0, 0.0, 0.0, 0.0)
    }

    fn window_rect(&mut self) -> Option<&mut WindowRect> {
        None
    }

    fn set_window_rect(&mut self, rect: Rect) {
        if let Some(window_rect) = self.window_rect() {
            window_rect.set_current(rect);
        }
    }
}

pub struct WindowRect {
    current: Rect,
    last: Rect,
}

impl WindowRect {
    pub fn new(initial: Rect) -> Self {
        Self {
            current: initial,
            last: initial,
        }
    }

    pub fn set_current(&mut self, rect: Rect) {
        self.current = rect;
    }

    pub fn changed(&self) -> bool {
        (self.current.w() != self.last.w())
            || (self.current.h() != self.last.h())
    }

    pub fn commit(&mut self) {
        self.last = self.current;
    }

    pub fn w(&self) -> f32 {
        self.current.w()
    }

    pub fn h(&self) -> f32 {
        self.current.h()
    }
}
