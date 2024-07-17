use avian3d::prelude::*;
use bevy::prelude::*;
use movement::PlayerMovementBundle;
use std::fmt::Display;

mod actions;
mod movement;

use crate::ObjectLayer;

use super::common::Health;

const MAX_PLAYERS: u8 = 24;
const PLAYER_RADIUS: f32 = 0.5;
const PLAYER_HEIGHT: f32 = 1.8;
const BASE_PLAYER_HEALTH: u16 = 500;
const BASE_SENSOR_STRENGTH: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerAssets>()
            .register_type::<Player>();
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
pub enum GameController {
    KeyBoard,
    Gamepad(Gamepad),
}

impl Display for GameController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::KeyBoard => String::from("Keyboard"),
                Self::Gamepad(gamepad) => format!("Gamepad {}", gamepad.id),
            }
        )
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Player {
    pub id: u8,
    pub controller: GameController,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub name: Name,
    pub health: Health,
    // PBR
    pub pbr: PbrBundle,
    // Physics
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub layer: CollisionLayers,
    // Input
    pub movement: PlayerMovementBundle,
}

impl PlayerBundle {
    pub fn new(player: u8, controller: GameController, assets: &PlayerAssets) -> Self {
        if player >= MAX_PLAYERS {
            panic!("{MAX_PLAYERS} players supported");
        }
        Self {
            player: Player {
                id: player,
                controller,
            },
            name: Name::new(format!("Player {}: {controller}", player + 1,)),
            health: Health::new(BASE_PLAYER_HEALTH),
            pbr: PbrBundle {
                mesh: assets.mesh.clone_weak(),
                material: assets.materials[player as usize].clone_weak(),
                ..default()
            },
            rigidbody: RigidBody::Kinematic,
            collider: Collider::capsule(PLAYER_RADIUS, PLAYER_HEIGHT),
            layer: CollisionLayers::new(ObjectLayer::Player, LayerMask::ALL),
            movement: PlayerMovementBundle::new(30.0, 0.9, controller),
        }
    }
}

#[derive(Resource)]
pub struct PlayerAssets {
    pub colors: [Color; MAX_PLAYERS as usize],
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<StandardMaterial>; MAX_PLAYERS as usize],
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let colors = [
            Color::srgb_u8(255, 0, 0),     // #FF0000 - Red
            Color::srgb_u8(255, 255, 0),   // #FFFF00 - Yellow
            Color::srgb_u8(0, 234, 255),   // #00EAFF - Cyan
            Color::srgb_u8(170, 0, 255),   // #AA00FF - Purple
            Color::srgb_u8(255, 127, 0),   // #FF7F00 - Orange
            Color::srgb_u8(191, 255, 0),   // #BFFF00 - Lime
            Color::srgb_u8(0, 149, 255),   // #0095FF - Sky Blue
            Color::srgb_u8(255, 0, 170),   // #FF00AA - Magenta
            Color::srgb_u8(255, 212, 0),   // #FFD400 - Gold
            Color::srgb_u8(106, 255, 0),   // #6AFF00 - Green
            Color::srgb_u8(0, 64, 255),    // #0040FF - Blue
            Color::srgb_u8(237, 185, 185), // #EDB9B9 - Light Pink
            Color::srgb_u8(185, 215, 237), // #B9D7ED - Light Blue
            Color::srgb_u8(231, 233, 185), // #E7E9B9 - Light Yellow
            Color::srgb_u8(220, 185, 237), // #DCB9ED - Light Purple
            Color::srgb_u8(185, 237, 224), // #B9EDE0 - Light Cyan
            Color::srgb_u8(143, 35, 35),   // #8F2323 - Dark Red
            Color::srgb_u8(35, 98, 143),   // #23628F - Dark Blue
            Color::srgb_u8(143, 106, 35),  // #8F6A23 - Dark Yellow
            Color::srgb_u8(107, 35, 143),  // #6B238F - Dark Purple
            Color::srgb_u8(79, 143, 35),   // #4F8F23 - Dark Green
            Color::srgb_u8(0, 0, 0),       // #000000 - Black
            Color::srgb_u8(115, 115, 115), // #737373 - Gray
            Color::srgb_u8(204, 204, 204), // #CCCCCC - Light Gray
        ];
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let materials = colors.map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                fog_enabled: false,
                ..default()
            })
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Capsule3d::new(PLAYER_RADIUS, PLAYER_HEIGHT));
        Self {
            colors,
            materials,
            mesh,
        }
    }
}
