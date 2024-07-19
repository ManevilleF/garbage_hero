use super::{common::Health, garbage::CollectorBundle};
use bevy::prelude::*;

mod assets;
mod input;
mod movement;
mod skills;

use assets::PlayerAssets;
pub use input::GameController;
use input::{PlayerInputBundle, PlayerInputPlugin};
use movement::{PlayerMovementBundle, PlayerMovementPlugin};
use skills::{PlayerSkillsBundle, PlayerSkillsPlugin};

const MAX_PLAYERS: u8 = 24;
const PLAYER_RADIUS: f32 = 0.5;
const PLAYER_HEIGHT: f32 = 1.8;
const BASE_PLAYER_HEALTH: u16 = 500;
const BASE_SENSOR_STRENGTH: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerInputPlugin, PlayerMovementPlugin, PlayerSkillsPlugin))
            .init_resource::<PlayerAssets>()
            .register_type::<PlayerAssets>()
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

impl Player {
    pub fn spawn(self, position: Vec3) -> impl FnOnce(&mut World) {
        move |world| {
            let assets = world.resource::<PlayerAssets>();
            let mut bundle = PlayerBundle::new(self, assets);
            bundle.pbr.transform.translation = position;
            world.spawn(bundle).with_children(|b| {
                let collector_bundle = CollectorBundle::new(3.0, 1.5);
                b.spawn(collector_bundle);
            });
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub name: Name,
    pub health: Health,
    pub input: PlayerInputBundle,
    pub movement: PlayerMovementBundle,
    pub skills: PlayerSkillsBundle,
    pub pbr: PbrBundle,
}

impl PlayerBundle {
    pub fn new(player: Player, assets: &PlayerAssets) -> Self {
        if player.id >= MAX_PLAYERS {
            panic!("{MAX_PLAYERS} players supported");
        }
        Self {
            name: Name::new(format!("Player {}: {}", player.id, player.controller)),
            health: Health::new(BASE_PLAYER_HEALTH),
            input: PlayerInputBundle::new(player.controller),
            movement: PlayerMovementBundle::new(30.0, 0.9),
            skills: PlayerSkillsBundle::new(),
            pbr: PbrBundle {
                mesh: assets.mesh.clone_weak(),
                material: assets.materials[player.id as usize].clone_weak(),
                ..default()
            },
            player,
        }
    }
}

pub fn spawn_players(mut connected_evr: EventReader<PlayerConnected>, mut commands: Commands) {
    for event in connected_evr.read() {
        commands.add(event.0.spawn(Vec3::Y * 3.0));
    }
}
