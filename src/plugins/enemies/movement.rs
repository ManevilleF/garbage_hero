use avian3d::prelude::CollidingEntities;
use bevy::prelude::*;

use crate::{
    plugins::{garbage::CollectorConfig, player::Player},
    Dead, GameState,
};

const PLUNGE_HEIGHT: f32 = 25.0;
const MAX_DISTANCE: f32 = 100.0;

pub struct EnemyMovementPlugin;

impl Plugin for EnemyMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemyMovement>()
            .register_type::<EnemyMovementState>()
            .add_systems(
                FixedUpdate,
                (move_enemy, detect_players).run_if(in_state(GameState::Running)),
            )
            .add_systems(PostUpdate, handle_state_change);
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct EnemyMovement {
    elapsed: f32,
    pub speed: f32,
    pub spawn_position: Vec3,
    pub anchor_position: Vec3,
}

impl EnemyMovement {
    #[inline]
    pub const fn new(speed: f32, position: Vec3) -> Self {
        Self {
            speed,
            spawn_position: position,
            anchor_position: position,
            elapsed: 0.0,
        }
    }
}

#[derive(Debug, Component, Reflect, Clone, Copy)]
#[reflect(Component)]
pub struct PlayerDetector {
    last_detection: f32,
    pub attack_cooldown: f32,
}

impl PlayerDetector {
    pub const fn new(cooldown: f32) -> Self {
        Self {
            last_detection: 0.0,
            attack_cooldown: cooldown,
        }
    }
}

#[derive(Debug, Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub enum EnemyMovementState {
    #[default]
    Idle,
    PrepareAttack(Vec3),
    PlungeAttack(Vec3),
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
                movement.anchor_position + delta
            }
            EnemyMovementState::PrepareAttack(target) => 'att: {
                if position.distance(target) < 1.0 {
                    *state = EnemyMovementState::PlungeAttack(Vec3::new(target.x, 0.5, target.z));
                    break 'att position;
                }
                let Ok(dir) = Dir3::new(target - position) else {
                    *state = EnemyMovementState::PlungeAttack(Vec3::new(target.x, 0.5, target.z));
                    break 'att position;
                };
                position + *dir * speed * 1.5 * dt
            }
            EnemyMovementState::PlungeAttack(target) => 'att: {
                if position.distance(target) < 1.0 {
                    *state = EnemyMovementState::Returning;
                    break 'att position;
                }
                let Ok(dir) = Dir3::new(target - position) else {
                    *state = EnemyMovementState::Returning;
                    break 'att position;
                };
                position + *dir * speed * 2.0 * dt
            }
            EnemyMovementState::Returning => {
                movement.anchor_position.x = position.x;
                movement.anchor_position.z = position.z;
                if movement.anchor_position.distance(movement.spawn_position) > MAX_DISTANCE {
                    *state = EnemyMovementState::PrepareAttack(movement.spawn_position)
                } else {
                    movement.elapsed = 0.0;
                    *state = EnemyMovementState::Idle;
                }
                position
            }
        };

        if let Ok(direction) = Dir3::new(position - target_position) {
            transform.look_to(direction, Dir3::Y);
        }
        transform.translation = target_position;
    }
}

fn detect_players(
    time: Res<Time>,
    mut detectors: Query<(&Parent, &mut PlayerDetector, &CollidingEntities)>,
    mut enemies: Query<&mut EnemyMovementState>,
    players: Query<&GlobalTransform, (With<Player>, Without<Dead>)>,
) {
    let dt = time.delta_seconds();
    for (parent, mut detector, collisions) in &mut detectors {
        detector.last_detection += dt;
        if detector.last_detection < detector.attack_cooldown {
            continue;
        }
        let mut state = enemies.get_mut(parent.get()).unwrap();
        if matches!(*state, EnemyMovementState::PrepareAttack(_)) {
            continue;
        }
        let Some(gtr) = collisions.iter().find_map(|e| players.get(*e).ok()) else {
            continue;
        };
        let target = gtr.translation();
        *state = EnemyMovementState::PrepareAttack(Vec3::new(target.x, PLUNGE_HEIGHT, target.z));
        detector.last_detection = 0.0;
    }
}

fn handle_state_change(
    enemies: Query<(&EnemyMovementState, &Children), Changed<EnemyMovementState>>,
    mut collectors: Query<&mut CollectorConfig>,
) {
    for (state, children) in &enemies {
        let mut configs = collectors.iter_many_mut(children);
        while let Some(mut config) = configs.fetch_next() {
            match state {
                EnemyMovementState::PrepareAttack(_) | EnemyMovementState::PlungeAttack(_) => {
                    config.enabled = false
                }
                EnemyMovementState::Idle | EnemyMovementState::Returning => config.enabled = true,
            }
        }
    }
}
