//! Game state machine — menu → playing → paused transitions.

use serde::{Deserialize, Serialize};

/// A game state with enter/exit lifecycle.
pub trait GameState: Send {
    /// Return the state's display name.
    fn name(&self) -> &str;
    /// Called when transitioning into this state.
    fn on_enter(&mut self, _world: &mut crate::World) {}
    /// Called when transitioning out of this state.
    fn on_exit(&mut self, _world: &mut crate::World) {}
    /// Called each frame while this state is active.
    fn update(&mut self, _world: &mut crate::World) {}
}

/// State machine resource managing game state transitions.
pub struct StateMachine {
    states: Vec<Box<dyn GameState>>,
    current: usize,
    pending_transition: Option<usize>,
}

impl StateMachine {
    /// Create a state machine with the given initial state.
    ///
    /// # Examples
    ///
    /// ```
    /// use kiran::state::{StateMachine, NamedState};
    ///
    /// let mut sm = StateMachine::new(Box::new(NamedState("menu".into())));
    /// let playing = sm.add_state(Box::new(NamedState("playing".into())));
    /// assert_eq!(sm.current_name(), "menu");
    /// assert_eq!(sm.state_count(), 2);
    /// ```
    pub fn new(initial: Box<dyn GameState>) -> Self {
        Self {
            states: vec![initial],
            current: 0,
            pending_transition: None,
        }
    }

    /// Add a state to the machine. Returns its index.
    pub fn add_state(&mut self, state: Box<dyn GameState>) -> usize {
        let idx = self.states.len();
        self.states.push(state);
        idx
    }

    /// Request a transition to a state by index.
    pub fn transition_to(&mut self, index: usize) {
        if index < self.states.len() {
            self.pending_transition = Some(index);
        }
    }

    /// Apply pending transition (call between frames).
    pub fn apply_transition(&mut self, world: &mut crate::World) {
        if let Some(next) = self.pending_transition.take() {
            self.states[self.current].on_exit(world);
            self.current = next;
            self.states[self.current].on_enter(world);
        }
    }

    /// Update the current state.
    pub fn update(&mut self, world: &mut crate::World) {
        self.states[self.current].update(world);
    }

    /// Current state name.
    #[must_use]
    pub fn current_name(&self) -> &str {
        self.states[self.current].name()
    }

    /// Current state index.
    #[must_use]
    #[inline]
    pub fn current_index(&self) -> usize {
        self.current
    }

    /// Number of registered states.
    #[must_use]
    #[inline]
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
}

/// Simple named state (no custom logic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedState(pub String);

impl GameState for NamedState {
    fn name(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_basic() {
        let mut sm = StateMachine::new(Box::new(NamedState("menu".into())));
        let playing = sm.add_state(Box::new(NamedState("playing".into())));
        let paused = sm.add_state(Box::new(NamedState("paused".into())));

        assert_eq!(sm.current_name(), "menu");
        assert_eq!(sm.state_count(), 3);

        let mut world = crate::World::new();

        sm.transition_to(playing);
        sm.apply_transition(&mut world);
        assert_eq!(sm.current_name(), "playing");

        sm.transition_to(paused);
        sm.apply_transition(&mut world);
        assert_eq!(sm.current_name(), "paused");
    }

    #[test]
    fn state_machine_no_transition() {
        let mut sm = StateMachine::new(Box::new(NamedState("idle".into())));
        let mut world = crate::World::new();
        sm.apply_transition(&mut world); // no pending
        assert_eq!(sm.current_name(), "idle");
    }

    #[test]
    fn state_machine_invalid_index() {
        let mut sm = StateMachine::new(Box::new(NamedState("menu".into())));
        sm.transition_to(999); // invalid
        let mut world = crate::World::new();
        sm.apply_transition(&mut world);
        assert_eq!(sm.current_name(), "menu"); // unchanged
    }

    #[test]
    fn state_machine_update() {
        let mut sm = StateMachine::new(Box::new(NamedState("game".into())));
        let mut world = crate::World::new();
        sm.update(&mut world); // should not panic
    }
}
