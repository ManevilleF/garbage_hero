use super::{
    Collected, DistributionShape, GarbageBody, GarbageItem, PointDistribution, ThrownItem,
};
use crate::{GameState, ObjectLayer, ParticleConfig};
use avian3d::prelude::*;
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    log,
    prelude::*,
};
use bevy_hanabi::{EffectProperties, EffectSpawner, ParticleEffect, ParticleEffectBundle};

pub struct CollectorPlugin;

impl Plugin for CollectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Collector>()
            .register_type::<CollectorConfig>()
            .add_systems(
                FixedUpdate,
                (auto_rotate, update_collected_position, collect_items)
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(PostUpdate, (update_radius, update_particles));

        #[cfg(feature = "debug")]
        app.add_systems(
            PostUpdate,
            draw_gizmos
                .after(avian3d::prelude::PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct CollectorConfig {
    pub enabled: bool,
    pub color: Color,
}

#[derive(Debug, Reflect)]
#[reflect(Component)]
pub struct Collector {
    min_radius: f32,
    growing: bool,
    distribution: PointDistribution,
    shape: DistributionShape,
    collected: Vec<Entity>,
}

#[derive(Debug, Component)]
pub struct OnCollectedFilterOut {
    pub layer: ObjectLayer,
}

impl Component for Collector {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            let Some(collector) = world.get::<Self>(entity) else {
                log::error!("on_remove hook triggered for {entity:?} without `Collector`");
                return;
            };
            let collected = collector.collected.clone();
            let mut commands = world.commands();
            for entity in collected {
                if let Some(mut cmd) = commands.get_entity(entity) {
                    cmd.remove::<Collected>();
                }
            }
        });
    }
}

#[derive(Bundle)]
pub struct CollectorBundle {
    pub spatial: SpatialBundle,
    pub collector: Collector,
    pub config: CollectorConfig,
    pub collider: Collider,
    pub sensor: Sensor,
    pub layer: CollisionLayers,
    pub name: Name,
    pub filter_out: OnCollectedFilterOut,
}

impl CollectorBundle {
    pub fn fixed(
        collector_radius: f32,
        max_distance: f32,
        color: Color,
        max_items: usize,
        max_points: usize,
        on_collected_filter: ObjectLayer,
    ) -> Self {
        Self {
            spatial: SpatialBundle::default(),
            collider: Collider::sphere(1.0),
            sensor: Sensor,
            collector: Collector::fixed(collector_radius, max_distance, max_items, max_points),
            layer: CollisionLayers::new(ObjectLayer::Collector, [ObjectLayer::Collectible]),
            name: Name::new("Garbage Collector"),
            config: CollectorConfig {
                enabled: false,
                color,
            },
            filter_out: OnCollectedFilterOut {
                layer: on_collected_filter,
            },
        }
    }

