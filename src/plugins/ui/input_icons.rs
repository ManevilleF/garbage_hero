use bevy::{prelude::*, utils::HashMap};
use leafwing_input_manager::prelude::*;

use crate::plugins::player::{GameController, PlayerInput};

const NOT_FOUND_ICON: &str = "kenney_input-prompts/Flairs/flair_disabled.png";
const NO_INPUT_ICON: &str = "kenney_input-prompts/Flairs/flair_disabled_cross.png";
const MOUSE_ICON: &str = "kenney_input-prompts/Keyboard&Mouse/mouse_small.png";
const CONTROLLER_ICON: &str = "kenney_input-prompts/Xbox/controller_xboxseries.png";

#[derive(Component, Debug, Clone)]
pub struct InputMapIcons {
    pub controller_icon: Handle<Image>,
    pub input_icons: HashMap<PlayerInput, Handle<Image>>,
}

impl InputMapIcons {
    pub fn new(
        map: &InputMap<PlayerInput>,
        controller: &GameController,
        server: &AssetServer,
    ) -> Self {
        let not_found_handle = server.load(NOT_FOUND_ICON);
        let not_input_handle = server.load(NO_INPUT_ICON);
        let controller_icon = server.load(match controller {
            GameController::KeyBoard => MOUSE_ICON,
            GameController::Gamepad(_) => CONTROLLER_ICON,
        });
        let input_icons = map
            .iter()
            .map(|(action, input)| {
                let icon = if input.is_empty() {
                    not_input_handle.clone()
                } else if let Some(path) = input_icon(input[0].clone()) {
                    server.load(path)
                } else {
                    not_found_handle.clone()
                };
                (*action, icon)
            })
            .collect();
        Self {
            controller_icon,
            input_icons,
        }
    }
}

pub fn input_icon(input: UserInput) -> Option<&'static str> {
    match input {
        UserInput::Single(i) => single_input_icon(i),
        UserInput::Chord(_) => None,
        UserInput::VirtualDPad(p) => virtual_dpad_icon(p),
        UserInput::VirtualAxis(a) => virtual_axis_icon(a),
    }
}

pub fn single_input_icon(input: InputKind) -> Option<&'static str> {
    match input {
        InputKind::GamepadButton(b) => xbox_icon(b),
        InputKind::SingleAxis(_) => None,
        InputKind::DualAxis(d) => dual_axis_icon(d),
        InputKind::PhysicalKey(k) => keyboard_icon(k),
        InputKind::Modifier(_) => None,
        InputKind::Mouse(b) => mouse_button_icon(b),
        InputKind::MouseWheel(w) => mouse_wheel_icon(w),
        InputKind::MouseMotion(_) => None,
        _ => None,
    }
}

pub fn virtual_dpad_icon(pad: VirtualDPad) -> Option<&'static str> {
    let path = if pad == VirtualDPad::arrow_keys() || pad == VirtualDPad::wasd() {
        "kenney_input-prompts/Keyboard&Mouse/keyboard_arrows_all.png"
    } else if pad == VirtualDPad::mouse_motion() {
        "kenney_input-prompts/Keyboard&Mouse/mouse_move.png"
    } else if pad == VirtualDPad::dpad() {
        "kenney_input-prompts/Xbox/xbox_dpad_all.png"
    } else {
        return None;
    };
    Some(path)
}

pub fn virtual_axis_icon(axis: VirtualAxis) -> Option<&'static str> {
    let path = if axis == VirtualAxis::ad() || axis == VirtualAxis::horizontal_arrow_keys() {
        "kenney_input-prompts/Keyboard&Mouse/keyboard_arrows_horizontal.png"
    } else if axis == VirtualAxis::ws() || axis == VirtualAxis::vertical_arrow_keys() {
        "kenney_input-prompts/Keyboard&Mouse/keyboard_arrows_vertical.png"
    } else if axis == VirtualAxis::horizontal_dpad() {
        "kenney_input-prompts/Xbox/xbox_dpad_horizontal.png"
    } else if axis == VirtualAxis::vertical_dpad() {
        "kenney_input-prompts/Xbox/xbox_dpad_vertical.png"
    } else {
        return None;
    };
    Some(path)
}

