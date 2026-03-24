//! NPC personality and emotion via bhava
//!
//! Provides ECS components for NPC personality traits and emotional state:
//! - [`Personality`] component wrapping bhava PersonalityProfile + MoodVector
//! - Mood stimulus system for event-driven emotion changes
//! - Behavioral prompt generation for AI-driven NPCs

use crate::world::{Entity, EventBus, World};

/// NPC personality and emotional state component.
pub struct Personality {
    /// Trait profile (formality, warmth, curiosity, patience, etc.)
    pub profile: bhava::traits::PersonalityProfile,
    /// Current emotional state.
    pub mood: bhava::mood::MoodVector,
    /// Whether this NPC is actively processing stimuli.
    pub active: bool,
}

impl Personality {
    /// Create a new personality with a name and neutral mood.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            profile: bhava::traits::PersonalityProfile::new(name),
            mood: bhava::mood::MoodVector::neutral(),
            active: true,
        }
    }

    /// Set a personality trait.
    pub fn set_trait(&mut self, kind: bhava::traits::TraitKind, level: bhava::traits::TraitLevel) {
        self.profile.set_trait(kind, level);
    }

    /// Get the dominant emotion.
    #[must_use]
    pub fn dominant_emotion(&self) -> bhava::mood::Emotion {
        self.mood.dominant_emotion()
    }

    /// Apply a mood stimulus (nudge an emotion). Intensity is clamped to [-1.0, 1.0].
    pub fn stimulate(&mut self, emotion: bhava::mood::Emotion, intensity: f32) {
        self.mood.nudge(emotion, intensity.clamp(-1.0, 1.0));
    }

    /// Decay mood toward neutral over time.
    pub fn decay_mood(&mut self, factor: f32) {
        self.mood.decay(factor);
    }

    /// Get the emotional intensity (how far from neutral).
    #[must_use]
    #[inline]
    pub fn emotional_intensity(&self) -> f32 {
        self.mood.intensity()
    }

    /// Generate a behavioral prompt string for AI (combines personality + mood).
    #[must_use]
    pub fn compose_prompt(&self) -> String {
        use std::fmt::Write;
        let personality_prompt = self.profile.compose_prompt();
        let dominant = self.dominant_emotion();
        let intensity = self.emotional_intensity();
        let mut out = personality_prompt;
        let _ = write!(
            out,
            "\nCurrent mood: {dominant:?} (intensity: {intensity:.2})"
        );
        out
    }
}

// ---------------------------------------------------------------------------
// Mood stimulus event
// ---------------------------------------------------------------------------

/// Event that stimulates an NPC's mood.
#[derive(Debug, Clone)]
pub struct MoodStimulus {
    pub entity: Entity,
    pub emotion: bhava::mood::Emotion,
    pub intensity: f32,
}

/// Process mood stimulus events from the event bus.
pub fn process_mood_stimuli(world: &mut World) {
    let stimuli = {
        let Some(bus) = world.get_resource_mut::<EventBus>() else {
            return;
        };
        bus.drain::<MoodStimulus>()
    };

    for stimulus in stimuli {
        if let Some(personality) = world.get_component_mut::<Personality>(stimulus.entity)
            && personality.active
        {
            personality.stimulate(stimulus.emotion, stimulus.intensity);
        }
    }
}

