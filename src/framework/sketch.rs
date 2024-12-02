// framework/sketch.rs
use nannou::prelude::*;

use super::metadata::SketchMetadata;

pub trait Sketch {
    type Model;

    fn metadata(&self) -> &SketchMetadata;
    fn model(&self, app: &App) -> Self::Model;
    fn update(&self, app: &App, model: &mut Self::Model, update: Update);
    fn view(&self, app: &App, model: &Self::Model, frame: Frame);
}
