use super::{
    common::Health,
    garbage::{CollectorBundle, CollectorParticlesBundle},
    Dead,
};
use crate::{ObjectLayer, ParticleConfig};
use bevy::prelude::*;

mod assets;
mod input;
mod movement;
mod skills;
mod ui;

use assets::{PlayerAimMarkerBundle, PlayerAssets, PlayerVisualsBundle, PlayerVisualsPlugin};
pub use input::{GameController, GamepadCategory, PlayerInput};
use input::{PlayerInputBundle, PlayerInputPlugin};
use movement::{PlayerMovementBundle, PlayerMovementPlugin};
pub use skills::{ActiveSkill, SkillState};
use skills::{PlayerSkillsBundle, PlayerSkillsPlugin};
use ui::PlayerUiPlugin;

const MAX_PLAYERS: u8 = 10;
const PLAYER_RADIUS: f32 = 0.8;
const PLAYER_HEIGHT: f32 = 1.5;
const BASE_PLAYER_HEALTH: u16 = 200;
const BASE_SENSOR_STRENGTH: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerVisualsPlugin,
            PlayerInputPlugin,
            PlayerMovementPlugin,
            PlayerSkillsPlugin,
            PlayerUiPlugin,
        ))
        .add_event::<PlayerConnected>()
        .register_type::<Player>()
        .register_type::<PlayerConnected>()
        .add_systems(Update, spawn_players);
    }
}

#[derive(Debug, Event, Reflect)]
pub struct PlayerConnected(pub Player);

#[derive(Debug, Component, Clone, Copy, Reflect)]
pub struct Player {
    pub id: u8,
    pub controller: GameController,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub name: Name,
    pub health: Health,
    pub input: PlayerInputBundle,
    pub movement: PlayerMovementBundle,
    pub skills: PlayerSkillsBundle,
    pub spatial: SpatialBundle,
}

impl PlayerBundle {
    pub fn new(player: Player, server: &AssetServer) -> Self {
        if player.id >= MAX_PLAYERS {
            panic!("{MAX_PLAYERS} players supported");
        }
        Self {
            name: Name::new(format!("Player {}: {}", player.id, player.controller)),
            health: Health::new(BASE_PLAYER_HEALTH),
            input: PlayerInputBundle::new(player.controller, server),
            movement: PlayerMovementBundle::new(100.0, 0.9),
            skills: PlayerSkillsBundle::new(),
            spatial: Default::default(),
            player,
        }
    }
}

pub fn spawn_players(
    mut commands: Commands,
    players: Query<&GlobalTransform, With<Player>>,
    mut connected_evr: EventReader<PlayerConnected>,
    assets: Res<PlayerAssets>,
    particles: Res<ParticleConfig>,
    asset_server: Res<AssetServer>,
) {
    let position = players
        .iter()
        .next()
        .map(|gtr| gtr.translation())
        .unwrap_or(Vec3::ZERO)
        + Vec3::ONE * 3.0;
    for PlayerConnected(player) in connected_evr.read() {
        let color = assets.colors[player.id as usize];
        // Offset
        let mut bundle = PlayerBundle::new(*player, &asset_server);
        bundle.spatial.transform.translation = position;

        let root_entity = commands
            .spawn((
                SpatialBundle::default(),
                Name::new(format!("{} Root", bundle.name)),
            ))
            .id();
        let player_entity = commands.spawn(bundle).set_parent(root_entity).id();

        let collector_entity = commands
            .spawn(CollectorBundle::growing(
                4.0,
                1.0,
                color,
                50,
                ObjectLayer::Player,
            ))
            .set_parent(player_entity)
            .id();
        commands
            .spawn(CollectorParticlesBundle::new(
                collector_entity,
                color,
                &particles,
            ))
            .set_parent(root_entity);
        commands
            .spawn(PlayerVisualsBundle::new(player.id as usize, &assets))
            .set_parent(player_entity);
        // Marker
        commands
            .spawn(PlayerAimMarkerBundle::new(
                player.id as usize,
                player_entity,
                &assets,
            ))
            .set_parent(root_entity);
    }
}

pub fn reset_players(world: &mut World) {
    let mut players = world.query_filtered::<(Entity, &mut Health, &mut Transform), With<Player>>();
    let mut entities = Vec::new();
    for (i, (entity, mut health, mut tr)) in players.iter_mut(world).enumerate() {
        health.reset();
        tr.translation.z = 0.0;
        tr.translation.x = i as f32 * 2.0;
        entities.push(entity);
    }
    for entity in entities {
        world.entity_mut(entity).remove::<Dead>();
    }
}
