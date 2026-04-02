use criterion::{Criterion, criterion_group, criterion_main};
use kiran::World;
use kiran::biology::{MetabolicProfile, Microbe, Physiology, PhysiologyGait, PlantState};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// Physiology
// ---------------------------------------------------------------------------

fn bench_physiology(c: &mut Criterion) {
    let mut group = c.benchmark_group("biology/physiology");

    group.bench_function("create", |b| {
        b.iter(|| black_box(Physiology::new(black_box(70.0))))
    });

    group.bench_function("with_gait", |b| {
        b.iter(|| {
            black_box(
                Physiology::new(black_box(70.0))
                    .with_gait(black_box(PhysiologyGait::Run))
                    .with_speed(black_box(3.0)),
            )
        })
    });

    group.bench_function("insert_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        b.iter(|| {
            world
                .insert_component(entity, Physiology::new(black_box(80.0)))
                .unwrap();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Microbe
// ---------------------------------------------------------------------------

fn bench_microbe(c: &mut Criterion) {
    let mut group = c.benchmark_group("biology/microbe");

    group.bench_function("create", |b| {
        b.iter(|| {
            black_box(Microbe::new(
                black_box("E. coli"),
                black_box(1000.0),
                black_box(1e9),
            ))
        })
    });

    group.bench_function("with_growth_rate", |b| {
        b.iter(|| black_box(Microbe::new("E. coli", 1000.0, 1e9).with_growth_rate(black_box(0.8))))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Metabolic profile
// ---------------------------------------------------------------------------

fn bench_metabolic(c: &mut Criterion) {
    let mut group = c.benchmark_group("biology/metabolic");

    group.bench_function("create", |b| b.iter(|| black_box(MetabolicProfile::new())));

    group.bench_function("is_energy_crisis", |b| {
        let m = MetabolicProfile::new();
        b.iter(|| black_box(m.is_energy_crisis()))
    });

    group.bench_function("is_anaerobic", |b| {
        let m = MetabolicProfile::new();
        b.iter(|| black_box(m.is_anaerobic()))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Plant
// ---------------------------------------------------------------------------

fn bench_plant(c: &mut Criterion) {
    let mut group = c.benchmark_group("biology/plant");

    group.bench_function("seed", |b| {
        b.iter(|| black_box(PlantState::seed(black_box("Oak"))))
    });

    group.bench_function("mature", |b| {
        b.iter(|| black_box(PlantState::mature(black_box("Pine"), black_box(10.0))))
    });

    group.bench_function("total_mass", |b| {
        let p = PlantState::mature("Pine", 10.0);
        b.iter(|| black_box(p.total_mass()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_physiology,
    bench_microbe,
    bench_metabolic,
    bench_plant,
);
criterion_main!(benches);
