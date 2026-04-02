//! Voice synthesis via svara, shabda, and prani
//!
//! Bridges the AGNOS voice stack with kiran's ECS:
//! - **svara** — Formant and vocal synthesis (glottal source, vocal tract, prosody)
//! - **shabda** — Grapheme-to-phoneme conversion (text → phoneme sequences)
//! - **prani** — Creature vocal synthesis (species-specific voices, emotion, fatigue)
//!
//! Core types:
//! - [`VoiceSource`] component for humanoid vocal synthesis
//! - [`CreatureVoiceSource`] component for creature vocalizations
//! - [`SpeechRequest`] event for triggering text-to-speech

use serde::{Deserialize, Serialize};

use crate::world::{Entity, World};

// ---------------------------------------------------------------------------
// svara — formant and vocal synthesis
// ---------------------------------------------------------------------------

/// Formant filter banks (resonant filtering for vowel/consonant shaping).
pub use svara::formant;
/// Glottal source models (pulse generation for voice excitation).
pub use svara::glottal;
/// Level-of-detail quality settings for synthesis.
pub use svara::lod as svara_lod;
/// Phoneme definitions and classification.
pub use svara::phoneme;
/// Synthesis thread pool.
pub use svara::pool as svara_pool;
/// Prosody contours (pitch, stress, intonation patterns).
pub use svara::prosody;
/// Batch rendering of phoneme sequences to audio buffers.
pub use svara::render as svara_render;
/// Phoneme sequencing and timing.
pub use svara::sequence;
/// Spectral analysis utilities.
pub use svara::spectral as svara_spectral;
/// Vocal tract modeling (nasal coupling, tract length, articulatory parameters).
pub use svara::tract;
/// Formant trajectory planning (smooth transitions between targets).
pub use svara::trajectory;
/// Voice profiles (speaker identity, effort, quality).
pub use svara::voice as svara_voice;

// ---------------------------------------------------------------------------
// shabda — grapheme-to-phoneme
// ---------------------------------------------------------------------------

/// G2P conversion engine (text → phoneme sequences).
pub use shabda::engine as g2p_engine;
/// Heteronym resolution (context-dependent pronunciation).
pub use shabda::heteronym;
/// Text normalization (numbers, abbreviations, punctuation).
pub use shabda::normalize;
/// Timing profiles for phoneme duration.
pub use shabda::prosody as g2p_prosody;
/// Phoneme rule sets.
pub use shabda::rules as g2p_rules;
/// SSML parsing for marked-up speech input.
pub use shabda::ssml;
/// Syllable segmentation.
pub use shabda::syllable;

// ---------------------------------------------------------------------------
// prani — creature vocal synthesis
// ---------------------------------------------------------------------------

/// Emotion state for vocal modulation.
pub use prani::emotion as creature_emotion;
/// Vocal fatigue modeling.
pub use prani::fatigue;
/// Voice presets for common species.
pub use prani::preset;
/// Call sequencing (bouts, phrases, patterns).
pub use prani::sequence as creature_sequence;
/// Species definitions and vocal tract parameters.
pub use prani::species;
/// Creature vocal tract modeling.
pub use prani::tract as creature_tract;
/// Vocalization types and call intents.
pub use prani::vocalization;
/// Creature voice configuration and synthesis.
pub use prani::voice as creature_voice;

// ---------------------------------------------------------------------------
// Voice source component (humanoid)
// ---------------------------------------------------------------------------

/// Humanoid voice source — text-to-speech driven by shabda + svara.
///
/// Attach to an NPC entity. When a [`SpeechRequest`] targets this entity,
/// the text is converted to phonemes (shabda) and synthesized (svara).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSource {
    /// Speaker identity name (selects voice profile).
    pub profile_name: String,
    /// Speaking rate multiplier (1.0 = normal).
    pub rate: f32,
    /// Pitch shift in semitones (0.0 = natural).
    pub pitch_shift: f32,
    /// Output volume.
    pub volume: f32,
    /// Whether this voice is currently speaking.
    #[serde(skip)]
    pub speaking: bool,
}

impl VoiceSource {
    /// Create a new voice source with the given profile name.
    pub fn new(profile_name: impl Into<String>) -> Self {
        Self {
            profile_name: profile_name.into(),
            rate: 1.0,
            pitch_shift: 0.0,
            volume: 1.0,
            speaking: false,
        }
    }

    /// Set the speaking rate.
    pub fn with_rate(mut self, rate: f32) -> Self {
        self.rate = rate;
        self
    }

    /// Set the pitch shift in semitones.
    pub fn with_pitch_shift(mut self, semitones: f32) -> Self {
        self.pitch_shift = semitones;
        self
    }

    /// Set the output volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
}

impl Default for VoiceSource {
    fn default() -> Self {
        Self::new("default")
    }
}

// ---------------------------------------------------------------------------
// Creature voice source component
// ---------------------------------------------------------------------------

/// Creature voice source — species-specific vocalizations driven by prani.
///
/// Attach to a creature entity. Vocalizations are influenced by the creature's
/// emotional state and fatigue level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureVoiceSource {
    /// Species identifier (selects vocal tract model).
    pub species_name: String,
    /// Current emotional arousal (0.0 = calm, 1.0 = agitated).
    pub arousal: f32,
    /// Current vocal fatigue (0.0 = fresh, 1.0 = exhausted).
    pub fatigue: f32,
    /// Output volume.
    pub volume: f32,
    /// Whether this creature is currently vocalizing.
    #[serde(skip)]
    pub vocalizing: bool,
}

