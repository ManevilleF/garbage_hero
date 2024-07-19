use bevy::prelude::*;

mod collector;
mod distribution;
mod items;

pub use collector::CollectorBundle;

use collector::*;
use distribution::*;
use items::*;

pub struct GarbagePlugin;

impl Plugin for GarbagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GarbageAssets>()
            .register_type::<GarbageItem>()
            .register_type::<Collected>()
            .register_type::<Collector>()
            .register_type::<CircularDistribution>();
        app.add_systems(
            FixedUpdate,
            (
                Collector::update_collected_position,
                Collector::collect_items,
            ),
        )
        .add_systems(PostUpdate, Collector::update_radius);
    }
}
