//! Animation state machine — blend trees and state transitions.
//!
//! Provides an [`AnimState`] ECS component that manages animation playback
//! and transitions between clips. Works with soorat's `AnimationClip` and
//! `Skeleton` types.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Animation state machine
// ---------------------------------------------------------------------------

/// A named animation state referencing a clip by index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimNode {
    /// Human-readable state name.
    pub name: String,
    /// Index into the animation clip list.
    pub clip_index: usize,
    /// Playback speed multiplier.
    pub speed: f32,
    /// Whether the clip loops.
    pub looping: bool,
}

impl AnimNode {
    /// Create a looping animation node.
    #[must_use]
    pub fn new(name: impl Into<String>, clip_index: usize) -> Self {
        Self {
            name: name.into(),
            clip_index,
            speed: 1.0,
            looping: true,
        }
    }

    /// Create a non-looping (one-shot) animation node.
    #[must_use]
    pub fn once(name: impl Into<String>, clip_index: usize) -> Self {
        Self {
            name: name.into(),
            clip_index,
            speed: 1.0,
            looping: false,
        }
    }
}

/// A transition between two animation states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimTransition {
    /// Source state index.
    pub from: usize,
    /// Target state index.
    pub to: usize,
    /// Crossfade duration in seconds.
    pub duration: f32,
    /// Parameter name that triggers this transition.
    pub trigger: String,
}

/// ECS component: animation state machine for an entity.
///
/// Manages which animation clip is active, crossfade blending between
/// states, and parameter-driven transitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimState {
    /// All states in the machine.
    pub nodes: Vec<AnimNode>,
    /// Transitions between states.
    pub transitions: Vec<AnimTransition>,
    /// Current active state index.
    pub current: usize,
    /// Playback time in the current clip (seconds).
    pub time: f32,
    /// Blend target (if transitioning).
    blend_target: Option<usize>,
    /// Blend progress (0.0 = source, 1.0 = target).
    blend_alpha: f32,
    /// Blend duration for the active transition.
    blend_duration: f32,
    /// Named bool parameters that drive transitions.
    params: Vec<(String, bool)>,
}

impl AnimState {
    /// Create an empty animation state machine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a state node, returns its index.
    pub fn add_node(&mut self, node: AnimNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    /// Add a transition.
    pub fn add_transition(
        &mut self,
        from: usize,
        to: usize,
        duration: f32,
        trigger: impl Into<String>,
    ) {
        self.transitions.push(AnimTransition {
            from,
            to,
            duration,
            trigger: trigger.into(),
        });
    }

    /// Set a parameter value.
    pub fn set_param(&mut self, name: &str, value: bool) {
        if let Some(p) = self.params.iter_mut().find(|(n, _)| n == name) {
            p.1 = value;
        } else {
            self.params.push((name.to_string(), value));
        }
    }

    /// Get a parameter value.
    #[must_use]
    pub fn get_param(&self, name: &str) -> bool {
        self.params
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| *v)
            .unwrap_or(false)
    }

    /// Advance the animation by dt seconds.
    ///
    /// Checks for triggered transitions, advances blend, updates playback time.
    /// Returns the current clip index (and optionally blend target + alpha).
    #[must_use]
    pub fn tick(&mut self, dt: f32) -> AnimPlayback {
        // Check for new transitions
        if self.blend_target.is_none() {
            for t in &self.transitions {
                if t.from == self.current && self.get_param(&t.trigger) {
                    self.blend_target = Some(t.to);
                    self.blend_alpha = 0.0;
                    self.blend_duration = t.duration;
                    break;
                }
            }
        }

        // Advance blend
        if let Some(target) = self.blend_target {
            if self.blend_duration <= 0.0 {
                self.current = target;
                self.blend_target = None;
                self.blend_alpha = 0.0;
                self.time = 0.0;
                // Reset trigger
                self.reset_trigger_for(target);
            } else {
                self.blend_alpha += dt / self.blend_duration;
                if self.blend_alpha >= 1.0 {
                    self.current = target;
                    self.blend_target = None;
                    self.blend_alpha = 0.0;
                    self.time = 0.0;
                    self.reset_trigger_for(target);
                }
            }
        }

        // Advance playback time
        if let Some(node) = self.nodes.get(self.current) {
            self.time += dt * node.speed;
        }

        AnimPlayback {
            clip_index: self
                .nodes
                .get(self.current)
                .map(|n| n.clip_index)
                .unwrap_or(0),
            time: self.time,
            blend: self.blend_target.map(|t| {
                let clip = self.nodes.get(t).map(|n| n.clip_index).unwrap_or(0);
                (clip, self.blend_alpha)
            }),
        }
    }

    /// Current state name.
    #[must_use]
    pub fn current_name(&self) -> &str {
        self.nodes
            .get(self.current)
            .map(|n| n.name.as_str())
            .unwrap_or("")
    }

    /// Whether a blend transition is in progress.
    #[must_use]
    #[inline]
    pub fn is_blending(&self) -> bool {
        self.blend_target.is_some()
    }

    /// Number of states.
    #[must_use]
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn reset_trigger_for(&mut self, target: usize) {
        if let Some(trigger) = self
            .transitions
            .iter()
            .find(|t| t.to == target)
            .map(|t| t.trigger.clone())
        {
            self.set_param(&trigger, false);
        }
    }
}

