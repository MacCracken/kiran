use criterion::{Criterion, criterion_group, criterion_main};
use kiran::World;
use kiran::personality::{MoodStimulus, Personality, decay_all_moods, process_mood_stimuli};
use kiran::world::EventBus;
use std::hint::black_box;

fn bench_personality(c: &mut Criterion) {
    let mut group = c.benchmark_group("personality");

    group.bench_function("create", |b| b.iter(|| Personality::new(black_box("NPC"))));

    group.bench_function("stimulate", |b| {
        let mut p = Personality::new("NPC");
        b.iter(|| {
            p.stimulate(black_box(bhava::mood::Emotion::Joy), black_box(0.5));
        })
    });

    group.bench_function("decay", |b| {
        let mut p = Personality::new("NPC");
        p.stimulate(bhava::mood::Emotion::Joy, 1.0);
        b.iter(|| {
            p.decay_mood(black_box(0.95));
        })
    });

    group.bench_function("compose_prompt", |b| {
        let mut p = Personality::new("Guard");
        p.set_trait(
            bhava::traits::TraitKind::Warmth,
            bhava::traits::TraitLevel::High,
        );
        p.stimulate(bhava::mood::Emotion::Joy, 0.5);
        b.iter(|| black_box(p.compose_prompt()))
    });

    group.bench_function("process_stimuli_100", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;
            for _ in 0..iters {
                let mut world = World::new();
                world.insert_resource(EventBus::new());

                let mut entities = Vec::new();
                for i in 0..100 {
                    let e = world.spawn();
                    world
                        .insert_component(e, Personality::new(format!("NPC{i}")))
                        .unwrap();
                    entities.push(e);
                }

                {
                    let bus = world.get_resource_mut::<EventBus>().unwrap();
                    for &e in &entities {
                        bus.publish(MoodStimulus {
                            entity: e,
                            emotion: bhava::mood::Emotion::Arousal,
                            intensity: 0.5,
                        });
                    }
                }

                let start = std::time::Instant::now();
                process_mood_stimuli(&mut world);
                total += start.elapsed();
            }
            total
        })
    });

    group.bench_function("decay_all_100", |b| {
        let mut world = World::new();
        for i in 0..100 {
            let e = world.spawn();
            let mut p = Personality::new(format!("NPC{i}"));
            p.stimulate(bhava::mood::Emotion::Joy, 1.0);
            world.insert_component(e, p).unwrap();
        }
        b.iter(|| {
            decay_all_moods(black_box(&mut world), black_box(0.95));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_personality);
criterion_main!(benches);
