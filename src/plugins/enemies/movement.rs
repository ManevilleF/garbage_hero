use bevy::prelude::*;

use super::body::Body;

pub struct EnemyMovementPlugin;

impl Plugin for EnemyMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemyMovement>()
            .add_systems(FixedUpdate, move_enemy);
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct EnemyMovement {
    pub time_offset: f32,
    pub speed: f32,
    pub spawn_position: Vec3,
}

fn move_enemy(mut enemies: Query<(&mut Transform, &EnemyMovement, &Body)>, time: Res<Time>) {
    let elapsed = time.elapsed_seconds();
    for (mut transform, movement, body) in &mut enemies {
        let elapsed_time = elapsed + movement.time_offset;
        let speed = movement.speed;

        // Figure-eight pattern

        let delta = Vec3::new(
            speed * elapsed_time.sin(),
            0.0,
            speed * (2.0 * elapsed_time).sin() / 2.0,
        );

        let coef = body.dorsal.len() as f32 * (1.0 / movement.speed);
        let new_translation = movement.spawn_position + delta * coef;
        if let Ok(direction) = Dir3::new(transform.translation - new_translation) {
            transform.look_to(direction, Dir3::Y);
        }
        transform.translation = new_translation;
    }
}
