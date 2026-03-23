//! kiran-input — Input handling: keyboard, mouse, gamepad
//!
//! Provides input event types and an [`InputState`] tracker that accumulates
//! events each frame and exposes query methods for pressed keys, mouse
//! position, and button state.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Key codes
// ---------------------------------------------------------------------------

/// Keyboard key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Digits
    Key0, Key1, Key2, Key3, Key4,
    Key5, Key6, Key7, Key8, Key9,

    // Arrows
    Up, Down, Left, Right,

    // Modifiers
    LShift, RShift,
    LCtrl, RCtrl,
    LAlt, RAlt,
    LSuper, RSuper,

    // Function keys
    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,

    // Common keys
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    CapsLock,

    // Punctuation / misc
    Minus,
    Equals,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    Comma,
    Period,
    Slash,
    Grave,
}

// ---------------------------------------------------------------------------
// Mouse
// ---------------------------------------------------------------------------

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

// ---------------------------------------------------------------------------
// Input events
// ---------------------------------------------------------------------------

/// An input event produced by the platform layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MouseMoved { x: f64, y: f64 },
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseScroll { dx: f64, dy: f64 },
}

// ---------------------------------------------------------------------------
// InputState
// ---------------------------------------------------------------------------

/// Accumulated input state for the current frame.
#[derive(Debug, Default)]
pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: HashSet<MouseButton>,
    mouse_x: f64,
    mouse_y: f64,
    scroll_dx: f64,
    scroll_dy: f64,
    /// Keys pressed this frame (for "just pressed" queries).
    just_pressed: HashSet<KeyCode>,
    /// Keys released this frame.
    just_released: HashSet<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a single input event, updating internal state.
    pub fn process_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::KeyPressed(key) => {
                if self.pressed_keys.insert(*key) {
                    self.just_pressed.insert(*key);
                }
            }
            InputEvent::KeyReleased(key) => {
                self.pressed_keys.remove(key);
                self.just_released.insert(*key);
            }
            InputEvent::MouseMoved { x, y } => {
                self.mouse_x = *x;
                self.mouse_y = *y;
            }
            InputEvent::MouseButtonPressed(btn) => {
                self.pressed_buttons.insert(*btn);
            }
            InputEvent::MouseButtonReleased(btn) => {
                self.pressed_buttons.remove(btn);
            }
            InputEvent::MouseScroll { dx, dy } => {
                self.scroll_dx += dx;
                self.scroll_dy += dy;
            }
        }
    }

    /// Whether a key is currently held down.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Whether a key was pressed this frame (edge-triggered).
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// Whether a key was released this frame.
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    /// Whether a mouse button is currently held.
    pub fn is_mouse_button_pressed(&self, btn: MouseButton) -> bool {
        self.pressed_buttons.contains(&btn)
    }

    /// Current mouse position.
    pub fn mouse_position(&self) -> (f64, f64) {
        (self.mouse_x, self.mouse_y)
    }

    /// Accumulated scroll delta this frame.
    pub fn scroll_delta(&self) -> (f64, f64) {
        (self.scroll_dx, self.scroll_dy)
    }

    /// Clear per-frame transient state (call at the start of each frame).
    pub fn clear_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.scroll_dx = 0.0;
        self.scroll_dy = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_press_release() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::KeyPressed(KeyCode::W));
        assert!(state.is_key_pressed(KeyCode::W));
        assert!(state.is_key_just_pressed(KeyCode::W));

        state.process_event(&InputEvent::KeyReleased(KeyCode::W));
        assert!(!state.is_key_pressed(KeyCode::W));
        assert!(state.is_key_just_released(KeyCode::W));
    }

    #[test]
    fn key_not_pressed() {
        let state = InputState::new();
        assert!(!state.is_key_pressed(KeyCode::A));
    }

    #[test]
    fn mouse_moved() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::MouseMoved { x: 100.0, y: 200.0 });
        assert_eq!(state.mouse_position(), (100.0, 200.0));
    }

    #[test]
    fn mouse_button() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::MouseButtonPressed(MouseButton::Left));
        assert!(state.is_mouse_button_pressed(MouseButton::Left));
        assert!(!state.is_mouse_button_pressed(MouseButton::Right));

        state.process_event(&InputEvent::MouseButtonReleased(MouseButton::Left));
        assert!(!state.is_mouse_button_pressed(MouseButton::Left));
    }

    #[test]
    fn mouse_scroll() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::MouseScroll { dx: 0.0, dy: 3.0 });
        state.process_event(&InputEvent::MouseScroll { dx: 0.0, dy: -1.0 });
        assert_eq!(state.scroll_delta(), (0.0, 2.0));
    }

    #[test]
    fn clear_frame_resets_transient() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::KeyPressed(KeyCode::Space));
        state.process_event(&InputEvent::MouseScroll { dx: 1.0, dy: 1.0 });

        assert!(state.is_key_just_pressed(KeyCode::Space));
        state.clear_frame();

        assert!(!state.is_key_just_pressed(KeyCode::Space));
        assert_eq!(state.scroll_delta(), (0.0, 0.0));
        // Key is still held down
        assert!(state.is_key_pressed(KeyCode::Space));
    }

    #[test]
    fn just_pressed_only_on_first_event() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::KeyPressed(KeyCode::A));
        state.process_event(&InputEvent::KeyPressed(KeyCode::A)); // duplicate
        assert!(state.is_key_just_pressed(KeyCode::A));

        state.clear_frame();
        assert!(!state.is_key_just_pressed(KeyCode::A));
        assert!(state.is_key_pressed(KeyCode::A));
    }

    #[test]
    fn multiple_keys() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::KeyPressed(KeyCode::W));
        state.process_event(&InputEvent::KeyPressed(KeyCode::LShift));

        assert!(state.is_key_pressed(KeyCode::W));
        assert!(state.is_key_pressed(KeyCode::LShift));
        assert!(!state.is_key_pressed(KeyCode::S));
    }

    #[test]
    fn default_mouse_position() {
        let state = InputState::new();
        assert_eq!(state.mouse_position(), (0.0, 0.0));
    }

    #[test]
    fn serde_input_event() {
        let event = InputEvent::KeyPressed(KeyCode::Escape);
        let json = serde_json::to_string(&event).unwrap();
        let decoded: InputEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn serde_mouse_event() {
        let event = InputEvent::MouseMoved { x: 42.0, y: 99.0 };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: InputEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }
}
