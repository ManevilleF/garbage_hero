use super::{MAX_PLAYERS, PLAYER_HEIGHT, PLAYER_RADIUS};
use bevy::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
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
