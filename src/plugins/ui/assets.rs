use bevy::prelude::*;
use leafwing_input_manager::buttonlike::MouseWheelDirection;

pub struct InputIcons {}

pub const fn playstation_icon(button: GamepadButtonType) -> Option<&'static str> {
    match button {
        GamepadButtonType::South => Some("PlayStation/playstation_button_color_cross.png"),
        GamepadButtonType::East => Some("PlayStation/playstation_button_color_triangle.png"),
        GamepadButtonType::North => Some("PlayStation/playstation_button_color_triangle.png"),
        GamepadButtonType::West => Some("PlayStation/playstation_button_color_square.png"),
        GamepadButtonType::LeftTrigger => Some("PlayStation/playstation_trigger_l1.png"),
        GamepadButtonType::LeftTrigger2 => Some("PlayStation/playstation_trigger_l2.png"),
        GamepadButtonType::RightTrigger => Some("PlayStation/playstation_trigger_r1.png"),
        GamepadButtonType::RightTrigger2 => Some("PlayStation/playstation_trigger_r2.png"),
        GamepadButtonType::Select => Some("PlayStation/playstation3_button_select.png"),
        GamepadButtonType::Start => Some("PlayStation/playstation3_button_start.png"),
        GamepadButtonType::LeftThumb => Some("PlayStation/playstation_stick_l.png"),
        GamepadButtonType::RightThumb => Some("PlayStation/playstation_stick_r.png"),
        GamepadButtonType::DPadUp => Some("PlayStation/playstation_dpad_up.png"),
        GamepadButtonType::DPadDown => Some("PlayStation/playstation_dpad_down.png"),
        GamepadButtonType::DPadLeft => Some("PlayStation/playstation_dpad_left.png"),
        GamepadButtonType::DPadRight => Some("PlayStation/playstation_dpad_right.png"),
        _ => None,
    }
}

pub const fn xbox_icon(button: GamepadButtonType) -> Option<&'static str> {
    match button {
        GamepadButtonType::South => Some("Xbox/xbox_button_color_a.png"),
        GamepadButtonType::East => Some("Xbox/xbox_button_color_b.png"),
        GamepadButtonType::North => Some("Xbox/xbox_button_color_y.png"),
        GamepadButtonType::West => Some("Xbox/xbox_button_color_x.png"),
        GamepadButtonType::LeftTrigger => Some("Xbox/xbox_lt.png"),
        GamepadButtonType::LeftTrigger2 => Some("Xbox/xbox_lb.png"),
        GamepadButtonType::RightTrigger => Some("Xbox/xbox_rt.png"),
        GamepadButtonType::RightTrigger2 => Some("Xbox/xbox_rb.png"),
        GamepadButtonType::Select => Some("Xbox/xbox_button_menu.png"),
        GamepadButtonType::Start => Some("Xbox/xbox_button_start.png"),
        GamepadButtonType::LeftThumb => Some("Xbox/xbox_stick_l.png"),
        GamepadButtonType::RightThumb => Some("Xbox/xbox_stick_r.png"),
        GamepadButtonType::DPadUp => Some("Xbox/xbox_dpad_up.png"),
        GamepadButtonType::DPadDown => Some("Xbox/xbox_dpad_down.png"),
        GamepadButtonType::DPadLeft => Some("Xbox/xbox_dpad_left.png"),
        GamepadButtonType::DPadRight => Some("Xbox/xbox_dpad_right.png"),
        _ => None,
    }
}

