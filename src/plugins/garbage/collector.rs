use avian3d::prelude::*;
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    log,
    prelude::*,
};

use crate::ObjectLayer;

use super::{ArcDistribution, CircularDistribution, GarbageItem};

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
                let Some(mut collector) = world.get_mut::<Collector>(collected.collector_entity)
                else {
                    log::error!("Cannot find collector of `Collected` entity {entity:?}");
                    return;
                };
                // Can be already removed
                collector.insert(entity);
                let Some(mut layer) = world.get_mut::<CollisionLayers>(entity) else {
                    log::error!("on_add hook triggered for {entity:?} without `CollisionLayers`");
                    return;
                };
                // Collected entities should no longer interact withsome things
                layer.filters.remove(ObjectLayer::Map);
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
                layer.filters.add(ObjectLayer::Map);
                layer.filters.add(ObjectLayer::Collector);

                // Reset gravity scale
                let Some(mut scale) = world.get_mut::<GravityScale>(entity) else {
                    log::warn!("on_add hook triggered for {entity:?} without `GravityScale`");
                    return;
                };
                scale.0 = 1.0;
            });
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Collector {
    pub circle_distrib: CircularDistribution,
    pub arc_distrib: ArcDistribution,
    collected: Vec<Entity>,
}

impl Collector {
    const ROTATION_SPEED: f32 = 2.0;
    const COLLECTED_SPEED: f32 = 10.0;

    pub fn new(min_radius: f32, max_distance: f32) -> Self {
        Self {
            circle_distrib: CircularDistribution::new(min_radius, max_distance),
            arc_distrib: ArcDistribution::new(min_radius, max_distance),
            collected: Vec::new(),
        }
    }

    pub fn radius(&self) -> f32 {
        self.circle_distrib.radius(self.len())
    }

    pub fn tick_circle_rotation(&mut self, dt: f32) {
        self.circle_distrib.rotate(dt * Self::ROTATION_SPEED);
    }

    pub fn update_radius(mut collectors: Query<(&mut Transform, &Self), Changed<Self>>) {
        for (mut tr, collector) in &mut collectors {
            tr.scale = Vec3::splat(collector.radius());
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.collected.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.collected.is_empty()
    }

    pub fn insert(&mut self, entity: Entity) {
        self.collected.push(entity);
        self.circle_distrib.set_amount(self.len());
        self.arc_distrib.set_amount(self.len());
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        let index = self.collected.iter().position(|e| *e == entity)?;
        let res = self.collected.remove(index);
        self.circle_distrib.set_amount(self.len());
        self.arc_distrib.set_amount(self.len());
        Some(res)
    }

    pub fn update_collected_position(
        time: Res<Time>,
        mut collected: Query<
            (&Transform, &mut LinearVelocity),
            (With<Collected>, With<GarbageItem>),
        >,
        mut collectors: Query<(&GlobalTransform, &mut Self)>,
    ) {
        let dt = time.delta_seconds();
        for (center_tr, mut collector) in &mut collectors {
            let center = center_tr.translation();
            collector.tick_circle_rotation(dt);
            let mut collected = collected.iter_many_mut(&collector.collected);
            let positions = collector.circle_distrib.points();
            let mut i = 0_usize;
            while let Some((tr, mut linvel)) = collected.fetch_next() {
                let target = positions[i];
                let target = center + Vec3::new(target.x, 1.0, target.y);
                let delta = target - tr.translation;

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
                println!("COLLECTED {item:?}");
                commands.entity(item).insert(Collected { collector_entity });
            }
        }
    }

    pub fn draw_gizmos(mut gizmos: Gizmos, collectors: Query<(&GlobalTransform, &Self)>) {
        use bevy::color::palettes::css::YELLOW;
        for (gt, collector) in &collectors {
            let translation = gt.translation();
            gizmos.circle(
                translation,
                Dir3::Y,
                collector.circle_distrib.radius(collector.collected.len()),
                Color::Srgba(YELLOW),
            );
            for pos in collector.circle_distrib.points() {
                let p = translation + Vec3::new(pos.x, 0.0, pos.y);
                gizmos.sphere(p, Quat::IDENTITY, 0.2, Color::Srgba(YELLOW));
            }
        }
    }
}

#[derive(Bundle)]
pub struct CollectorBundle {
    pub transform: TransformBundle,
    pub collectible_sensor: Collector,
    pub collider: Collider,
    pub sensor: Sensor,
    pub layer: CollisionLayers,
    pub name: Name,
}

impl CollectorBundle {
    pub fn new(min_radius: f32, max_distance: f32) -> Self {
        Self {
            transform: TransformBundle::default(),
            collider: Collider::sphere(1.0),
            sensor: Sensor,
            collectible_sensor: Collector::new(min_radius, max_distance),
            layer: CollisionLayers::new(ObjectLayer::Collector, [ObjectLayer::Collectible]),
            name: Name::new("Garbage Collector"),
        }
    }
}
