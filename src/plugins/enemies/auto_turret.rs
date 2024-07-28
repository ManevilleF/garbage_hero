use super::{
    assets::EnemyAssets, Enemy, PlayerDetectorBundle, SpawnTurret, TargetPlayer, ENEMY_COLOR,
};
use crate::{
    plugins::{
        garbage::{Collector, CollectorBundle, CollectorParticlesBundle},
        particles::DeathEffect,
    },
    Damage, GameState, Health, ObjectLayer, ParticleConfig,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::{thread_rng, Rng};
use std::f32::consts::TAU;

const BASE_HEALTH: u16 = 50;
const BASE_DAMAGE: u16 = 10;

const IMPULSE_SPEED: f32 = 100.0;
const IDLE_TRESHOLD: f32 = 10.0;
const MIN_ITEMS: usize = 5;

pub struct AutoTurretPlugin;

impl Plugin for AutoTurretPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TurretState>()
            .add_systems(Update, spawn_turret)
            .add_systems(
                FixedUpdate,
                (behave, detect_players).run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Bundle)]
pub struct AutoTurretBundle {
    pub pbr: PbrBundle,
    pub enemy: Enemy,
    pub state: TurretState,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub restitution: Restitution,
    pub lin_damping: LinearDamping,
    pub ang_damping: AngularDamping,
    pub health: Health,
    pub damage: Damage,
    pub name: Name,
    pub death: DeathEffect,
}

impl AutoTurretBundle {
    pub fn new(pos: Vec3, assets: &EnemyAssets) -> Self {
        Self {
            pbr: PbrBundle {
                material: assets.materials[0].clone_weak(),
                mesh: assets.mesh.clone_weak(),
                transform: Transform::from_translation(pos),
                ..default()
            },
            enemy: Enemy,
            rigidbody: RigidBody::Dynamic,
            collider: assets.collider.clone(),
            layers: CollisionLayers::new(ObjectLayer::Enemy, LayerMask::ALL),
            restitution: Restitution::new(1.0),
            lin_damping: LinearDamping(1.0),
            ang_damping: AngularDamping(1.5),
            health: Health::new(BASE_HEALTH),
            damage: Damage(BASE_DAMAGE),
            name: Name::new("Auto Turret"),
            state: TurretState::default(),
            death: DeathEffect {
                color: Color::BLACK,
                radius: 1.0,
            },
        }
    }
}

#[derive(Debug, Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub enum TurretState {
    #[default]
    Idle,
    Shoot(Dir2),
}

fn behave(
    mut commands: Commands,
    mut enemies: Query<(Entity, &LinearVelocity, &mut TurretState, &Children)>,
    collectors: Query<&Collector>,
) {
    for (entity, linvel, mut state, children) in &mut enemies {
        let collector = collectors.iter_many(children).next().unwrap();
        match *state {
            TurretState::Idle => {
                if collector.len() < MIN_ITEMS && linvel.length_squared() < IDLE_TRESHOLD {
                    // TOO: use a rng resource
                    let mut rng = thread_rng();
                    let angle = rng.gen_range(0.0..=TAU);
                    commands.entity(entity).insert(ExternalImpulse::new(
                        Vec3::new(angle.cos(), 0.0, angle.sin()) * IMPULSE_SPEED,
                    ));
                }
            }
            TurretState::Shoot(dir) => {
                if let Some(command) = collector.throw_collected(dir, 50.0) {
                    commands.add(command);
                }
                *state = TurretState::Idle;
            }
        }
    }
}

fn detect_players(
    mut commands: Commands,
    mut enemies: Query<
        (Entity, &GlobalTransform, &mut TurretState, &TargetPlayer),
        Added<TargetPlayer>,
    >,
) {
    for (entity, gtr, mut state, target) in &mut enemies {
        if matches!(*state, TurretState::Idle) {
            let pos = gtr.translation().xz();
            if let Ok(dir) = Dir2::new(target.xz() - pos) {
                *state = TurretState::Shoot(dir);
            }
        }
        commands.entity(entity).remove::<TargetPlayer>();
    }
}

fn spawn_turret(
    mut events: EventReader<SpawnTurret>,
    mut commands: Commands,
    assets: Res<EnemyAssets>,
    particles: Res<ParticleConfig>,
) {
    for event in events.read() {
        let enemy = commands
            .spawn(AutoTurretBundle::new(
                Vec3::new(event.position.x, 2.0, event.position.y),
                &assets,
            ))
            .id();
        let mut collector_bundle =
            CollectorBundle::growing(5.0, 2.0, ENEMY_COLOR, 10, ObjectLayer::Enemy);
        collector_bundle.config.enabled = true;
        let collector = commands.spawn(collector_bundle).set_parent(enemy).id();
        commands
            .spawn(PlayerDetectorBundle::sphere(40.0, 0.5))
            .set_parent(enemy);
        commands.spawn(CollectorParticlesBundle::new(
            collector,
            ENEMY_COLOR,
            &particles,
        ));
    }
}
