use bevy::{ecs::world::Command, prelude::*};
use std::f32::consts::PI;

mod body;
mod builds;
mod collected;
mod collector;
mod distribution;
mod items;
mod throw;

pub use body::{GarbageBody, GarbageBodyPlugin};

pub use builds::{AvailableItemBuilds, SpawnBuild};
pub use collected::Collected;
pub use collector::{Collector, CollectorBundle, CollectorConfig, CollectorParticlesBundle};
pub use distribution::{DistributionShape, PointDistribution};
pub use items::{GarbageAssets, GarbageBundle, GarbageItem};
pub use throw::ThrownItem;

use builds::ItemBuildsPlugin;
use collected::CollectedPlugin;
use collector::CollectorPlugin;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;
use throw::ThrowPlugin;

use super::map::MAP_SIZE;

pub struct GarbagePlugin;

impl Plugin for GarbagePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CollectedPlugin,
            CollectorPlugin,
            ItemBuildsPlugin,
            ThrowPlugin,
            GarbageBodyPlugin,
        ))
        .init_resource::<GarbageAssets>()
        .register_type::<GarbageAssets>()
        .register_type::<GarbageItem>()
        .register_type::<PointDistribution>();
    }
}

pub fn spawn_some_garbage(amount: usize) -> impl FnOnce(&mut World) {
    move |world| {
        let square = Rectangle::new(MAP_SIZE.x - 2.0, MAP_SIZE.y - 2.0);
        let mut rng = thread_rng();
        let assets = world.resource::<GarbageAssets>();
        let bundles: Vec<_> = (0..amount)
            .map(|_| {
                let item = GarbageItem::iter().choose(&mut rng).unwrap();
                let pos = square.sample_interior(&mut rng);
                let position = Vec3::new(pos.x, 1.0, pos.y);
                let mut bundle = GarbageBundle::new(item, assets);
                bundle.pbr.transform.translation = position;
                bundle
            })
            .collect();
        world.spawn_batch(bundles);
    }
}

pub fn spawn_builds(amount: usize) -> impl FnOnce(&mut World) {
    move |world| {
        let square = Rectangle::new(MAP_SIZE.x - 20.0, MAP_SIZE.y - 20.0);
        let mut rng = thread_rng();
        let builds = world.resource::<AvailableItemBuilds>();
        let commands: Vec<_> = (0..amount)
            .map(|_| {
                let handle = builds.values().choose(&mut rng).unwrap().clone_weak();
                let pos = square.sample_interior(&mut rng);
                let position = Vec3::new(pos.x, 1.0, pos.y);
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
