use avian3d::prelude::*;
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    log,
    prelude::*,
};

use crate::ObjectLayer;

use super::{ArcDistribution, CircularDistribution, GarbageItem};

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
                collector.remove(entity);

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
    pub circle_distrib: CircularDistribution,
    pub arc_distrib: ArcDistribution,
    collected: Vec<Entity>,
}

impl Collector {
    const ROTATION_SPEED: f32 = 10.0;
    const COLLECTED_SPEED: f32 = 50.0;

    pub fn new(min_radius: f32, max_distance: f32) -> Self {
        Self {
            circle_distrib: CircularDistribution::new(min_radius, max_distance),
            arc_distrib: ArcDistribution::new(min_radius, max_distance),
            collected: Vec::new(),
        }
    }

    pub fn radius(&self) -> f32 {
        self.circle_distrib.radius(self.collected.len())
    }

    pub fn tick_circle_rotation(&mut self, dt: f32) {
        self.circle_distrib.rotate(dt * Self::ROTATION_SPEED);
    }

    pub fn update_radius(mut collectors: Query<(&mut Transform, &Self), Changed<Self>>) {
        for (mut tr, collector) in &mut collectors {
            tr.scale = Vec3::splat(collector.radius());
        }
    }

    pub fn insert(&mut self, entity: Entity) {
        self.collected.push(entity);
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        let index = self.collected.iter().position(|e| *e == entity)?;
        Some(self.collected.remove(index))
    }

    pub fn update_collected_position(
        time: Res<Time>,
        mut collected: Query<
            (&Transform, &mut LinearVelocity),
            (With<Collected>, With<GarbageItem>),
        >,
        mut collectors: Query<(&Transform, &mut Self)>,
    ) {
        let dt = time.delta_seconds();
        for (center_tr, mut collector) in &mut collectors {
            collector.tick_circle_rotation(dt);
            let mut collected = collected.iter_many_mut(&collector.collected);
            let positions = collector.circle_distrib.points();
            let mut i = 0_usize;
            while let Some((tr, mut linvel)) = collected.fetch_next() {
                let target = positions[i];
                let target = Vec3::new(target.x, center_tr.translation.y, target.y);
                let delta = tr.translation - target;

                linvel.0 = delta * Self::COLLECTED_SPEED;
                i += 1;
            }
        }
    }

    pub fn throw_collected(&self, direction: Dir2, force: f32) -> Option<impl FnOnce(&mut World)> {
        let (index, _) = self.circle_distrib.find_closest_aligned_point(direction)?;
        let Some(entity) = self.collected.get(index).copied() else {
            log::error!("Collector and circular distribution are out of sync, No entity found at index {index:?}");
            return None;
        };
        let direction = Vec3::new(direction.x, 0.0, direction.y);
        Some(move |world: &mut World| {
            let mut entity_cmd = world.entity_mut(entity);
            entity_cmd
                .insert(ExternalImpulse::new(direction * force))
                .remove::<Collected>();
        })
    }

    pub fn collect_items(
        mut commands: Commands,
        collectors: Query<(Entity, &CollidingEntities), With<Self>>,
        items: Query<Entity, (With<GarbageItem>, Without<Collected>)>,
    ) {
        for (collector_entity, collision) in &collectors {
            for item in items.iter_many(&collision.0) {
                commands.entity(item).insert(Collected { collector_entity });
            }
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
    pub fn new(min_radius: f32, max_distance: f32) -> Self {
        Self {
            transform: TransformBundle::default(),
            collider: Collider::sphere(1.0),
            sensor: Sensor,
            collectible_sensor: Collector::new(min_radius, max_distance),
            layer: CollisionLayers::new(ObjectLayer::Collector, [ObjectLayer::Collectible]),
        }
    }
}
