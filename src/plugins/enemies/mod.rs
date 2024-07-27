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

const ENEMY_COLOR: Color = Color::BLACK;

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
    pub fn new(size: usize, radius: f32, max_distance: f32, items_per_point: usize) -> Self {
        Self {
            collector: CollectorBundle::fixed(
                radius,
                max_distance,
                ENEMY_COLOR,
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
    pub fn sphere(radius: f32, cooldown: f32) -> Self {
        Self {
            spatial: SpatialBundle::default(),
            sensor: Sensor,
            collider: Collider::sphere(radius),
            layers: CollisionLayers::new(ObjectLayer::Enemy, ObjectLayer::Player),
            detector: PlayerDetector::new(cooldown),
        }
    }

    pub fn cone(cooldown: f32) -> Self {
        Self {
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 5.0)
                    .with_rotation(Quat::from_rotation_y(PI) * Quat::from_rotation_x(FRAC_PI_2)),
                ..default()
            },
            sensor: Sensor,
            collider: Collider::cone(15.0, 15.0),
            layers: CollisionLayers::new(ObjectLayer::Enemy, ObjectLayer::Player),
            detector: PlayerDetector::new(cooldown),
        }
    }
}

fn spawn_worm(mut commands: Commands, assets: Res<EnemyAssets>, particles: Res<ParticleConfig>) {
    const SIZE: usize = 15;
    let enemy = commands
        .spawn(WormBundle::new(Vec3::Y * 2.0, &assets, SIZE))
        .id();
    let mut collector_bundle = EnemyCollectorBundle::new(SIZE, 5.0, 1.5, 4);
    collector_bundle.collector.config.enabled = true;
    let collector = commands.spawn(collector_bundle).set_parent(enemy).id();
    commands
        .spawn(PlayerDetectorBundle::cone(3.0))
        .set_parent(enemy);
    commands.spawn(CollectorParticlesBundle::new(
        collector,
        ENEMY_COLOR,
        &particles,
    ));
}
