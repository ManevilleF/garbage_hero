use bevy::prelude::*;
use std::f32::consts::TAU;

/// Computes the ideal circle radius for `amount` of points separated by `max_distance`
///
/// # Arguments
///
/// * `min_radius` - The minimum radius
/// * `max_distance` - The maximum distance between points
/// * `amount` - The number of points to be distributed.
///
/// # Returns
///
/// * The calculated radius which ensures the points are evenly distributed around the circle.
fn radius(min_radius: f32, max_distance: f32, amount: usize) -> f32 {
    let circumference = amount as f32 * max_distance;
    let calculated_radius = circumference / TAU;

    calculated_radius.max(min_radius)
}

/// Rotates a given point by a specified angle
///
/// # Arguments
///
/// * `point` - The point to be rotated.
/// * `angle` - The angle to rotate the point by.
///
/// # Returns
///
/// * The rotated point as a `Vec2`.
fn rotated_point(point: &Vec2, angle: f32) -> Vec2 {
    let rotated_x = point.x.mul_add(angle.cos(), -point.y * angle.sin());
    let rotated_y = point.x.mul_add(angle.sin(), point.y * angle.cos());
    Vec2::new(rotated_x, rotated_y)
}

/// A component for managing circular distributions of points.
///
/// This struct allows for the generation and retrieval of points distributed
/// evenly in an oriented circle.
#[derive(Debug, Component, Reflect)]
pub struct CircularDistribution {
    /// The minimum radius of the circle .
    pub min_radius: f32,
    /// The maximum distance between points.
    pub max_distance: f32,
    /// Cached angle for oriented rotation
    current_angle: f32,
    /// Cached points for the distribution.
    points: Vec<Vec2>,
}

impl CircularDistribution {
    #[must_use]
    #[inline]
    /// Constructs a new circular distribution of minimum radius `min_radius` and
    /// `max_distance` between points
    pub const fn new(min_radius: f32, max_distance: f32) -> Self {
        Self {
            min_radius,
            max_distance,
            current_angle: 0.0,
            points: Vec::new(),
        }
    }

    pub fn radius(&self, amount: usize) -> f32 {
        radius(self.min_radius, self.max_distance, amount)
    }

    /// Updates the amount of points in the circle, effectively updating the points caches
    ///
    /// # Arguments
    ///
    /// * `amount` - The number of points to be distributed.
    pub fn set_amount(&mut self, amount: usize) {
        if amount == self.points.len() {
            return;
        }
        let radius = self.radius(amount);
        let theta = TAU / (amount as f32);
        // Circle
        self.points = (0..amount)
            .map(|i| {
                let step = theta.mul_add(i as f32, self.current_angle);
                let x = radius * step.cos();
                let y = radius * step.sin();
                Vec2::new(x, y)
            })
            .collect();
    }

    pub fn rotate(&mut self, angle: f32) {
        self.current_angle += angle;
        self.points
            .iter_mut()
            .for_each(|point| *point = rotated_point(point, angle));
    }

    /// Retrieves the points
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }

    /// Finds the closest point to a given direction in the circular distribution.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction vector to compare against.
    ///
    /// # Returns
    ///
    /// * The closest point as a Vec2.
    pub fn find_closest_aligned_point(&self, direction: Dir2) -> Option<(usize, Vec2)> {
        // Convert direction to an angle
        let target_angle = direction.y.atan2(direction.x);

        let mut index = 0;
        let mut current_diff: f32 = f32::MAX;
        // Find the point with the minimum angular difference
        for (i, point) in self.points.iter().enumerate() {
            let angle = point.y.atan2(point.x);
            let diff = (target_angle - angle).abs();

            if diff < current_diff {
                index = i;
                current_diff = diff;
            }
        }

        self.points.get(index).map(|p| (index, *p))
    }
}

/// A component for managing circular distributions of points.
///
/// This struct allows for the generation and retrieval of points distributed
/// evenly in an oriented circle.
#[derive(Debug, Component, Reflect)]
pub struct ArcDistribution {
    /// The minimum radius of the arc.
    pub min_radius: f32,
    /// The maximum distance between points.
    pub max_distance: f32,
    /// Cached angle for oriented rotation
    current_angle: f32,
    /// Cached points for the distribution.
    points: Vec<Vec2>,
}

impl ArcDistribution {
    #[must_use]
    #[inline]
    /// Constructs a new arc distribution of minimum radius `min_radius` and
    /// `max_distance` between points
    pub const fn new(min_radius: f32, max_distance: f32) -> Self {
        Self {
            min_radius,
            max_distance,
            current_angle: 0.0,
            points: Vec::new(),
        }
    }

    pub fn radius(&self, amount: usize) -> f32 {
        radius(self.min_radius, self.max_distance, amount)
    }

    /// Updates the amount of points in the circle, effectively updating the points caches
    ///
    /// # Arguments
    ///
    /// * `amount` - The number of points to be distributed.
    pub fn set_amount(&mut self, amount: usize) {
        if amount == self.points.len() {
            return;
        }
        let radius = self.radius(amount);
        let theta = TAU / (amount as f32);
        let max_arc_points = amount / 4;
        self.points = (0..amount)
            .map(|i| {
                let r = ((i / max_arc_points) as f32).mul_add(self.max_distance, radius);
                let inner = i % max_arc_points;
                let mut step = theta.mul_add(inner as f32, self.current_angle);
                if inner % 2 == 0 {
                    step = -step;
                }
                let x = r * step.cos();
                let y = r * step.sin();
                Vec2::new(x, y)
            })
            .collect();
    }

    pub fn set_direction(&mut self, direction: Dir2) {
        let angle = direction.y.atan2(direction.x);
        let angle_diff = angle - self.current_angle;
        self.current_angle = angle;
        self.points
            .iter_mut()
            .for_each(|point| *point = rotated_point(point, angle_diff));
    }

    /// Retrieves the points
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }
}
