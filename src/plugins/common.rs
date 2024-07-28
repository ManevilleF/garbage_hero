use avian3d::prelude::*;
use bevy::{log, prelude::*};

use super::player::Player;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>()
            .register_type::<Damage>()
            .register_type::<Dead>()
            .add_systems(First, despawn_deads)
            .add_systems(PreUpdate, handle_death)
            .add_systems(Update, (direct_damage, tick_health));
    }
}

#[derive(Debug, Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub current: u16,
    pub max: u16,
    damage_cooldown: f32,
}

impl Health {
    const INVINCIBLE_TIME: f32 = 0.5;

    pub const fn new(max: u16) -> Self {
        Self {
            current: max,
            max,
            damage_cooldown: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.damage_cooldown += dt;
    }

    /// Returns `true` if health is still over 0
    pub fn damage(&mut self, amount: u16) -> bool {
        if self.damage_cooldown >= Self::INVINCIBLE_TIME {
            self.current = self.current.saturating_sub(amount);
            self.damage_cooldown = 0.0;
        }
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

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Damage(pub u16);

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Dead;

fn tick_health(time: Res<Time>, mut healths: Query<&mut Health, Without<Dead>>) {
    let dt = time.delta_seconds();
    for mut health in &mut healths {
        health.tick(dt);
    }
}

fn direct_damage(
    mut events: EventReader<CollisionStarted>,
    mut entities: Query<(Option<&Damage>, Option<&mut Health>), Or<(With<Health>, With<Damage>)>>,
) {
    for CollisionStarted(a, b) in events.read() {
        let Ok([(damage_a, health_a), (damage_b, health_b)]) = entities.get_many_mut([*a, *b])
        else {
            continue;
        };
        if let Some((damage, mut health)) = damage_a.zip(health_b) {
            health.damage(damage.0);
        }
        if let Some((damage, mut health)) = damage_b.zip(health_a) {
            health.damage(damage.0);
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
