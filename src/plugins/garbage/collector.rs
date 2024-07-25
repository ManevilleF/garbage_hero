use super::{Collected, DistributionShape, GarbageItem, PointDistribution, ThrownItem};
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
            .add_systems(
                FixedUpdate,
                (auto_rotate, update_collected_position, collect_items)
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(PostUpdate, (update_radius, update_particles));

        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, draw_gizmos);
    }
}

#[derive(Debug, Reflect)]
#[reflect(Component)]
pub struct Collector {
    pub enabled: bool,
    pub color: Color,
    distribution: PointDistribution,
    shape: DistributionShape,
    collected: Vec<Entity>,
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
                commands.entity(entity).remove::<Collected>();
            }
        });
    }
}

#[derive(Bundle)]
pub struct CollectorBundle {
    pub spatial: SpatialBundle,
    pub collectible_sensor: Collector,
    pub collider: Collider,
    pub sensor: Sensor,
    pub layer: CollisionLayers,
    pub name: Name,
}

impl CollectorBundle {
    pub fn new(min_radius: f32, max_distance: f32, color: Color) -> Self {
        Self {
            spatial: SpatialBundle::default(),
            collider: Collider::sphere(1.0),
            sensor: Sensor,
            collectible_sensor: Collector::new(min_radius, max_distance, color),
            layer: CollisionLayers::new(ObjectLayer::Collector, [ObjectLayer::Collectible]),
            name: Name::new("Garbage Collector"),
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
    const MAX_ITEMS: usize = 75;
    const COLLECTOR_RADIUS_COEF: f32 = 1.2;

    pub fn new(min_radius: f32, max_distance: f32, color: Color) -> Self {
        Self {
            enabled: false,
            color,
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

    #[inline]
    pub fn len(&self) -> usize {
        self.collected.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.collected.is_empty()
    }

    pub fn insert(&mut self, entity: Entity, dir: Option<Dir2>) -> bool {
        if self.len() >= Self::MAX_ITEMS {
            return false;
        }
        match dir.and_then(|d| self.distribution.find_closest_aligned_point(d)) {
            Some((index, _)) => {
                self.collected.insert(index, entity);
            }
            None => {
                self.collected.push(entity);
            }
        }
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
            entity_cmd
                .insert((
                    LinearVelocity::default(),
                    ExternalImpulse::new(direction * force * mass),
                    ThrownItem::new(collector_entity),
                ))
                .remove::<Collected>();
        })
    }
}

pub fn update_radius(mut collectors: Query<(&mut Transform, &Collector), Changed<Collector>>) {
    for (mut tr, collector) in &mut collectors {
        tr.scale = Vec3::splat(collector.radius() * Collector::COLLECTOR_RADIUS_COEF);
    }
}

fn update_particles(
    mut particles: Query<(
        &mut Transform,
        &mut EffectSpawner,
        &mut EffectProperties,
        &CollectorParticles,
    )>,
    collectors: Query<(&GlobalTransform, Ref<Collector>)>,
) {
    for (mut tr, mut spawner, mut properties, target) in &mut particles {
        let Ok((gtr, collector)) = collectors.get(target.0) else {
            log::error!("Collector particles target is invalid");
            continue;
        };
        tr.translation = gtr.translation();
        if !collector.is_changed() {
            continue;
        }
        spawner.set_active(collector.enabled);
        properties.set("radius", collector.radius().into());
    }
}

fn update_collected_position(
    mut collected: Query<(&Transform, &mut LinearVelocity), (With<Collected>, Without<ThrownItem>)>,
    mut collectors: Query<(&GlobalTransform, &Collector)>,
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

            linvel.0 = delta * Collector::COLLECTED_SPEED;
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
    collectors: Query<(Entity, &CollidingEntities, &Collector)>,
    items: Query<Entity, (With<GarbageItem>, Without<Collected>, Without<ThrownItem>)>,
) {
    for (collector_entity, collision, collector) in &collectors {
        if !collector.enabled {
            continue;
        }
        for item in items.iter_many(&collision.0) {
            commands.entity(item).insert(Collected { collector_entity });
        }
    }
}

#[cfg(feature = "debug")]
fn draw_gizmos(mut gizmos: Gizmos, collectors: Query<(&GlobalTransform, &Collector)>) {
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
