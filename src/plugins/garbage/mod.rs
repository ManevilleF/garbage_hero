use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        world::Command,
    },
    log,
    prelude::*,
};

mod builds;
mod collector;
mod distribution;
mod items;

use bevy_mod_outline::{OutlineBundle, OutlineVolume};
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
        .add_systems(PostUpdate, (Collector::update_radius, reset_thrown_items))
        .add_systems(PostProcessCollisions, filter_thrown_collisions);

        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, Collector::draw_gizmos);
    }
}

#[derive(Debug, Reflect)]
#[reflect(Component)]
pub struct ThrownItem {
    pub collector_entity: Entity,
}

impl Component for ThrownItem {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .on_add(|mut world, entity, _| {
                let thrown = world.get::<Self>(entity).unwrap();
                let Some(collector) = world.get::<Collector>(thrown.collector_entity) else {
                    log::error!("Thrown entity {entity:?} collector does not exist");
                    return;
                };
                let color = collector.color;
                let mut commands = world.commands();
                commands.entity(entity).insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 3.0,
                        colour: color,
                    },
                    ..default()
                });
            })
            .on_remove(|mut world, entity, _| {
                let mut commands = world.commands();
                commands.entity(entity).remove::<OutlineBundle>();
            });
    }
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

fn filter_thrown_collisions(
    mut collisions: ResMut<Collisions>,
    thrown: Query<&ThrownItem>,
    collected: Query<&Collected>,
) {
    collisions.retain(|contact| {
        let entities = [contact.entity1, contact.entity2];
        let mut thrown_item_collector: Option<Entity> = None;
        let mut collected_item_collector: Option<Entity> = None;

        for &entity in &entities {
            if let Ok(thrown_item) = thrown.get(entity) {
                thrown_item_collector = Some(thrown_item.collector_entity);
            }
            if let Ok(collected_item) = collected.get(entity) {
                collected_item_collector = Some(collected_item.collector_entity);
            }
        }

        // If both a thrown item and a collected item are found, compare their collector_entity fields
        if let (Some(thrown_collector), Some(collected_collector)) =
            (thrown_item_collector, collected_item_collector)
        {
            thrown_collector != collected_collector
        } else {
            // If either is None, we don't have a match and thus don't filter out the collision
            true
        }
    });
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