pub const fn steamdeck_icon(button: GamepadButtonType) -> Option<&'static str> {
    match button {
        GamepadButtonType::South => Some("SteamDeck/steamdeck_button_a.png"),
        GamepadButtonType::East => Some("SteamDeck/steamdeck_button_b.png"),
        GamepadButtonType::North => Some("SteamDeck/steamdeck_button_y.png"),
        GamepadButtonType::West => Some("SteamDeck/steamdeck_button_x.png"),
        GamepadButtonType::LeftTrigger => Some("SteamDeck/steamdeck_button_l1.png"),
        GamepadButtonType::LeftTrigger2 => Some("SteamDeck/steamdeck_button_l2.png"),
        GamepadButtonType::RightTrigger => Some("SteamDeck/steamdeck_button_r1.png"),
        GamepadButtonType::RightTrigger2 => Some("SteamDeck/steamdeck_button_r2.png"),
        GamepadButtonType::Select => Some("SteamDeck/steamdeck_button_options.png"),
        GamepadButtonType::Start => Some("SteamDeck/steamdeck_button_quickaccess.png"),
        GamepadButtonType::LeftThumb => Some("SteamDeck/steamdeck_stick_l.png"),
        GamepadButtonType::RightThumb => Some("SteamDeck/steamdeck_stick_r.png"),
        GamepadButtonType::DPadUp => Some("SteamDeck/steamdeck_dpad_up.png"),
        GamepadButtonType::DPadDown => Some("SteamDeck/steamdeck_dpad_down.png"),
        GamepadButtonType::DPadLeft => Some("SteamDeck/steamdeck_dpad_left.png"),
        GamepadButtonType::DPadRight => Some("SteamDeck/steamdeck_dpad_right.png"),
        _ => None,
    }
}

pub const fn mouse_button_icon(button: MouseButton) -> Option<&'static str> {
    match button {
        MouseButton::Left => Some("KeyboardMouse/mouse_left.png"),
        MouseButton::Right => Some("KeyboardMouse/mouse_right.png"),
        MouseButton::Middle => Some("KeyboardMouse/mouse_scroll.png"),
        MouseButton::Back => Some("KeyboardMouse/mouse_small.png"),
        MouseButton::Forward => Some("KeyboardMouse/mouse_small.png"),
        _ => None,
    }
}

pub const fn mouse_wheel_icon(direction: MouseWheelDirection) -> Option<&'static str> {
    match direction {
        MouseWheelDirection::Up => Some("KeyboardMouse/mouse_scroll_up.png"),
        MouseWheelDirection::Down => Some("KeyboardMouse/mouse_scroll_down.png"),
        _ => None,
    }
}