pub fn dual_axis_icon(axis: DualAxis) -> Option<&'static str> {
    let path = if axis == DualAxis::left_stick() {
        "kenney_input-prompts/Xbox/xbox_stick_l.png"
    } else if axis == DualAxis::right_stick() {
        "kenney_input-prompts/Xbox/xbox_stick_r.png"
    } else if axis == DualAxis::mouse_motion() {
        "kenney_input-prompts/Keyboard&Mouse/mouse_move.png"
    } else {
        return None;
    };
    Some(path)
}

#[rustfmt::skip]
pub const fn playstation_icon(button: GamepadButtonType) -> Option<&'static str> {
    match button {
        GamepadButtonType::South => Some("kenney_input-prompts/PlayStation/playstation_button_color_cross.png"),
        GamepadButtonType::East => Some("kenney_input-prompts/PlayStation/playstation_button_color_triangle.png"),
        GamepadButtonType::North => Some("kenney_input-prompts/PlayStation/playstation_button_color_triangle.png"),
        GamepadButtonType::West => Some("kenney_input-prompts/PlayStation/playstation_button_color_square.png"),
        GamepadButtonType::LeftTrigger => Some("kenney_input-prompts/PlayStation/playstation_trigger_l1.png"),
        GamepadButtonType::LeftTrigger2 => Some("kenney_input-prompts/PlayStation/playstation_trigger_l2.png"),
        GamepadButtonType::RightTrigger => Some("kenney_input-prompts/PlayStation/playstation_trigger_r1.png"),
        GamepadButtonType::RightTrigger2 => Some("kenney_input-prompts/PlayStation/playstation_trigger_r2.png"),
        GamepadButtonType::Select => Some("kenney_input-prompts/PlayStation/playstation3_button_select.png"),
        GamepadButtonType::Start => Some("kenney_input-prompts/PlayStation/playstation3_button_start.png"),
        GamepadButtonType::LeftThumb => Some("kenney_input-prompts/PlayStation/playstation_stick_l.png"),
        GamepadButtonType::RightThumb => Some("kenney_input-prompts/PlayStation/playstation_stick_r.png"),
        GamepadButtonType::DPadUp => Some("kenney_input-prompts/PlayStation/playstation_dpad_up.png"),
        GamepadButtonType::DPadDown => Some("kenney_input-prompts/PlayStation/playstation_dpad_down.png"),
        GamepadButtonType::DPadLeft => Some("kenney_input-prompts/PlayStation/playstation_dpad_left.png"),
        GamepadButtonType::DPadRight => Some("kenney_input-prompts/PlayStation/playstation_dpad_right.png"),
        _ => None,
    }
}

#[rustfmt::skip]
pub const fn xbox_icon(button: GamepadButtonType) -> Option<&'static str> {
    match button {
        GamepadButtonType::South => Some("kenney_input-prompts/Xbox/xbox_button_color_a.png"),
        GamepadButtonType::East => Some("kenney_input-prompts/Xbox/xbox_button_color_b.png"),
        GamepadButtonType::North => Some("kenney_input-prompts/Xbox/xbox_button_color_y.png"),
        GamepadButtonType::West => Some("kenney_input-prompts/Xbox/xbox_button_color_x.png"),
        GamepadButtonType::LeftTrigger => Some("kenney_input-prompts/Xbox/xbox_lb.png"),
        GamepadButtonType::LeftTrigger2 => Some("kenney_input-prompts/Xbox/xbox_lt.png"),
        GamepadButtonType::RightTrigger => Some("kenney_input-prompts/Xbox/xbox_rb.png"),
        GamepadButtonType::RightTrigger2 => Some("kenney_input-prompts/Xbox/xbox_rt.png"),
        GamepadButtonType::Select => Some("kenney_input-prompts/Xbox/xbox_button_start.png"),
        GamepadButtonType::Start => Some("kenney_input-prompts/Xbox/xbox_button_menu.png"),
        GamepadButtonType::LeftThumb => Some("kenney_input-prompts/Xbox/xbox_stick_l.png"),
        GamepadButtonType::RightThumb => Some("kenney_input-prompts/Xbox/xbox_stick_r.png"),
        GamepadButtonType::DPadUp => Some("kenney_input-prompts/Xbox/xbox_dpad_up.png"),
        GamepadButtonType::DPadDown => Some("kenney_input-prompts/Xbox/xbox_dpad_down.png"),
        GamepadButtonType::DPadLeft => Some("kenney_input-prompts/Xbox/xbox_dpad_left.png"),
        GamepadButtonType::DPadRight => Some("kenney_input-prompts/Xbox/xbox_dpad_right.png"),
        _ => None,
    }
}

