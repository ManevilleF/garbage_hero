use avian3d::prelude::*;
use bevy::prelude::*;

use crate::ObjectLayer;

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
}

impl MapElementBundle {
    pub fn new_cube(assets: &MapAssets) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: assets.cube_mesh.clone_weak(),
                material: assets.default_mat.clone_weak(),
                ..default()
            },
            collider: Collider::cuboid(1.0, 1.0, 1.0),
            layers: CollisionLayers::new(ObjectLayer::Map, LayerMask::ALL),
            body: RigidBody::Static,
            name: Name::new("Map cube"),
        }
    }
}

fn create_default_ground(mut commands: Commands, assets: Res<MapAssets>) {
    let mut bundle = MapElementBundle::new_cube(&assets);
    bundle.pbr.transform.translation.y = -3.0;
    bundle.pbr.transform.scale = Vec3::new(100.0, 1.0, 100.0);
    bundle.name = Name::new("Ground");
    commands.spawn(bundle);
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
            base_color: Color::WHITE,
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