    pub fn growing(
        min_radius: f32,
        max_distance: f32,
        color: Color,
        max_items: usize,
        on_collected_filter: ObjectLayer,
    ) -> Self {
        Self {
            spatial: SpatialBundle::default(),
            collider: Collider::sphere(1.0),
            sensor: Sensor,
            collector: Collector::growing(min_radius, max_distance, max_items),
            layer: CollisionLayers::new(ObjectLayer::Collector, [ObjectLayer::Collectible]),
            name: Name::new("Garbage Collector"),
            config: CollectorConfig {
                enabled: false,
                color,
            },
            filter_out: OnCollectedFilterOut {
                layer: on_collected_filter,
            },
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CollectorParticles(pub Entity);

#[derive(Bundle)]
pub struct CollectorParticlesBundle {
    pub collector: CollectorParticles,
    pub particles: ParticleEffectBundle,
    pub name: Name,
}

impl CollectorParticlesBundle {
    pub fn new(collector_entity: Entity, color: Color, particles: &ParticleConfig) -> Self {
        Self {
            collector: CollectorParticles(collector_entity),
            particles: ParticleEffectBundle {
                effect: ParticleEffect::new(particles.collector_effect.clone()),
                effect_properties: EffectProperties::default()
                    .with_properties([("color".to_owned(), ParticleConfig::color_to_value(color))]),
                ..default()
            },
            name: Name::new(format!("Collector {collector_entity:?} particles")),
        }
    }
}

impl Collector {
    const ANGULAR_SPEED: f32 = 10.0;
    const COLLECTED_SPEED: f32 = 10.0;
    const COLLECTOR_RADIUS_COEF: f32 = 1.2;

    pub fn fixed(
        collection_radius: f32,
        max_distance: f32,
        max_items: usize,
        max_points: usize,
    ) -> Self {
        let mut distribution = PointDistribution::new(0.0, max_distance);
        distribution.update(max_points, DistributionShape::Circle);
        Self {
            min_radius: collection_radius,
            growing: false,
            distribution,
            shape: DistributionShape::Circle,
            collected: Vec::with_capacity(max_items),
        }
    }

    pub fn growing(min_radius: f32, max_distance: f32, max_items: usize) -> Self {
        Self {
            min_radius,
            growing: true,
            distribution: PointDistribution::new(min_radius, max_distance),
            shape: DistributionShape::Circle,
            collected: Vec::with_capacity(max_items),
        }
    }

    pub fn radius(&self) -> f32 {
        if self.growing {
            self.distribution.radius(self.len())
        } else {
            self.min_radius
        }
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

    #[inline]
    pub fn len(&self) -> usize {
        self.collected.len()
    }

    #[inline]
    pub fn points_len(&self) -> usize {
        self.distribution.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.collected.is_empty()
    }

    pub fn insert(&mut self, entity: Entity, dir: Option<Dir2>) -> bool {
        if self.len() >= self.collected.capacity() {
            return false;
        }
        match (
            self.growing,
            dir.and_then(|d| self.distribution.find_closest_aligned_point(d)),
        ) {
            (true, Some((index, _))) => {
                self.collected.insert(index, entity);
            }
            _ => {
                self.collected.push(entity);
            }
        }
        if self.growing {
            self.distribution.update(self.len(), self.shape);
        }
        true
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        let index = self.collected.iter().position(|e| *e == entity)?;
        let res = self.collected.remove(index);
        if self.growing {
            self.distribution.update(self.len(), self.shape);
        }
        Some(res)
    }

    pub fn set_shape(&mut self, shape: DistributionShape) {
        log::info!("Collector shape is now a `{shape}`");
        self.shape = shape;

        if self.growing {
            self.distribution.update(self.len(), self.shape);
        } else {
            self.distribution.update(self.points_len(), self.shape);
        }
    }

    #[inline]
    pub const fn shape(&self) -> DistributionShape {
        self.shape
    }

    pub fn throw_collected(&self, direction: Dir2, force: f32) -> Option<impl FnOnce(&mut World)> {
        let (index, _) = self.distribution.find_closest_aligned_point(direction)?;
        let Some(entity) = self.collected.get(index).copied() else {
            log::error!("Collector and circular distribution are out of sync, No entity found at index {index:?}");
            return None;
        };
        let direction = Vec3::new(direction.x, 0.0, direction.y);
        Some(move |world: &mut World| {
            let mass = world
                .get::<ColliderMassProperties>(entity)
                .map(|p| p.mass.0)
                .unwrap_or(1.0);
            let collected = world.get::<Collected>(entity).unwrap();
            let collector_entity = collected.collector_entity;
            let mut entity_cmd = world.entity_mut(entity);
            entity_cmd.remove::<Collected>().insert((
                LinearVelocity::default(),
                ExternalImpulse::new(direction * force * mass),
                ThrownItem::new(collector_entity),
            ));
        })
    }
}

pub fn update_radius(mut collectors: Query<(&mut Transform, &Collector), Changed<Collector>>) {
    for (mut tr, collector) in &mut collectors {
        tr.scale = Vec3::splat(collector.radius() * Collector::COLLECTOR_RADIUS_COEF);
    }
}

fn update_particles(
    mut commands: Commands,
    mut particles: Query<(
        Entity,
        &mut Transform,
        &mut EffectSpawner,
        &mut EffectProperties,
        &CollectorParticles,
    )>,
    collectors: Query<(&GlobalTransform, Ref<Collector>, &CollectorConfig)>,
) {
    for (entity, mut tr, mut spawner, mut properties, target) in &mut particles {
        let Ok((gtr, collector, config)) = collectors.get(target.0) else {
            log::error!("Collector particles target is invalid");
            spawner.set_active(false);
            commands.entity(entity).despawn();
            continue;
        };
        tr.translation = gtr.translation();
        spawner.set_active(config.enabled);
        if collector.is_changed() {
            properties.set("radius", collector.radius().into());
        }
    }
}

fn update_collected_position(
    mut collected: Query<(&Transform, &mut LinearVelocity), (With<Collected>, Without<ThrownItem>)>,
    collectors: Query<(&GlobalTransform, &Collector, Option<&GarbageBody>)>,
) {
    for (center_tr, collector, body) in &collectors {
        let center = center_tr.translation();
        let mut collected = collected.iter_many_mut(&collector.collected);
        let positions = match body {
            Some(b) => b.compute_3d_positions(collector.len(), &collector.distribution),
            None => collector
                .distribution
                .points()
                .iter()
                .map(|p| center + Vec3::new(p.x, 1.0, p.y))
                .collect(),
        };
        let mut i = 0_usize;
        while let Some((tr, mut linvel)) = collected.fetch_next() {
            let Some(target) = positions.get(i).copied() else {
                log::error!("Collector has more positions than it has collected items");
                continue;
            };
            let delta = (target - tr.translation) * Collector::COLLECTED_SPEED;

            linvel.0 = delta.clamp_length_max(GarbageItem::MAX_SPEED);
            i += 1;
        }
    }
}

fn auto_rotate(time: Res<Time>, mut collectors: Query<(&GlobalTransform, &mut Collector)>) {
    let dt = time.delta_seconds();
    for (gtr, mut collector) in &mut collectors {
        match collector.shape {
            DistributionShape::Circle => {
                let radius = collector.radius();
                collector
                    .distribution
                    .rotate(Collector::rotation_angle(radius, dt));
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

fn collect_items(
    mut commands: Commands,
    collectors: Query<(Entity, &CollidingEntities, &CollectorConfig), With<Collector>>,
    items: Query<Entity, (With<GarbageItem>, Without<Collected>, Without<ThrownItem>)>,
) {
    for (collector_entity, collision, config) in &collectors {
        if !config.enabled {
            continue;
        }
        for item in items.iter_many(&collision.0) {
            commands.entity(item).insert(Collected { collector_entity });
        }
    }
}

#[cfg(feature = "debug")]
fn draw_gizmos(
    mut gizmos: Gizmos,
    collectors: Query<(&GlobalTransform, &Collector, Option<&GarbageBody>)>,
) {
    use bevy::color::palettes::css::DARK_GRAY;
    let color = Color::Srgba(DARK_GRAY);

    for (gt, collector, body) in &collectors {
        let translation = gt.translation();
        gizmos.circle(translation, Dir3::Y, collector.radius(), color);
        let positions = match body {
            Some(b) => b.compute_3d_positions(collector.len(), &collector.distribution),
            None => collector
                .distribution
                .points()
                .iter()
                .map(|p| translation + Vec3::new(p.x, 1.0, p.y))
                .collect(),
        };
        for pos in positions {
            gizmos.sphere(pos, Quat::IDENTITY, 0.2, color);
        }
    }
}
