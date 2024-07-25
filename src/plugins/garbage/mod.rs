use bevy::{ecs::world::Command, prelude::*};
use std::f32::consts::PI;

mod builds;
mod collected;
mod collector;
mod distribution;
mod items;
mod throw;

pub use builds::{AvailableItemBuilds, SpawnBuild};
pub use collected::Collected;
pub use collector::{Collector, CollectorBundle, CollectorParticlesBundle};
pub use distribution::DistributionShape;
pub use items::{GarbageAssets, GarbageBundle, GarbageItem};
pub use throw::ThrownItem;

use builds::ItemBuildsPlugin;
use collected::CollectedPlugin;
use collector::CollectorPlugin;
use distribution::*;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;
use throw::ThrowPlugin;

pub struct GarbagePlugin;

impl Plugin for GarbagePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CollectedPlugin,
            CollectorPlugin,
            ItemBuildsPlugin,
            ThrowPlugin,
        ))
        .init_resource::<GarbageAssets>()
        .register_type::<GarbageAssets>()
        .register_type::<GarbageItem>()
        .register_type::<PointDistribution>();
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
