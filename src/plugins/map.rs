use avian3d::prelude::*;
use bevy::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Map>();
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Map;

#[derive(Bundle)]
pub struct MapElementBundle {
    pub pbr: PbrBundle,
    pub collider: Collider,
    pub layers: CollisionLayers,
}

#[derive(Debug, Resource, Reflect)]
pub struct MapAssets {}
