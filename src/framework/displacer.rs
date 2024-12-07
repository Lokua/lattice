use nannou::prelude::*;
use std::sync::Arc;

impl std::fmt::Debug for Displacer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Displacer")
            .field("position", &self.position)
            .field("radius", &self.radius)
            .field("strength", &self.strength)
            .field("custom_distance_fn", &"<function>")
            .finish()
    }
}

pub type CustomDistanceFn =
    Option<Arc<dyn Fn(Vec2, Vec2) -> f32 + Send + Sync>>;

pub struct Displacer {
    pub position: Vec2,
    pub radius: f32,
    pub strength: f32,
    pub custom_distance_fn: CustomDistanceFn,
}

impl Displacer {
    pub fn new(
        position: Vec2,
        radius: f32,
        strength: f32,
        custom_distance_fn: CustomDistanceFn,
    ) -> Self {
        Self {
            position,
            radius,
            strength,
            custom_distance_fn,
        }
    }

    pub fn update(&mut self, state: Option<DisplacerState>) {
        if let Some(state) = state {
            if let Some(position) = state.position {
                self.position = position;
            }
            if let Some(radius) = state.radius {
                self.radius = radius;
            }
            if let Some(strength) = state.strength {
                self.strength = strength;
            }
        }
    }

    pub fn influence(&self, grid_point: Vec2) -> Vec2 {
        let radius = self.radius.max(f32::EPSILON);

        let distance_to_center = match &self.custom_distance_fn {
            Some(f) => f(grid_point, self.position),
            None => grid_point.distance(self.position),
        };

        if distance_to_center == 0.0 {
            return vec2(0.0, 0.0);
        }

        let proximity = 1.0 - distance_to_center / (radius * 2.0);
        // let proximity = 1.0 - distance_to_center / (radius + self.strength);
        let distance_factor = proximity.max(0.0);
        let angle = (grid_point.y - self.position.y)
            .atan2(grid_point.x - self.position.x);
        let force = self.strength * distance_factor.powi(2);
        let dx = angle.cos() * force;
        let dy = angle.sin() * force;

        vec2(dx, dy)
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    pub fn set_custom_distance_fn(
        &mut self,
        custom_distance_fn: CustomDistanceFn,
    ) {
        self.custom_distance_fn = custom_distance_fn;
    }
}

#[derive(Debug, Clone)]
pub struct DisplacerState {
    pub position: Option<Vec2>,
    pub radius: Option<f32>,
    pub strength: Option<f32>,
}

impl DisplacerState {
    pub fn new() -> Self {
        Self {
            position: None,
            radius: None,
            strength: None,
        }
    }

    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = Some(strength);
        self
    }
}
