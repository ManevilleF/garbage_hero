use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

use super::GameController;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MoveAction>()
            .register_type::<MovementSpeed>()
            .register_type::<MovementDampingFactor>()
            .register_type::<ControllerGravity>()
            .add_plugins(InputManagerPlugin::<MoveAction>::default())
            .add_systems(
                Update,
                (apply_gravity, apply_movement, apply_movement_damping).chain(),
            );
    }
}

#[derive(Bundle)]
pub struct PlayerMovementBundle {
    pub speed: MovementSpeed,
    pub gravity: ControllerGravity,
    pub damping: MovementDampingFactor,
    pub movement: InputManagerBundle<MoveAction>,
}

impl PlayerMovementBundle {
    pub fn new(speed: f32, damping_factor: f32, controller: GameController) -> Self {
        Self {
            speed: MovementSpeed(speed),
            gravity: ControllerGravity(-9.81),
            damping: MovementDampingFactor(damping_factor),
            movement: InputManagerBundle::with_map(MoveAction::input_map(controller)),
        }
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct MovementSpeed(pub f32);

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct ControllerGravity(pub f32);

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct MovementDampingFactor(pub f32);

#[derive(Debug, Clone, Copy, Actionlike, PartialEq, Eq, Reflect, Hash)]
pub enum MoveAction {
    Directional(DirectionMove),
    Axis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash, EnumIter)]
pub enum DirectionMove {
    Left,
    Right,
    Up,
    Down,
}

impl DirectionMove {
    #[inline]
    pub const fn to_vec2(self) -> Vec2 {
        match self {
            Self::Up => Vec2::Y,
            Self::Down => Vec2::NEG_Y,
            Self::Right => Vec2::X,
            Self::Left => Vec2::NEG_X,
        }
    }
}

impl MoveAction {
    pub fn input_map(controller: GameController) -> InputMap<Self> {
        use DirectionMove::*;
        use MoveAction::*;

        let mut map = InputMap::default();
        match controller {
            GameController::Gamepad(gamepad) => {
                map.set_gamepad(gamepad)
                    .insert(Axis, DualAxis::left_stick())
                    .insert(Directional(Up), GamepadButtonType::DPadUp)
                    .insert(Directional(Down), GamepadButtonType::DPadDown)
                    .insert(Directional(Left), GamepadButtonType::DPadLeft)
                    .insert(Directional(Right), GamepadButtonType::DPadRight);
            }
            GameController::KeyBoard => {
                map.insert_one_to_many(Directional(Up), [KeyCode::ArrowUp, KeyCode::KeyW])
                    .insert_one_to_many(Directional(Down), [KeyCode::ArrowDown, KeyCode::KeyS])
                    .insert_one_to_many(Directional(Left), [KeyCode::ArrowLeft, KeyCode::KeyA])
                    .insert_one_to_many(Directional(Right), [KeyCode::ArrowRight, KeyCode::KeyD]);
            }
        }
        map
    }

    pub fn get_movement(state: &ActionState<Self>) -> Option<Vec2> {
        use MoveAction::*;

        if state.pressed(&Axis) {
            let dir = state.clamped_axis_pair(&Axis)?.xy();
            return Some(dir);
        }

        let axis: Vec2 = DirectionMove::iter()
            .filter(|&dir| state.pressed(&Directional(dir)))
            .map(DirectionMove::to_vec2)
            .sum();

        axis.try_normalize()
    }
}

/// Applies movement input to player controllers.
pub fn apply_movement(
    mut controllers: Query<(
        &mut LinearVelocity,
        &ActionState<MoveAction>,
        &MovementSpeed,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut velocity, action_state, speed) in &mut controllers {
        if let Some(dir) = MoveAction::get_movement(action_state) {
            velocity.x += dir.x * dt * speed.0;
            velocity.z += dir.y * dt * speed.0;
        }
    }
}

/// Applies [`ControllerGravity`] to player controllers.
fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&ControllerGravity, &mut LinearVelocity)>,
) {
    let delta_time = time.delta_seconds();
    for (gravity, mut velocity) in &mut controllers {
        velocity.0 += gravity.0 * delta_time;
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut velocity) in &mut query {
        velocity.x *= damping_factor.0;
        velocity.z *= damping_factor.0;
    }
}
