use criterion::{Criterion, criterion_group, criterion_main};
use kiran::astronomy::{CelestialBody, WeatherCondition, WeatherZone};
use kiran::chemistry::{ChemicalBody, GeologicalBody, RadioactiveSource};
use kiran::lore::{CultureProfile, StochasticSource};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// Chemistry
// ---------------------------------------------------------------------------

fn bench_chemistry(c: &mut Criterion) {
    let mut group = c.benchmark_group("science/chemistry");

    group.bench_function("chemical_body_create", |b| {
        b.iter(|| black_box(ChemicalBody::new(black_box("NaCl"), black_box(298.0))))
    });

    group.bench_function("chemical_body_builder", |b| {
        b.iter(|| {
            black_box(
                ChemicalBody::new("H2SO4", 298.0)
                    .with_concentration(black_box(0.1))
                    .with_ph(black_box(1.0)),
            )
        })
    });

    group.bench_function("geological_body_create", |b| {
        b.iter(|| {
            black_box(GeologicalBody::new(
                black_box("Granite"),
                black_box(7.0),
                black_box(2700.0),
            ))
        })
    });

    group.bench_function("radioactive_source_create", |b| {
        b.iter(|| {
            black_box(RadioactiveSource::new(
                black_box("U-238"),
                black_box(1.41e17),
                black_box(1e3),
            ))
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Astronomy
// ---------------------------------------------------------------------------

fn bench_astronomy(c: &mut Criterion) {
    let mut group = c.benchmark_group("science/astronomy");

    group.bench_function("celestial_circular", |b| {
        b.iter(|| {
            black_box(CelestialBody::circular(
                black_box("Earth"),
                black_box(5.972e24),
                black_box(1.496e11),
            ))
        })
    });

    group.bench_function("celestial_elliptical", |b| {
        b.iter(|| {
            black_box(CelestialBody::elliptical(
                black_box("Mars"),
                black_box(6.417e23),
                black_box(2.279e11),
                black_box(0.0934),
            ))
        })
    });

    group.bench_function("weather_standard", |b| {
        b.iter(|| black_box(WeatherZone::standard()))
    });

    group.bench_function("weather_builder", |b| {
        b.iter(|| {
            black_box(
                WeatherZone::standard()
                    .with_temperature(black_box(260.0))
                    .with_humidity(black_box(0.9))
                    .with_wind(black_box(15.0), black_box(1.5))
                    .with_condition(black_box(WeatherCondition::Snow)),
            )
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// World/Lore
// ---------------------------------------------------------------------------

fn bench_lore(c: &mut Criterion) {
    let mut group = c.benchmark_group("science/lore");

    group.bench_function("culture_profile_create", |b| {
        b.iter(|| {
            black_box(CultureProfile::new(
                black_box("Roman Empire"),
                black_box("la"),
                black_box("julian"),
            ))
        })
    });

    group.bench_function("culture_profile_builder", |b| {
        b.iter(|| {
            black_box(
                CultureProfile::new("Norse", "no", "gregorian").with_era(black_box("Viking Age")),
            )
        })
    });

    group.bench_function("stochastic_uniform", |b| {
        b.iter(|| black_box(StochasticSource::uniform(black_box(0.0), black_box(1.0))))
    });

    group.bench_function("stochastic_normal", |b| {
        b.iter(|| black_box(StochasticSource::normal(black_box(100.0), black_box(15.0))))
    });

    group.finish();
}

criterion_group!(benches, bench_chemistry, bench_astronomy, bench_lore);
criterion_main!(benches);
