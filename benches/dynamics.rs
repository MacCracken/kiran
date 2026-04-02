use criterion::{Criterion, criterion_group, criterion_main};
use kiran::World;
use kiran::dynamics::{AeroSurface, EmField, MaterialBody, ThermalBody, ThermalPhase};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// EM field
// ---------------------------------------------------------------------------

fn bench_em_field(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamics/em_field");

    group.bench_function("create", |b| {
        b.iter(|| EmField::new(black_box([1.0, 0.0, 0.0]), black_box([0.0, 0.0, 1.0])))
    });

    group.bench_function("create_electric_only", |b| {
        b.iter(|| EmField::electric_only(black_box([5.0, 0.0, 0.0])))
    });

    group.bench_function("insert_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        b.iter(|| {
            world
                .insert_component(entity, EmField::electric_only(black_box([1.0, 0.0, 0.0])))
                .unwrap();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Material body
// ---------------------------------------------------------------------------

fn bench_material_body(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamics/material");

    group.bench_function("create_steel", |b| {
        b.iter(|| black_box(MaterialBody::steel()))
    });

    group.bench_function("create_aluminum", |b| {
        b.iter(|| black_box(MaterialBody::aluminum()))
    });

    group.bench_function("is_yielded", |b| {
        let m = MaterialBody::steel();
        b.iter(|| black_box(m.is_yielded(black_box(300e6))))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Thermal body
// ---------------------------------------------------------------------------

fn bench_thermal_body(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamics/thermal");

    group.bench_function("create", |b| {
        b.iter(|| {
            black_box(ThermalBody::new(
                black_box(300.0),
                black_box(50.0),
                black_box(500.0),
                black_box(1.0),
            ))
        })
    });

    group.bench_function("apply_heat", |b| {
        let mut t = ThermalBody::new(300.0, 50.0, 500.0, 1.0);
        b.iter(|| {
            t.apply_heat(black_box(100.0));
        })
    });

    group.bench_function("with_phase", |b| {
        b.iter(|| {
            black_box(
                ThermalBody::new(373.0, 0.6, 4186.0, 1.0)
                    .with_phase(black_box(ThermalPhase::Liquid)),
            )
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Aero surface
// ---------------------------------------------------------------------------

fn bench_aero_surface(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamics/aero");

    group.bench_function("create", |b| {
        b.iter(|| {
            black_box(
                AeroSurface::new(black_box(16.0), black_box(0.02), black_box(8.0))
                    .with_oswald(black_box(0.9)),
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_em_field,
    bench_material_body,
    bench_thermal_body,
    bench_aero_surface,
);
criterion_main!(benches);
