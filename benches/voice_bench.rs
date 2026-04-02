use criterion::{Criterion, criterion_group, criterion_main};
use kiran::World;
use kiran::voice::{CreatureVoiceSource, VoiceSource};
use std::hint::black_box;

fn bench_voice(c: &mut Criterion) {
    let mut group = c.benchmark_group("voice");

    group.bench_function("voice_source_create", |b| {
        b.iter(|| black_box(VoiceSource::new(black_box("narrator"))))
    });

    group.bench_function("voice_source_builder", |b| {
        b.iter(|| {
            black_box(
                VoiceSource::new("guard")
                    .with_rate(black_box(1.2))
                    .with_pitch_shift(black_box(-2.0))
                    .with_volume(black_box(0.8)),
            )
        })
    });

    group.bench_function("creature_voice_create", |b| {
        b.iter(|| black_box(CreatureVoiceSource::new(black_box("wolf"))))
    });

    group.bench_function("creature_voice_builder", |b| {
        b.iter(|| {
            black_box(
                CreatureVoiceSource::new("bird")
                    .with_arousal(black_box(0.7))
                    .with_fatigue(black_box(0.3))
                    .with_volume(black_box(0.9)),
            )
        })
    });

    group.bench_function("voice_insert_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        b.iter(|| {
            world
                .insert_component(entity, VoiceSource::new(black_box("bard")))
                .unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_voice);
criterion_main!(benches);
