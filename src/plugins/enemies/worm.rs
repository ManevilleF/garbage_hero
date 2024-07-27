use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    plugins::{garbage::CollectorConfig, particles::DeathEffect, player::Player},
    Damage, Dead, GameState, Health, ObjectLayer,
};

use super::{assets::EnemyAssets, Enemy, PlayerDetector};

const PLUNGE_HEIGHT: f32 = 25.0;
const MAX_DISTANCE: f32 = 100.0;

const BASE_HEALTH: u16 = 100;
const BASE_DAMAGE: u16 = 10;

pub struct WormPlugin;

impl Plugin for WormPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WormMovement>()
            .register_type::<WormState>()
            .add_systems(
                FixedUpdate,
                (behave, detect_players).run_if(in_state(GameState::Running)),
            )
            .add_systems(PostUpdate, handle_state_change);
    }
}

#[derive(Bundle)]
pub struct WormBundle {
    pub pbr: PbrBundle,
    pub enemy: Enemy,
    pub movement: WormMovement,
    pub state: WormState,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub scale: GravityScale,
    pub health: Health,
    pub damage: Damage,
    pub name: Name,
    pub death: DeathEffect,
}

impl WormBundle {
    pub fn new(pos: Vec3, assets: &EnemyAssets, size: usize) -> Self {
        Self {
            pbr: PbrBundle {
                material: assets.materials[0].clone_weak(),
                mesh: assets.mesh.clone_weak(),
                transform: Transform::from_translation(pos),
                ..default()
            },
            enemy: Enemy,
            movement: WormMovement::new(size as f32 * 1.5, pos),
            rigidbody: RigidBody::Kinematic,
            scale: GravityScale(0.0),
            collider: assets.collider.clone(),
            layers: CollisionLayers::new(ObjectLayer::Enemy, LayerMask::ALL),
            health: Health::new(BASE_HEALTH),
            damage: Damage(BASE_DAMAGE),
            name: Name::new("Worm"),
            state: WormState::default(),
            death: DeathEffect {
                color: Color::BLACK,
                radius: 1.0,
            },
        }
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct WormMovement {
    elapsed: f32,
    pub speed: f32,
    pub spawn_position: Vec3,
    pub anchor_position: Vec3,
}

impl WormMovement {
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

#[derive(Debug, Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub enum WormState {
    #[default]
    Idle,
    PrepareAttack(Vec3),
    PlungeAttack(Vec3),
    Returning,
}

fn behave(
    mut enemies: Query<(&mut Transform, &mut WormMovement, &mut WormState)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut transform, mut movement, mut state) in &mut enemies {
        let position = transform.translation;
        let speed = movement.speed;
        let target_position = match *state {
            WormState::Idle => {
                // Figure-eight pattern
                let delta = Vec3::new(
                    speed * movement.elapsed.sin(),
                    0.0,
                    speed * (2.0 * movement.elapsed).sin() / 2.0,
                );
                movement.elapsed += dt;
                movement.anchor_position + delta
            }
            WormState::PrepareAttack(target) => 'att: {
                if position.distance(target) < 1.0 {
                    *state = WormState::PlungeAttack(Vec3::new(target.x, 0.5, target.z));
                    break 'att position;
                }
                let Ok(dir) = Dir3::new(target - position) else {
                    *state = WormState::PlungeAttack(Vec3::new(target.x, 0.5, target.z));
                    break 'att position;
                };
                position + *dir * speed * 1.5 * dt
            }
            WormState::PlungeAttack(target) => 'att: {
                if position.distance(target) < 1.0 {
                    *state = WormState::Returning;
                    break 'att position;
                }
                let Ok(dir) = Dir3::new(target - position) else {
                    *state = WormState::Returning;
                    break 'att position;
                };
                position + *dir * speed * 2.0 * dt
            }
            WormState::Returning => {
                movement.anchor_position.x = position.x;
                movement.anchor_position.z = position.z;
                if movement.anchor_position.distance(movement.spawn_position) > MAX_DISTANCE {
                    *state = WormState::PrepareAttack(movement.spawn_position)
                } else {
                    movement.elapsed = 0.0;
                    *state = WormState::Idle;
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
    mut enemies: Query<&mut WormState>,
    players: Query<&GlobalTransform, (With<Player>, Without<Dead>)>,
) {
    let dt = time.delta_seconds();
    for (parent, mut detector, collisions) in &mut detectors {
        detector.last_detection += dt;
        if detector.last_detection < detector.attack_cooldown {
            continue;
        }
        let mut state = enemies.get_mut(parent.get()).unwrap();
        if matches!(*state, WormState::PrepareAttack(_)) {
            continue;
        }
        let Some(gtr) = collisions.iter().find_map(|e| players.get(*e).ok()) else {
            continue;
        };
        let target = gtr.translation();
        *state = WormState::PrepareAttack(Vec3::new(target.x, PLUNGE_HEIGHT, target.z));
        detector.last_detection = 0.0;
    }
}

fn handle_state_change(
    enemies: Query<(&WormState, &Children), Changed<WormState>>,
    mut collectors: Query<&mut CollectorConfig>,
) {
    for (state, children) in &enemies {
        let mut configs = collectors.iter_many_mut(children);
        while let Some(mut config) = configs.fetch_next() {
            match state {
                WormState::PrepareAttack(_) | WormState::PlungeAttack(_) => config.enabled = false,
                WormState::Idle | WormState::Returning => config.enabled = true,
            }
        }
    }
}
