#![no_main]

use libfuzzer_sys::fuzz_target;
use kiran::input::{
    GamepadAxis, GamepadButton, InputEvent, InputState, KeyCode, MouseButton, TouchPhase,
};

/// Map a byte to one of the KeyCode variants (first 69 fit nicely).
fn key_from_byte(b: u8) -> KeyCode {
    match b % 69 {
        0 => KeyCode::A,
        1 => KeyCode::B,
        2 => KeyCode::C,
        3 => KeyCode::D,
        4 => KeyCode::E,
        5 => KeyCode::F,
        6 => KeyCode::G,
        7 => KeyCode::H,
        8 => KeyCode::I,
        9 => KeyCode::J,
        10 => KeyCode::K,
        11 => KeyCode::L,
        12 => KeyCode::M,
        13 => KeyCode::N,
        14 => KeyCode::O,
        15 => KeyCode::P,
        16 => KeyCode::Q,
        17 => KeyCode::R,
        18 => KeyCode::S,
        19 => KeyCode::T,
        20 => KeyCode::U,
        21 => KeyCode::V,
        22 => KeyCode::W,
        23 => KeyCode::X,
        24 => KeyCode::Y,
        25 => KeyCode::Z,
        26 => KeyCode::Key0,
        27 => KeyCode::Key1,
        28 => KeyCode::Key2,
        29 => KeyCode::Key3,
        30 => KeyCode::Key4,
        31 => KeyCode::Key5,
        32 => KeyCode::Key6,
        33 => KeyCode::Key7,
        34 => KeyCode::Key8,
        35 => KeyCode::Key9,
        36 => KeyCode::Up,
        37 => KeyCode::Down,
        38 => KeyCode::Left,
        39 => KeyCode::Right,
        40 => KeyCode::LShift,
        41 => KeyCode::RShift,
        42 => KeyCode::LCtrl,
        43 => KeyCode::RCtrl,
        44 => KeyCode::LAlt,
        45 => KeyCode::RAlt,
        46 => KeyCode::LSuper,
        47 => KeyCode::RSuper,
        48 => KeyCode::F1,
        49 => KeyCode::F2,
        50 => KeyCode::F3,
        51 => KeyCode::F4,
        52 => KeyCode::F5,
        53 => KeyCode::F6,
        54 => KeyCode::F7,
        55 => KeyCode::F8,
        56 => KeyCode::F9,
        57 => KeyCode::F10,
        58 => KeyCode::F11,
        59 => KeyCode::F12,
        60 => KeyCode::Space,
        61 => KeyCode::Enter,
        62 => KeyCode::Escape,
        63 => KeyCode::Tab,
        64 => KeyCode::Backspace,
        65 => KeyCode::Delete,
        66 => KeyCode::Home,
        67 => KeyCode::End,
        _ => KeyCode::PageUp,
    }
}

fn mouse_button_from_byte(b: u8) -> MouseButton {
    match b % 5 {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Back,
        _ => MouseButton::Forward,
    }
}

fn gamepad_button_from_byte(b: u8) -> GamepadButton {
    match b % 10 {
        0 => GamepadButton::South,
        1 => GamepadButton::East,
        2 => GamepadButton::West,
        3 => GamepadButton::North,
        4 => GamepadButton::LeftBumper,
        5 => GamepadButton::RightBumper,
        6 => GamepadButton::LeftTrigger,
        7 => GamepadButton::RightTrigger,
        8 => GamepadButton::DPadUp,
        _ => GamepadButton::DPadDown,
    }
}

fn gamepad_axis_from_byte(b: u8) -> GamepadAxis {
    match b % 4 {
        0 => GamepadAxis::LeftStickX,
        1 => GamepadAxis::LeftStickY,
        2 => GamepadAxis::RightStickX,
        _ => GamepadAxis::RightStickY,
    }
}

fn touch_phase_from_byte(b: u8) -> TouchPhase {
    match b % 4 {
        0 => TouchPhase::Started,
        1 => TouchPhase::Moved,
        2 => TouchPhase::Ended,
        _ => TouchPhase::Cancelled,
    }
}

/// Decode a pair of bytes into an f64 in [-1000, 1000].
fn f64_from_bytes(a: u8, b: u8) -> f64 {
    let raw = i16::from_le_bytes([a, b]);
    (raw as f64) / 32.0
}

fuzz_target!(|data: &[u8]| {
    let mut state = InputState::new();

    // Each event is decoded from a variable-length chunk starting with an
    // opcode byte.  We consume bytes greedily.
    let mut pos = 0;
    while pos < data.len() {
        let opcode = data[pos];
        pos += 1;

        let event = match opcode % 11 {
            // KeyPressed — 1 extra byte
            0 if pos < data.len() => {
                let e = InputEvent::KeyPressed(key_from_byte(data[pos]));
                pos += 1;
                e
            }
            // KeyReleased — 1 extra byte
            1 if pos < data.len() => {
                let e = InputEvent::KeyReleased(key_from_byte(data[pos]));
                pos += 1;
                e
            }
            // MouseMoved — 4 extra bytes
            2 if pos + 3 < data.len() => {
                let x = f64_from_bytes(data[pos], data[pos + 1]);
                let y = f64_from_bytes(data[pos + 2], data[pos + 3]);
                pos += 4;
                InputEvent::MouseMoved { x, y }
            }
            // MouseButtonPressed — 1 extra byte
            3 if pos < data.len() => {
                let e = InputEvent::MouseButtonPressed(mouse_button_from_byte(data[pos]));
                pos += 1;
                e
            }
            // MouseButtonReleased — 1 extra byte
            4 if pos < data.len() => {
                let e = InputEvent::MouseButtonReleased(mouse_button_from_byte(data[pos]));
                pos += 1;
                e
            }
            // MouseScroll — 4 extra bytes
            5 if pos + 3 < data.len() => {
                let dx = f64_from_bytes(data[pos], data[pos + 1]);
                let dy = f64_from_bytes(data[pos + 2], data[pos + 3]);
                pos += 4;
                InputEvent::MouseScroll { dx, dy }
            }
            // GamepadButtonPressed — 1 extra byte
            6 if pos < data.len() => {
                let e = InputEvent::GamepadButtonPressed(gamepad_button_from_byte(data[pos]));
                pos += 1;
                e
            }
            // GamepadButtonReleased — 1 extra byte
            7 if pos < data.len() => {
                let e = InputEvent::GamepadButtonReleased(gamepad_button_from_byte(data[pos]));
                pos += 1;
                e
            }
            // GamepadAxisMoved — 3 extra bytes
            8 if pos + 2 < data.len() => {
                let axis = gamepad_axis_from_byte(data[pos]);
                let value = f64_from_bytes(data[pos + 1], data[pos + 2]);
                pos += 3;
                InputEvent::GamepadAxisMoved { axis, value }
            }
            // Touch — 5 extra bytes (id, x, y, phase)
            9 if pos + 4 < data.len() => {
                let id = data[pos] as u64;
                let x = f64_from_bytes(data[pos + 1], data[pos + 2]);
                let y = f64_from_bytes(data[pos + 3], data[pos + 4]);
                // Use opcode's high bits for phase variety
                let phase = touch_phase_from_byte(data[pos]);
                pos += 5;
                InputEvent::Touch { id, x, y, phase }
            }
            // end_frame — call end_frame to reset per-frame state and continue
            10 => {
                state.clear_frame();
                continue;
            }
            // Not enough bytes for the chosen event — stop.
            _ => break,
        };

        state.process_event(&event);
    }
});
