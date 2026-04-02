//! Input handling: keyboard, mouse, gamepad
//!
//! Provides input event types and an [`InputState`] tracker that accumulates
//! events each frame and exposes query methods for pressed keys, mouse
//! position, and button state.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Key codes
// ---------------------------------------------------------------------------

/// Keyboard key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum KeyCode {
    // Letters
    /// The A key.
    A,
    /// The B key.
    B,
    /// The C key.
    C,
    /// The D key.
    D,
    /// The E key.
    E,
    /// The F key.
    F,
    /// The G key.
    G,
    /// The H key.
    H,
    /// The I key.
    I,
    /// The J key.
    J,
    /// The K key.
    K,
    /// The L key.
    L,
    /// The M key.
    M,
    /// The N key.
    N,
    /// The O key.
    O,
    /// The P key.
    P,
    /// The Q key.
    Q,
    /// The R key.
    R,
    /// The S key.
    S,
    /// The T key.
    T,
    /// The U key.
    U,
    /// The V key.
    V,
    /// The W key.
    W,
    /// The X key.
    X,
    /// The Y key.
    Y,
    /// The Z key.
    Z,

    // Digits
    /// The 0 key.
    Key0,
    /// The 1 key.
    Key1,
    /// The 2 key.
    Key2,
    /// The 3 key.
    Key3,
    /// The 4 key.
    Key4,
    /// The 5 key.
    Key5,
    /// The 6 key.
    Key6,
    /// The 7 key.
    Key7,
    /// The 8 key.
    Key8,
    /// The 9 key.
    Key9,

    // Arrows
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,

    // Modifiers
    /// Left Shift key.
    LShift,
    /// Right Shift key.
    RShift,
    /// Left Control key.
    LCtrl,
    /// Right Control key.
    RCtrl,
    /// Left Alt key.
    LAlt,
    /// Right Alt key.
    RAlt,
    /// Left Super (Windows/Command) key.
    LSuper,
    /// Right Super (Windows/Command) key.
    RSuper,

    // Function keys
    /// The F1 key.
    F1,
    /// The F2 key.
    F2,
    /// The F3 key.
    F3,
    /// The F4 key.
    F4,
    /// The F5 key.
    F5,
    /// The F6 key.
    F6,
    /// The F7 key.
    F7,
    /// The F8 key.
    F8,
    /// The F9 key.
    F9,
    /// The F10 key.
    F10,
    /// The F11 key.
    F11,
    /// The F12 key.
    F12,

    // Common keys
    /// Space bar.
    Space,
    /// Enter / Return key.
    Enter,
    /// Escape key.
    Escape,
    /// Tab key.
    Tab,
    /// Backspace key.
    Backspace,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Caps Lock key.
    CapsLock,

    // Punctuation / misc
    /// Minus / hyphen key.
    Minus,
    /// Equals key.
    Equals,
    /// Left bracket key.
    LeftBracket,
    /// Right bracket key.
    RightBracket,
    /// Backslash key.
    Backslash,
    /// Semicolon key.
    Semicolon,
    /// Apostrophe key.
    Apostrophe,
    /// Comma key.
    Comma,
    /// Period key.
    Period,
    /// Forward slash key.
    Slash,
    /// Grave accent / backtick key.
    Grave,
}

// ---------------------------------------------------------------------------
// Mouse
// ---------------------------------------------------------------------------

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
    /// Back mouse button.
    Back,
    /// Forward mouse button.
    Forward,
}

// ---------------------------------------------------------------------------
// Gamepad
// ---------------------------------------------------------------------------

/// Gamepad button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GamepadButton {
    /// A / Cross button.
    South,
    /// B / Circle button.
    East,
    /// X / Square button.
    West,
    /// Y / Triangle button.
    North,
    /// D-pad up.
    DPadUp,
    /// D-pad down.
    DPadDown,
    /// D-pad left.
    DPadLeft,
    /// D-pad right.
    DPadRight,
    /// Left bumper / L1.
    LeftBumper,
    /// Right bumper / R1.
    RightBumper,
    /// Left stick press / L3.
    LeftStick,
    /// Right stick press / R3.
    RightStick,
    /// Start / Options button.
    Start,
    /// Select / Share button.
    Select,
    /// Home / Guide button.
    Home,
}

/// Gamepad axis identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GamepadAxis {
    /// Left stick horizontal axis.
    LeftStickX,
    /// Left stick vertical axis.
    LeftStickY,
    /// Right stick horizontal axis.
    RightStickX,
    /// Right stick vertical axis.
    RightStickY,
    /// Left trigger / L2 axis.
    LeftTrigger,
    /// Right trigger / R2 axis.
    RightTrigger,
}

