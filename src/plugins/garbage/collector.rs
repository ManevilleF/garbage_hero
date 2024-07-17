use avian3d::prelude::*;
use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        entity::EntityHashSet,
    },
    log,
    prelude::*,
};

use crate::ObjectLayer;

use super::GarbageItem;

#[derive(Debug, Reflect)]
pub struct Collected {
    pub collector_entity: Entity,
}

impl Component for Collected {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .on_add(|mut world, entity, _| {
                let Some(mut layer) = world.get_mut::<CollisionLayers>(entity) else {
                    log::error!("on_add hook triggered for {entity:?} without `CollisionLayers`");
                    return;
                };
                // Collected entities should no longer interact withsome things
                layer.filters.remove(ObjectLayer::Map);
                layer.filters.remove(ObjectLayer::Collector);
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
                collector.collected.remove(&entity);

                let Some(mut layer) = world.get_mut::<CollisionLayers>(entity) else {
                    log::error!("on_add hook triggered for {entity:?} without `CollisionLayers`");
                    return;
                };
                // Reset filter
                layer.filters.add(ObjectLayer::Map);
                layer.filters.add(ObjectLayer::Collector);
            });
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Collector {
    pub base_radius: f32,
    pub base_strength: usize,
    pub collected: EntityHashSet,
}

impl Collector {
    const RADIUS_COEFFICIENT: f32 = 0.1;
    const ROTATION_SPEED: f32 = 10.0;

    pub fn strength(&self) -> usize {
        self.base_strength
            .saturating_add(usize::MAX - self.collected.len())
    }

    pub fn radius(&self) -> f32 {
        (self.collected.len() as f32).mul_add(Self::RADIUS_COEFFICIENT, self.base_radius)
    }

    pub fn update_radius(mut collectors: Query<(&mut Transform, &Self), Changed<Self>>) {
        for (mut tr, collector) in &mut collectors {
            tr.scale = Vec3::splat(collector.radius());
        }
    }

    pub fn update_collected_position(
        time: Res<Time>,
        mut collected: Query<&mut Transform, (With<Collected>, With<GarbageItem>)>,
        collectors: Query<(&Transform, &Self)>,
    ) {
        for (center_tr, collector) in &collectors {
            let mut collected = collected.iter_many_mut(&collector.collected);
            while let Some(mut tr) = collected.fetch_next() {
                let rotation =
                    Quat::from_axis_angle(Vec3::Y, Self::ROTATION_SPEED * time.delta_seconds());
                let desired_position =
                    center_tr.translation + rotation.mul_vec3(Vec3::Z * collector.radius());

                // TODO: Add elasticity or use velocity instead
                tr.translation = desired_position;
                tr.rotation = rotation;
            }
        }
    }

    pub fn throw_collected(
        &mut self,
        entity: Entity,
        direction: Dir3,
        force: f32,
    ) -> impl FnOnce(&mut World) {
        if !self.collected.remove(&entity) {
            panic!("Can't throw non collected entity");
        }
        move |world| {
            let mut entity_cmd = world.entity_mut(entity);
            entity_cmd
                .insert(ExternalImpulse::new(*direction * force))
                .remove::<Collected>();
        }
    }
}

#[derive(Bundle)]
pub struct CollectorBundle {
    pub transform: TransformBundle,
    pub collider: Collider,
    pub sensor: Sensor,
    pub collectible_sensor: Collector,
    pub layer: CollisionLayers,
}

impl CollectorBundle {
    pub fn new(base_radius: f32, base_strength: usize) -> Self {
        Self {
            transform: TransformBundle::default(),
            collider: Collider::sphere(1.0),
            sensor: Sensor,
            collectible_sensor: Collector {
                base_strength,
                base_radius,
                collected: EntityHashSet::default(),
            },
            layer: CollisionLayers::new(ObjectLayer::Collector, [ObjectLayer::Collectible]),
        }
    }
}
