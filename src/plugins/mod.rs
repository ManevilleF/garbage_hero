mod camera;
mod common;
#[cfg(feature = "debug")]
mod debug;
mod enemies;
mod garbage;
mod light;
mod map;
mod particles;
mod player;
mod splash;
mod ui;

pub use camera::CameraPlugin;
pub use common::*;
#[cfg(feature = "debug")]
pub use debug::DebugPlugin;
pub use enemies::{spawn_enemies, EnemiesPlugin, Enemy};
pub use garbage::{spawn_builds, spawn_some_garbage, GarbageItem, GarbagePlugin};
pub use light::LightPlugin;
pub use map::{spawn_game_starters, MapPlugin};
pub use particles::{ParticleConfig, ParticlesPlugin};
pub use player::{reset_players, Player, PlayerPlugin};
#[cfg(not(feature = "debug"))]
pub use splash::SplashScreenPlugin;

pub mod utils {
    use bevy::render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    };

    // Creates a colorful test pattern
    pub fn uv_debug_texture() -> Image {
        const TEXTURE_SIZE: usize = 8;

        let mut palette: [u8; 32] = [
            255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102,
            255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
        ];

        let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
        for y in 0..TEXTURE_SIZE {
            let offset = TEXTURE_SIZE * y * 4;
            texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
            palette.rotate_right(4);
        }

        Image::new_fill(
            Extent3d {
                width: TEXTURE_SIZE as u32,
                height: TEXTURE_SIZE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}