// ---------------------------------------------------------------------------
// Input events
// ---------------------------------------------------------------------------

/// An input event produced by the platform layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InputEvent {
    /// A key was pressed.
    KeyPressed(KeyCode),
    /// A key was released.
    KeyReleased(KeyCode),
    /// The mouse cursor moved.
    MouseMoved {
        /// Cursor X position.
        x: f64,
        /// Cursor Y position.
        y: f64,
    },
    /// A mouse button was pressed.
    MouseButtonPressed(MouseButton),
    /// A mouse button was released.
    MouseButtonReleased(MouseButton),
    /// The mouse scroll wheel moved.
    MouseScroll {
        /// Horizontal scroll delta.
        dx: f64,
        /// Vertical scroll delta.
        dy: f64,
    },
    /// A gamepad button was pressed.
    GamepadButtonPressed(GamepadButton),
    /// A gamepad button was released.
    GamepadButtonReleased(GamepadButton),
    /// A gamepad axis value changed.
    GamepadAxisMoved {
        /// The axis that moved.
        axis: GamepadAxis,
        /// New axis value (-1.0 to 1.0).
        value: f64,
    },
    /// Touch began/moved/ended.
    Touch {
        /// Unique touch point identifier.
        id: u64,
        /// Touch X position.
        x: f64,
        /// Touch Y position.
        y: f64,
        /// Current phase of the touch.
        phase: TouchPhase,
    },
    /// Text character input (for UI text fields).
    TextInput(char),
    /// Cursor lock/unlock request.
    CursorLock(bool),
}

/// Touch event phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TouchPhase {
    /// Touch began.
    Started,
    /// Touch point moved.
    Moved,
    /// Touch ended (finger lifted).
    Ended,
    /// Touch was cancelled by the system.
    Cancelled,
}

// ---------------------------------------------------------------------------
// Action mapping
// ---------------------------------------------------------------------------

/// A logical game action (e.g., "jump", "fire", "move_left").
pub type ActionName = String;

/// A physical input binding that triggers an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InputBinding {
    /// Keyboard key binding.
    Key(KeyCode),
    /// Mouse button binding.
    Mouse(MouseButton),
    /// Gamepad button binding.
    Gamepad(GamepadButton),
    /// Gamepad axis positive direction binding.
    GamepadAxisPositive(GamepadAxis),
    /// Gamepad axis negative direction binding.
    GamepadAxisNegative(GamepadAxis),
}

/// Maps logical actions to physical input bindings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionMap {
    bindings: HashMap<ActionName, Vec<InputBinding>>,
}

impl ActionMap {
    /// Create a new empty action map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a physical input to a logical action.
    pub fn bind(&mut self, action: impl Into<String>, binding: InputBinding) {
        self.bindings
            .entry(action.into())
            .or_default()
            .push(binding);
    }

    /// Check if an action is currently active (any binding pressed).
    pub fn is_action_pressed(&self, action: &str, state: &InputState) -> bool {
        let Some(bindings) = self.bindings.get(action) else {
            return false;
        };
        bindings.iter().any(|b| match b {
            InputBinding::Key(key) => state.is_key_pressed(*key),
            InputBinding::Mouse(btn) => state.is_mouse_button_pressed(*btn),
            InputBinding::Gamepad(btn) => state.is_gamepad_button_pressed(*btn),
            InputBinding::GamepadAxisPositive(axis) => state.gamepad_axis(*axis) > 0.5,
            InputBinding::GamepadAxisNegative(axis) => state.gamepad_axis(*axis) < -0.5,
        })
    }

    /// Check if an action was just pressed this frame.
    pub fn is_action_just_pressed(&self, action: &str, state: &InputState) -> bool {
        let Some(bindings) = self.bindings.get(action) else {
            return false;
        };
        bindings.iter().any(|b| match b {
            InputBinding::Key(key) => state.is_key_just_pressed(*key),
            InputBinding::Mouse(btn) => state.is_mouse_button_just_pressed(*btn),
            InputBinding::Gamepad(btn) => state.is_gamepad_button_just_pressed(*btn),
            _ => false,
        })
    }

