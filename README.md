# Kiran

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

Modular game engine built in Rust, designed for AI-driven game development. Composes AGNOS shared crates for physics ([impetus](https://github.com/MacCracken/impetus)), math ([hisab](https://github.com/MacCracken/hisab)), audio ([dhvani](https://github.com/MacCracken/dhvani)), and rendering ([aethersafta](https://github.com/MacCracken/aethersafta)).

For AI NPCs and headless simulation, see [joshua](https://github.com/MacCracken/joshua) (builds on kiran).

## Architecture

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms, spatial structures)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── aethersafta  — rendering (wgpu, scene graph)  [planned]
  ├── dhvani       — audio (spatial, DSP, mixing)    [planned]
  ├── ranga        — image processing (textures)     [planned]
  ├── majra        — multiplayer (pub/sub, relay)    [planned]
  ├── kavach       — scripting sandbox (WASM)        [planned]
  └── bhava        — NPC emotion/personality         [planned]
```

## Crates

| Crate | Description |
|-------|-------------|
| `kiran-core` | ECS world, generational entity allocator, game clock, event bus |
| `kiran-scene` | TOML scene format, loading, entity spawning |
| `kiran-input` | Keyboard, mouse, gamepad, edge-triggered queries |
| `kiran-render` | Renderer trait, camera, sprites/meshes, NullRenderer (headless) |
| `kiran-physics` | Impetus integration bridge |
| `kiran-ai` | AGNOS daimon/hoosh integration |

## Quick Start

```rust
use kiran_core::{World, GameClock};
use kiran_scene::load_scene;
use kiran_input::{InputState, KeyCode};

// Load a scene
let scene = load_scene(include_str!("level.toml")).unwrap();
let mut world = World::new();
let entities = kiran_scene::spawn_scene(&mut world, &scene);

// Game loop
let mut clock = GameClock::new(1.0 / 60.0);
loop {
    clock.tick(delta_time);
    // input → physics → game logic → render
}
```

## Scene Format (TOML)

```toml
[scene]
name = "tavern"
ambient_light = [0.2, 0.2, 0.3]

[[entity]]
name = "bartender"
type = "npc"
position = [5.0, 0.0, 3.0]

[entity.ai]
model = "mistral:7b-q4"
personality = "blue-shirt-guy"

[[entity]]
name = "table"
type = "static"
position = [3.0, 0.0, 2.0]
model = "models/tavern/table.glb"

[entity.physics]
body_type = "static"
collider = { shape = "box", half_extents = [0.5, 0.5, 0.5] }
```

## CLI

```sh
kiran check level.toml    # Validate scene file
kiran run level.toml      # Load and run
```

## Building

```sh
cargo build --workspace
cargo test --workspace
```

## Roadmap

See [docs/development/roadmap.md](docs/development/roadmap.md) — V0.1 (core) done, V0.2 (rendering) through V1.0 (production) planned.

## License

GPL-3.0 — see [LICENSE](LICENSE).
