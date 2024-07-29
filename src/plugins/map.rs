use std::f32::consts::FRAC_PI_4;

use crate::{ObjectLayer, StartGame};
use avian3d::prelude::*;
use bevy::{
    math::Affine2,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::texture::{
        ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
    },
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
                material: assets.ground_mat.clone_weak(),
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
    use bevy::color::palettes::css::{GREEN, RED, ROYAL_BLUE};

    const OFFSET: f32 = 30.0;

    let particles = world.resource::<ParticleConfig>();
    let easy_bundle = GameStarterBundle::new(
        StartGame {
            worm_count: 2,
            turret_count: 3,
        },
        "Easy",
        Color::Srgba(GREEN),
        Vec2::new(-OFFSET, 0.0),
        particles,
    );
    let normal_bundle = GameStarterBundle::new(
        StartGame {
            worm_count: 3,
            turret_count: 5,
        },
        "Medium",
        Color::Srgba(ROYAL_BLUE),
        Vec2::new(0.0, -OFFSET),
        particles,
    );
    let hard_bundle = GameStarterBundle::new(
        StartGame {
            worm_count: 4,
            turret_count: 7,
        },
        "Hard",
        Color::Srgba(RED),
        Vec2::new(OFFSET, 0.0),
        particles,
    );
    let assets = world.resource::<MapAssets>();
    let mesh = assets.spawner_mesh.clone_weak();
    let [easy_mat, normal_mat, hard_mat] = [
        assets.easy_mat.clone_weak(),
        assets.normal_mat.clone_weak(),
        assets.hard_mat.clone_weak(),
    ];
    world.spawn((
        easy_bundle,
        mesh.clone_weak(),
        easy_mat,
        NotShadowCaster,
        NotShadowReceiver,
    ));

    world.spawn((
        normal_bundle,
        mesh.clone_weak(),
        normal_mat,
        NotShadowCaster,
        NotShadowReceiver,
    ));
    world.spawn((
        hard_bundle,
        mesh,
        hard_mat,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct MapAssets {
    cube_mesh: Handle<Mesh>,
    spawner_mesh: Handle<Mesh>,
    easy_mat: Handle<StandardMaterial>,
    normal_mat: Handle<StandardMaterial>,
    hard_mat: Handle<StandardMaterial>,
    ground_mat: Handle<StandardMaterial>,
}

impl FromWorld for MapAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        let wood_texture = server.load_with_settings("wood-planks/texture.jpg", |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // rewriting mode to repeat image,
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        });
        let easy_texture = server.load("textures/easy.png");
        let normal_texture = server.load("textures/normal.png");
        let hard_texture = server.load("textures/hard.png");
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let ground_mat = materials.add(StandardMaterial {
            base_color: Color::Srgba(bevy::color::palettes::css::DARK_SALMON),
            base_color_texture: Some(wood_texture),
            uv_transform: Affine2::from_mat2(Mat2::from_scale_angle(MAP_SIZE / 20.0, FRAC_PI_4)),
            ..default()
        });
        let easy_mat = materials.add(StandardMaterial {
            base_color_texture: Some(easy_texture),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let normal_mat = materials.add(StandardMaterial {
            base_color_texture: Some(normal_texture),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let hard_mat = materials.add(StandardMaterial {
            base_color_texture: Some(hard_texture),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let spawner_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)));
        Self {
            cube_mesh,
            spawner_mesh,
            easy_mat,
            normal_mat,
            hard_mat,
            ground_mat,
        }
    }
}