    /// Get the axis value for an action (0.0 if not bound or not active).
    pub fn action_axis(&self, action: &str, state: &InputState) -> f64 {
        let Some(bindings) = self.bindings.get(action) else {
            return 0.0;
        };
        for b in bindings {
            match b {
                InputBinding::GamepadAxisPositive(axis)
                | InputBinding::GamepadAxisNegative(axis) => {
                    let v = state.gamepad_axis(*axis);
                    if v.abs() > 0.1 {
                        return v;
                    }
                }
                InputBinding::Key(key) => {
                    if state.is_key_pressed(*key) {
                        return 1.0;
                    }
                }
                _ => {}
            }
        }
        0.0
    }

    /// Number of registered actions.
    pub fn action_count(&self) -> usize {
        self.bindings.len()
    }
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
    mouse_dx: f64,
    mouse_dy: f64,
    mouse_initialized: bool,
    scroll_dx: f64,
    scroll_dy: f64,
    /// Keys pressed this frame (for "just pressed" queries).
    just_pressed: HashSet<KeyCode>,
    /// Keys released this frame.
    just_released: HashSet<KeyCode>,
    /// Mouse buttons pressed this frame.
    just_pressed_buttons: HashSet<MouseButton>,
    /// Mouse buttons released this frame.
    just_released_buttons: HashSet<MouseButton>,
    /// Gamepad buttons currently held.
    pressed_gamepad: HashSet<GamepadButton>,
    /// Gamepad buttons pressed this frame.
    just_pressed_gamepad: HashSet<GamepadButton>,
    /// Gamepad buttons released this frame.
    just_released_gamepad: HashSet<GamepadButton>,
    /// Gamepad axis values.
    gamepad_axes: HashMap<GamepadAxis, f64>,
    /// Active touch points.
    touches: HashMap<u64, (f64, f64, TouchPhase)>,
    /// Text input characters this frame.
    text_input: Vec<char>,
    /// Whether cursor is locked.
    cursor_locked: bool,
    /// Active input context (e.g., `"gameplay"`, `"menu"`, `"ui"`).
    pub context: String,
}