#[rustfmt::skip]
pub const fn steamdeck_icon(button: GamepadButtonType) -> Option<&'static str> {
    match button {
        GamepadButtonType::South => Some("kenney_input-prompts/SteamDeck/steamdeck_button_a.png"),
        GamepadButtonType::East => Some("kenney_input-prompts/SteamDeck/steamdeck_button_b.png"),
        GamepadButtonType::North => Some("kenney_input-prompts/SteamDeck/steamdeck_button_y.png"),
        GamepadButtonType::West => Some("kenney_input-prompts/SteamDeck/steamdeck_button_x.png"),
        GamepadButtonType::LeftTrigger => Some("kenney_input-prompts/SteamDeck/steamdeck_button_l1.png"),
        GamepadButtonType::LeftTrigger2 => Some("kenney_input-prompts/SteamDeck/steamdeck_button_l2.png"),
        GamepadButtonType::RightTrigger => Some("kenney_input-prompts/SteamDeck/steamdeck_button_r1.png"),
        GamepadButtonType::RightTrigger2 => Some("kenney_input-prompts/SteamDeck/steamdeck_button_r2.png"),
        GamepadButtonType::Select => Some("kenney_input-prompts/SteamDeck/steamdeck_button_quickaccess.png"),
        GamepadButtonType::Start => Some("kenney_input-prompts/SteamDeck/steamdeck_button_options.png"),
        GamepadButtonType::LeftThumb => Some("kenney_input-prompts/SteamDeck/steamdeck_stick_l.png"),
        GamepadButtonType::RightThumb => Some("kenney_input-prompts/SteamDeck/steamdeck_stick_r.png"),
        GamepadButtonType::DPadUp => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_up.png"),
        GamepadButtonType::DPadDown => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_down.png"),
        GamepadButtonType::DPadLeft => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_left.png"),
        GamepadButtonType::DPadRight => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_right.png"),
        _ => None,
    }
}

#[rustfmt::skip]
pub const fn mouse_button_icon(button: MouseButton) -> Option<&'static str> {
    match button {
        MouseButton::Left => Some("kenney_input-prompts/Keyboard&Mouse/mouse_left.png"),
        MouseButton::Right => Some("kenney_input-prompts/Keyboard&Mouse/mouse_right.png"),
        MouseButton::Middle => Some("kenney_input-prompts/Keyboard&Mouse/mouse_scroll.png"),
        MouseButton::Back => Some("kenney_input-prompts/Keyboard&Mouse/mouse_small.png"),
        MouseButton::Forward => Some("kenney_input-prompts/Keyboard&Mouse/mouse_small.png"),
        _ => None,
    }
}

#[rustfmt::skip]
pub const fn mouse_wheel_icon(direction: MouseWheelDirection) -> Option<&'static str> {
    match direction {
        MouseWheelDirection::Up => Some("kenney_input-prompts/Keyboard&Mouse/mouse_scroll_up.png"),
        MouseWheelDirection::Down => Some("kenney_input-prompts/Keyboard&Mouse/mouse_scroll_down.png"),
        _ => None,
    }
}