/// Decay all NPC moods toward neutral. Call once per frame.
pub fn decay_all_moods(world: &mut World, factor: f32) {
    let entities: Vec<Entity> = world
        .query::<Personality>()
        .iter()
        .filter(|(_, p)| p.active)
        .map(|(e, _)| *e)
        .collect();

    for entity in entities {
        if let Some(personality) = world.get_component_mut::<Personality>(entity) {
            personality.decay_mood(factor);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bhava::mood::Emotion;
    use bhava::traits::{TraitKind, TraitLevel};

    #[test]
    fn personality_new() {
        let p = Personality::new("Guard");
        assert!(p.active);
        assert!((p.emotional_intensity()).abs() < 0.01);
    }

    #[test]
    fn personality_set_trait() {
        let mut p = Personality::new("Merchant");
        p.set_trait(TraitKind::Curiosity, TraitLevel::High);
        let level = p.profile.get_trait(TraitKind::Curiosity);
        assert_eq!(level, TraitLevel::High);
    }

    #[test]
    fn personality_stimulate() {
        let mut p = Personality::new("Villager");
        p.stimulate(Emotion::Joy, 0.8);
        assert!(p.emotional_intensity() > 0.0);
        assert_eq!(p.dominant_emotion(), Emotion::Joy);
    }

    #[test]
    fn personality_decay() {
        let mut p = Personality::new("Soldier");
        p.stimulate(Emotion::Frustration, 1.0);
        let before = p.emotional_intensity();
        p.decay_mood(0.5);
        assert!(p.emotional_intensity() < before);
    }

    #[test]
    fn personality_compose_prompt() {
        let mut p = Personality::new("Innkeeper");
        p.set_trait(TraitKind::Warmth, TraitLevel::Highest);
        p.stimulate(Emotion::Joy, 0.5);
        let prompt = p.compose_prompt();
        assert!(prompt.contains("Joy"));
    }

    #[test]
    fn personality_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Personality::new("NPC")).unwrap();
        assert!(world.has_component::<Personality>(e));
    }

    #[test]
    fn mood_stimulus_system() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let npc = world.spawn();
        world
            .insert_component(npc, Personality::new("Guard"))
            .unwrap();

        // Publish stimulus
        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(MoodStimulus {
                entity: npc,
                emotion: Emotion::Arousal,
                intensity: 0.7,
            });
        }

        process_mood_stimuli(&mut world);

        let p = world.get_component::<Personality>(npc).unwrap();
        assert!(p.mood.get(Emotion::Arousal) > 0.0);
    }

    #[test]
    fn mood_stimulus_inactive_npc() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let npc = world.spawn();
        let mut personality = Personality::new("Sleeping");
        personality.active = false;
        world.insert_component(npc, personality).unwrap();

        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(MoodStimulus {
                entity: npc,
                emotion: Emotion::Frustration,
                intensity: 1.0,
            });
        }

        process_mood_stimuli(&mut world);

        let p = world.get_component::<Personality>(npc).unwrap();
        assert!((p.emotional_intensity()).abs() < 0.01); // unchanged
    }

    #[test]
    fn decay_all_moods_system() {
        let mut world = World::new();

        let npc1 = world.spawn();
        let mut p1 = Personality::new("A");
        p1.stimulate(Emotion::Joy, 1.0);
        world.insert_component(npc1, p1).unwrap();

        let npc2 = world.spawn();
        let mut p2 = Personality::new("B");
        p2.stimulate(Emotion::Trust, 1.0);
        world.insert_component(npc2, p2).unwrap();

        decay_all_moods(&mut world, 0.5);

        let p1 = world.get_component::<Personality>(npc1).unwrap();
        let p2 = world.get_component::<Personality>(npc2).unwrap();
        assert!(p1.mood.get(Emotion::Joy) < 1.0);
        assert!(p2.mood.get(Emotion::Trust) < 1.0);
    }

    #[test]
    fn multiple_stimuli_accumulate() {
        let mut world = World::new();
        world.insert_resource(EventBus::new());

        let npc = world.spawn();
        world
            .insert_component(npc, Personality::new("Target"))
            .unwrap();

        {
            let bus = world.get_resource_mut::<EventBus>().unwrap();
            bus.publish(MoodStimulus {
                entity: npc,
                emotion: Emotion::Joy,
                intensity: 0.3,
            });
            bus.publish(MoodStimulus {
                entity: npc,
                emotion: Emotion::Joy,
                intensity: 0.3,
            });
        }

        process_mood_stimuli(&mut world);

        let p = world.get_component::<Personality>(npc).unwrap();
        assert!(p.mood.get(Emotion::Joy) > 0.5);
    }

    #[test]
    fn compose_prompt_empty_profile() {
        let p = Personality::new("Blank");
        let prompt = p.compose_prompt();
        assert!(!prompt.is_empty());
    }

    #[test]
    fn query_personality_entities() {
        let mut world = World::new();
        let npc1 = world.spawn();
        let npc2 = world.spawn();
        let _non_npc = world.spawn();

        world.insert_component(npc1, Personality::new("A")).unwrap();
        world.insert_component(npc2, Personality::new("B")).unwrap();

        let npcs = world.query::<Personality>();
        assert_eq!(npcs.len(), 2);
    }

    #[test]
    fn personality_multiple_traits() {
        let mut p = Personality::new("Complex");
        p.set_trait(TraitKind::Warmth, TraitLevel::Highest);
        p.set_trait(TraitKind::Humor, TraitLevel::High);
        p.set_trait(TraitKind::Patience, TraitLevel::Low);

        assert_eq!(p.profile.get_trait(TraitKind::Warmth), TraitLevel::Highest);
        assert_eq!(p.profile.get_trait(TraitKind::Humor), TraitLevel::High);
        assert_eq!(p.profile.get_trait(TraitKind::Patience), TraitLevel::Low);
    }

    #[test]
    fn stimulate_clamps_intensity() {
        let mut p = Personality::new("Clamped");
        p.stimulate(Emotion::Joy, 5.0); // over 1.0 → clamped to 1.0
        assert!(p.mood.get(Emotion::Joy) <= 1.0);
    }

    #[test]
    fn mood_blend_multiple_emotions() {
        let mut p = Personality::new("Mixed");
        p.stimulate(Emotion::Joy, 0.5);
        p.stimulate(Emotion::Frustration, 0.8);
        // Frustration is stronger
        assert_eq!(p.dominant_emotion(), Emotion::Frustration);
    }
}
