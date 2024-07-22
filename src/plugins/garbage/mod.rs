use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

mod collector;
mod distribution;
mod items;

pub use collector::{Collector, CollectorBundle};
pub use distribution::DistributionShape;
pub use items::{GarbageAssets, GarbageBundle, GarbageItem};

use collector::*;
use distribution::*;
use rand::{seq::IteratorRandom, thread_rng};
use strum::IntoEnumIterator;

pub struct GarbagePlugin;

impl Plugin for GarbagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GarbageAssets>()
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
        .add_systems(PostUpdate, (Collector::update_radius, handle_thrown_items));

        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, Collector::draw_gizmos);
    }
}

#[derive(Debug, Reflect, Component)]
#[reflect(Component)]
pub struct ThrownItem;

fn handle_thrown_items(
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