#[rustfmt::skip]
pub const fn keyboard_icon(key: KeyCode) -> Option<&'static str> {
    match key {
        KeyCode::KeyA => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_a.png"),
        KeyCode::KeyB => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_b.png"),
        KeyCode::KeyC => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_c.png"),
        KeyCode::KeyD => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_d.png"),
        KeyCode::KeyE => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_e.png"),
        KeyCode::KeyF => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_f.png"),
        KeyCode::KeyG => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_g.png"),
        KeyCode::KeyH => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_h.png"),
        KeyCode::KeyI => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_i.png"),
        KeyCode::KeyJ => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_j.png"),
        KeyCode::KeyK => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_k.png"),
        KeyCode::KeyL => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_l.png"),
        KeyCode::KeyM => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_m.png"),
        KeyCode::KeyN => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_n.png"),
        KeyCode::KeyO => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_o.png"),
        KeyCode::KeyP => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_p.png"),
        KeyCode::KeyQ => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_q.png"),
        KeyCode::KeyR => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_r.png"),
        KeyCode::KeyS => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_s.png"),
        KeyCode::KeyT => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_t.png"),
        KeyCode::KeyU => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_u.png"),
        KeyCode::KeyV => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_v.png"),
        KeyCode::KeyW => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_w.png"),
        KeyCode::KeyX => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_x.png"),
        KeyCode::KeyY => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_y.png"),
        KeyCode::KeyZ => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_z.png"),
        KeyCode::Digit0 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_0.png"),
        KeyCode::Digit1 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_1.png"),
        KeyCode::Digit2 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_2.png"),
        KeyCode::Digit3 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_3.png"),
        KeyCode::Digit4 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_4.png"),
        KeyCode::Digit5 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_5.png"),
        KeyCode::Digit6 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_6.png"),
        KeyCode::Digit7 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_7.png"),
        KeyCode::Digit8 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_8.png"),
        KeyCode::Digit9 => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_9.png"),
        KeyCode::Escape => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_escape.png"),
        KeyCode::Backspace => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_backspace.png"),
        KeyCode::Enter => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_enter.png"),
        KeyCode::Tab => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_tab.png"),
        KeyCode::Space => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_space.png"),
        KeyCode::Minus => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_minus.png"),
        KeyCode::Equal => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_equals.png"),
        KeyCode::BracketLeft => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_bracket_open.png"),
        KeyCode::BracketRight => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_bracket_close.png"),
        KeyCode::Backslash => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_slash_back.png"),
        KeyCode::Semicolon => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_semicolon.png"),
        KeyCode::Quote => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_quote.png"),
        KeyCode::Comma => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_comma.png"),
        KeyCode::Period => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_period.png"),
        KeyCode::Slash => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_slash_forward.png"),
        KeyCode::CapsLock => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_capslock.png"),
        KeyCode::Insert => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_insert.png"),
        KeyCode::Delete => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_delete.png"),
        KeyCode::Home => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_home.png"),
        KeyCode::End => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_end.png"),
        KeyCode::PageUp => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_page_up.png"),
        KeyCode::PageDown => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_page_down.png"),
        KeyCode::ArrowUp => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_arrow_up.png"),
        KeyCode::ArrowDown => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_arrow_down.png"),
        KeyCode::ArrowLeft => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_arrow_left.png"),
        KeyCode::ArrowRight => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_arrow_right.png"),
        KeyCode::ControlLeft => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_ctrl.png"),
        KeyCode::ControlRight => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_ctrl.png"),
        KeyCode::ShiftLeft => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_shift.png"),
        KeyCode::ShiftRight => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_shift.png"),
        KeyCode::AltLeft => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_alt.png"),
        KeyCode::AltRight => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_alt.png"),
        KeyCode::Meta => Some("kenney_input-prompts/Keyboard&Mouse/keyboard_command.png"),
        _ => None,
    }
}
