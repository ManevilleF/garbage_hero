use crate::ObjectLayer;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use super::{input::PlayerInputAction, PLAYER_HEIGHT, PLAYER_RADIUS};

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MovementSpeed>()
            .register_type::<MovementDampingFactor>()
            .register_type::<ControllerGravity>()
            .add_systems(
                Update,
                (apply_gravity, apply_movement, apply_movement_damping).chain(),
            );
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct MovementSpeed(pub f32);

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct ControllerGravity(pub f32);

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct MovementDampingFactor(pub f32);

#[derive(Bundle)]
pub struct PlayerMovementBundle {
    pub speed: MovementSpeed,
    pub gravity: ControllerGravity,
    pub damping: MovementDampingFactor,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub layer: CollisionLayers,
    pub constraints: LockedAxes,
}

impl PlayerMovementBundle {
    pub fn new(speed: f32, damping_factor: f32) -> Self {
        Self {
            speed: MovementSpeed(speed),
            gravity: ControllerGravity(-9.81),
            damping: MovementDampingFactor(damping_factor),
            rigidbody: RigidBody::Kinematic,
            collider: Collider::capsule(PLAYER_RADIUS, PLAYER_HEIGHT),
            layer: CollisionLayers::new(ObjectLayer::Player, LayerMask::ALL),
            constraints: LockedAxes::new().lock_rotation_x().lock_rotation_z(),
        }
    }
}

/// Applies movement input to player controllers.
pub fn apply_movement(
    mut controllers: Query<(
        &mut LinearVelocity,
        &ActionState<PlayerInputAction>,
        &MovementSpeed,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut velocity, action_state, speed) in &mut controllers {
        if let Some(dir) = PlayerInputAction::get_movement(action_state) {
            velocity.x += dir.x * dt * speed.0;
            velocity.z += dir.y * dt * speed.0;
        }
    }
}

/// Applies [`ControllerGravity`] to player controllers.
fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&ControllerGravity, &mut LinearVelocity)>,
) {
    let delta_time = time.delta_seconds();
    for (gravity, mut velocity) in &mut controllers {
        velocity.0 += gravity.0 * delta_time;
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut velocity) in &mut query {
        velocity.x *= damping_factor.0;
        velocity.z *= damping_factor.0;
    }
}