impl InputState {
    /// Create a new empty input state.
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
                if self.mouse_initialized {
                    self.mouse_dx += x - self.mouse_x;
                    self.mouse_dy += y - self.mouse_y;
                }
                self.mouse_x = *x;
                self.mouse_y = *y;
                self.mouse_initialized = true;
            }
            InputEvent::MouseButtonPressed(btn) => {
                if self.pressed_buttons.insert(*btn) {
                    self.just_pressed_buttons.insert(*btn);
                }
            }
            InputEvent::MouseButtonReleased(btn) => {
                self.pressed_buttons.remove(btn);
                self.just_released_buttons.insert(*btn);
            }
            InputEvent::MouseScroll { dx, dy } => {
                self.scroll_dx += dx;
                self.scroll_dy += dy;
            }
            InputEvent::GamepadButtonPressed(btn) => {
                if self.pressed_gamepad.insert(*btn) {
                    self.just_pressed_gamepad.insert(*btn);
                }
            }
            InputEvent::GamepadButtonReleased(btn) => {
                self.pressed_gamepad.remove(btn);
                self.just_released_gamepad.insert(*btn);
            }
            InputEvent::GamepadAxisMoved { axis, value } => {
                self.gamepad_axes.insert(*axis, *value);
            }
            InputEvent::Touch { id, x, y, phase } => match phase {
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    self.touches.remove(id);
                }
                _ => {
                    self.touches.insert(*id, (*x, *y, *phase));
                }
            },
            InputEvent::TextInput(ch) => {
                self.text_input.push(*ch);
            }
            InputEvent::CursorLock(locked) => {
                self.cursor_locked = *locked;
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

    /// Whether a mouse button was pressed this frame (edge-triggered).
    pub fn is_mouse_button_just_pressed(&self, btn: MouseButton) -> bool {
        self.just_pressed_buttons.contains(&btn)
    }

    /// Whether a mouse button was released this frame.
    pub fn is_mouse_button_just_released(&self, btn: MouseButton) -> bool {
        self.just_released_buttons.contains(&btn)
    }

    /// Current mouse position.
    pub fn mouse_position(&self) -> (f64, f64) {
        (self.mouse_x, self.mouse_y)
    }

    /// Whether a gamepad button is currently held.
    pub fn is_gamepad_button_pressed(&self, btn: GamepadButton) -> bool {
        self.pressed_gamepad.contains(&btn)
    }

    /// Whether a gamepad button was pressed this frame.
    pub fn is_gamepad_button_just_pressed(&self, btn: GamepadButton) -> bool {
        self.just_pressed_gamepad.contains(&btn)
    }

    /// Whether a gamepad button was released this frame.
    pub fn is_gamepad_button_just_released(&self, btn: GamepadButton) -> bool {
        self.just_released_gamepad.contains(&btn)
    }

    /// Get the current value of a gamepad axis (-1.0 to 1.0, 0.0 if not active).
    pub fn gamepad_axis(&self, axis: GamepadAxis) -> f64 {
        self.gamepad_axes.get(&axis).copied().unwrap_or(0.0)
    }

    /// Mouse movement delta this frame (for FPS camera control).
    pub fn mouse_delta(&self) -> (f64, f64) {
        (self.mouse_dx, self.mouse_dy)
    }

    /// Accumulated scroll delta this frame.
    pub fn scroll_delta(&self) -> (f64, f64) {
        (self.scroll_dx, self.scroll_dy)
    }

    /// Get active touch points.
    pub fn touches(&self) -> &HashMap<u64, (f64, f64, TouchPhase)> {
        &self.touches
    }

    /// Get text input characters this frame.
    pub fn text_input(&self) -> &[char] {
        &self.text_input
    }

    /// Whether the cursor is locked.
    pub fn is_cursor_locked(&self) -> bool {
        self.cursor_locked
    }

    /// Set the active input context (e.g., "gameplay", "menu").
    pub fn set_context(&mut self, context: impl Into<String>) {
        self.context = context.into();
    }

    /// Clear per-frame transient state (call at the start of each frame).
    pub fn clear_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.just_pressed_buttons.clear();
        self.just_released_buttons.clear();
        self.just_pressed_gamepad.clear();
        self.just_released_gamepad.clear();
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
        self.scroll_dx = 0.0;
        self.scroll_dy = 0.0;
        self.text_input.clear();
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

    #[test]
    fn mouse_button_edge_triggers() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::MouseButtonPressed(MouseButton::Left));
        assert!(state.is_mouse_button_just_pressed(MouseButton::Left));
        assert!(!state.is_mouse_button_just_released(MouseButton::Left));

        state.clear_frame();
        assert!(!state.is_mouse_button_just_pressed(MouseButton::Left));
        assert!(state.is_mouse_button_pressed(MouseButton::Left));

        state.process_event(&InputEvent::MouseButtonReleased(MouseButton::Left));
        assert!(state.is_mouse_button_just_released(MouseButton::Left));
        assert!(!state.is_mouse_button_pressed(MouseButton::Left));
    }

    #[test]
    fn all_mouse_button_variants() {
        let mut state = InputState::new();
        let buttons = [
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::Back,
            MouseButton::Forward,
        ];
        for btn in &buttons {
            state.process_event(&InputEvent::MouseButtonPressed(*btn));
        }
        for btn in &buttons {
            assert!(state.is_mouse_button_pressed(*btn));
        }
    }

    #[test]
    fn rapid_press_release_same_frame() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::KeyPressed(KeyCode::A));
        state.process_event(&InputEvent::KeyReleased(KeyCode::A));

        // Key was pressed and released in same frame
        assert!(!state.is_key_pressed(KeyCode::A));
        assert!(state.is_key_just_pressed(KeyCode::A));
        assert!(state.is_key_just_released(KeyCode::A));
    }

    #[test]
    fn serde_all_event_variants() {
        let events = vec![
            InputEvent::KeyPressed(KeyCode::A),
            InputEvent::KeyReleased(KeyCode::Z),
            InputEvent::MouseMoved { x: 0.0, y: 0.0 },
            InputEvent::MouseButtonPressed(MouseButton::Middle),
            InputEvent::MouseButtonReleased(MouseButton::Back),
            InputEvent::MouseScroll { dx: -1.5, dy: 2.5 },
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let decoded: InputEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(*event, decoded);
        }
    }

    #[test]
    fn mouse_delta() {
        let mut state = InputState::new();
        // First move initializes position, no delta
        state.process_event(&InputEvent::MouseMoved { x: 100.0, y: 200.0 });
        assert_eq!(state.mouse_delta(), (0.0, 0.0));

        state.clear_frame();

        // Second move produces delta
        state.process_event(&InputEvent::MouseMoved { x: 110.0, y: 205.0 });
        assert_eq!(state.mouse_delta(), (10.0, 5.0));

        state.clear_frame();
        assert_eq!(state.mouse_delta(), (0.0, 0.0));
    }

    #[test]
    fn mouse_delta_accumulates() {
        let mut state = InputState::new();
        // Initialize
        state.process_event(&InputEvent::MouseMoved { x: 10.0, y: 20.0 });
        // Subsequent moves accumulate delta
        state.process_event(&InputEvent::MouseMoved { x: 30.0, y: 25.0 });
        assert_eq!(state.mouse_delta(), (20.0, 5.0));
    }

    // -- Gamepad tests --

    #[test]
    fn gamepad_button_press_release() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::GamepadButtonPressed(GamepadButton::South));
        assert!(state.is_gamepad_button_pressed(GamepadButton::South));
        assert!(state.is_gamepad_button_just_pressed(GamepadButton::South));

        state.process_event(&InputEvent::GamepadButtonReleased(GamepadButton::South));
        assert!(!state.is_gamepad_button_pressed(GamepadButton::South));
        assert!(state.is_gamepad_button_just_released(GamepadButton::South));
    }

    #[test]
    fn gamepad_axis() {
        let mut state = InputState::new();
        assert_eq!(state.gamepad_axis(GamepadAxis::LeftStickX), 0.0);

        state.process_event(&InputEvent::GamepadAxisMoved {
            axis: GamepadAxis::LeftStickX,
            value: 0.75,
        });
        assert_eq!(state.gamepad_axis(GamepadAxis::LeftStickX), 0.75);
    }

    #[test]
    fn gamepad_clear_frame() {
        let mut state = InputState::new();
        state.process_event(&InputEvent::GamepadButtonPressed(GamepadButton::North));
        state.clear_frame();
        assert!(!state.is_gamepad_button_just_pressed(GamepadButton::North));
        assert!(state.is_gamepad_button_pressed(GamepadButton::North));
    }

    // -- Action mapping tests --

    #[test]
    fn action_map_bind_and_check() {
        let mut map = ActionMap::new();
        map.bind("jump", InputBinding::Key(KeyCode::Space));
        map.bind("jump", InputBinding::Gamepad(GamepadButton::South));
        assert_eq!(map.action_count(), 1);

        let mut state = InputState::new();
        assert!(!map.is_action_pressed("jump", &state));

        state.process_event(&InputEvent::KeyPressed(KeyCode::Space));
        assert!(map.is_action_pressed("jump", &state));
    }

    #[test]
    fn action_map_just_pressed() {
        let mut map = ActionMap::new();
        map.bind("fire", InputBinding::Mouse(MouseButton::Left));

        let mut state = InputState::new();
        state.process_event(&InputEvent::MouseButtonPressed(MouseButton::Left));
        assert!(map.is_action_just_pressed("fire", &state));

        state.clear_frame();
        assert!(!map.is_action_just_pressed("fire", &state));
    }

    #[test]
    fn action_map_gamepad_axis() {
        let mut map = ActionMap::new();
        map.bind(
            "move_right",
            InputBinding::GamepadAxisPositive(GamepadAxis::LeftStickX),
        );

        let mut state = InputState::new();
        assert_eq!(map.action_axis("move_right", &state), 0.0);

        state.process_event(&InputEvent::GamepadAxisMoved {
            axis: GamepadAxis::LeftStickX,
            value: 0.8,
        });
        assert!((map.action_axis("move_right", &state) - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn action_map_key_as_axis() {
        let mut map = ActionMap::new();
        map.bind("move_right", InputBinding::Key(KeyCode::D));

        let mut state = InputState::new();
        assert_eq!(map.action_axis("move_right", &state), 0.0);

        state.process_event(&InputEvent::KeyPressed(KeyCode::D));
        assert_eq!(map.action_axis("move_right", &state), 1.0);
    }

    #[test]
    fn action_map_unknown_action() {
        let map = ActionMap::new();
        let state = InputState::new();
        assert!(!map.is_action_pressed("nonexistent", &state));
        assert_eq!(map.action_axis("nonexistent", &state), 0.0);
    }

    #[test]
    fn action_map_serde() {
        let mut map = ActionMap::new();
        map.bind("jump", InputBinding::Key(KeyCode::Space));
        let json = serde_json::to_string(&map).unwrap();
        let decoded: ActionMap = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.action_count(), 1);
    }

    #[test]
    fn gamepad_button_serde() {
        let btn = GamepadButton::South;
        let json = serde_json::to_string(&btn).unwrap();
        let decoded: GamepadButton = serde_json::from_str(&json).unwrap();
        assert_eq!(btn, decoded);
    }

    #[test]
    fn gamepad_event_serde() {
        let event = InputEvent::GamepadAxisMoved {
            axis: GamepadAxis::RightTrigger,
            value: 0.5,
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: InputEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }
}