impl CreatureVoiceSource {
    /// Create a new creature voice source for the given species.
    pub fn new(species_name: impl Into<String>) -> Self {
        Self {
            species_name: species_name.into(),
            arousal: 0.0,
            fatigue: 0.0,
            volume: 1.0,
            vocalizing: false,
        }
    }

    /// Set the emotional arousal level.
    pub fn with_arousal(mut self, arousal: f32) -> Self {
        self.arousal = arousal.clamp(0.0, 1.0);
        self
    }

    /// Set the fatigue level.
    pub fn with_fatigue(mut self, fatigue: f32) -> Self {
        self.fatigue = fatigue.clamp(0.0, 1.0);
        self
    }

    /// Set the output volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
}

// ---------------------------------------------------------------------------
// Speech request event
// ---------------------------------------------------------------------------

/// Event requesting an entity to speak text.
#[derive(Debug, Clone)]
pub struct SpeechRequest {
    /// Target entity with a [`VoiceSource`] component.
    pub entity: Entity,
    /// Text to speak.
    pub text: String,
}

impl SpeechRequest {
    /// Create a speech request for the given entity and text.
    pub fn new(entity: Entity, text: impl Into<String>) -> Self {
        Self {
            entity,
            text: text.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Vocalize request event
// ---------------------------------------------------------------------------

/// Event requesting a creature to vocalize.
#[derive(Debug, Clone)]
pub struct VocalizeRequest {
    /// Target entity with a [`CreatureVoiceSource`] component.
    pub entity: Entity,
    /// Intent of the vocalization (alarm, mating call, territorial, etc.).
    pub intent: String,
}

impl VocalizeRequest {
    /// Create a vocalize request for the given entity and intent.
    pub fn new(entity: Entity, intent: impl Into<String>) -> Self {
        Self {
            entity,
            intent: intent.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Process speech requests from the event bus, marking voice sources as speaking.
pub fn process_speech_requests(world: &mut World) {
    let requests = {
        let Some(bus) = world.get_resource_mut::<crate::world::EventBus>() else {
            return;
        };
        bus.drain::<SpeechRequest>()
    };

    for req in requests {
        if let Some(voice) = world.get_component_mut::<VoiceSource>(req.entity) {
            voice.speaking = true;
        }
    }
}

/// Process vocalize requests from the event bus, marking creature voices as active.
pub fn process_vocalize_requests(world: &mut World) {
    let requests = {
        let Some(bus) = world.get_resource_mut::<crate::world::EventBus>() else {
            return;
        };
        bus.drain::<VocalizeRequest>()
    };

    for req in requests {
        if let Some(voice) = world.get_component_mut::<CreatureVoiceSource>(req.entity) {
            voice.vocalizing = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::EventBus;

    #[test]
    fn voice_source_builder() {
        let v = VoiceSource::new("narrator")
            .with_rate(1.2)
            .with_pitch_shift(-2.0)
            .with_volume(0.8);
        assert_eq!(v.profile_name, "narrator");
        assert_eq!(v.rate, 1.2);
        assert_eq!(v.pitch_shift, -2.0);
        assert_eq!(v.volume, 0.8);
        assert!(!v.speaking);
    }

    #[test]
    fn voice_source_default() {
        let v = VoiceSource::default();
        assert_eq!(v.profile_name, "default");
        assert_eq!(v.rate, 1.0);
    }

    #[test]
    fn creature_voice_builder() {
        let v = CreatureVoiceSource::new("wolf")
            .with_arousal(0.7)
            .with_fatigue(0.3)
            .with_volume(0.9);
        assert_eq!(v.species_name, "wolf");
        assert_eq!(v.arousal, 0.7);
        assert_eq!(v.fatigue, 0.3);
        assert!(!v.vocalizing);
    }

    #[test]
    fn creature_voice_clamps() {
        let v = CreatureVoiceSource::new("bird")
            .with_arousal(5.0)
            .with_fatigue(-1.0);
        assert_eq!(v.arousal, 1.0);
        assert_eq!(v.fatigue, 0.0);
    }

    #[test]
    fn speech_request_system() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let npc = world.spawn();
        world
            .insert_component(npc, VoiceSource::new("guard"))
            .unwrap();

        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(SpeechRequest::new(npc, "Halt!"));
        }

        process_speech_requests(&mut world);

        let voice = world.get_component::<VoiceSource>(npc).unwrap();
        assert!(voice.speaking);
    }

    #[test]
    fn vocalize_request_system() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let creature = world.spawn();
        world
            .insert_component(creature, CreatureVoiceSource::new("wolf"))
            .unwrap();

        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(VocalizeRequest::new(creature, "howl"));
        }

        process_vocalize_requests(&mut world);

        let voice = world
            .get_component::<CreatureVoiceSource>(creature)
            .unwrap();
        assert!(voice.vocalizing);
    }

    #[test]
    fn voice_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, VoiceSource::new("bard")).unwrap();
        assert!(world.has_component::<VoiceSource>(e));
    }

    #[test]
    fn creature_voice_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, CreatureVoiceSource::new("cat"))
            .unwrap();
        assert!(world.has_component::<CreatureVoiceSource>(e));
    }
}
