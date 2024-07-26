use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

pub struct EnemyBodyPlugin;

impl Plugin for EnemyBodyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Body>()
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

#[derive(Debug, Copy, Clone, Reflect)]
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
            let prev_point = self.points[i - 1];
            let current_point = &mut self.points[i];
            let direction =
                Dir3::new(current_point.position - prev_point.position).unwrap_or(Dir3::Z);
            let distance = self.point_radius;

            // Apply distance constraint
            current_point.position = prev_point.position + direction * distance;
            current_point.direction = direction;

            // Apply angle constraint
            let mut angle = prev_point.direction.angle_between(*direction);
            if angle.abs() > self.max_angle {
                angle = self.max_angle * angle.signum();
                let rotation = Quat::from_axis_angle(
                    direction.cross(*prev_point.direction).normalize(),
                    angle,
                );
                current_point.direction = Dir3::new(rotation * *current_point.direction)
                    .unwrap_or(current_point.direction);
                current_point.position = prev_point.position + current_point.direction * distance;
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
pub struct Body {
    // Anchored to Transform translation
    pub dorsal: Chain,
    pub offset: f32,
}

impl Body {
    pub fn new(amount: usize, pos: Vec3, radius: f32) -> Self {
        Self {
            dorsal: Chain::new(amount, pos, radius, FRAC_PI_2, 1.0),
            offset: radius,
        }
    }
}

fn update_bodies(mut bodies: Query<(&GlobalTransform, &mut Body)>) {
    for (gtr, mut body) in &mut bodies {
        let forward = gtr.forward();
        body.dorsal.points[0].position = gtr.translation() + forward * body.offset;
        body.dorsal.points[0].direction = forward;
        body.dorsal.update();
    }
}

#[cfg(feature = "debug")]
fn draw_gizmos(mut gizmos: Gizmos, bodies: Query<&Body>) {
    for body in &bodies {
        // dorsal
        for point in &body.dorsal.points {
            gizmos.circle(
                point.position,
                point.direction,
                body.dorsal.point_radius,
                Color::BLACK,
            );
            gizmos.arrow(
                point.position,
                point.position + *point.direction,
                Color::BLACK,
            );
        }
    }
}
