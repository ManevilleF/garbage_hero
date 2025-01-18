use super::{skills::PlayerSkill, Player, PlayerConnected};
use crate::{plugins::ui::input_icons::InputMapIcons, PauseGame};
use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent},
        keyboard::KeyboardInput,
    },
    log,
    prelude::*,
    utils::HashMap,
};
use leafwing_input_manager::prelude::*;
use std::fmt::Display;
use strum::Display;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerInput>()
            .add_plugins(InputManagerPlugin::<PlayerInput>::default())
            .add_systems(PostUpdate, (handle_new_controllers, pause_game));
    }
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash)]
pub enum GameController {
    KeyBoard,
    Gamepad {
        gamepad: Entity,
        category: GamepadCategory,
    },
}

#[derive(Debug, Clone, Copy, Reflect, Default, Display, PartialEq, Eq, Hash)]
pub enum GamepadCategory {
    Xbox,
    PlayStation,
    Steam,
    #[default]
    Unknown,
}

impl GamepadCategory {
    pub fn from_name(name: &str) -> Self {
        let name = name.to_lowercase();
        if name.contains("xbox") {
            Self::Xbox
        } else if name.contains("dualshock") || name.contains("ps") || name.contains("playstation")
        {
            Self::PlayStation
        } else if name.contains("steam") {
            Self::Steam
        } else {
            Self::Unknown
        }
    }
}

impl Display for GameController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::KeyBoard => String::from("Keyboard"),
                Self::Gamepad { category, .. } => format!("{category} Gamepad"),
            }
        )
    }
}
#[derive(Bundle)]
pub struct PlayerInputBundle {
    pub input: InputManagerBundle<PlayerInput>,
    pub icons: InputMapIcons,
}

impl PlayerInputBundle {
    pub fn new(controller: GameController, server: &AssetServer) -> Self {
        let (map, icons) = PlayerInput::input_map(controller);
        Self {
            input: InputManagerBundle::with_map(map),
            icons,
        }
    }
}

#[derive(Debug, Clone, Copy, Actionlike, PartialEq, Eq, Reflect, Hash)]
#[non_exhaustive]
pub enum PlayerInput {
    Move,
    Aim,
    Pause,
    Skill(PlayerSkill),
}

impl Display for PlayerInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Move => "Move".into(),
                Self::Aim => "Aim".into(),
                Self::Pause => "Pause".into(),
                Self::Skill(skill) => skill.to_string(),
            }
        )
    }
}

impl PlayerInput {
    pub fn input_map(controller: GameController) -> InputMap<Self> {
        use PlayerInput::*;
        use PlayerSkill::*;

        match controller {
            GameController::Gamepad { gamepad, .. } => {
                let map = InputMap::default()
                    .with_gamepad(gamepad)
                    .with(Pause, GamepadButton::Start)
                    .with_dual_axis(Move, GamepadStick::LEFT)
                    .with_dual_axis(Move, VirtualDPad::dpad())
                    .with_dual_axis(Aim, GamepadStick::RIGHT)
                    .with(Skill(Collect), GamepadButton::South)
                    .with(Skill(Shoot), GamepadButton::RightTrigger2)
                    .with(Skill(Defend), GamepadButton::LeftTrigger2)
                    .with(Skill(Dash), GamepadButton::East);
                map
            }
            GameController::KeyBoard => {
                let map = InputMap::default()
                    .with(Pause, KeyCode::Escape)
                    .with_dual_axis(Move, VirtualDPad::arrow_keys())
                    .with_dual_axis(Move, VirtualDPad::wasd())
                    .with_dual_axis(Aim, MouseMove::default())
                    .with_one_to_many(Skill(Collect), [KeyCode::ShiftLeft, KeyCode::ShiftRight])
                    .with(Skill(Shoot), MouseButton::Left)
                    .with(Skill(Defend), MouseButton::Right)
                    .with(Skill(Dash), KeyCode::Space);
                map
            }
        }
    }

    pub fn get_movement(state: &ActionState<Self>) -> Option<Vec2> {
        if state.pressed(&Self::Move) {
            let dir = state.clamped_axis_pair(&Self::Move).try_normalize()?;
            return Some(dir);
        }
        None
    }
}

pub fn handle_new_controllers(
    mut gamepad_evr: EventReader<GamepadConnectionEvent>,
    mut keyboard_evr: EventReader<KeyboardInput>,
    players: Query<&Player>,
    mut player_connected_evw: EventWriter<PlayerConnected>,
) {
    let players: HashMap<GameController, u8> =
        players.iter().map(|p| (p.controller, p.id)).collect();
    let new_player_id = || players.values().max().copied().map(|v| v + 1).unwrap_or(0);
    for event in gamepad_evr.read() {
        match &event.connection {
            GamepadConnection::Connected { name, .. } => {
                let category = GamepadCategory::from_name(name);
                let controller = GameController::Gamepad {
                    gamepad: event.gamepad,
                    category,
                };
                log::info!("New controller detected: {controller}");
                if !players.contains_key(&controller) {
                    player_connected_evw.send(PlayerConnected(Player {
                        controller,
                        id: new_player_id(),
                    }));
                }
            }
            GamepadConnection::Disconnected => {
                log::info!("A player disconnected");
                // TODO: Handle disconnected player
            }
        }
    }
    if players.get(&GameController::KeyBoard).is_none() && !keyboard_evr.is_empty() {
        log::info!("Keyboard controller detected");
        player_connected_evw.send(PlayerConnected(Player {
            controller: GameController::KeyBoard,
            id: new_player_id(),
        }));
    }
    keyboard_evr.clear();
}

fn pause_game(
    players: Query<(&Player, &ActionState<PlayerInput>)>,
    mut pause_evw: EventWriter<PauseGame>,
) {
    for (player, state) in &players {
        if state.just_pressed(&PlayerInput::Pause) {
            log::info!("Pause triggered by player {}", player.id);
            pause_evw.send_default();
        }
    }
}
