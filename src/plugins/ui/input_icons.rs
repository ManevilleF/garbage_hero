use bevy::{platform::collections::HashMap, prelude::*};

use crate::plugins::player::{Binding, GameController, GamepadCategory, PlayerInput, PlayerInputs};

const NOT_FOUND_ICON: &str = "kenney_input-prompts/Flairs/flair_disabled.png";
const NO_INPUT_ICON: &str = "kenney_input-prompts/Flairs/flair_disabled_cross.png";
const MOUSE_ICON: &str = "kenney_input-prompts/Keyboard&Mouse/mouse_small.png";

#[derive(Component, Debug, Clone)]
pub struct InputMapIcons {
    pub controller_icon: Handle<Image>,
    pub input_icons: HashMap<PlayerInput, Handle<Image>>,
}

impl GamepadCategory {
    pub const fn controller_icon(self) -> &'static str {
        match self {
            Self::Xbox => "kenney_input-prompts/Xbox/controller_xboxseries.png",
            Self::PlayStation => "kenney_input-prompts/PlayStation/controller_playstation5.png",
            Self::Steam => "kenney_input-prompts/SteamDeck/controller_steamdeck.png",
            Self::Unknown => "kenney_input-prompts/PlayStation/controller_playstation2.png",
        }
    }
}

impl InputMapIcons {
    pub fn new(map: &PlayerInputs, controller: &GameController, server: &AssetServer) -> Self {
        let not_found_handle = server.load(NOT_FOUND_ICON);
        let (category, icon_path) = match controller {
            GameController::KeyBoard => (GamepadCategory::Unknown, MOUSE_ICON),
            GameController::Gamepad { category, .. } => (*category, category.controller_icon()),
        };
        let controller_icon = server.load(icon_path);
        let input_icons = map
            .0
            .iter()
            .map(|(action, input)| {
                let icon = if let Some(path) = input_icon(input, category) {
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

pub fn input_icon(input: &Binding, category: GamepadCategory) -> Option<&'static str> {
    match input {
        Binding::MouseButton(button) => mouse_button_icon(*button),
        Binding::GamepadButton(b) => gamepad_button_icon(*b, category),
        Binding::KeyCode(key_code) => keyboard_icon(*key_code),
        Binding::Keys(key_codes) => keyboard_icon(key_codes[0]),
        Binding::MouseMove => Some("kenney_input-prompts/Keyboard&Mouse/mouse_move.png"),
        Binding::LeftSick => Some(left_stick_icon(category)),
        Binding::RightStick => Some(right_stick_icon(category)),
        Binding::Dpad => Some(virtual_dpad_icon(category)),
        Binding::Wasd | Binding::ArrowKeys => {
            Some("kenney_input-prompts/Keyboard&Mouse/keyboard_arrows_all.png")
        }
    }
}

pub const fn gamepad_button_icon(
    button: GamepadButton,
    category: GamepadCategory,
) -> Option<&'static str> {
    match category {
        GamepadCategory::Xbox => xbox_button_icon(button),
        GamepadCategory::Steam => steamdeck_button_icon(button),
        GamepadCategory::PlayStation | GamepadCategory::Unknown => playstation_button_icon(button),
    }
}

pub const fn virtual_dpad_icon(category: GamepadCategory) -> &'static str {
    match category {
        GamepadCategory::Xbox => "kenney_input-prompts/Xbox/xbox_dpad_all.png",
        GamepadCategory::PlayStation | GamepadCategory::Unknown => {
            "kenney_input-prompts/PlayStation/playstation_dpad_all.png"
        }
        GamepadCategory::Steam => "kenney_input-prompts/SteamDeck/steamdeck_dpad_all.png",
    }
}

pub const fn left_stick_icon(category: GamepadCategory) -> &'static str {
    match category {
        GamepadCategory::Xbox => "kenney_input-prompts/Xbox/xbox_stick_l.png",
        GamepadCategory::PlayStation | GamepadCategory::Unknown => {
            "kenney_input-prompts/PlayStation/playstation_stick_l.png"
        }
        GamepadCategory::Steam => "kenney_input-prompts/SteamDeck/steamdeck_stick_l.png",
    }
}

pub const fn right_stick_icon(category: GamepadCategory) -> &'static str {
    match category {
        GamepadCategory::Xbox => "kenney_input-prompts/Xbox/xbox_stick_r.png",
        GamepadCategory::PlayStation | GamepadCategory::Unknown => {
            "kenney_input-prompts/PlayStation/playstation_stick_r.png"
        }
        GamepadCategory::Steam => "kenney_input-prompts/SteamDeck/steamdeck_stick_r.png",
    }
}

