# Testing Strategy

## Current Stats

- 587 unit tests
- 27 doc tests
- 84% coverage
- Target: 80%+ coverage maintained

## Unit Test Patterns

The standard pattern: create a `World`, insert components, run logic, assert results.

```rust
#[test]
fn spawn_and_query_component() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert_component(e, 42u32).unwrap();

    let val = world.get_component::<u32>(e).unwrap();
    assert_eq!(*val, 42);
}
```

Test error paths explicitly — dead entities, missing components, stale handles:

```rust
#[test]
fn dead_entity_returns_error() {
    let mut world = World::new();
    let e = world.spawn();
    world.despawn(e).unwrap();

    assert!(world.get_component::<u32>(e).is_none());
}
```

## Integration Test Patterns

Full scheduler runs exercise the system pipeline end-to-end:

```rust
#[test]
fn scheduler_runs_stages_in_order() {
    let mut world = World::new();
    let mut scheduler = Scheduler::new();

    scheduler.add_system(Box::new(FnSystem::new(
        "physics_step", SystemStage::Physics, |world: &mut World| {
            // simulate
        },
    )));
    scheduler.add_system(Box::new(FnSystem::new(
        "render_submit", SystemStage::Render, |world: &mut World| {
            // draw
        },
    )));

    scheduler.run(&mut world);
}
```

## Headless Rendering

Use `NullRenderer` for render-path tests without a GPU:

```rust
use kiran::render::{NullRenderer, Renderer, RenderConfig, DrawCommand};

let mut r = NullRenderer::new();
r.init(&RenderConfig::default()).unwrap();
r.begin_frame().unwrap();
r.submit(DrawCommand::Clear([0.0, 0.0, 0.0, 1.0])).unwrap();
r.end_frame().unwrap();
assert_eq!(r.frame_count, 1);
```

## Benchmark Patterns

Benchmarks use criterion and require feature gates:

```rust
// benches/engine.rs (requires `rendering` feature)
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_spawn_entities(c: &mut Criterion) {
    c.bench_function("spawn/10k_entities", |b| {
        b.iter(|| {
            let mut world = World::new();
            for _ in 0..10_000 { world.spawn(); }
        });
    });
}
```

Run benchmarks and track history:

```bash
./scripts/bench-history.sh                # appends to bench-history.csv
cargo bench --bench engine                # criterion HTML reports in target/
cargo bench --bench personality            # requires `behavior` feature
```
