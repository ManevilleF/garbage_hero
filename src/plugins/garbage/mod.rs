use std::f32::consts::PI;

use avian3d::prelude::LinearVelocity;
use bevy::{ecs::world::Command, prelude::*};

mod builds;
mod collector;
mod distribution;
mod items;

pub use builds::{AvailableItemBuilds, ItemBuild, SpawnBuild};
pub use collector::{Collected, Collector, CollectorBundle};
pub use distribution::DistributionShape;
pub use items::{GarbageAssets, GarbageBundle, GarbageItem};

use builds::ItemBuildsPlugin;
use distribution::*;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;

pub struct GarbagePlugin;

impl Plugin for GarbagePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ItemBuildsPlugin)
            .init_resource::<GarbageAssets>()
            .register_type::<GarbageAssets>()
            .register_type::<GarbageItem>()
            .register_type::<Collected>()
            .register_type::<Collector>()
            .register_type::<ThrownItem>()
            .register_type::<PointDistribution>();
        app.add_systems(
            FixedUpdate,
            (
                Collector::auto_rotate,
                Collector::update_collected_position,
                Collector::collect_items,
            ),
        )
        .add_systems(PostUpdate, (Collector::update_radius, reset_thrown_items));

        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, Collector::draw_gizmos);
    }
}

#[derive(Debug, Reflect, Component)]
#[reflect(Component)]
pub struct ThrownItem {
    pub collector_entity: Entity,
}

impl ThrownItem {
    pub const fn new(collector_entity: Entity) -> Self {
        Self { collector_entity }
    }
}

fn reset_thrown_items(
    mut commands: Commands,
    items: Query<(Entity, &LinearVelocity), With<ThrownItem>>,
) {
    const TRESHOLD: f32 = 1.0;

    for (entity, linvel) in &items {
        if linvel.0.length_squared() <= TRESHOLD {
            commands.entity(entity).remove::<ThrownItem>();
        }
    }
}

pub fn spawn_some_garbage<S>(amount: usize, origin: Vec3, shape: S) -> impl FnOnce(&mut World)
where
    S: ShapeSample<Output = Vec3>,
{
    move |world| {
        let mut rng = thread_rng();
        let assets = world.resource::<GarbageAssets>();
        let bundles: Vec<_> = (0..amount)
            .map(|_| {
                let item = GarbageItem::iter().choose(&mut rng).unwrap();
                let position = origin + shape.sample_interior(&mut rng);
                let mut bundle = GarbageBundle::new(item, assets);
                bundle.pbr.transform.translation = position;
                bundle
            })
            .collect();
        world.spawn_batch(bundles);
    }
}

pub fn spawn_builds(amount: usize, origin: Vec3, range: f32) -> impl FnOnce(&mut World) {
    move |world| {
        let mut rng = thread_rng();
        let builds = world.resource::<AvailableItemBuilds>();
        let commands: Vec<_> = (0..amount)
            .map(|_| {
                let handle = builds.values().choose(&mut rng).unwrap().clone_weak();
                let pos = Circle::new(range).sample_interior(&mut rng);
                let position = origin + Vec3::new(pos.x, 1.0, pos.y);
                let angle = rng.gen_range(0.0..PI);
                SpawnBuild {
                    handle,
                    position,
                    angle,
                }
            })
            .collect();
        for command in commands {
            command.apply(world);
        }
    }
}
