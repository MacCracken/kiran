//! Audio integration via dhvani, naad, shravan, goonj, garjan, and ghurni
//!
//! Bridges the AGNOS audio stack with kiran's ECS:
//! - **dhvani** — Audio engine (graph processor, clock, playback)
//! - **naad** — Synthesis primitives (oscillators, filters, envelopes, voice pools)
//! - **shravan** — Audio codecs (WAV, FLAC, Ogg, MP3, Opus, resampling)
//! - **goonj** — Spatial acoustics (occlusion, reverb, ambisonics) — see [`crate::acoustics`]
//! - **garjan** — Environmental sound synthesis (weather, impacts, footsteps, water)
//! - **ghurni** — Mechanical sound synthesis (engines, gears, motors, turbines)
//!
//! Core types:
//! - [`AudioEngine`] resource wrapping dhvani's graph processor
//! - [`SoundSource`] component for entities that emit sound
//! - [`AudioListener`] component for the entity that "hears" (usually the camera)
//! - [`SoundTrigger`] component for event-driven audio playback
//! - [`EnvironmentSound`] component for procedural environmental audio
//! - [`MechanicalSound`] component for RPM-driven mechanical audio

use serde::{Deserialize, Serialize};

use crate::world::{Entity, EventBus, World};

// ---------------------------------------------------------------------------
// naad — synthesis primitives
// ---------------------------------------------------------------------------

/// Dynamics processing (compressor, limiter, gate).
pub use naad::dynamics;
/// Audio effects (delay, chorus, flanger, etc.).
pub use naad::effects;
/// ADSR envelope generators.
pub use naad::envelope;
/// Equalizer (parametric EQ bands).
pub use naad::eq;
/// Audio filters (biquad, low-pass, high-pass, band-pass, etc.).
pub use naad::filter as synth_filter;
/// Modulation sources and routing.
pub use naad::modulation;
/// Noise generators (white, pink, brown).
pub use naad::noise;
/// Audio synthesis oscillators and waveforms.
pub use naad::oscillator;
/// Spatial panning utilities.
pub use naad::panning;
/// Reverb algorithms.
pub use naad::reverb;
/// Voice pool management for polyphonic synthesis.
pub use naad::voice;
/// Wavetable synthesis.
pub use naad::wavetable;

// ---------------------------------------------------------------------------
// shravan — audio codecs
// ---------------------------------------------------------------------------

/// Audio codec detection and decoding.
pub use shravan::codec;
/// Audio format metadata (sample rate, channels, duration, bit depth).
pub use shravan::format;

// ---------------------------------------------------------------------------
// garjan — environmental sound synthesis
// ---------------------------------------------------------------------------

/// Fire and combustion sounds.
pub use garjan::fire;
/// Footstep synthesis.
pub use garjan::footstep;
/// Impact sounds (hits, drops, crashes).
pub use garjan::impact;
/// Ambient texture loops.
pub use garjan::texture as ambient_texture;
/// Water sounds (surf, streams, underwater).
pub use garjan::water;
/// Weather sounds (thunder, rain, wind).
pub use garjan::weather;

// ---------------------------------------------------------------------------
// ghurni — mechanical sound synthesis
// ---------------------------------------------------------------------------

/// Internal combustion engine synthesis.
pub use ghurni::engine as mech_engine;
/// Gear and transmission sounds.
pub use ghurni::gear;
/// Mechanical sound mixer.
pub use ghurni::mixer as mech_mixer;
/// Electric motor synthesis.
pub use ghurni::motor;
/// Synthesizer trait for all mechanical sources.
pub use ghurni::traits as mech_traits;
/// Turbine synthesis.
pub use ghurni::turbine;

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
    /// Playback pitch/speed (1.0 = normal, 2.0 = double speed).
    #[serde(default = "default_pitch")]
    pub pitch: f32,
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
    /// Which mix bus this sound belongs to.
    #[serde(default)]
    pub bus: MixBus,
    /// Fade state (0.0 = silent, 1.0 = full, transitioning).
    #[serde(skip)]
    pub fade: f32,
}

fn default_pitch() -> f32 {
    1.0
}

fn default_max_distance() -> f32 {
    50.0
}

impl Default for SoundSource {
    fn default() -> Self {
        Self {
            source: String::new(),
            volume: 1.0,
            pitch: 1.0,
            spatial: true,
            looping: false,
            playing: false,
            max_distance: 50.0,
            bus: MixBus::SFX,
            fade: 1.0,
        }
    }
}

impl SoundSource {
    /// Create a new sound source from a file path.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            ..Default::default()
        }
    }

    /// Set playback volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Enable looping playback.
    pub fn with_looping(mut self) -> Self {
        self.looping = true;
        self
    }

    /// Disable spatial audio (play as 2D).
    pub fn non_spatial(mut self) -> Self {
        self.spatial = false;
        self
    }

    /// Set maximum audible distance.
    pub fn with_max_distance(mut self, dist: f32) -> Self {
        self.max_distance = dist;
        self
    }

    /// Set playback pitch/speed.
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    /// Set the mix bus for this source.
    pub fn with_bus(mut self, bus: MixBus) -> Self {
        self.bus = bus;
        self
    }

    /// Start a fade in (from 0 to 1).
    pub fn fade_in(&mut self) {
        self.fade = 0.0;
        self.playing = true;
    }

    /// Start a fade out (fade will decrease toward 0 via step_fade).
    pub fn fade_out(&mut self) {
        // Mark fade direction as decreasing — step_fade handles both directions
        self.fade = self.fade.min(1.0 - f32::EPSILON);
    }

    /// Step the fade by dt. Returns effective volume (volume * fade).
    /// Fades in when fade < 1.0 after fade_in(), fades out after fade_out().
    #[inline]
    pub fn step_fade(&mut self, dt: f32, duration: f32) -> f32 {
        if self.fade < 1.0 {
            self.fade = (self.fade + dt / duration).min(1.0);
        }
        self.volume * self.fade
    }
}

