use super::{Player, PlayerConnected, skills::PlayerSkill};
use crate::{PauseGame, plugins::ui::input_icons::InputMapIcons};
use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent},
        keyboard::KeyboardInput,
    },
    log,
    platform::collections::HashMap,
    prelude::*,
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

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash, Component)]
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
                Self::Gamepad { gamepad, category } => format!("{category} Gamepad {:?}", gamepad),
            }
        )
    }
}
#[derive(Bundle)]
pub struct PlayerInputBundle {
    pub controller: GameController,
    pub inputs: PlayerInputs,
    pub map: InputMap<PlayerInput>,
    pub icons: InputMapIcons,
}

impl PlayerInputBundle {
    pub fn new(controller: GameController, server: &AssetServer) -> Self {
        let inputs = PlayerInputs::new(controller);
        let map = PlayerInput::input_map(controller, &inputs);
        let icons = InputMapIcons::new(&inputs, &controller, server);
        Self {
            controller,
            inputs,
            map,
            icons,
        }
    }
}

#[derive(Debug, Clone, Copy, Actionlike, PartialEq, Eq, Reflect, Hash)]
#[non_exhaustive]
pub enum PlayerInput {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    Aim,
    Pause,
    Skill(PlayerSkill),
}

#[derive(Debug, Clone, Component)]
pub struct PlayerInputs(pub(crate) HashMap<PlayerInput, Binding>);

#[derive(Debug, Clone)]
pub enum Binding {
    KeyCode(KeyCode),
    Keys(Vec<KeyCode>),
    GamepadButton(GamepadButton),
    MouseMove,
    MouseButton(MouseButton),
    LeftSick,
    RightStick,
    Dpad,
    Wasd,
    ArrowKeys,
}

impl PlayerInputs {
    pub fn new(controller: GameController) -> Self {
        let mut map = HashMap::new();
        use PlayerInput::*;
        use PlayerSkill::*;

        match controller {
            GameController::Gamepad { .. } => {
                map.extend([
                    (Move, Binding::LeftSick),
                    (Move, Binding::Dpad),
                    (Aim, Binding::RightStick),
                    (Skill(Collect), Binding::GamepadButton(GamepadButton::South)),
                    (
                        Skill(Shoot),
                        Binding::GamepadButton(GamepadButton::RightTrigger2),
                    ),
                    (
                        Skill(Defend),
                        Binding::GamepadButton(GamepadButton::LeftTrigger2),
                    ),
                    (Skill(Dash), Binding::GamepadButton(GamepadButton::East)),
                    (Pause, Binding::GamepadButton(GamepadButton::Start)),
                ]);
            }
            GameController::KeyBoard => {
                map.extend([
                    (Move, Binding::ArrowKeys),
                    (Move, Binding::Wasd),
                    (Aim, Binding::MouseMove),
                    (
                        Skill(Collect),
                        Binding::Keys(vec![KeyCode::ShiftLeft, KeyCode::ShiftRight]),
                    ),
                    (Skill(Shoot), Binding::MouseButton(MouseButton::Left)),
                    (Skill(Defend), Binding::MouseButton(MouseButton::Right)),
                    (Skill(Dash), Binding::KeyCode(KeyCode::Space)),
                    (Pause, Binding::KeyCode(KeyCode::Escape)),
                ]);
            }
        }
        Self(map)
    }
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
    pub fn input_map(controller: GameController, inputs: &PlayerInputs) -> InputMap<Self> {
        let mut map = InputMap::default();
        for (input, binding) in inputs.0.iter() {
            match binding {
                Binding::KeyCode(key) => {
                    map.insert(*input, *key);
                }
                Binding::Keys(keys) => {
                    map.insert_one_to_many(*input, keys.clone());
                }
                Binding::GamepadButton(button) => {
                    map.insert(*input, *button);
                }
                Binding::MouseMove => {
                    map.insert_dual_axis(*input, MouseMove::default());
                }
                Binding::MouseButton(button) => {
                    map.insert(*input, *button);
                }
                Binding::LeftSick => {
                    map.insert_dual_axis(*input, GamepadStick::LEFT);
                }
                Binding::RightStick => {
                    map.insert_dual_axis(*input, GamepadStick::RIGHT);
                }
                Binding::Dpad => {
                    map.insert_dual_axis(*input, VirtualDPad::dpad());
                }
                Binding::Wasd => {
                    map.insert_dual_axis(*input, VirtualDPad::wasd());
                }
                Binding::ArrowKeys => {
                    map.insert_dual_axis(*input, VirtualDPad::arrow_keys());
                }
            }
        }

        match controller {
            GameController::Gamepad { gamepad, .. } => {
                map.set_gamepad(gamepad);
            }
            GameController::KeyBoard => {}
        }
        map
    }

    pub fn get_movement(state: &ActionState<Self>) -> Option<Vec2> {
        state.clamped_axis_pair(&Self::Move).xy().try_normalize()
    }
}

pub fn handle_new_controllers(
    mut gamepad_evr: MessageReader<GamepadConnectionEvent>,
    mut keyboard_evr: MessageReader<KeyboardInput>,
    players: Query<&Player>,
    mut player_connected_evw: MessageWriter<PlayerConnected>,
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
                    player_connected_evw.write(PlayerConnected(Player {
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
        player_connected_evw.write(PlayerConnected(Player {
            controller: GameController::KeyBoard,
            id: new_player_id(),
        }));
    }
    keyboard_evr.clear();
}

fn pause_game(
    players: Query<(&Player, &ActionState<PlayerInput>)>,
    mut pause_evw: MessageWriter<PauseGame>,
) {
    for (player, state) in &players {
        if state.just_pressed(&PlayerInput::Pause) {
            log::info!("Pause triggered by player {}", player.id);
            pause_evw.write_default();
        }
    }
}
