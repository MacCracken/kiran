//! Audio integration via dhvani
//!
//! Provides spatial audio for the game engine:
//! - [`AudioEngine`] resource wrapping dhvani's graph processor
//! - [`SoundSource`] component for entities that emit sound
//! - [`AudioListener`] component for the entity that "hears" (usually the camera)
//! - [`SoundTrigger`] component for event-driven audio playback

use serde::{Deserialize, Serialize};

use crate::world::{Entity, EventBus, World};

// ---------------------------------------------------------------------------
// Sound source component
// ---------------------------------------------------------------------------

/// A sound-emitting component attached to an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundSource {
    /// Path to the audio file.
    pub source: String,
    /// Playback volume (0.0 = silent, 1.0 = full).
    pub volume: f32,
    /// Whether this sound is spatial (affected by distance/panning).
    pub spatial: bool,
    /// Whether the sound loops.
    pub looping: bool,
    /// Playback state.
    #[serde(skip)]
    pub playing: bool,
    /// Maximum audible distance (spatial only).
    #[serde(default = "default_max_distance")]
    pub max_distance: f32,
}

fn default_max_distance() -> f32 {
    50.0
}

impl Default for SoundSource {
    fn default() -> Self {
        Self {
            source: String::new(),
            volume: 1.0,
            spatial: true,
            looping: false,
            playing: false,
            max_distance: 50.0,
        }
    }
}

impl SoundSource {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            ..Default::default()
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn with_looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn non_spatial(mut self) -> Self {
        self.spatial = false;
        self
    }

    pub fn with_max_distance(mut self, dist: f32) -> Self {
        self.max_distance = dist;
        self
    }
}

// ---------------------------------------------------------------------------
// Audio listener
// ---------------------------------------------------------------------------

/// Marks an entity as the audio listener (where sound is "heard" from).
/// Typically attached to the camera entity. Only one listener should be active.
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioListener;

// ---------------------------------------------------------------------------
// Sound triggers
// ---------------------------------------------------------------------------

/// What kind of event triggers a sound.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerKind {
    /// Play when a collision starts with this entity.
    CollisionStart,
    /// Play when a collision stops.
    CollisionStop,
    /// Play on a named custom action.
    Action(String),
}

/// Links an event to a sound effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundTrigger {
    pub kind: TriggerKind,
    pub source: String,
    pub volume: f32,
}

impl SoundTrigger {
    pub fn on_collision(source: impl Into<String>) -> Self {
        Self {
            kind: TriggerKind::CollisionStart,
            source: source.into(),
            volume: 1.0,
        }
    }

    pub fn on_action(action: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            kind: TriggerKind::Action(action.into()),
            source: source.into(),
            volume: 1.0,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
}

// ---------------------------------------------------------------------------
// Audio engine resource
// ---------------------------------------------------------------------------

/// The audio engine resource — manages dhvani audio graph and playback.
pub struct AudioEngine {
    /// Master volume (0.0–1.0).
    pub master_volume: f32,
    /// dhvani audio graph for processing.
    graph: dhvani::graph::Graph,
    /// Audio clock for synchronization.
    clock: dhvani::AudioClock,
    /// Sample rate.
    sample_rate: u32,
    /// Buffer size in frames.
    buffer_size: usize,
    /// Listener entity (if set).
    listener: Option<Entity>,
}

impl AudioEngine {
    /// Create a new audio engine with the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            master_volume: 1.0,
            graph: dhvani::graph::Graph::new(),
            clock: {
                let mut c = dhvani::AudioClock::new(sample_rate);
                c.start();
                c
            },
            sample_rate,
            buffer_size: 1024,
            listener: None,
        }
    }

    /// Set the listener entity.
    pub fn set_listener(&mut self, entity: Entity) {
        self.listener = Some(entity);
    }

    /// Get the current listener entity.
    pub fn listener(&self) -> Option<Entity> {
        self.listener
    }

    /// Get the sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the buffer size.
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Get the current playback position in seconds.
    pub fn position_secs(&self) -> f64 {
        self.clock.position_secs()
    }

    /// Access the underlying dhvani graph.
    pub fn graph(&self) -> &dhvani::graph::Graph {
        &self.graph
    }

    /// Mutably access the underlying dhvani graph.
    pub fn graph_mut(&mut self) -> &mut dhvani::graph::Graph {
        &mut self.graph
    }

    /// Advance the audio clock by one buffer.
    pub fn advance(&mut self) {
        self.clock.advance(self.buffer_size as u64);
    }

    /// Compute spatial gain for a sound source based on distance from listener.
    pub fn spatial_gain(distance: f32, max_distance: f32) -> f32 {
        if distance >= max_distance {
            0.0
        } else {
            (1.0 - distance / max_distance).clamp(0.0, 1.0)
        }
    }

    /// Compute stereo pan value (-1.0 left, 0.0 center, 1.0 right) from a relative position.
    pub fn spatial_pan(relative_x: f32, max_distance: f32) -> f32 {
        if max_distance <= 0.0 {
            return 0.0;
        }
        (relative_x / max_distance).clamp(-1.0, 1.0)
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new(44100)
    }
}

