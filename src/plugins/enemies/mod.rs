use std::f32::consts::{FRAC_PI_2, PI};

use super::{
    garbage::{CollectorBundle, GarbageBody},
    ParticleConfig,
};
use crate::{plugins::garbage::CollectorParticlesBundle, ObjectLayer};
use avian3d::prelude::*;
use bevy::prelude::*;

mod assets;
mod worm;

use assets::{EnemyAssets, EnemyAssetsPlugin};
use worm::{WormBundle, WormPlugin};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WormPlugin, EnemyAssetsPlugin))
            .register_type::<Enemy>();

        app.add_systems(Startup, spawn_worm);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Enemy;

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
            body: GarbageBody::new(size, Vec3::ZERO, 2.5, -1.0),
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

#[derive(Bundle)]
pub struct PlayerDetectorBundle {
    pub spatial: SpatialBundle,
    pub sensor: Sensor,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub detector: PlayerDetector,
}

impl PlayerDetectorBundle {
    pub fn new() -> Self {
        Self {
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 5.0)
                    .with_rotation(Quat::from_rotation_y(PI) * Quat::from_rotation_x(FRAC_PI_2)),
                ..default()
            },
            sensor: Sensor,
            collider: Collider::cone(15.0, 15.0),
            layers: CollisionLayers::new(ObjectLayer::Enemy, ObjectLayer::Player),
            detector: PlayerDetector::new(3.0),
        }
    }
}

fn spawn_worm(mut commands: Commands, assets: Res<EnemyAssets>, particles: Res<ParticleConfig>) {
    const SIZE: usize = 15;
    let enemy = commands
        .spawn(WormBundle::new(Vec3::Y * 2.0, &assets, SIZE))
        .id();
    let mut collector_bundle = EnemyCollectorBundle::new(SIZE, Color::BLACK, 4);
    collector_bundle.collector.config.enabled = true;
    let collector = commands.spawn(collector_bundle).set_parent(enemy).id();
    commands
        .spawn(PlayerDetectorBundle::new())
        .set_parent(enemy);
    commands.spawn(CollectorParticlesBundle::new(
        collector,
        Color::BLACK,
        &particles,
    ));
}
