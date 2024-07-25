use avian3d::{collision::CollidingEntities, prelude::*};
use bevy::{log, prelude::*};

use super::{
    garbage::{Collected, ThrownItem},
    player::Player,
};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>()
            .register_type::<Damage>()
            .register_type::<Dead>()
            .add_systems(First, despawn_deads)
            .add_systems(PreUpdate, handle_death)
            .add_systems(Update, (direct_damage, velocity_damage));
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
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Damage(pub u16);

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Dead;

fn direct_damage(damage: Query<(&Damage, &CollidingEntities)>, mut healths: Query<&mut Health>) {
    for (damage, collision) in &damage {
        let mut healths = healths.iter_many_mut(&collision.0);
        while let Some(mut health) = healths.fetch_next() {
            health.damage(damage.0);
        }
    }
}

fn velocity_damage(
    mut events: EventReader<CollisionStarted>,
    mut healths: Query<
        (
            Option<&LinearVelocity>,
            &mut Health,
            Option<&Collected>,
            Option<&ThrownItem>,
        ),
        (Without<Damage>, With<Collider>),
    >,
) {
    const VEL_TRESHOLD: f32 = 10.0;
    const DAMAGE_RATIO: f32 = 0.1;

    for CollisionStarted(a, b) in events.read() {
        let Ok(
            [(linvel_a, mut health_a, collected_a, thrown_a), (linvel_b, mut health_b, collected_b, thrown_b)],
        ) = healths.get_many_mut([*a, *b])
        else {
            // debug ?
            continue;
        };
        // Skip damage if both objects are collected by the same entity
        if collected_a
            .map(|c| c.collector_entity)
            .or_else(|| thrown_a.map(|t| t.collector_entity))
            .zip(
                collected_b
                    .map(|c| c.collector_entity)
                    .or_else(|| thrown_b.map(|t| t.collector_entity)),
            )
            .map(|(a, b)| a == b)
            .unwrap_or(false)
        {
            continue;
        }
        let velocity = linvel_a.map(|v| v.0).unwrap_or(Vec3::ZERO)
            + linvel_b.map(|v| v.0).unwrap_or(Vec3::ZERO);
        let length = velocity.length().floor();
        let damage = ((length - VEL_TRESHOLD) * DAMAGE_RATIO).floor() as u16;
        if damage > 0 {
            health_a.damage(damage);
            health_b.damage(damage);
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