// ---------------------------------------------------------------------------
// Mix buses
// ---------------------------------------------------------------------------

/// Audio mix bus identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MixBus {
    Master,
    Music,
    SFX,
    Ambient,
    Dialogue,
    UI,
}

/// Volume settings per mix bus.
#[derive(Debug, Clone)]
pub struct MixBusVolumes {
    volumes: std::collections::HashMap<MixBus, f32>,
}

impl Default for MixBusVolumes {
    fn default() -> Self {
        let mut volumes = std::collections::HashMap::new();
        volumes.insert(MixBus::Master, 1.0);
        volumes.insert(MixBus::Music, 0.7);
        volumes.insert(MixBus::SFX, 1.0);
        volumes.insert(MixBus::Ambient, 0.5);
        volumes.insert(MixBus::Dialogue, 1.0);
        volumes.insert(MixBus::UI, 0.8);
        Self { volumes }
    }
}

impl MixBusVolumes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the volume for a bus (0.0–1.0).
    pub fn set(&mut self, bus: MixBus, volume: f32) {
        self.volumes.insert(bus, volume.clamp(0.0, 1.0));
    }

    /// Get the volume for a bus.
    pub fn get(&self, bus: MixBus) -> f32 {
        self.volumes.get(&bus).copied().unwrap_or(1.0)
    }

    /// Get the effective volume for a bus (bus volume * master volume).
    pub fn effective(&self, bus: MixBus) -> f32 {
        if bus == MixBus::Master {
            self.get(MixBus::Master)
        } else {
            self.get(bus) * self.get(MixBus::Master)
        }
    }

    /// Mute a bus (set to 0).
    pub fn mute(&mut self, bus: MixBus) {
        self.volumes.insert(bus, 0.0);
    }
}

// ---------------------------------------------------------------------------
// Sound action event
// ---------------------------------------------------------------------------

/// Event published when a named action triggers a sound.
#[derive(Debug, Clone)]
pub struct SoundActionEvent {
    pub action: String,
    pub entity: Entity,
}

/// Process sound triggers from the event bus.
/// Call this each frame to check for collision/action events and mark sounds for playback.
pub fn process_sound_triggers(world: &mut World) {
    // Check for collision events if physics feature is enabled
    #[cfg(feature = "physics")]
    {
        use crate::physics::PhysicsCollisionEvent;

        if let Some(bus) = world.get_resource_mut::<EventBus>() {
            let collision_events = bus.drain::<PhysicsCollisionEvent>();
            for event in collision_events {
                match event {
                    PhysicsCollisionEvent::Started { entity_a, entity_b } => {
                        // Check if either entity has a collision trigger
                        for entity in [entity_a, entity_b] {
                            apply_trigger(world, entity, TriggerKind::CollisionStart);
                        }
                    }
                    PhysicsCollisionEvent::Stopped { entity_a, entity_b } => {
                        for entity in [entity_a, entity_b] {
                            apply_trigger(world, entity, TriggerKind::CollisionStop);
                        }
                    }
                }
            }
        }
    }

    // Check for action events
    if let Some(bus) = world.get_resource_mut::<EventBus>() {
        let action_events = bus.drain::<SoundActionEvent>();
        for event in action_events {
            apply_trigger(world, event.entity, TriggerKind::Action(event.action));
        }
    }
}

