use crate::ObjectLayer;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use super::{assets::PlayerAssets, input::PlayerInputAction, Player, PLAYER_HEIGHT, PLAYER_RADIUS};

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MovementSpeed>()
            .register_type::<MovementDampingFactor>()
            .add_systems(
                Update,
                (apply_gravity, apply_movement, apply_movement_damping).chain(),
            );
        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, draw_gizmos);
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct MovementDampingFactor(pub f32);

#[derive(Bundle)]
pub struct PlayerMovementBundle {
    pub speed: MovementSpeed,
    pub damping: MovementDampingFactor,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub layer: CollisionLayers,
    pub margin: CollisionMargin,
    pub constraints: LockedAxes,
    pub angular_damping: AngularDamping,
    pub gravity_scale: GravityScale,
}

impl PlayerMovementBundle {
    pub fn new(speed: f32, damping_factor: f32) -> Self {
        Self {
            speed: MovementSpeed(speed),
            damping: MovementDampingFactor(damping_factor),
            rigidbody: RigidBody::Dynamic,
            collider: Collider::capsule(PLAYER_RADIUS, PLAYER_HEIGHT),
            layer: CollisionLayers::new(ObjectLayer::Player, LayerMask::ALL),
            margin: CollisionMargin(0.05),
            constraints: LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            angular_damping: AngularDamping(10.0),
            gravity_scale: GravityScale(1.0),
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
            velocity.x -= dir.x * dt * speed.0;
            velocity.z += dir.y * dt * speed.0;
        }
    }
}

/// Applies [`Gravity`] to player controllers.
fn apply_gravity(
    time: Res<Time>,
    gravity: Res<Gravity>,
    mut controllers: Query<&mut LinearVelocity, With<MovementSpeed>>,
) {
    let delta_time = time.delta_seconds();
    for mut velocity in &mut controllers {
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

fn draw_gizmos(
    mut gizmos: Gizmos,
    players: Query<(&Player, &GlobalTransform, &LinearVelocity)>,
    assets: Res<PlayerAssets>,
) {
    for (player, gtr, vel) in &players {
        let position = gtr.translation();
        let forward = gtr.forward();
        let color = assets.colors[player.id as usize];
        gizmos.arrow(position, position + vel.0, color);
        gizmos.ray(position, *forward, color);
    }
}