#[rustfmt::skip]
pub const fn playstation_button_icon(button: GamepadButton) -> Option<&'static str> {
    match button {
        GamepadButton::South => Some("kenney_input-prompts/PlayStation/playstation_button_color_cross.png"),
        GamepadButton::East => Some("kenney_input-prompts/PlayStation/playstation_button_color_circle.png"),
        GamepadButton::North => Some("kenney_input-prompts/PlayStation/playstation_button_color_triangle.png"),
        GamepadButton::West => Some("kenney_input-prompts/PlayStation/playstation_button_color_square.png"),
        GamepadButton::LeftTrigger => Some("kenney_input-prompts/PlayStation/playstation_trigger_l1.png"),
        GamepadButton::LeftTrigger2 => Some("kenney_input-prompts/PlayStation/playstation_trigger_l2.png"),
        GamepadButton::RightTrigger => Some("kenney_input-prompts/PlayStation/playstation_trigger_r1.png"),
        GamepadButton::RightTrigger2 => Some("kenney_input-prompts/PlayStation/playstation_trigger_r2.png"),
        GamepadButton::Select => Some("kenney_input-prompts/PlayStation/playstation3_button_select.png"),
        GamepadButton::Start => Some("kenney_input-prompts/PlayStation/playstation3_button_start.png"),
        GamepadButton::LeftThumb => Some("kenney_input-prompts/PlayStation/playstation_stick_l.png"),
        GamepadButton::RightThumb => Some("kenney_input-prompts/PlayStation/playstation_stick_r.png"),
        GamepadButton::DPadUp => Some("kenney_input-prompts/PlayStation/playstation_dpad_up.png"),
        GamepadButton::DPadDown => Some("kenney_input-prompts/PlayStation/playstation_dpad_down.png"),
        GamepadButton::DPadLeft => Some("kenney_input-prompts/PlayStation/playstation_dpad_left.png"),
        GamepadButton::DPadRight => Some("kenney_input-prompts/PlayStation/playstation_dpad_right.png"),
        _ => None,
    }
}

#[rustfmt::skip]
pub const fn xbox_button_icon(button: GamepadButton) -> Option<&'static str> {
    match button {
        GamepadButton::South => Some("kenney_input-prompts/Xbox/xbox_button_color_a.png"),
        GamepadButton::East => Some("kenney_input-prompts/Xbox/xbox_button_color_b.png"),
        GamepadButton::North => Some("kenney_input-prompts/Xbox/xbox_button_color_y.png"),
        GamepadButton::West => Some("kenney_input-prompts/Xbox/xbox_button_color_x.png"),
        GamepadButton::LeftTrigger => Some("kenney_input-prompts/Xbox/xbox_lb.png"),
        GamepadButton::LeftTrigger2 => Some("kenney_input-prompts/Xbox/xbox_lt.png"),
        GamepadButton::RightTrigger => Some("kenney_input-prompts/Xbox/xbox_rb.png"),
        GamepadButton::RightTrigger2 => Some("kenney_input-prompts/Xbox/xbox_rt.png"),
        GamepadButton::Select => Some("kenney_input-prompts/Xbox/xbox_button_start.png"),
        GamepadButton::Start => Some("kenney_input-prompts/Xbox/xbox_button_menu.png"),
        GamepadButton::LeftThumb => Some("kenney_input-prompts/Xbox/xbox_stick_l.png"),
        GamepadButton::RightThumb => Some("kenney_input-prompts/Xbox/xbox_stick_r.png"),
        GamepadButton::DPadUp => Some("kenney_input-prompts/Xbox/xbox_dpad_up.png"),
        GamepadButton::DPadDown => Some("kenney_input-prompts/Xbox/xbox_dpad_down.png"),
        GamepadButton::DPadLeft => Some("kenney_input-prompts/Xbox/xbox_dpad_left.png"),
        GamepadButton::DPadRight => Some("kenney_input-prompts/Xbox/xbox_dpad_right.png"),
        _ => None,
    }
}

#[rustfmt::skip]
pub const fn steamdeck_button_icon(button: GamepadButton) -> Option<&'static str> {
    match button {
        GamepadButton::South => Some("kenney_input-prompts/SteamDeck/steamdeck_button_a.png"),
        GamepadButton::East => Some("kenney_input-prompts/SteamDeck/steamdeck_button_b.png"),
        GamepadButton::North => Some("kenney_input-prompts/SteamDeck/steamdeck_button_y.png"),
        GamepadButton::West => Some("kenney_input-prompts/SteamDeck/steamdeck_button_x.png"),
        GamepadButton::LeftTrigger => Some("kenney_input-prompts/SteamDeck/steamdeck_button_l1.png"),
        GamepadButton::LeftTrigger2 => Some("kenney_input-prompts/SteamDeck/steamdeck_button_l2.png"),
        GamepadButton::RightTrigger => Some("kenney_input-prompts/SteamDeck/steamdeck_button_r1.png"),
        GamepadButton::RightTrigger2 => Some("kenney_input-prompts/SteamDeck/steamdeck_button_r2.png"),
        GamepadButton::Select => Some("kenney_input-prompts/SteamDeck/steamdeck_button_quickaccess.png"),
        GamepadButton::Start => Some("kenney_input-prompts/SteamDeck/steamdeck_button_options.png"),
        GamepadButton::LeftThumb => Some("kenney_input-prompts/SteamDeck/steamdeck_stick_l.png"),
        GamepadButton::RightThumb => Some("kenney_input-prompts/SteamDeck/steamdeck_stick_r.png"),
        GamepadButton::DPadUp => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_up.png"),
        GamepadButton::DPadDown => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_down.png"),
        GamepadButton::DPadLeft => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_left.png"),
        GamepadButton::DPadRight => Some("kenney_input-prompts/SteamDeck/steamdeck_dpad_right.png"),
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
