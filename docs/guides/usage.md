# Usage Patterns

## Philosophy

Kiran follows a strict composition model: small components, thin systems, data-driven scenes. The engine owns the ECS, game loop, and scene format. Domain-specific work (physics, rendering, audio, AI) lives in feature-gated AGNOS crates that kiran orchestrates.

## World, Scene, Systems, Game Loop

Every kiran game follows the same lifecycle:

```rust
use kiran::{World, Scheduler, FnSystem, SystemStage, GameClock};
use kiran::scene::load_scene;

fn main() -> kiran::Result<()> {
    // 1. Create world
    let mut world = World::new();

    // 2. Load scene from TOML
    let scene_toml = std::fs::read_to_string("level.toml")?;
    let scene = load_scene(&scene_toml)?;
    kiran::scene::spawn_scene(&mut world, &scene)?;

    // 3. Register systems
    let mut scheduler = Scheduler::new();
    scheduler.add_system(Box::new(FnSystem::new(
        "player_movement", SystemStage::GameLogic, move |world: &mut World| {
            // query components, update state
        },
    )));

    // 4. Game loop
    let mut clock = GameClock::new();
    loop {
        clock.tick(1.0 / 60.0);
        world.insert_resource(clock.clone());
        scheduler.run(&mut world);
    }
}
```

Systems execute in stage order: `Input` (0) -> `Physics` (1) -> `GameLogic` (2) -> `Render` (3). Within a stage, topological sort resolves dependencies.

## Component Design

Prefer small, single-purpose components over large monolithic ones:

```rust
// Good: composable, queryable independently
struct Position(Vec3);
struct Velocity(Vec3);
struct Health(f32);

// Bad: kitchen-sink component
struct PlayerData { pos: Vec3, vel: Vec3, hp: f32, name: String, /* ... */ }
```

Attach components to entities, compose behavior through systems:

```rust
let entity = world.spawn();
world.insert_component(entity, Position(Vec3::ZERO))?;
world.insert_component(entity, Velocity(Vec3::new(1.0, 0.0, 0.0)))?;
world.insert_component(entity, Health(100.0))?;
```

## Feature Gate Selection

Choose features based on your game type:

| Game Type | Recommended Features |
|-----------|---------------------|
| 2D arcade | `rendering` |
| 3D action | `rendering`, `physics`, `audio`, `navigation` |
| Multiplayer FPS | `rendering`, `physics`, `audio`, `multiplayer`, `navigation` |
| Simulation | `physics`, `fluids`, `dynamics`, `biology`, `chemistry` |
| AI sandbox | `ai`, `behavior`, `scripting` |
| Full engine | `full` (all 15 feature gates, 46 deps) |

```toml
# Cargo.toml — pull only what you need
[dependencies]
kiran = { version = "0.26", features = ["rendering", "physics", "audio"] }
```

## Scene Format (TOML)

Scenes are data files, not code. Entities, prefabs, materials, and hierarchy are declared in TOML:

```toml
name = "level_01"
description = "Tutorial level"

[[entities]]
name = "player"
position = [0.0, 1.0, 0.0]
tags = ["controllable"]

[[entities]]
name = "sun"
position = [0.0, 50.0, 0.0]
light_intensity = 1.0
```

Scenes support prefab templates, child hierarchies, material definitions, physics bodies, and sound sources. Hot reload via `SceneReloader` watches the file and applies diffs at runtime.
