use super::{collector::OnCollectedFilterOut, Collector};
use avian3d::prelude::*;
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    log,
    prelude::*,
};

use crate::ObjectLayer;

pub struct CollectedPlugin;

impl Plugin for CollectedPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Collected>();
    }
}

#[derive(Debug, Reflect)]
#[reflect(Component)]
pub struct Collected {
    pub collector_entity: Entity,
}

impl Component for Collected {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .on_add(|mut world, entity, _| {
                let Some(collected) = world.get::<Self>(entity) else {
                    log::error!("on_remove hook triggered for {entity:?} without `Collected`");
                    return;
                };
                let collector_entity = collected.collector_entity;
                let collected_pos = world
                    .get::<GlobalTransform>(entity)
                    .map(|gtr| gtr.translation().xz())
                    .unwrap();
                let collector_pos = world
                    .get::<GlobalTransform>(collected.collector_entity)
                    .map(|gtr| gtr.translation().xz())
                    .unwrap();
                let dir = Dir2::new(collected_pos - collector_pos).ok();

                let Some(mut collector) = world.get_mut::<Collector>(collector_entity) else {
                    log::error!("Cannot find collector of `Collected` entity {entity:?}");
                    return;
                };
                if !collector.insert(entity, dir) {
                    let mut commands = world.commands();
                    commands.entity(entity).remove::<Self>();
                    return;
                };

                let filter_out = world
                    .get::<OnCollectedFilterOut>(collector_entity)
                    .map(|filter| filter.layer);
                let Some(mut layer) = world.get_mut::<CollisionLayers>(entity) else {
                    log::error!("on_add hook triggered for {entity:?} without `CollisionLayers`");
                    return;
                };
                // Collected entities should no longer interact withsome things
                if let Some(filter_out) = filter_out {
                    layer.filters.remove(filter_out);
                }
                layer.filters.remove(ObjectLayer::Collector);
                let Some(mut scale) = world.get_mut::<GravityScale>(entity) else {
                    log::warn!("on_add hook triggered for {entity:?} without `GravityScale`");
                    return;
                };
                scale.0 = 0.0;
            })
            .on_remove(|mut world, entity, _| {
                let Some(collected) = world.get::<Self>(entity) else {
                    log::error!("on_remove hook triggered for {entity:?} without `Collected`");
                    return;
                };
                let Some(mut collector) = world.get_mut::<Collector>(collected.collector_entity)
                else {
                    log::error!("Cannot find collector of `Collected` entity {entity:?}");
                    return;
                };
                // Can be already removed
                collector.remove(entity);

                let Some(mut layer) = world.get_mut::<CollisionLayers>(entity) else {
                    log::error!("on_add hook triggered for {entity:?} without `CollisionLayers`");
                    return;
                };
                // Reset filter
                layer.filters = LayerMask::ALL;

                // Reset gravity scale
                let Some(mut scale) = world.get_mut::<GravityScale>(entity) else {
                    log::warn!("on_add hook triggered for {entity:?} without `GravityScale`");
                    return;
                };
                scale.0 = 1.0;
            });
    }
}
