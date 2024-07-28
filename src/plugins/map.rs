use crate::{ObjectLayer, StartGame};
use avian3d::prelude::*;
use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_hanabi::{EffectProperties, ParticleEffect, ParticleEffectBundle};

use super::ParticleConfig;

pub const MAP_SIZE: Vec2 = Vec2::new(200.0, 200.);

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Map>()
            .init_resource::<MapAssets>()
            .register_type::<MapAssets>()
            .add_systems(Startup, (create_default_ground, spawn_game_starters))
            .add_systems(Update, handle_game_starters);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Map;

#[derive(Bundle)]
pub struct MapElementBundle {
    pub pbr: PbrBundle,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub name: Name,
    pub body: RigidBody,
    pub friction: Friction,
    pub restitution: Restitution,
}

#[derive(Bundle)]
pub struct InvisibleMapElementBundle {
    pub spatial: SpatialBundle,
    pub collider: Collider,
    pub layers: CollisionLayers,
    pub name: Name,
    pub body: RigidBody,
    pub friction: Friction,
    pub restitution: Restitution,
}

impl MapElementBundle {
    pub fn new_cube(assets: &MapAssets) -> Self {
        let mut mask = LayerMask::ALL;
        mask.remove(ObjectLayer::Map);
        Self {
            pbr: PbrBundle {
                mesh: assets.cube_mesh.clone_weak(),
                material: assets.default_mat.clone_weak(),
                ..default()
            },
            collider: Collider::cuboid(1.0, 1.0, 1.0),
            layers: CollisionLayers::new(ObjectLayer::Map, mask),
            body: RigidBody::Static,
            name: Name::new("Map cube"),
            friction: Friction::new(0.7),
            restitution: Restitution::new(0.2),
        }
    }
}

impl InvisibleMapElementBundle {
    pub fn new_cube() -> Self {
        let mut mask = LayerMask::ALL;
        mask.remove(ObjectLayer::Map);
        Self {
            spatial: Default::default(),
            collider: Collider::cuboid(1.0, 1.0, 1.0),
            layers: CollisionLayers::new(ObjectLayer::Map, mask),
            body: RigidBody::Static,
            name: Name::new("Invisible Wall"),
            friction: Friction::new(0.5),
            restitution: Restitution::new(0.8),
        }
    }
}

fn create_default_ground(mut commands: Commands, assets: Res<MapAssets>) {
    let mut ground = MapElementBundle::new_cube(&assets);
    ground.pbr.transform.scale = Vec3::new(MAP_SIZE.x - 1.0, 1.0, MAP_SIZE.y - 1.0);
    ground.name = Name::new("Ground");
    commands.spawn(ground);
    let mut left = InvisibleMapElementBundle::new_cube();
    left.spatial.transform.translation = Vec3::new(-MAP_SIZE.x / 2.0, 0.0, 0.0);
    left.spatial.transform.scale = Vec3::new(1.0, 100.0, MAP_SIZE.y);
    let mut right = InvisibleMapElementBundle::new_cube();
    right.spatial.transform.translation = Vec3::new(MAP_SIZE.x / 2.0, 0.0, 0.0);
    right.spatial.transform.scale = Vec3::new(1.0, 100.0, MAP_SIZE.y);
    let mut bot = InvisibleMapElementBundle::new_cube();
    bot.spatial.transform.translation = Vec3::new(0.0, 0.0, -MAP_SIZE.y / 2.0);
    bot.spatial.transform.scale = Vec3::new(MAP_SIZE.x, 100.0, 1.0);
    let mut top = InvisibleMapElementBundle::new_cube();
    top.spatial.transform.translation = Vec3::new(0.0, 0.0, MAP_SIZE.y / 2.0);
    top.spatial.transform.scale = Vec3::new(MAP_SIZE.x, 100.0, 1.0);
    commands.spawn_batch([left, right, top, bot]);
}

#[derive(Bundle)]
struct GameStarterBundle {
    pub name: Name,
    pub data: StartGame,
    pub collider: Collider,
    pub sensor: Sensor,
    pub particles: ParticleEffectBundle,
    pub layer: CollisionLayers,
}

impl GameStarterBundle {
    pub fn new(
        data: StartGame,
        name: &str,
        color: Color,
        pos: Vec2,
        particles: &ParticleConfig,
    ) -> Self {
        Self {
            name: Name::new(name.to_owned()),
            data,
            collider: Collider::sphere(4.0),
            sensor: Sensor,
            particles: ParticleEffectBundle {
                transform: Transform::from_xyz(pos.x, 1.0, pos.y),
                effect: ParticleEffect::new(particles.collector_effect.clone()),
                effect_properties: EffectProperties::default()
                    .with_properties([("color".to_owned(), ParticleConfig::color_to_value(color))])
                    .with_properties([("radius".to_owned(), 5.0.into())]),
                ..default()
            },
            layer: CollisionLayers::new(ObjectLayer::Map, ObjectLayer::Player),
        }
    }
}

fn handle_game_starters(starters: Query<(&StartGame, &CollidingEntities)>, mut commands: Commands) {
    for (data, collision) in &starters {
        if collision.is_empty() {
            continue;
        }
        commands.add(*data);
    }
}

pub fn spawn_game_starters(world: &mut World) {
    const OFFSET: f32 = 30.0;

    use bevy::color::palettes::css::{GREEN, RED, ROYAL_BLUE};
    let assets = world.resource::<MapAssets>();
    let (mesh, mat) = (
        assets.spawner_mesh.clone_weak(),
        assets.spawner_mat.clone_weak(),
    );
    let particles = world.resource::<ParticleConfig>();
    for bundle in [
        GameStarterBundle::new(
            StartGame {
                worm_count: 2,
                turret_count: 3,
            },
            "Easy",
            Color::Srgba(GREEN),
            Vec2::new(-OFFSET, 0.0),
            particles,
        ),
        GameStarterBundle::new(
            StartGame {
                worm_count: 3,
                turret_count: 5,
            },
            "Medium",
            Color::Srgba(ROYAL_BLUE),
            Vec2::new(0.0, -OFFSET),
            particles,
        ),
        GameStarterBundle::new(
            StartGame {
                worm_count: 4,
                turret_count: 7,
            },
            "Hard",
            Color::Srgba(RED),
            Vec2::new(OFFSET, 0.0),
            particles,
        ),
    ] {
        world.spawn((
            bundle,
            mesh.clone_weak(),
            mat.clone_weak(),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }
}

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct MapAssets {
    cube_mesh: Handle<Mesh>,
    spawner_mesh: Handle<Mesh>,
    spawner_mat: Handle<StandardMaterial>,
    default_mat: Handle<StandardMaterial>,
}

impl FromWorld for MapAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let default_mat = materials.add(StandardMaterial {
            base_color: Color::Srgba(bevy::color::palettes::css::DARK_SALMON),
            ..default()
        });
        let spawner_mat = materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.3),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let sphere_mesh = meshes.add(Sphere::new(5.0));
        Self {
            cube_mesh,
            spawner_mesh: sphere_mesh,
            spawner_mat,
            default_mat,
        }
    }
}