/// Apply a trigger of the given kind to an entity, updating its SoundSource.
fn apply_trigger(world: &mut World, entity: Entity, kind: TriggerKind) {
    // Compare kind first (cheap) before cloning the trigger (allocates String)
    let matches = world
        .get_component::<SoundTrigger>(entity)
        .is_some_and(|t| t.kind == kind);
    if !matches {
        return;
    }
    // Now clone only the fields we need
    let trigger = world.get_component::<SoundTrigger>(entity).unwrap();
    let source_path = trigger.source.clone();
    let volume = trigger.volume;

    if let Some(source) = world.get_component_mut::<SoundSource>(entity) {
        source.source = source_path;
        source.volume = volume;
        source.playing = true;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_source_builder() {
        let src = SoundSource::new("sounds/hit.wav")
            .with_volume(0.8)
            .with_looping()
            .with_max_distance(100.0);
        assert_eq!(src.source, "sounds/hit.wav");
        assert_eq!(src.volume, 0.8);
        assert!(src.looping);
        assert!(src.spatial);
        assert_eq!(src.max_distance, 100.0);
    }

    #[test]
    fn sound_source_non_spatial() {
        let src = SoundSource::new("music/theme.ogg").non_spatial();
        assert!(!src.spatial);
    }

    #[test]
    fn sound_source_default() {
        let src = SoundSource::default();
        assert!(src.source.is_empty());
        assert_eq!(src.volume, 1.0);
        assert!(src.spatial);
        assert!(!src.looping);
        assert!(!src.playing);
    }

    #[test]
    fn sound_trigger_collision() {
        let trigger = SoundTrigger::on_collision("sounds/bump.wav").with_volume(0.5);
        assert_eq!(trigger.kind, TriggerKind::CollisionStart);
        assert_eq!(trigger.volume, 0.5);
    }

    #[test]
    fn sound_trigger_action() {
        let trigger = SoundTrigger::on_action("jump", "sounds/jump.wav");
        assert_eq!(trigger.kind, TriggerKind::Action("jump".into()));
    }

    #[test]
    fn audio_engine_defaults() {
        let engine = AudioEngine::default();
        assert_eq!(engine.sample_rate(), 44100);
        assert_eq!(engine.buffer_size(), 1024);
        assert_eq!(engine.master_volume, 1.0);
        assert!(engine.listener().is_none());
    }

    #[test]
    fn audio_engine_set_listener() {
        let mut engine = AudioEngine::new(48000);
        let entity = Entity::new(5, 0);
        engine.set_listener(entity);
        assert_eq!(engine.listener(), Some(entity));
    }

    #[test]
    fn audio_engine_advance() {
        let mut engine = AudioEngine::new(44100);
        assert_eq!(engine.position_secs(), 0.0);
        engine.advance();
        assert!(engine.position_secs() > 0.0);
    }

    #[test]
    fn spatial_gain_calculation() {
        assert_eq!(AudioEngine::spatial_gain(0.0, 50.0), 1.0);
        assert_eq!(AudioEngine::spatial_gain(25.0, 50.0), 0.5);
        assert_eq!(AudioEngine::spatial_gain(50.0, 50.0), 0.0);
        assert_eq!(AudioEngine::spatial_gain(100.0, 50.0), 0.0);
    }

    #[test]
    fn spatial_pan_calculation() {
        assert_eq!(AudioEngine::spatial_pan(0.0, 50.0), 0.0);
        assert!((AudioEngine::spatial_pan(25.0, 50.0) - 0.5).abs() < f32::EPSILON);
        assert_eq!(AudioEngine::spatial_pan(-25.0, 50.0), -0.5);
        // Clamped
        assert_eq!(AudioEngine::spatial_pan(100.0, 50.0), 1.0);
        assert_eq!(AudioEngine::spatial_pan(-100.0, 50.0), -1.0);
        // Zero max distance
        assert_eq!(AudioEngine::spatial_pan(10.0, 0.0), 0.0);
    }

    #[test]
    fn sound_source_serde_roundtrip() {
        let src = SoundSource::new("test.wav").with_volume(0.7).with_looping();
        let json = serde_json::to_string(&src).unwrap();
        let decoded: SoundSource = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.source, "test.wav");
        assert_eq!(decoded.volume, 0.7);
        assert!(decoded.looping);
        // playing is skipped in serde
        assert!(!decoded.playing);
    }

    #[test]
    fn action_event_triggers_sound() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let entity = world.spawn();
        world
            .insert_component(entity, SoundSource::new("sounds/jump.wav").with_volume(0.5))
            .unwrap();
        world
            .insert_component(entity, SoundTrigger::on_action("jump", "sounds/jump.wav"))
            .unwrap();

        // Publish action event
        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(SoundActionEvent {
                action: "jump".into(),
                entity,
            });
        }

        process_sound_triggers(&mut world);

        let source = world.get_component::<SoundSource>(entity).unwrap();
        assert!(source.playing);
    }

    #[test]
    fn sound_source_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, SoundSource::new("ambient.wav"))
            .unwrap();

        assert!(world.has_component::<SoundSource>(e));
        let src = world.get_component::<SoundSource>(e).unwrap();
        assert_eq!(src.source, "ambient.wav");
    }

    #[test]
    fn action_trigger_updates_source_path() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let entity = world.spawn();
        world
            .insert_component(entity, SoundSource::new("placeholder.wav"))
            .unwrap();
        world
            .insert_component(
                entity,
                SoundTrigger::on_action("attack", "sounds/sword.wav"),
            )
            .unwrap();

        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(SoundActionEvent {
                action: "attack".into(),
                entity,
            });
        }

        process_sound_triggers(&mut world);

        let source = world.get_component::<SoundSource>(entity).unwrap();
        assert!(source.playing);
        // Source path should be updated from trigger
        assert_eq!(source.source, "sounds/sword.wav");
    }

    #[test]
    fn wrong_action_does_not_trigger() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let entity = world.spawn();
        world
            .insert_component(entity, SoundSource::new("idle.wav"))
            .unwrap();
        world
            .insert_component(entity, SoundTrigger::on_action("jump", "sounds/jump.wav"))
            .unwrap();

        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(SoundActionEvent {
                action: "attack".into(), // wrong action
                entity,
            });
        }

        process_sound_triggers(&mut world);

        let source = world.get_component::<SoundSource>(entity).unwrap();
        assert!(!source.playing);
        assert_eq!(source.source, "idle.wav"); // unchanged
    }

    #[test]
    fn audio_listener_component() {
        let mut world = World::new();
        let cam = world.spawn();
        world.insert_component(cam, AudioListener).unwrap();
        assert!(world.has_component::<AudioListener>(cam));
    }

    #[test]
    fn sound_trigger_serde_roundtrip() {
        let trigger = SoundTrigger::on_action("jump", "sounds/jump.wav").with_volume(0.8);
        let json = serde_json::to_string(&trigger).unwrap();
        let decoded: SoundTrigger = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.kind, TriggerKind::Action("jump".into()));
        assert_eq!(decoded.source, "sounds/jump.wav");
        assert_eq!(decoded.volume, 0.8);
    }

    #[test]
    fn spatial_gain_negative_distance() {
        // Negative distance should still clamp to max 1.0
        assert_eq!(AudioEngine::spatial_gain(-5.0, 50.0), 1.0);
    }

    // -- Mix bus tests --

    #[test]
    fn mix_bus_defaults() {
        let buses = MixBusVolumes::new();
        assert_eq!(buses.get(MixBus::Master), 1.0);
        assert_eq!(buses.get(MixBus::Music), 0.7);
        assert_eq!(buses.get(MixBus::SFX), 1.0);
    }

    #[test]
    fn mix_bus_set_get() {
        let mut buses = MixBusVolumes::new();
        buses.set(MixBus::Music, 0.3);
        assert_eq!(buses.get(MixBus::Music), 0.3);
    }

    #[test]
    fn mix_bus_effective_volume() {
        let mut buses = MixBusVolumes::new();
        buses.set(MixBus::Master, 0.5);
        buses.set(MixBus::SFX, 0.8);
        assert!((buses.effective(MixBus::SFX) - 0.4).abs() < f32::EPSILON);
        assert_eq!(buses.effective(MixBus::Master), 0.5);
    }

    #[test]
    fn mix_bus_mute() {
        let mut buses = MixBusVolumes::new();
        buses.mute(MixBus::Music);
        assert_eq!(buses.get(MixBus::Music), 0.0);
        assert_eq!(buses.effective(MixBus::Music), 0.0);
    }

    #[test]
    fn mix_bus_clamp() {
        let mut buses = MixBusVolumes::new();
        buses.set(MixBus::SFX, 2.0);
        assert_eq!(buses.get(MixBus::SFX), 1.0);
        buses.set(MixBus::SFX, -1.0);
        assert_eq!(buses.get(MixBus::SFX), 0.0);
    }

    #[test]
    fn mix_bus_serde() {
        let bus = MixBus::Dialogue;
        let json = serde_json::to_string(&bus).unwrap();
        let decoded: MixBus = serde_json::from_str(&json).unwrap();
        assert_eq!(bus, decoded);
    }
}
