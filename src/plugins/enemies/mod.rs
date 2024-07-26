use super::garbage::GarbageBody;
use crate::ObjectLayer;
use avian3d::prelude::*;
use bevy::prelude::*;

mod assets;
mod movement;

use assets::EnemyAssets;
use movement::{EnemyMovement, EnemyMovementPlugin};

const BASE_HEALTH: u16 = 50;
const BASE_DAMAGE: u16 = 10;

use super::{Damage, Health};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnemyMovementPlugin)
            .init_resource::<EnemyAssets>()
            .register_type::<Enemy>();

        app.add_systems(Startup, spawn_enemy);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub pbr: PbrBundle,
    pub enemy: Enemy,
    pub movement: EnemyMovement,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub scale: GravityScale,
    pub health: Health,
    pub damage: Damage,
    pub name: Name,
    pub body: GarbageBody,
}

impl EnemyBundle {
    pub fn new(pos: Vec3, assets: &EnemyAssets, size: usize) -> Self {
        Self {
            pbr: PbrBundle {
                material: assets.worm_head_mat.clone_weak(),
                mesh: assets.worm_head_mesh.clone_weak(),
                transform: Transform::from_translation(pos),
                ..default()
            },
            enemy: Enemy,
            movement: EnemyMovement {
                time_offset: 0.0,
                speed: 5.0,
                spawn_position: pos,
            },
            rigidbody: RigidBody::Kinematic,
            scale: GravityScale(0.0),
            collider: assets.worm_head_collider.clone(),
            layers: CollisionLayers::new(ObjectLayer::Enemy, LayerMask::ALL),
            health: Health::new(BASE_HEALTH),
            damage: Damage(BASE_DAMAGE),
            name: Name::new("Worm"),
            body: GarbageBody::new(size, pos, 1.0),
        }
    }
}

fn spawn_enemy(mut commands: Commands, assets: Res<EnemyAssets>) {
    commands.spawn(EnemyBundle::new(Vec3::Y * 2.0, &assets, 20));
}
