use bevy::prelude::*;

mod collector;
mod items;

use collector::*;
use items::*;

pub struct GarbagePlugin;

impl Plugin for GarbagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GarbageAssets>()
            .register_type::<GarbageItem>()
            .register_type::<Collected>()
            .register_type::<Collector>();
        app.add_systems(FixedUpdate, (Collector::update_collected_position))
            .add_systems(PostUpdate, (Collector::update_radius));
    }
}
