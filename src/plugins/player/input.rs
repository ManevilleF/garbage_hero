use super::skills::PlayerSkill;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use std::fmt::Display;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerInputAction>()
            .add_plugins(InputManagerPlugin::<PlayerInputAction>::default());
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
#[derive(Bundle)]
pub struct PlayerInputBundle {
    pub input: InputManagerBundle<PlayerInputAction>,
}

impl PlayerInputBundle {
    pub fn new(controller: GameController) -> Self {
        Self {
            input: InputManagerBundle::with_map(PlayerInputAction::input_map(controller)),
        }
    }
}

#[derive(Debug, Clone, Copy, Actionlike, PartialEq, Eq, Reflect, Hash)]
pub enum PlayerInputAction {
    Move,
    Aim,
    Skill(PlayerSkill),
}

impl PlayerInputAction {
    pub fn input_map(controller: GameController) -> InputMap<Self> {
        use PlayerInputAction::*;
        use PlayerSkill::*;

        let mut map = InputMap::default();
        match controller {
            GameController::Gamepad(gamepad) => {
                map.set_gamepad(gamepad)
                    .insert(Move, DualAxis::left_stick())
                    .insert(Move, VirtualDPad::dpad())
                    .insert(Skill(Collect), GamepadButtonType::South)
                    .insert(Skill(Shoot), GamepadButtonType::RightTrigger)
                    .insert(Skill(Defend), GamepadButtonType::LeftTrigger)
                    .insert(Skill(Dash), GamepadButtonType::East)
                    .insert(Skill(Burst), GamepadButtonType::West);
            }
            GameController::KeyBoard => {
                map.insert(Move, VirtualDPad::arrow_keys())
                    .insert(Move, VirtualDPad::wasd())
                    .insert(Skill(Collect), KeyCode::KeyE)
                    .insert(Skill(Shoot), MouseButton::Left)
                    .insert(Skill(Defend), MouseButton::Right)
                    .insert(Skill(Dash), KeyCode::Space)
                    .insert(Skill(Burst), KeyCode::KeyQ);
            }
        }
        map
    }

    pub fn get_movement(state: &ActionState<Self>) -> Option<Vec2> {
        if state.pressed(&Self::Move) {
            let dir = state.clamped_axis_pair(&Self::Move)?.xy().try_normalize()?;
            return Some(dir);
        }
        None
    }
}
