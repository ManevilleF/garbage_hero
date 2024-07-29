use avian3d::prelude::{ExternalImpulse, LinearVelocity};
use bevy::{log, prelude::*, utils::HashMap};
use leafwing_input_manager::action_state::ActionState;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{
    plugins::{
        camera::CameraParams,
        garbage::{Collector, CollectorConfig, DistributionShape},
    },
    Dead, GameState,
};

use super::{input::PlayerInput, GameController, Player};

pub struct PlayerSkillsPlugin;

impl Plugin for PlayerSkillsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerAim>()
            .register_type::<SkillState>()
            .register_type::<ActiveSkill>()
            .add_systems(
                Update,
                (
                    (update_aim, apply_aim).chain(),
                    (update_skills, (collector_skills, throw_skill, dash_skill)).chain(),
                )
                    .run_if(in_state(GameState::Running)),
            );
        #[cfg(feature = "debug")]
        app.add_systems(
            PostUpdate,
            draw_gizmos
                .after(avian3d::prelude::PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash, EnumIter, Display)]
pub enum PlayerSkill {
    Collect,
    Shoot,
    Dash,
    Defend,
}

impl PlayerSkill {
    pub const fn cooldown(self) -> f32 {
        match self {
            Self::Collect => 0.0,
            Self::Shoot => 0.05,
            Self::Dash => 0.5,
            Self::Defend => 0.0,
        }
    }
}

#[derive(Debug, Reflect, Component, Clone)]
#[reflect(Component)]
pub struct SkillState {
    pub cooldowns: HashMap<PlayerSkill, f32>,
}

#[derive(Debug, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct ActiveSkill {
    pub active: Option<PlayerSkill>,
}

impl Default for SkillState {
    fn default() -> Self {
        Self {
            cooldowns: PlayerSkill::iter().map(|s| (s, 0.0)).collect(),
        }
    }
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
    pub state: SkillState,
    pub active: ActiveSkill,
}

impl PlayerSkillsBundle {
    pub fn new() -> Self {
        Self {
            aim: PlayerAim::new(),
            state: SkillState::default(),
            active: ActiveSkill::default(),
        }
    }
}

fn update_aim(
    mut gizmos: Gizmos,
    mut players: Query<
        (
            &mut PlayerAim,
            &Player,
            &GlobalTransform,
            &ActionState<PlayerInput>,
        ),
        Without<Dead>,
    >,
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
            GameController::Gamepad { .. } => {
                let Some(dir) = action_state
                    .clamped_axis_pair(&PlayerInput::Aim)
                    .map(Vec2::from)
                else {
                    continue;
                };
                let direction = Dir2::new(dir * Vec2::new(1.0, -1.0)).unwrap_or(Dir2::Y);
                let mut dir = aim.map_unchanged(|aim| &mut aim.dir);
                dir.set_if_neq(direction);
            }
        }
    }
}

fn update_skills(
    time: Res<Time>,
    mut players: Query<(
        &mut SkillState,
        &mut ActiveSkill,
        &ActionState<PlayerInput>,
        Has<Dead>,
    )>,
) {
    let dt = time.delta_seconds();
    for (mut state, mut active, input, dead) in &mut players {
        state
            .cooldowns
            .values_mut()
            .for_each(|cooldown| *cooldown = (*cooldown - dt).max(0.0));

        if dead {
            active.active = None;
            continue;
        }
        if let Some(skill) = active.active {
            if !input.pressed(&PlayerInput::Skill(skill)) {
                active.active = None;
                state.cooldowns.insert(skill, skill.cooldown());
            } else {
                continue;
            }
        }
        for (skill, cooldown) in &state.cooldowns {
            if *cooldown <= 0.0 && input.pressed(&PlayerInput::Skill(*skill)) {
                active.active = Some(*skill);
                break;
            }
        }
    }
}

fn collector_skills(
    players: Query<(&Children, &ActiveSkill), (With<Player>, Changed<ActiveSkill>)>,
    mut collectors: Query<(&mut Collector, &mut CollectorConfig)>,
) {
    for (children, active) in &players {
        let mut collectors = collectors.iter_many_mut(children);
        while let Some((mut collector, mut config)) = collectors.fetch_next() {
            let shape = if active.active == Some(PlayerSkill::Defend) {
                DistributionShape::Arc
            } else {
                DistributionShape::Circle
            };
            if collector.shape() != shape {
                collector.set_shape(shape);
            }
            let enabled = active.active == Some(PlayerSkill::Collect);
            if config.enabled != enabled {
                config.enabled = enabled;
            }
        }
    }
}

fn throw_skill(
    mut commands: Commands,
    players: Query<(&Player, &Children, &ActiveSkill, &PlayerAim), Changed<ActiveSkill>>,
    collectors: Query<&Collector>,
) {
    for (player, children, active, aim) in &players {
        if active.active != Some(PlayerSkill::Shoot) {
            continue;
        }
        for collector in collectors.iter_many(children) {
            if let Some(command) = collector.throw_collected(aim.direction2(), 70.0) {
                commands.add(command);
            } else {
                log::info!("Player {}, Nothing to shoot", player.id);
            }
        }
    }
}

fn dash_skill(
    mut commands: Commands,
    players: Query<(Entity, &PlayerAim, &ActiveSkill, &LinearVelocity), Changed<ActiveSkill>>,
) {
    const DASH_SPEED: f32 = 500.0;

    for (entity, aim, skill, linvel) in &players {
        if skill.active != Some(PlayerSkill::Dash) {
            continue;
        }
        let direction = (linvel.length_squared() > 1.0)
            .then(|| Vec3::new(linvel.x, 0.0, linvel.z).normalize())
            .unwrap_or(*aim.direction3());
        commands
            .entity(entity)
            .insert(ExternalImpulse::new(direction * DASH_SPEED));
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
