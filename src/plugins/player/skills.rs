use bevy::{log, prelude::*};
use leafwing_input_manager::action_state::ActionState;
use strum::EnumIter;

use crate::plugins::camera::CameraParams;

use super::{input::PlayerInputAction, GameController, Player};

pub struct PlayerSkillsPlugin;

impl Plugin for PlayerSkillsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerAim>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash, EnumIter)]
pub enum PlayerSkill {
    Collect,
    Shoot,
    Dash,
    Defend,
    Burst,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct PlayerAim(Dir2);

#[derive(Bundle)]
pub struct PlayerSkillsBundle {
    pub aim: PlayerAim,
}

impl PlayerSkillsBundle {
    pub fn new() -> Self {
        Self {
            aim: PlayerAim(Dir2::Y),
        }
    }
}

fn update_aim(
    mut players: Query<(
        &mut PlayerAim,
        &Player,
        &GlobalTransform,
        &ActionState<PlayerInputAction>,
    )>,
    camera: CameraParams,
) {
    for (mut aim, player, gtr, action_state) in &mut players {
        match player.controller {
            GameController::KeyBoard => {
                let Some(ray) = camera.mouse_ray() else {
                    continue;
                };
                let player_pos = gtr.translation();
                let Some(dist) = ray.intersect_plane(player_pos, InfinitePlane3d::new(Dir3::Y))
                else {
                    log::error!(
                        "Failed to compute camera ray intersection for player {}",
                        player.id
                    );
                    continue;
                };
                let target = ray.origin + ray.direction * dist;
                let Ok(direction) = Dir2::new(target.xz() - player_pos.xz()) else {
                    log::error!(
                        "Failed to normalize direction between camera ray and player {}",
                        player.id
                    );
                    continue;
                };
                aim.set_if_neq(PlayerAim(direction));
            }
            GameController::Gamepad(_) => {
                let Some(dir) = action_state
                    .clamped_axis_pair(&PlayerInputAction::Aim)
                    .map(Vec2::from)
                else {
                    continue;
                };
                let Ok(direction) = Dir2::new(dir) else {
                    log::error!(
                        "Failed to normalize aim direction {dir:?} for player {}",
                        player.id
                    );
                    continue;
                };
                aim.set_if_neq(PlayerAim(direction));
            }
        }
    }
}
