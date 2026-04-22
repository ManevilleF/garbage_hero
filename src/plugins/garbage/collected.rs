use super::{Collector, collector::OnCollectedFilterOut};
use avian3d::prelude::*;
use bevy::{
    ecs::{
        component::{Immutable, StorageType},
        lifecycle::ComponentHook,
    },
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
    type Mutability = Immutable;

    fn on_add() -> Option<ComponentHook> {
        Some(|mut world, ctx| {
            let collected = world.get::<Self>(ctx.entity).unwrap();
            let collector_entity = collected.collector_entity;
            let collected_pos = world
                .get::<GlobalTransform>(ctx.entity)
                .map(|gtr| gtr.translation().xz())
                .unwrap();
            let collector_pos = world
                .get::<GlobalTransform>(collected.collector_entity)
                .map(|gtr| gtr.translation().xz())
                .unwrap();
            let dir = Dir2::new(collected_pos - collector_pos).ok();

            let Some(mut collector) = world.get_mut::<Collector>(collector_entity) else {
                log::error!(
                    "Cannot find collector of `Collected` entity {:?}",
                    ctx.entity
                );
                return;
            };
            if !collector.insert(ctx.entity, dir) {
                let mut commands = world.commands();
                commands.entity(ctx.entity).remove::<Self>();
                return;
            };

            let filter_out = world
                .get::<OnCollectedFilterOut>(collector_entity)
                .map(|filter| filter.layer);
            let Some(mut layer) = world.get::<CollisionLayers>(ctx.entity).cloned() else {
                log::error!(
                    "on_add hook triggered for {:?} without `CollisionLayers`",
                    ctx.entity
                );
                return;
            };
            // Collected entities should no longer interact withsome things
            if let Some(filter_out) = filter_out {
                layer.filters.remove(filter_out);
            }
            layer.filters.remove(ObjectLayer::Collector);
            world.commands().entity(ctx.entity).insert(layer);
            let Some(mut scale) = world.get_mut::<GravityScale>(ctx.entity) else {
                log::warn!(
                    "on_add hook triggered for {:?} without `GravityScale`",
                    ctx.entity
                );
                return;
            };
            scale.0 = 0.0;
        })
    }

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world, ctx| {
            let collected = world.get::<Self>(ctx.entity).unwrap();
            if let Some(mut collector) = world.get_mut::<Collector>(collected.collector_entity) {
                collector.remove(ctx.entity);
            };

            let Some(mut layer) = world.get::<CollisionLayers>(ctx.entity).cloned() else {
                log::error!(
                    "on_add hook triggered for {:?} without `CollisionLayers`",
                    ctx.entity
                );
                return;
            };
            // Reset filter
            layer.filters = LayerMask::ALL;
            world.commands().entity(ctx.entity).insert(layer);

            // Reset gravity scale
            let Some(mut scale) = world.get_mut::<GravityScale>(ctx.entity) else {
                log::warn!(
                    "on_add hook triggered for {:?} without `GravityScale`",
                    ctx.entity
                );
                return;
            };
            scale.0 = 1.0;
        })
    }
}
