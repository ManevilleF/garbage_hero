use crate::ObjectLayer;
use avian3d::prelude::*;
use bevy::prelude::*;

pub const MAP_SIZE: Vec2 = Vec2::new(200.0, 200.);

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Map>()
            .init_resource::<MapAssets>()
            .register_type::<MapAssets>()
            .add_systems(Startup, create_default_ground);
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

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct MapAssets {
    cube_mesh: Handle<Mesh>,
    default_mat: Handle<StandardMaterial>,
}

impl FromWorld for MapAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let default_mat = materials.add(StandardMaterial {
            base_color: Color::Srgba(bevy::color::palettes::css::DARK_SALMON),
            ..default()
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        Self {
            cube_mesh,
            default_mat,
        }
    }
}
