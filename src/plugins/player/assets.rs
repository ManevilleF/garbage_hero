use std::{f32::consts::PI, time::Duration};

use crate::Dead;

use super::{skills::PlayerAim, Player, MAX_PLAYERS};
use avian3d::prelude::LinearVelocity;
use bevy::{
    animation::RepeatAnimation,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

pub struct PlayerVisualsPlugin;

impl Plugin for PlayerVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerAssets>()
            .register_type::<PlayerAssets>()
            .register_type::<PlayerAimMarker>()
            .register_type::<CharacterAnimations>()
            .register_type::<RootPlayer>()
            .add_systems(Update, (setup_animations, player_animations))
            .add_systems(
                PostUpdate,
                update_marker
                    .after(avian3d::prelude::PhysicsSet::Sync)
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

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

#[derive(Component, Debug, Reflect)]
pub struct RootPlayer(Entity);

fn setup_animations(
    mut commands: Commands,
    assets: Res<PlayerAssets>,
    players: Query<(Entity, &Player)>,
    ancestors: Query<&Parent>,
    animations: Query<Entity, Added<AnimationPlayer>>,
) {
    for entity in &animations {
        let ancestors = ancestors.iter_ancestors(entity);
        let Some((root, player)) = players.iter_many(ancestors).next() else {
            continue;
        };
        commands.entity(entity).insert((
            assets.animation_graphs[player.id as usize].clone_weak(),
            assets.animations[player.id as usize].clone(),
            RootPlayer(root),
        ));
    }
}

fn player_animations(
    players: Query<(Has<Dead>, &LinearVelocity), With<Player>>,
    mut animations: Query<(&mut AnimationPlayer, &CharacterAnimations, &RootPlayer)>,
) {
    for (mut anim_player, animations, root) in &mut animations {
        let (is_dead, linvel) = players.get(root.0).unwrap();
        if is_dead {
            anim_player.stop(animations.idle);
            anim_player.stop(animations.running);
            anim_player.play(animations.death);
        } else if linvel.length_squared() > 1.0 {
            anim_player
                .animation_mut(animations.idle)
                .map(|a| a.rewind().set_repeat(RepeatAnimation::Never));
            anim_player.play(animations.running).repeat();
        } else {
            anim_player
                .animation_mut(animations.running)
                .map(|a| a.rewind().set_repeat(RepeatAnimation::Never));
            anim_player.play(animations.idle).repeat();
        };
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
                transform: Transform::from_xyz(0.0, 0.55, 0.0),
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

#[derive(Debug, Component, Reflect, Clone)]
pub struct CharacterAnimations {
    pub idle: AnimationNodeIndex,
    pub running: AnimationNodeIndex,
    pub death: AnimationNodeIndex,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    pub colors: [Color; MAX_PLAYERS as usize],
    pub scenes: [Handle<Scene>; MAX_PLAYERS as usize],
    pub animation_graphs: [Handle<AnimationGraph>; MAX_PLAYERS as usize],
    pub animations: [CharacterAnimations; MAX_PLAYERS as usize],
    pub marker_mats: [Handle<StandardMaterial>; MAX_PLAYERS as usize],
    pub marker_mesh: Handle<Mesh>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let colors = [
            Color::srgb_u8(255, 0, 0),     // #FF0000 - Red
            Color::srgb_u8(0, 234, 255),   // #00EAFF - Cyan
            Color::srgb_u8(255, 127, 0),   // #FF7F00 - Orange
            Color::srgb_u8(170, 0, 255),   // #AA00FF - Purple
            Color::srgb_u8(191, 255, 0),   // #BFFF00 - Lime
            Color::srgb_u8(0, 149, 255),   // #0095FF - Sky Blue
            Color::srgb_u8(255, 0, 170),   // #FF00AA - Magenta
            Color::srgb_u8(106, 255, 0),   // #6AFF00 - Green
            Color::srgb_u8(0, 64, 255),    // #0040FF - Blue
            Color::srgb_u8(255, 255, 255), // #FFFFFF - White
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
            "kenney_mini-characters/Models/glb/character-male-e.glb",
            "kenney_mini-characters/Models/glb/character-female-e.glb",
            "kenney_mini-characters/Models/glb/character-male-b.glb",
            "kenney_mini-characters/Models/glb/character-female-b.glb",
            "kenney_mini-characters/Models/glb/character-male-c.glb",
            "kenney_mini-characters/Models/glb/character-female-c.glb",
            "kenney_mini-characters/Models/glb/character-male-d.glb",
            "kenney_mini-characters/Models/glb/character-female-d.glb",
            "kenney_mini-characters/Models/glb/character-male-f.glb",
            "kenney_mini-characters/Models/glb/character-female-f.glb",
        ];
        let scenes = characters.map(|path| server.load(format!("{path}#Scene0")));
        let mut animation_graphs: [_; MAX_PLAYERS as usize] =
            std::array::from_fn(|_| AnimationGraph::new());
        let animations = std::array::from_fn(|i| {
            let path = characters[i];
            let graph = &mut animation_graphs[i];
            let idle = graph.add_clip(
                server.load(GltfAssetLabel::Animation(1).from_asset(path)),
                1.0,
                graph.root,
            );
            let running = graph.add_clip(
                server.load(GltfAssetLabel::Animation(3).from_asset(path)),
                1.0,
                graph.root,
            );
            let death = graph.add_clip(
                server.load(GltfAssetLabel::Animation(9).from_asset(path)),
                1.0,
                graph.root,
            );

            CharacterAnimations {
                idle,
                running,
                death,
            }
        });
        let mut graphs = world.resource_mut::<Assets<AnimationGraph>>();
        let animation_graphs = animation_graphs.map(|graph| graphs.add(graph));
        Self {
            colors,
            scenes,
            animations,
            animation_graphs,
            marker_mats,
            marker_mesh,
        }
    }
}
