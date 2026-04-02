//! Example: Create NPC personalities, stimulate emotions, generate AI prompts.

use kiran::personality::{MoodStimulus, Personality, process_mood_stimuli};
use kiran::{EventBus, World};

fn main() {
    let mut world = World::new();
    world.insert_resource(EventBus::new());

    // Create a guard NPC with specific traits
    let guard = world.spawn();
    let mut guard_personality = Personality::new("Guard Captain");
    guard_personality.set_trait(
        bhava::traits::TraitKind::Patience,
        bhava::traits::TraitLevel::Low,
    );
    guard_personality.set_trait(
        bhava::traits::TraitKind::Formality,
        bhava::traits::TraitLevel::High,
    );
    world.insert_component(guard, guard_personality).unwrap();

    // Create a merchant NPC
    let merchant = world.spawn();
    let mut merchant_personality = Personality::new("Merchant");
    merchant_personality.set_trait(
        bhava::traits::TraitKind::Warmth,
        bhava::traits::TraitLevel::Highest,
    );
    merchant_personality.set_trait(
        bhava::traits::TraitKind::Humor,
        bhava::traits::TraitLevel::High,
    );
    world
        .insert_component(merchant, merchant_personality)
        .unwrap();

    // Stimulate emotions via event bus
    {
        let bus = world.get_resource_mut::<EventBus>().unwrap();
        bus.publish(MoodStimulus {
            entity: guard,
            emotion: bhava::mood::Emotion::Frustration,
            intensity: 0.8,
        });
        bus.publish(MoodStimulus {
            entity: merchant,
            emotion: bhava::mood::Emotion::Joy,
            intensity: 0.6,
        });
    }

    process_mood_stimuli(&mut world);

    // Print personality state and AI prompts
    for (entity, label) in [(guard, "Guard"), (merchant, "Merchant")] {
        let p = world.get_component::<Personality>(entity).unwrap();
        println!("--- {label} ---");
        println!("  Dominant emotion: {:?}", p.dominant_emotion());
        println!("  Emotional intensity: {:.2}", p.emotional_intensity());
        println!("  AI prompt:\n{}\n", p.compose_prompt());
    }

    // Decay moods and show the change
    kiran::personality::decay_all_moods(&mut world, 0.5);
    let guard_p = world.get_component::<Personality>(guard).unwrap();
    println!(
        "After decay — Guard intensity: {:.2}",
        guard_p.emotional_intensity()
    );
}
