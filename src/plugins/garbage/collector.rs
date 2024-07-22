use avian3d::prelude::*;
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    log,
    prelude::*,
};

use crate::ObjectLayer;

use super::{DistributionShape, GarbageItem, PointDistribution};

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
                if !collector.insert(entity) {
                    return;
                };
                let Some(mut layer) = world.get_mut::<CollisionLayers>(entity) else {
                    log::error!("on_add hook triggered for {entity:?} without `CollisionLayers`");
                    return;
                };
                // Collected entities should no longer interact withsome things
                layer.filters.remove(ObjectLayer::Player);
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
                layer.filters.add(ObjectLayer::Player);
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
    distribution: PointDistribution,
    shape: DistributionShape,
    collected: Vec<Entity>,
}

impl Collector {
    const ANGULAR_SPEED: f32 = 10.0;
    const COLLECTED_SPEED: f32 = 10.0;
    const MAX_ITEMS: usize = 100;

    pub fn new(min_radius: f32, max_distance: f32) -> Self {
        Self {
            distribution: PointDistribution::new(min_radius, max_distance),
            shape: DistributionShape::Circle,
            collected: Vec::with_capacity(Self::MAX_ITEMS),
        }
    }

    pub fn radius(&self) -> f32 {
        self.distribution.radius(self.len())
    }

    /// Calculate the rotation angle in radians for a constant linear speed.
    ///
    /// # Arguments
    ///
    /// * `radius` - The radius of the circular path (m).
    /// * `dt` - The time interval over which to calculate the rotation angle (s).
    ///
    /// # Returns
    ///
    /// * `f32` - The rotation angle in radians.
    fn rotation_angle(radius: f32, dt: f32) -> f32 {
        let angular_speed = Self::ANGULAR_SPEED / radius;
        angular_speed * dt
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

    pub fn insert(&mut self, entity: Entity) -> bool {
        if self.len() >= Self::MAX_ITEMS {
            return false;
        }
        self.collected.push(entity);
        self.distribution.update(self.len(), self.shape);
        true
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        let index = self.collected.iter().position(|e| *e == entity)?;
        let res = self.collected.remove(index);
        self.distribution.update(self.len(), self.shape);
        Some(res)
    }

    pub fn set_shape(&mut self, shape: DistributionShape) {
        log::info!("Collector shape is now a `{shape}`");
        self.shape = shape;
        self.distribution.update(self.len(), self.shape);
    }

    #[inline]
    pub const fn shape(&self) -> DistributionShape {
        self.shape
    }

    pub fn update_collected_position(
        mut collected: Query<(&Transform, &mut LinearVelocity), With<Collected>>,
        mut collectors: Query<(&GlobalTransform, &Self)>,
    ) {
        for (center_tr, collector) in &mut collectors {
            let center = center_tr.translation();
            let mut collected = collected.iter_many_mut(&collector.collected);
            let positions = collector.distribution.points();
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

    pub fn auto_rotate(time: Res<Time>, mut collectors: Query<(&GlobalTransform, &mut Self)>) {
        let dt = time.delta_seconds();
        for (gtr, mut collector) in &mut collectors {
            match collector.shape {
                DistributionShape::Circle => {
                    let radius = collector.radius();
                    collector
                        .distribution
                        .rotate(Self::rotation_angle(radius, dt));
                }
                DistributionShape::Arc => {
                    let dir = gtr.forward().xz();
                    collector
                        .distribution
                        .set_direction(Dir2::new_unchecked(dir));
                }
            }
        }
    }

    pub fn throw_collected(&self, direction: Dir2, force: f32) -> Option<impl FnOnce(&mut World)> {
        let (index, _) = self.distribution.find_closest_aligned_point(direction)?;
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
        use bevy::color::palettes::css::DARK_GRAY;
        let color = Color::Srgba(DARK_GRAY);

        for (gt, collector) in &collectors {
            let translation = gt.translation();
            gizmos.circle(
                translation,
                Dir3::Y,
                collector.distribution.radius(collector.collected.len()),
                color,
            );
            for pos in collector.distribution.points() {
                let p = translation + Vec3::new(pos.x, 0.0, pos.y);
                gizmos.sphere(p, Quat::IDENTITY, 0.2, color);
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
