use avian3d::prelude::*;
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    log,
    prelude::*,
};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};

use crate::Damage;

use super::{collector::CollectorConfig, Collected};

pub struct ThrowPlugin;

const THROW_DAMAGE: u16 = 10;

impl Plugin for ThrowPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ThrownItem>()
            .add_systems(Update, reset_thrown_items)
            .add_systems(PostProcessCollisions, filter_thrown_collisions);
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
                let Some(config) = world.get::<CollectorConfig>(thrown.collector_entity) else {
                    log::error!("Thrown entity {entity:?} collector config does not exist");
                    return;
                };
                let color = config.color;
                let mut commands = world.commands();
                commands.entity(entity).insert((
                    OutlineBundle {
                        outline: OutlineVolume {
                            visible: true,
                            width: 3.0,
                            colour: color,
                        },
                        ..default()
                    },
                    Damage(THROW_DAMAGE),
                ));
            })
            .on_remove(|mut world, entity, _| {
                let mut commands = world.commands();
                commands
                    .entity(entity)
                    .remove::<OutlineBundle>()
                    .remove::<Damage>();
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
    const TRESHOLD: f32 = 9.0;

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
