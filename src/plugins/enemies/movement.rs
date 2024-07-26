use bevy::prelude::*;

use crate::plugins::garbage::GarbageBody;

pub struct EnemyMovementPlugin;

impl Plugin for EnemyMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemyMovement>()
            .register_type::<EnemyMovementState>()
            .add_systems(FixedUpdate, move_enemy);
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct EnemyMovement {
    elapsed: f32,
    pub speed: f32,
    pub spawn_position: Vec3,
}

impl EnemyMovement {
    #[inline]
    pub const fn new(speed: f32, position: Vec3) -> Self {
        Self {
            speed,
            spawn_position: position,
            elapsed: 0.0,
        }
    }
}

#[derive(Debug, Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub enum EnemyMovementState {
    #[default]
    Idle,
    Attack(Vec3),
    Returning,
}

fn move_enemy(
    mut enemies: Query<(&mut Transform, &mut EnemyMovement, &mut EnemyMovementState)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut transform, mut movement, mut state) in &mut enemies {
        let position = transform.translation;
        let speed = movement.speed;
        let target_position = match *state {
            EnemyMovementState::Idle => {
                // Figure-eight pattern
                let delta = Vec3::new(
                    speed * movement.elapsed.sin(),
                    0.0,
                    speed * (2.0 * movement.elapsed).sin() / 2.0,
                );
                movement.elapsed += dt;
                movement.spawn_position + delta
            }
            EnemyMovementState::Attack(target) => 'att: {
                const TARGET_Y: f32 = 10.0;
                let Ok(dir) = Dir3::new(target - position) else {
                    *state = EnemyMovementState::Returning;
                    break 'att position;
                };
                if transform.translation.y < TARGET_Y {
                    let target = Vec3::new(
                        dir.x.mul_add(speed, position.x),
                        TARGET_Y,
                        dir.z.mul_add(speed, position.z),
                    );
                    break 'att target;
                }
                (target - position) * speed * 3.0 * dt
            }
            EnemyMovementState::Returning => {
                movement.spawn_position = position;
                movement.elapsed = 0.0;
                *state = EnemyMovementState::Idle;
                position
            }
        };

        if let Ok(direction) = Dir3::new(position - target_position) {
            transform.look_to(direction, Dir3::Y);
        }
        transform.translation = target_position;
    }
}
