use avian3d::prelude::*;
use bevy::{
    ecs::{
        component::{Mutable, StorageType},
        lifecycle::ComponentHook,
        system::SystemParam,
    },
    log,
    prelude::*,
};
use bevy_mod_outline::OutlineVolume;

use crate::Damage;

use super::{Collected, collector::CollectorConfig};

pub struct ThrowPlugin;

const THROW_DAMAGE: u16 = 10;
const THROW_MIN_TIMER: f32 = 1.0;

impl Plugin for ThrowPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ThrownItem>()
            .add_systems(Update, update_thrown_items)
            .add_systems(Update, on_thrown_items);
    }
}

#[derive(Debug, Reflect)]
#[reflect(Component)]
pub struct ThrownItem {
    pub collector_entity: Entity,
    timer: f32,
    force: Vec3,
}

impl Component for ThrownItem {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn on_add() -> Option<ComponentHook> {
        Some(|mut world, ctx| {
            let thrown = world.get::<Self>(ctx.entity).unwrap();
            let Some(config) = world.get::<CollectorConfig>(thrown.collector_entity) else {
                log::error!(
                    "Thrown entity {:?} collector config does not exist",
                    ctx.entity
                );
                return;
            };
            let color = config.color;
            let mut commands = world.commands();
            commands.entity(ctx.entity).insert((
                OutlineVolume {
                    visible: true,
                    width: 3.0,
                    colour: color,
                },
                Damage(THROW_DAMAGE),
            ));
        })
    }

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world, ctx| {
            let mut commands = world.commands();
            commands
                .entity(ctx.entity)
                .remove::<OutlineVolume>()
                .remove::<Damage>();
        })
    }
}

impl ThrownItem {
    pub const fn new(collector_entity: Entity, force: Vec3) -> Self {
        Self {
            collector_entity,
            timer: 0.0,
            force,
        }
    }
}

fn update_thrown_items(
    time: Res<Time>,
    mut commands: Commands,
    mut items: Query<(Entity, &mut ThrownItem, &LinearVelocity)>,
) {
    const TRESHOLD: f32 = 12.0;

    let dt = time.delta_secs();
    for (entity, mut thrown, linvel) in &mut items {
        thrown.timer += dt;
        if thrown.timer >= THROW_MIN_TIMER && linvel.0.length_squared() <= TRESHOLD {
            commands.entity(entity).remove::<ThrownItem>();
        }
    }
}

fn on_thrown_items(mut items: Query<(Forces, &ThrownItem), Added<ThrownItem>>) {
    for (mut forces, item) in &mut items {
        forces.apply_linear_impulse(item.force);
    }
}

#[derive(SystemParam)]
pub struct ThrownItemHooks<'w, 's> {
    thrown: Query<'w, 's, &'static ThrownItem>,
    collected: Query<'w, 's, &'static Collected>,
}

impl CollisionHooks for ThrownItemHooks<'_, '_> {
    fn filter_pairs(&self, collider1: Entity, collider2: Entity, _: &mut Commands) -> bool {
        let entities = [collider1, collider2];
        let mut thrown_item_collector: Option<Entity> = None;
        let mut collected_item_collector: Option<Entity> = None;

        for &entity in &entities {
            if let Ok(thrown_item) = self.thrown.get(entity) {
                thrown_item_collector = Some(thrown_item.collector_entity);
            }
            if let Ok(collected_item) = self.collected.get(entity) {
                collected_item_collector = Some(collected_item.collector_entity);
            }
        }

        // If both a thrown item and a collected item are found, compare their
        // collector_entity fields
        if let (Some(thrown_collector), Some(collected_collector)) =
            (thrown_item_collector, collected_item_collector)
        {
            thrown_collector != collected_collector
        } else {
            // If either is None, we don't have a match and thus don't filter out the
            // collision
            true
        }
    }
}
