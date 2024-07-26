use std::f32::consts::PI;

use bevy::prelude::*;

use crate::plugins::garbage::PointDistribution;

pub struct GarbageBodyPlugin;

impl Plugin for GarbageBodyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GarbageBody>()
            .add_systems(FixedUpdate, update_bodies);

        #[cfg(feature = "debug")]
        app.add_systems(
            PostUpdate,
            draw_gizmos
                .after(avian3d::prelude::PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Debug, Reflect)]
struct Point {
    position: Vec3,
    direction: Dir3,
}

#[derive(Debug, Reflect)]
pub struct Chain {
    points: Vec<Point>,
    point_radius: f32,
    // rads
    max_angle: f32,
    min_y: f32,
}

impl Chain {
    #[inline]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn update(&mut self) {
        let len = self.points.len();
        if len <= 1 {
            return;
        }

        for i in 1..len {
            let prev_pos = self.points[i - 1].position;
            let prev_dir = self.points[i - 1].direction;
            let current_point = &mut self.points[i];
            let direction = Dir3::new(current_point.position - prev_pos).unwrap_or(Dir3::Z);
            let distance = self.point_radius;

            // Apply distance constraint
            current_point.position = prev_pos + direction * distance;
            current_point.direction = direction;

            // Apply angle constraint
            let mut angle = prev_dir.angle_between(*direction);
            if angle.abs() > self.max_angle {
                angle = self.max_angle * angle.signum();
                let rotation = Quat::from_axis_angle(direction.cross(*prev_dir).normalize(), angle);
                current_point.direction = Dir3::new(rotation * *current_point.direction)
                    .unwrap_or(current_point.direction);
                current_point.position = prev_pos + current_point.direction * distance;
            }

            // Ensure no point falls below MIN_Y
            if current_point.position.y < self.min_y {
                current_point.position.y = self.min_y;
            }
        }
    }

    pub fn new(
        count: usize,
        base_pos: Vec3,
        point_radius: f32,
        max_angle: f32,
        min_y: f32,
    ) -> Self {
        let base_dir = Dir3::Z;
        let points = (0..count)
            .map(|i| {
                let position = base_pos + *base_dir * i as f32;
                Point {
                    position,
                    direction: base_dir,
                }
            })
            .collect();

        Self {
            points,
            point_radius,
            max_angle,
            min_y,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GarbageBody {
    // Anchored to Transform translation
    pub dorsal: Chain,
    pub offset: f32,
}

impl GarbageBody {
    pub fn new(amount: usize, pos: Vec3, radius: f32, offset: f32) -> Self {
        Self {
            dorsal: Chain::new(amount, pos, radius, PI, 1.0),
            offset,
        }
    }

    pub fn full_length(&self) -> f32 {
        self.dorsal.len() as f32 * self.dorsal.point_radius
    }

    pub fn compute_3d_positions(&self, len: usize, distribution: &PointDistribution) -> Vec<Vec3> {
        let mut res = Vec::with_capacity(len);
        let mut i = 0_usize;
        let mut j = 0_usize;
        while i < len && j < self.dorsal.len() {
            let dorsal_point = &self.dorsal.points[j];
            let rot = Quat::from_rotation_arc(Vec3::Y, *dorsal_point.direction);
            let points = distribution.points();
            for p in points {
                let p = rot * Vec3::new(p.x, 0.0, p.y);
                res.push(dorsal_point.position + p);
            }
            i += points.len();
            j += 1;
        }
        res
    }
}

fn update_bodies(mut bodies: Query<(&GlobalTransform, &mut GarbageBody)>) {
    for (gtr, mut body) in &mut bodies {
        let forward = gtr.forward();
        body.dorsal.points[0].position = gtr.translation() + forward * body.offset;
        body.dorsal.points[0].direction = forward;
        body.dorsal.update();
    }
}

#[cfg(feature = "debug")]
fn draw_gizmos(mut gizmos: Gizmos, bodies: Query<&GarbageBody>) {
    for body in &bodies {
        // dorsal
        for point in &body.dorsal.points {
            // gizmos.circle(
            //     point.position,
            //     point.direction,
            //     body.dorsal.point_radius,
            //     Color::BLACK,
            // );
            gizmos.arrow(
                point.position,
                point.position + *point.direction * body.dorsal.point_radius,
                Color::BLACK,
            );
        }
    }
}
