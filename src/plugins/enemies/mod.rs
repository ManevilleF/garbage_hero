use std::f32::consts::{FRAC_PI_2, PI};

use super::{map::MAP_SIZE, player::Player, Dead};
use crate::{ObjectLayer, StartGame};
use avian3d::prelude::*;
use bevy::prelude::*;

mod assets;
mod auto_turret;
mod worm;

use assets::EnemyAssetsPlugin;
use auto_turret::AutoTurretPlugin;
use rand::thread_rng;
use worm::WormPlugin;

const ENEMY_COLOR: Color = Color::BLACK;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WormPlugin, AutoTurretPlugin, EnemyAssetsPlugin))
            .register_type::<Enemy>()
            .register_type::<TargetPlayer>()
            .add_event::<SpawnTurret>()
            .add_event::<SpawnWorm>()
            .add_systems(Update, handle_game_start)
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

fn detect_players(
    mut commands: Commands,
    time: Res<Time>,
    mut detectors: Query<(&Parent, &mut PlayerDetector, &CollidingEntities)>,
    players: Query<&GlobalTransform, (With<Player>, Without<Dead>)>,
) {
    let dt = time.delta_seconds();
    for (parent, mut detector, collisions) in &mut detectors {
        detector.last_detection += dt;
        if detector.last_detection < detector.attack_cooldown {
            continue;
        }
        let Some(gtr) = collisions.iter().find_map(|e| players.get(*e).ok()) else {
            continue;
        };
        let target = gtr.translation();
        commands.entity(parent.get()).insert(TargetPlayer(target));
        detector.last_detection = 0.0;
    }
}

#[derive(Event, Reflect)]
pub struct SpawnWorm {
    pub size: usize,
    pub position: Vec2,
}

#[derive(Event, Reflect)]
pub struct SpawnTurret {
    pub position: Vec2,
}

fn handle_game_start(
    mut events: EventReader<StartGame>,
    mut spawn_worm_evw: EventWriter<SpawnWorm>,
    mut spawn_turret_evw: EventWriter<SpawnTurret>,
) {
    let Some(StartGame {
        worm_count,
        turret_count,
    }) = events.read().next().copied()
    else {
        return;
    };
    events.clear();
    let square = Rectangle::new(MAP_SIZE.x - 20.0, MAP_SIZE.y - 20.0);
    let mut rng = thread_rng();
    for i in 0..=worm_count {
        let position = square.sample_interior(&mut rng);
        spawn_worm_evw.send(SpawnWorm {
            size: 10 + i,
            position,
        });
    }
    for _ in 0..=turret_count {
        let position = square.sample_interior(&mut rng);
        spawn_turret_evw.send(SpawnTurret { position });
    }
}
