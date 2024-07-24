use std::f32::consts::PI;

use super::{skills::PlayerAim, MAX_PLAYERS};
use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_mod_outline::{AsyncSceneInheritOutline, OutlineBundle, OutlineVolume};

pub struct PlayerVisualsPlugin;

impl Plugin for PlayerVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerAssets>()
            .register_type::<PlayerAssets>()
            .register_type::<PlayerAimMarker>()
            .add_systems(PostUpdate, update_marker);
    }
}

#[derive(Bundle)]
pub struct PlayerVisualsBundle {
    pub scene: SceneBundle,
    pub outline: OutlineBundle,
    pub outline_inherit: AsyncSceneInheritOutline,
}

impl PlayerVisualsBundle {
    pub fn new(id: usize, assets: &PlayerAssets) -> Self {
        let color = assets.colors[id];
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
            outline: OutlineBundle {
                outline: OutlineVolume {
                    visible: true,
                    width: 3.0,
                    colour: color,
                },
                ..default()
            },
            outline_inherit: AsyncSceneInheritOutline,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAimMarker(Entity);

#[derive(Bundle)]
pub struct PlayerAimMarkerBundle {
    pub pbr: PbrBundle,
    pub marker: PlayerAimMarker,
    pub name: Name,
    pub no_shadow_caster: NotShadowCaster,
    pub no_shadow_receiver: NotShadowReceiver,
}

impl PlayerAimMarkerBundle {
    pub fn new(id: usize, player_entity: Entity, assets: &PlayerAssets) -> Self {
        Self {
            pbr: PbrBundle {
                transform: Transform::from_xyz(0.0, 0.05, 0.0),
                mesh: assets.marker_mesh.clone_weak(),
                material: assets.marker_mats[id].clone_weak(),
                ..default()
            },
            marker: PlayerAimMarker(player_entity),
            name: Name::new(format!("Player {id} aim marker")),
            no_shadow_caster: NotShadowCaster,
            no_shadow_receiver: NotShadowReceiver,
        }
    }
}

fn update_marker(
    players: Query<(&GlobalTransform, &PlayerAim)>,
    mut markers: Query<(&mut Transform, &PlayerAimMarker)>,
) {
    for (mut tr, PlayerAimMarker(player)) in &mut markers {
        let Ok((gtr, aim)) = players.get(*player) else {
            continue;
        };
        let target = gtr.translation();
        tr.translation.x = target.x;
        tr.translation.z = target.z;
        tr.look_to(aim.direction3(), Dir3::Y);
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    pub colors: [Color; MAX_PLAYERS as usize],
    pub scenes: [Handle<Scene>; MAX_PLAYERS as usize],
    pub marker_mats: [Handle<StandardMaterial>; MAX_PLAYERS as usize],
    pub marker_mesh: Handle<Mesh>,
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
        let marker_mats = colors.map(|c| {
            materials.add(StandardMaterial {
                base_color: c,
                unlit: true,
                fog_enabled: false,
                ..default()
            })
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let marker_mesh = meshes.add(Triangle3d::new(
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::new(-0.5, 0.0, -2.0),
            Vec3::new(0.5, 0.0, -2.0),
        ));
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
        Self {
            colors,
            scenes,
            marker_mats,
            marker_mesh,
        }
    }
}
