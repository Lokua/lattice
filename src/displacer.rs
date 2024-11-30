use nannou::prelude::*;

#[derive(Debug, Clone)]
pub struct Displacer {
    pub position: Vec2,
    pub radius: f32,
    pub strength: f32, // Added strength as a property
}

impl Displacer {
    /// Create a new Displacer with position, radius, and strength
    pub fn new(position: Vec2, radius: f32, strength: f32) -> Self {
        Self {
            position,
            radius,
            strength,
        }
    }

    /// Update the position, radius, or strength of the Displacer
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

    /// Influence a grid point based on the Displacer's state
    pub fn influence(&self, grid_point: Vec2) -> Vec2 {
        let radius = self.radius.max(f32::EPSILON);
        let distance_to_center = grid_point.distance(self.position);

        if distance_to_center == 0.0 {
            return vec2(0.0, 0.0);
        }

        let proximity = 1.0 - distance_to_center / (radius * 2.0);
        let distance_factor = proximity.max(0.0);
        let angle = (grid_point.y - self.position.y)
            .atan2(grid_point.x - self.position.x);
        let force = self.strength * distance_factor.powi(2);
        let dx = angle.cos() * force;
        let dy = angle.sin() * force;

        vec2(dx, dy)
    }
}

/// Optional state updates for Displacer
#[derive(Debug, Clone)]
pub struct DisplacerState {
    pub position: Option<Vec2>,
    pub radius: Option<f32>,
    pub strength: Option<f32>, // Added strength to the state
}
