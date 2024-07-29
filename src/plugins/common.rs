use std::ops::Deref;

use avian3d::prelude::*;
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    log,
    prelude::*,
};
use bevy_mod_outline::OutlineVolume;

use super::player::Player;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>()
            .register_type::<Damage>()
            .register_type::<Dead>()
            .register_type::<Invincible>()
            .add_systems(First, despawn_deads)
            .add_systems(PreUpdate, handle_death)
            .add_systems(Update, (direct_damage, tick_invincibility));
    }
}

#[derive(Debug, Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub current: u16,
    pub max: u16,
}

impl Health {
    pub const fn new(max: u16) -> Self {
        Self { current: max, max }
    }

    /// Returns `true` if health is still over 0
    pub fn damage(&mut self, amount: u16) -> bool {
        self.current = self.current.saturating_sub(amount);
        self.current > 0
    }

    pub fn heal(&mut self, amount: u16) {
        self.current += amount;
        self.current = self.current.min(self.max);
    }

    pub fn increase_max(&mut self, amount: u16) {
        self.max += amount;
        self.heal(amount);
    }

    pub fn ratio(&self) -> f32 {
        self.current as f32 / self.max as f32
    }

    pub fn reset(&mut self) {
        self.current = self.max;
    }
}

#[derive(Debug, Reflect)]
#[reflect(Component)]
pub struct Invincible(pub f32);

impl Invincible {
    pub fn tick(&mut self, dt: f32) -> bool {
        self.0 -= dt;
        self.0 > 0.0
    }

    pub fn player() -> Self {
        Self(0.1)
    }
}

impl Component for Invincible {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .on_add(|mut world, entity, _| {
                if let Some(mut volume) = world.get_mut::<OutlineVolume>(entity) {
                    volume.visible = true;
                } else if let Some(children) =
                    world.get::<Children>(entity).map(|c| c.deref().to_vec())
                {
                    for entity in children {
                        if let Some(mut volume) = world.get_mut::<OutlineVolume>(entity) {
                            volume.visible = true;
                            break;
                        }
                    }
                }
            })
            .on_remove(|mut world, entity, _| {
                if let Some(mut volume) = world.get_mut::<OutlineVolume>(entity) {
                    volume.visible = false;
                } else if let Some(children) =
                    world.get::<Children>(entity).map(|c| c.deref().to_vec())
                {
                    for entity in children {
                        if let Some(mut volume) = world.get_mut::<OutlineVolume>(entity) {
                            volume.visible = false;
                            break;
                        }
                    }
                }
            });
    }
}

impl Default for Invincible {
    fn default() -> Self {
        Self(0.03)
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Damage(pub u16);

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Dead;

fn tick_invincibility(
    mut commands: Commands,
    time: Res<Time>,
    mut invincibility: Query<(Entity, &mut Invincible)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut invincible) in &mut invincibility {
        if !invincible.tick(dt) {
            commands.entity(entity).remove::<Invincible>();
        }
    }
}

fn direct_damage(
    mut commands: Commands,
    mut events: EventReader<CollisionStarted>,
    mut entities: Query<
        (
            Option<&Damage>,
            Option<&mut Health>,
            Has<Invincible>,
            Has<Player>,
        ),
        Or<(With<Health>, With<Damage>)>,
    >,
) {
    for CollisionStarted(a, b) in events.read() {
        let Ok(
            [(damage_a, health_a, invicible_a, is_player_a), (damage_b, health_b, invicible_b, is_player_b)],
        ) = entities.get_many_mut([*a, *b])
        else {
            continue;
        };
        if !invicible_b {
            if let Some((damage, mut health)) = damage_a.zip(health_b) {
                health.damage(damage.0);
                commands.entity(*b).insert(if is_player_b {
                    Invincible::player()
                } else {
                    Invincible::default()
                });
            }
        }
        if !invicible_a {
            if let Some((damage, mut health)) = damage_b.zip(health_a) {
                health.damage(damage.0);
                commands.entity(*a).insert(if is_player_a {
                    Invincible::player()
                } else {
                    Invincible::default()
                });
            }
        }
    }
}

fn handle_death(mut commands: Commands, entities: Query<(Entity, &Health), Changed<Health>>) {
    for (entity, health) in &entities {
        if health.current > 0 {
            continue;
        }
        commands.entity(entity).insert(Dead);
    }
}

fn despawn_deads(mut commands: Commands, entities: Query<(Entity, Option<&Player>), With<Dead>>) {
    for (entity, player) in &entities {
        if let Some(player) = player {
            // TODO
            log::info!("Player died: {}", player.id);
        } else {
            // TODO: Spawn animation
            commands.entity(entity).despawn_recursive();
        }
    }
}