pub const fn keyboard_icon(key: KeyCode) -> Option<&'static str> {
    match key {
        KeyCode::KeyA => Some("KeyboardMouse/keyboard_a.png"),
        KeyCode::KeyB => Some("KeyboardMouse/keyboard_b.png"),
        KeyCode::KeyC => Some("KeyboardMouse/keyboard_c.png"),
        KeyCode::KeyD => Some("KeyboardMouse/keyboard_d.png"),
        KeyCode::KeyE => Some("KeyboardMouse/keyboard_e.png"),
        KeyCode::KeyF => Some("KeyboardMouse/keyboard_f.png"),
        KeyCode::KeyG => Some("KeyboardMouse/keyboard_g.png"),
        KeyCode::KeyH => Some("KeyboardMouse/keyboard_h.png"),
        KeyCode::KeyI => Some("KeyboardMouse/keyboard_i.png"),
        KeyCode::KeyJ => Some("KeyboardMouse/keyboard_j.png"),
        KeyCode::KeyK => Some("KeyboardMouse/keyboard_k.png"),
        KeyCode::KeyL => Some("KeyboardMouse/keyboard_l.png"),
        KeyCode::KeyM => Some("KeyboardMouse/keyboard_m.png"),
        KeyCode::KeyN => Some("KeyboardMouse/keyboard_n.png"),
        KeyCode::KeyO => Some("KeyboardMouse/keyboard_o.png"),
        KeyCode::KeyP => Some("KeyboardMouse/keyboard_p.png"),
        KeyCode::KeyQ => Some("KeyboardMouse/keyboard_q.png"),
        KeyCode::KeyR => Some("KeyboardMouse/keyboard_r.png"),
        KeyCode::KeyS => Some("KeyboardMouse/keyboard_s.png"),
        KeyCode::KeyT => Some("KeyboardMouse/keyboard_t.png"),
        KeyCode::KeyU => Some("KeyboardMouse/keyboard_u.png"),
        KeyCode::KeyV => Some("KeyboardMouse/keyboard_v.png"),
        KeyCode::KeyW => Some("KeyboardMouse/keyboard_w.png"),
        KeyCode::KeyX => Some("KeyboardMouse/keyboard_x.png"),
        KeyCode::KeyY => Some("KeyboardMouse/keyboard_y.png"),
        KeyCode::KeyZ => Some("KeyboardMouse/keyboard_z.png"),
        KeyCode::Digit0 => Some("KeyboardMouse/keyboard_0.png"),
        KeyCode::Digit1 => Some("KeyboardMouse/keyboard_1.png"),
        KeyCode::Digit2 => Some("KeyboardMouse/keyboard_2.png"),
        KeyCode::Digit3 => Some("KeyboardMouse/keyboard_3.png"),
        KeyCode::Digit4 => Some("KeyboardMouse/keyboard_4.png"),
        KeyCode::Digit5 => Some("KeyboardMouse/keyboard_5.png"),
        KeyCode::Digit6 => Some("KeyboardMouse/keyboard_6.png"),
        KeyCode::Digit7 => Some("KeyboardMouse/keyboard_7.png"),
        KeyCode::Digit8 => Some("KeyboardMouse/keyboard_8.png"),
        KeyCode::Digit9 => Some("KeyboardMouse/keyboard_9.png"),
        KeyCode::Escape => Some("KeyboardMouse/keyboard_escape.png"),
        KeyCode::Backspace => Some("KeyboardMouse/keyboard_backspace.png"),
        KeyCode::Enter => Some("KeyboardMouse/keyboard_enter.png"),
        KeyCode::Tab => Some("KeyboardMouse/keyboard_tab.png"),
        KeyCode::Space => Some("KeyboardMouse/keyboard_space.png"),
        KeyCode::Minus => Some("KeyboardMouse/keyboard_minus.png"),
        KeyCode::Equal => Some("KeyboardMouse/keyboard_equals.png"),
        KeyCode::BracketLeft => Some("KeyboardMouse/keyboard_bracket_open.png"),
        KeyCode::BracketRight => Some("KeyboardMouse/keyboard_bracket_close.png"),
        KeyCode::Backslash => Some("KeyboardMouse/keyboard_slash_back.png"),
        KeyCode::Semicolon => Some("KeyboardMouse/keyboard_semicolon.png"),
        KeyCode::Quote => Some("KeyboardMouse/keyboard_quote.png"),
        KeyCode::Comma => Some("KeyboardMouse/keyboard_comma.png"),
        KeyCode::Period => Some("KeyboardMouse/keyboard_period.png"),
        KeyCode::Slash => Some("KeyboardMouse/keyboard_slash_forward.png"),
        KeyCode::CapsLock => Some("KeyboardMouse/keyboard_capslock.png"),
        KeyCode::Insert => Some("KeyboardMouse/keyboard_insert.png"),
        KeyCode::Delete => Some("KeyboardMouse/keyboard_delete.png"),
        KeyCode::Home => Some("KeyboardMouse/keyboard_home.png"),
        KeyCode::End => Some("KeyboardMouse/keyboard_end.png"),
        KeyCode::PageUp => Some("KeyboardMouse/keyboard_page_up.png"),
        KeyCode::PageDown => Some("KeyboardMouse/keyboard_page_down.png"),
        KeyCode::ArrowUp => Some("KeyboardMouse/keyboard_arrow_up.png"),
        KeyCode::ArrowDown => Some("KeyboardMouse/keyboard_arrow_down.png"),
        KeyCode::ArrowLeft => Some("KeyboardMouse/keyboard_arrow_left.png"),
        KeyCode::ArrowRight => Some("KeyboardMouse/keyboard_arrow_right.png"),
        KeyCode::ControlLeft => Some("KeyboardMouse/keyboard_ctrl.png"),
        KeyCode::ControlRight => Some("KeyboardMouse/keyboard_ctrl.png"),
        KeyCode::ShiftLeft => Some("KeyboardMouse/keyboard_shift.png"),
        KeyCode::ShiftRight => Some("KeyboardMouse/keyboard_shift.png"),
        KeyCode::AltLeft => Some("KeyboardMouse/keyboard_alt.png"),
        KeyCode::AltRight => Some("KeyboardMouse/keyboard_alt.png"),
        KeyCode::Meta => Some("KeyboardMouse/keyboard_command.png"),
        _ => None,
    }
}
