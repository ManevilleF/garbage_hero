use super::{Dead, map::MAP_SIZE, player::Player, spawn_some_garbage};
use crate::ObjectLayer;
use avian3d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

mod assets;
mod auto_turret;
mod worm;

use assets::EnemyAssetsPlugin;
use auto_turret::AutoTurretPlugin;
use rand::rng;
use worm::WormPlugin;

const ENEMY_COLOR: Color = Color::BLACK;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WormPlugin, AutoTurretPlugin, EnemyAssetsPlugin))
            .register_type::<Enemy>()
            .register_type::<TargetPlayer>()
            .add_message::<SpawnTurret>()
            .add_message::<SpawnWorm>()
            .add_systems(FixedUpdate, detect_players);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Enemy;

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

#[derive(Component, Reflect, Deref)]
pub struct TargetPlayer(Vec3);

#[derive(Bundle)]
pub struct PlayerDetectorBundle {
    pub transform: Transform,
    pub sensor: Sensor,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub detector: PlayerDetector,
}

impl PlayerDetectorBundle {
    pub fn sphere(radius: f32, cooldown: f32) -> Self {
        Self {
            transform: Transform::default(),
            sensor: Sensor,
            collider: Collider::sphere(radius),
            layers: CollisionLayers::new(ObjectLayer::Enemy, ObjectLayer::Player),
            detector: PlayerDetector::new(cooldown),
        }
    }

    pub fn cone(cooldown: f32) -> Self {
        Self {
            transform: Transform::from_xyz(0.0, 0.0, 5.0)
                .with_rotation(Quat::from_rotation_y(PI) * Quat::from_rotation_x(FRAC_PI_2)),
            sensor: Sensor,
            collider: Collider::cone(15.0, 15.0),
            layers: CollisionLayers::new(ObjectLayer::Enemy, ObjectLayer::Player),
            detector: PlayerDetector::new(cooldown),
        }
    }
}

fn detect_players(
    mut commands: Commands,
    time: Res<Time>,
    mut detectors: Query<(&ChildOf, &mut PlayerDetector, &CollidingEntities)>,
    players: Query<&GlobalTransform, (With<Player>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    for (parent, mut detector, collisions) in &mut detectors {
        detector.last_detection += dt;
        if detector.last_detection < detector.attack_cooldown {
            continue;
        }
        let Some(gtr) = collisions.iter().find_map(|e| players.get(*e).ok()) else {
            continue;
        };
        let target = gtr.translation();
        commands
            .entity(parent.parent())
            .insert(TargetPlayer(target));
        detector.last_detection = 0.0;
    }
}

#[derive(Message, Reflect)]
pub struct SpawnWorm {
    pub size: usize,
    pub position: Vec2,
}

#[derive(Message, Reflect)]
pub struct SpawnTurret {
    pub position: Vec2,
}

pub fn spawn_enemies(worms: usize, turrets: usize, world: &mut World) {
    let square = Rectangle::new(MAP_SIZE.x - 20.0, MAP_SIZE.y - 20.0);
    let mut rng = rng();
    for i in 0..worms {
        let position = square.sample_interior(&mut rng);
        world.write_message(SpawnWorm {
            size: 12 + i,
            position,
        });
        spawn_some_garbage((12 + i) * 4, Some(Vec2::new(10.0, 10.0)), Some(position))(world);
    }
    for _ in 0..turrets {
        let position = square.sample_interior(&mut rng);
        world.write_message(SpawnTurret { position });
    }
}
