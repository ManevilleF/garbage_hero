use super::{
    garbage::{CollectorBundle, GarbageBody},
    ParticleConfig,
};
use crate::{plugins::garbage::CollectorParticlesBundle, ObjectLayer};
use avian3d::prelude::*;
use bevy::prelude::*;

mod assets;
mod movement;

use assets::EnemyAssets;
use movement::{EnemyMovement, EnemyMovementPlugin, EnemyMovementState};

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
    pub state: EnemyMovementState,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub scale: GravityScale,
    pub health: Health,
    pub damage: Damage,
    pub name: Name,
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
            movement: EnemyMovement::new((size * 2) as f32, pos),
            rigidbody: RigidBody::Kinematic,
            scale: GravityScale(0.0),
            collider: assets.worm_head_collider.clone(),
            layers: CollisionLayers::new(ObjectLayer::Enemy, LayerMask::ALL),
            health: Health::new(BASE_HEALTH),
            damage: Damage(BASE_DAMAGE),
            name: Name::new("Worm"),
            state: EnemyMovementState::default(),
        }
    }
}

#[derive(Bundle)]
pub struct EnemyCollectorBundle {
    pub collector: CollectorBundle,
    pub body: GarbageBody,
}

impl EnemyCollectorBundle {
    pub fn new(size: usize, color: Color, items_per_point: usize) -> Self {
        Self {
            collector: CollectorBundle::fixed(
                5.0,
                1.5,
                color,
                size * items_per_point,
                items_per_point,
                ObjectLayer::Enemy,
            ),
            body: GarbageBody::new(size, Vec3::ZERO, 2.0, 0.0),
        }
    }
}

fn spawn_enemy(mut commands: Commands, assets: Res<EnemyAssets>, particles: Res<ParticleConfig>) {
    const SIZE: usize = 10;
    let enemy = commands
        .spawn(EnemyBundle::new(Vec3::Y * 2.0, &assets, SIZE))
        .id();
    let mut collector_bundle = EnemyCollectorBundle::new(SIZE, Color::BLACK, 4);
    collector_bundle.collector.config.enabled = true;
    let collector = commands.spawn(collector_bundle).set_parent(enemy).id();
    commands.spawn(CollectorParticlesBundle::new(
        collector,
        Color::BLACK,
        &particles,
    ));
}
