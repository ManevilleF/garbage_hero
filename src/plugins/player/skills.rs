use bevy::{log, prelude::*};
use leafwing_input_manager::action_state::ActionState;
use strum::EnumIter;

use crate::plugins::camera::CameraParams;

use super::{assets::PlayerAssets, input::PlayerInputAction, GameController, Player};

pub struct PlayerSkillsPlugin;

impl Plugin for PlayerSkillsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerAim>()
            .add_systems(Update, (update_aim, apply_aim).chain());
        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, draw_gizmos);
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

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAim {
    pub dir: Dir2,
    pub max_rotation_speed: f32,
}

impl PlayerAim {
    pub const fn new() -> Self {
        Self {
            dir: Dir2::Y,
            max_rotation_speed: 5.0,
        }
    }
    pub fn direction3(&self) -> Dir3 {
        Dir3::new_unchecked(Vec3::new(self.dir.x, 0.0, self.dir.y))
    }

    pub const fn direction2(&self) -> Dir2 {
        self.dir
    }
}

#[derive(Bundle)]
pub struct PlayerSkillsBundle {
    pub aim: PlayerAim,
}

impl PlayerSkillsBundle {
    pub fn new() -> Self {
        Self {
            aim: PlayerAim::new(),
        }
    }
}

fn update_aim(
    mut gizmos: Gizmos,
    mut players: Query<(
        &mut PlayerAim,
        &Player,
        &GlobalTransform,
        &ActionState<PlayerInputAction>,
    )>,
    camera: CameraParams,
) {
    for (aim, player, gtr, action_state) in &mut players {
        match player.controller {
            GameController::KeyBoard => {
                let Some(ray) = camera.mouse_ray() else {
                    continue;
                };
                let player_pos = gtr.translation();
                let Some(dist) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Y))
                else {
                    log::error!(
                        "Failed to compute camera ray intersection for player {}",
                        player.id
                    );
                    continue;
                };
                let target = ray.origin + ray.direction * dist;
                gizmos.sphere(target, Quat::default(), 0.1, Color::BLACK);
                let Ok(direction) = Dir2::new(target.xz() - player_pos.xz()) else {
                    log::error!(
                        "Failed to normalize direction between camera ray and player {}",
                        player.id
                    );
                    continue;
                };
                let mut dir = aim.map_unchanged(|aim| &mut aim.dir);
                dir.set_if_neq(direction);
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
                let mut dir = aim.map_unchanged(|aim| &mut aim.dir);
                dir.set_if_neq(direction);
            }
        }
    }
}

fn apply_aim(time: Res<Time>, mut players: Query<(&mut Transform, &PlayerAim)>) {
    let dt = time.delta_seconds();
    for (mut tr, aim) in &mut players {
        let current = tr.forward().xz();
        let target: Vec2 = *aim.direction2();
        let target_angle = target.angle_between(current);
        let max_step = aim.max_rotation_speed * dt;
        let angle = target_angle.clamp(-max_step, max_step);
        tr.rotate_axis(Dir3::Y, angle);
    }
}

fn draw_gizmos(mut gizmos: Gizmos, players: Query<(&PlayerAim, &GlobalTransform)>) {
    use bevy::color::palettes::css::GRAY;
    for (aim, gtr) in &players {
        let position = gtr.translation();
        let forward = gtr.forward();
        let color = Color::Srgba(GRAY);
        gizmos.arrow(position, position + *aim.direction3() * 3.0, color);
        gizmos.ray(position, *forward * 2.0, color);
        gizmos.short_arc_3d_between(
            position,
            position + *forward * 2.0,
            position + *aim.direction3() * 2.0,
            color,
        );
    }
}
