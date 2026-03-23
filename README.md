# Kiran

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

Modular game engine built in Rust, designed for AI-driven game development. Composes AGNOS shared crates for physics ([impetus](https://github.com/MacCracken/impetus)), math ([hisab](https://github.com/MacCracken/hisab)), audio ([dhvani](https://github.com/MacCracken/dhvani)), rendering ([soorat](https://github.com/MacCracken/soorat)), and optics ([prakash](https://github.com/MacCracken/prakash)).

For AI NPCs and headless simulation, see [joshua](https://github.com/MacCracken/joshua) (builds on kiran).

## Architecture

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms, spatial structures)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── soorat       — rendering (wgpu, sprites, meshes, window management)
  ├── prakash      — optics (ray tracing, spectral color, PBR primitives)
  ├── dhvani       — audio (spatial, DSP, mixing)
  ├── ranga        — image processing (textures)     [planned]
  ├── majra        — multiplayer (pub/sub, relay)    [planned]
  ├── kavach       — scripting sandbox (WASM)        [planned]
  └── bhava        — NPC emotion/personality         [planned]
```

## Modules

| Module | Description |
|--------|-------------|
| `world` | ECS world, generational entity allocator, game clock, event bus, scheduler |
| `scene` | TOML scene format, loading, hierarchy, prefabs, materials |
| `input` | Keyboard, mouse, gamepad, edge-triggered queries |
| `render` | Renderer trait, camera controllers, sprites/meshes, NullRenderer |
| `gpu` | Soorat rendering backend (feature: `rendering`) |
| `audio` | dhvani spatial audio integration (feature: `audio`) |
| `physics` | Impetus physics bridge, raycasting, debug shapes (feature: `physics`) |
| `ai` | AGNOS daimon/hoosh integration (feature: `ai`) |
| `script` | Script engine, message passing, WASM bridge |
| `reload` | Scene hot reload, file watcher, live diff updates |

## Quick Start

```rust
use kiran::{World, GameClock, Scheduler, FnSystem, SystemStage};
use kiran::scene::{load_scene, spawn_scene};
use kiran::input::InputState;

// Load a scene
let scene = load_scene(include_str!("level.toml")).unwrap();
let mut world = World::new();
spawn_scene(&mut world, &scene).unwrap();

// Set up systems
let mut scheduler = Scheduler::new();
scheduler.add_system(Box::new(FnSystem::new("tick", SystemStage::Input, |world| {
    let clock = world.get_resource_mut::<GameClock>().unwrap();
    clock.tick(1.0 / 60.0);
})));

// Game loop
loop {
    scheduler.run(&mut world);
}
```

## Feature Flags

| Feature | Dependency | Description |
|---------|-----------|-------------|
| `audio` | dhvani | Spatial audio, sound triggers |
| `physics` | impetus | Rigid bodies, collision, raycasting |
| `rendering` | soorat | GPU rendering via wgpu |
| `ai` | reqwest, tokio | Daimon/hoosh AI integration |

## Building

```sh
cargo build
cargo test
cargo test --all-features
```

## Roadmap

See [docs/development/roadmap.md](docs/development/roadmap.md).

## License

GPL-3.0 — see [LICENSE](LICENSE).
