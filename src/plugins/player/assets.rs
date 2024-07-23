use std::f32::consts::PI;

use super::MAX_PLAYERS;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct PlayerVisualsBundle {
    pub scene: SceneBundle,
}

impl PlayerVisualsBundle {
    pub fn new(id: usize, assets: &PlayerAssets) -> Self {
        Self {
            scene: SceneBundle {
                scene: assets.scenes[id].clone_weak(),
                transform: Transform {
                    translation: Vec3::new(0.0, -1.5, 0.0),
                    scale: Vec3::splat(3.0),
                    rotation: Quat::from_rotation_y(PI),
                },
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct PlayerMarkerBundle {
    pub pbr: PbrBundle,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    pub colors: [Color; MAX_PLAYERS as usize],
    pub scenes: [Handle<Scene>; MAX_PLAYERS as usize],
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
        let server = world.resource::<AssetServer>();
        let characters = [
            "character-male-a",
            "character-female-a",
            "character-male-b",
            "character-female-b",
            "character-male-c",
            "character-female-c",
            "character-male-d",
            "character-female-d",
            "character-male-e",
            "character-female-e",
            "character-male-f",
            "character-female-f",
        ];
        let characters = characters.map(|name| {
            server.load(format!(
                "kenney_mini-characters/Models/glb/{name}.glb#Scene0"
            ))
        });
        let scenes = std::array::from_fn(|i| characters[i % characters.len()].clone());
        Self { colors, scenes }
    }
}
