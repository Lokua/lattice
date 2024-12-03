use super::controls::Controls;

pub struct SketchConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub fps: f64,
    pub bpm: f32,
    pub w: i32,
    pub h: i32,
}

pub trait SketchModel {
    fn controls(&mut self) -> Option<&mut Controls> {
        None
    }
}
