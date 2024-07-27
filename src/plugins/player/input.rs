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
    pub input: InputManagerBundle<PlayerInput>,
    pub icons: InputMapIcons,
}

impl PlayerInputBundle {
    pub fn new(controller: GameController, server: &AssetServer) -> Self {
        let map = PlayerInput::input_map(controller);
        let icons = InputMapIcons::new(&map, &controller, server);
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

        let mut map = InputMap::default();
        match controller {
            GameController::Gamepad(gamepad) => {
                map.set_gamepad(gamepad)
                    .insert(Pause, GamepadButtonType::Start)
                    .insert(Move, DualAxis::left_stick())
                    .insert(Move, VirtualDPad::dpad())
                    .insert(Aim, DualAxis::right_stick())
                    .insert(Skill(Collect), GamepadButtonType::South)
                    .insert(Skill(Shoot), GamepadButtonType::RightTrigger2)
                    .insert(Skill(Defend), GamepadButtonType::LeftTrigger2)
                    .insert(Skill(Dash), GamepadButtonType::East)
                    .insert(Skill(Burst), GamepadButtonType::West);
            }
            GameController::KeyBoard => {
                map.insert(Pause, KeyCode::Escape)
                    .insert(Move, VirtualDPad::arrow_keys())
                    .insert(Move, VirtualDPad::wasd())
                    .insert(Aim, DualAxis::mouse_motion())
                    .insert_one_to_many(Skill(Collect), [KeyCode::ShiftLeft, KeyCode::ShiftRight])
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
        let controller = GameController::Gamepad(event.gamepad);
        match &event.connection {
            GamepadConnection::Connected(info) => {
                log::info!("New controller detected: {controller}: {info:?}");
                if !players.contains_key(&controller) {
                    player_connected_evw.send(PlayerConnected(Player {
                        controller,
                        id: new_player_id(),
                    }));
                }
            }
            GamepadConnection::Disconnected => {
                if let Some(player) = players.get(&controller) {
                    log::info!("Player {player} disconnected");
                    // TODO: Handle disconnected player
                }
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