/// Sound pool — limits concurrent sounds of the same type.
#[derive(Debug, Clone)]
pub struct SoundPool {
    /// Maximum concurrent sounds.
    pub max_voices: usize,
    /// Currently playing count.
    pub active: usize,
}

impl SoundPool {
    /// Create a pool with the given voice limit.
    pub fn new(max_voices: usize) -> Self {
        Self {
            max_voices,
            active: 0,
        }
    }

    /// Try to acquire a voice. Returns true if allowed.
    pub fn try_play(&mut self) -> bool {
        if self.active < self.max_voices {
            self.active += 1;
            true
        } else {
            false
        }
    }

    /// Release a voice.
    pub fn release(&mut self) {
        self.active = self.active.saturating_sub(1);
    }

    /// Is the pool full?
    #[must_use]
    #[inline]
    pub fn is_full(&self) -> bool {
        self.active >= self.max_voices
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
#[non_exhaustive]
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
    /// What event triggers this sound.
    pub kind: TriggerKind,
    /// Path to the audio file.
    pub source: String,
    /// Playback volume (0.0–1.0).
    pub volume: f32,
}

impl SoundTrigger {
    /// Create a trigger that fires on collision start.
    pub fn on_collision(source: impl Into<String>) -> Self {
        Self {
            kind: TriggerKind::CollisionStart,
            source: source.into(),
            volume: 1.0,
        }
    }

    /// Create a trigger that fires on a named action.
    pub fn on_action(action: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            kind: TriggerKind::Action(action.into()),
            source: source.into(),
            volume: 1.0,
        }
    }

    /// Set trigger playback volume.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MixBus {
    /// Master bus — scales all other buses.
    Master,
    /// Background music.
    Music,
    /// Sound effects (default).
    #[default]
    SFX,
    /// Ambient/environmental sounds.
    Ambient,
    /// Character dialogue.
    Dialogue,
    /// UI feedback sounds.
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
    /// Create default mix bus volumes.
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
    /// The action name that was triggered.
    pub action: String,
    /// The entity that triggered the action.
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
// Environmental sound component
// ---------------------------------------------------------------------------

/// Environmental sound source — procedural audio driven by garjan.
///
/// Attach to an entity to generate weather, impact, water, or ambient sounds.
/// The `kind` field selects the synthesis model; `intensity` and `variation`
/// control the sound character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSound {
    /// What type of environmental sound this produces.
    pub kind: EnvironmentSoundKind,
    /// Intensity (0.0 = silent, 1.0 = full).
    pub intensity: f32,
    /// Random variation seed (different values produce different textures).
    pub variation: u64,
    /// Playback volume.
    pub volume: f32,
    /// Whether this source is active.
    pub active: bool,
}

/// Kind of environmental sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EnvironmentSoundKind {
    /// Thunder crack/rumble.
    Thunder,
    /// Rain (light to heavy).
    Rain,
    /// Wind (breeze to gale).
    Wind,
    /// Fire/combustion.
    Fire,
    /// Water (surf, streams, drips).
    Water,
    /// Ambient texture loop.
    Ambient,
    /// Footsteps on terrain.
    Footstep,
    /// Object impact.
    Impact,
}

impl EnvironmentSound {
    /// Create a new environmental sound of the given kind.
    pub fn new(kind: EnvironmentSoundKind) -> Self {
        Self {
            kind,
            intensity: 1.0,
            variation: 0,
            volume: 1.0,
            active: true,
        }
    }

    /// Set the intensity.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// Set the variation seed.
    pub fn with_variation(mut self, variation: u64) -> Self {
        self.variation = variation;
        self
    }

    /// Set playback volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
}

// ---------------------------------------------------------------------------
// Mechanical sound component
// ---------------------------------------------------------------------------

/// Mechanical sound source — RPM-driven audio driven by ghurni.
///
/// Attach to an entity representing a machine (vehicle, motor, fan, etc.).
/// The synthesis model produces sound based on `rpm` and `load`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanicalSound {
    /// What type of machine this represents.
    pub kind: MechanicalSoundKind,
    /// Revolutions per minute.
    pub rpm: f32,
    /// Mechanical load factor (0.0 = idle, 1.0 = full load).
    pub load: f32,
    /// Playback volume.
    pub volume: f32,
    /// Whether this source is active.
    pub active: bool,
}

/// Kind of mechanical sound source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MechanicalSoundKind {
    /// Internal combustion engine.
    Engine,
    /// Electric motor.
    Motor,
    /// Gear/transmission.
    Gear,
    /// Turbine.
    Turbine,
    /// Chain drive.
    Chain,
    /// Belt drive.
    Belt,
}

impl MechanicalSound {
    /// Create a new mechanical sound of the given kind.
    pub fn new(kind: MechanicalSoundKind) -> Self {
        Self {
            kind,
            rpm: 0.0,
            load: 0.0,
            volume: 1.0,
            active: true,
        }
    }

    /// Set the RPM.
    pub fn with_rpm(mut self, rpm: f32) -> Self {
        self.rpm = rpm;
        self
    }

    /// Set the load factor.
    pub fn with_load(mut self, load: f32) -> Self {
        self.load = load;
        self
    }

    /// Set playback volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
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