/// Result of an animation tick — what to sample.
#[derive(Debug, Clone)]
pub struct AnimPlayback {
    /// Primary clip index to sample.
    pub clip_index: usize,
    /// Current time in the clip.
    pub time: f32,
    /// If blending: (target clip index, blend alpha 0..1).
    pub blend: Option<(usize, f32)>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anim_state_basic() {
        let mut state = AnimState::new();
        let idle = state.add_node(AnimNode::new("idle", 0));
        let walk = state.add_node(AnimNode::new("walk", 1));
        assert_eq!(idle, 0);
        assert_eq!(walk, 1);
        assert_eq!(state.node_count(), 2);
        assert_eq!(state.current_name(), "idle");
    }

    #[test]
    fn anim_state_tick_advances_time() {
        let mut state = AnimState::new();
        state.add_node(AnimNode::new("idle", 0));
        let playback = state.tick(0.5);
        assert!((playback.time - 0.5).abs() < 0.01);
        assert_eq!(playback.clip_index, 0);
        assert!(playback.blend.is_none());
    }

    #[test]
    fn anim_state_transition() {
        let mut state = AnimState::new();
        state.add_node(AnimNode::new("idle", 0));
        state.add_node(AnimNode::new("walk", 1));
        state.add_transition(0, 1, 0.5, "walk");

        // No trigger — stays idle
        let _ = state.tick(0.1);
        assert_eq!(state.current_name(), "idle");
        assert!(!state.is_blending());

        // Trigger transition
        state.set_param("walk", true);
        let playback = state.tick(0.1);
        assert!(state.is_blending());
        assert!(playback.blend.is_some());
        let (blend_clip, alpha) = playback.blend.unwrap();
        assert_eq!(blend_clip, 1);
        assert!(alpha > 0.0);
    }

    #[test]
    fn anim_state_transition_completes() {
        let mut state = AnimState::new();
        state.add_node(AnimNode::new("idle", 0));
        state.add_node(AnimNode::new("walk", 1));
        state.add_transition(0, 1, 0.2, "go");

        state.set_param("go", true);
        let _ = state.tick(0.1);
        assert!(state.is_blending());

        let _ = state.tick(0.15); // exceeds blend duration
        assert!(!state.is_blending());
        assert_eq!(state.current_name(), "walk");
    }

    #[test]
    fn anim_state_instant_transition() {
        let mut state = AnimState::new();
        state.add_node(AnimNode::new("idle", 0));
        state.add_node(AnimNode::new("run", 1));
        state.add_transition(0, 1, 0.0, "run");

        state.set_param("run", true);
        let _ = state.tick(0.01);
        assert_eq!(state.current_name(), "run");
        assert!(!state.is_blending());
    }

    #[test]
    fn anim_state_speed_multiplier() {
        let mut state = AnimState::new();
        let mut node = AnimNode::new("fast", 0);
        node.speed = 2.0;
        state.add_node(node);
        let playback = state.tick(0.5);
        assert!((playback.time - 1.0).abs() < 0.01);
    }

    #[test]
    fn anim_state_params() {
        let mut state = AnimState::new();
        assert!(!state.get_param("jump"));
        state.set_param("jump", true);
        assert!(state.get_param("jump"));
        state.set_param("jump", false);
        assert!(!state.get_param("jump"));
    }

    #[test]
    fn anim_state_trigger_resets_after_transition() {
        let mut state = AnimState::new();
        state.add_node(AnimNode::new("idle", 0));
        state.add_node(AnimNode::new("walk", 1));
        state.add_transition(0, 1, 0.1, "go");

        state.set_param("go", true);
        let _ = state.tick(0.2); // complete transition
        // Trigger should be reset
        assert!(!state.get_param("go"));
    }

    #[test]
    fn anim_state_serde_roundtrip() {
        let mut state = AnimState::new();
        state.add_node(AnimNode::new("idle", 0));
        state.add_node(AnimNode::new("walk", 1));
        state.add_transition(0, 1, 0.3, "go");

        let json = serde_json::to_string(&state).unwrap();
        let decoded: AnimState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.node_count(), 2);
        assert_eq!(decoded.transitions.len(), 1);
    }

    #[test]
    fn anim_node_once() {
        let node = AnimNode::once("attack", 3);
        assert!(!node.looping);
        assert_eq!(node.clip_index, 3);
    }

    #[test]
    fn anim_state_no_nodes() {
        let mut state = AnimState::new();
        let playback = state.tick(0.1);
        assert_eq!(playback.clip_index, 0);
        assert_eq!(state.current_name(), "");
    }

    #[test]
    fn anim_state_chain_transitions() {
        let mut state = AnimState::new();
        state.add_node(AnimNode::new("idle", 0));
        state.add_node(AnimNode::new("walk", 1));
        state.add_node(AnimNode::new("run", 2));
        state.add_transition(0, 1, 0.1, "walk");
        state.add_transition(1, 2, 0.1, "run");

        // idle → walk
        state.set_param("walk", true);
        let _ = state.tick(0.2);
        assert_eq!(state.current_name(), "walk");

        // walk → run
        state.set_param("run", true);
        let _ = state.tick(0.2);
        assert_eq!(state.current_name(), "run");
    }
}
