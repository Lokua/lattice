use nannou::{
    glam::Vec2,
    rand::{random_f32, random_range},
};

use super::prelude::*;

// https://github.com/Lokua/p5/blob/main/src/sketches/drop3.mjs

pub struct DropZone {
    pub center: Vec2,
}

impl DropZone {
    pub fn new(center: Vec2) -> Self {
        Self { center }
    }

    pub fn point_within_circular_zone(
        &self,
        inner_radius: f32,
        outer_radius: f32,
    ) -> Vec2 {
        let angle = random_f32() * TWO_PI;
        let radius = f32::sqrt(random_range(
            inner_radius * inner_radius,
            outer_radius * outer_radius,
        ));
        Vec2::new(
            self.center.x + radius * angle.cos(),
            self.center.y + radius * angle.sin(),
        )
    }

    pub fn point_within_rectangular_zone_advanced(
        &self,
        inner_radius: f32,
        outer_radius: f32,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
    ) -> Vec2 {
        let random = || {
            (
                self.center.x + random_range(-x_min, x_max),
                self.center.y + random_range(-y_min, y_max),
            )
        };
        let mut point = Vec2::from(random());
        while !self.is_in_rectangular_zone(point, inner_radius, outer_radius) {
            point = Vec2::from(random());
        }
        point
    }

    pub fn point_within_rectangular_zone(
        &self,
        inner_radius: f32,
        outer_radius: f32,
    ) -> Vec2 {
        self.point_within_rectangular_zone_advanced(
            inner_radius,
            outer_radius,
            outer_radius,
            outer_radius,
            outer_radius,
            outer_radius,
        )
    }

    pub fn point_within_rectangular_zone_top_bottom(
        &self,
        inner_radius: f32,
        outer_radius: f32,
    ) -> Vec2 {
        self.point_within_rectangular_zone_advanced(
            inner_radius,
            outer_radius,
            inner_radius,
            inner_radius,
            outer_radius,
            outer_radius,
        )
    }

    pub fn point_within_rectangular_zone_top(
        &self,
        inner_radius: f32,
        outer_radius: f32,
    ) -> Vec2 {
        self.point_within_rectangular_zone_advanced(
            inner_radius,
            outer_radius,
            inner_radius,
            inner_radius,
            inner_radius,
            outer_radius,
        )
    }
    pub fn point_within_rectangular_zone_right(
        &self,
        inner_radius: f32,
        outer_radius: f32,
    ) -> Vec2 {
        self.point_within_rectangular_zone_advanced(
            inner_radius,
            outer_radius,
            inner_radius,
            outer_radius,
            inner_radius,
            inner_radius,
        )
    }
    pub fn point_within_rectangular_zone_bottom(
        &self,
        inner_radius: f32,
        outer_radius: f32,
    ) -> Vec2 {
        self.point_within_rectangular_zone_advanced(
            inner_radius,
            outer_radius,
            inner_radius,
            inner_radius,
            outer_radius,
            inner_radius,
        )
    }
    pub fn point_within_rectangular_zone_left(
        &self,
        inner_radius: f32,
        outer_radius: f32,
    ) -> Vec2 {
        self.point_within_rectangular_zone_advanced(
            inner_radius,
            outer_radius,
            outer_radius,
            inner_radius,
            inner_radius,
            inner_radius,
        )
    }

    pub fn is_in_rectangular_zone(
        &self,
        point: Vec2,
        inner_radius: f32,
        outer_radius: f32,
    ) -> bool {
        let delta = (point - self.center).abs();
        let max_dist = delta.x.max(delta.y);
        max_dist >= inner_radius && max_dist <= outer_radius
    }
}
