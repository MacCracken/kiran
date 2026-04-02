# Kiran

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

Modular game engine built in Rust, designed for AI-driven game development. Composes AGNOS shared crates for physics ([impetus](https://github.com/MacCracken/impetus)), math ([hisab](https://github.com/MacCracken/hisab)), audio ([dhvani](https://github.com/MacCracken/dhvani)), rendering ([soorat](https://github.com/MacCracken/soorat)), optics ([prakash](https://github.com/MacCracken/prakash)), multiplayer ([majra](https://github.com/MacCracken/majra)), and scripting ([kavach](https://github.com/MacCracken/kavach)).

For AI NPCs and headless simulation, see [joshua](https://github.com/MacCracken/joshua). For the visual editor, see [salai](https://github.com/MacCracken/salai).

## Architecture

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── soorat       — rendering (wgpu, PBR, shadows, animation, terrain, text, UI)
  ├── prakash      — optics (ray tracing, spectral color, PBR math)
  ├── dhvani       — audio (spatial, DSP, mixing)
  ├── majra        — multiplayer (pub/sub, relay)
  ├── kavach       — scripting sandbox (WASM)
  └── bhava        — NPC emotion/personality         [planned]
```

## Modules

| Module | Feature | Description |
|--------|---------|-------------|
| `world` | core | ECS world, Vec arena O(1) storage, scheduler, change detection |
| `scene` | core | TOML scene format, hierarchy, prefabs, materials |
| `input` | core | Keyboard, mouse, edge-triggered queries |
| `render` | core | Renderer trait, camera controllers, NullRenderer |
| `gpu` | `rendering` | soorat GPU backend (sprites, meshes, PBR, shadows, animation) |
| `audio` | `audio` | dhvani spatial audio, sound triggers |
| `physics` | `physics` | impetus bridge, raycasting, debug shapes, 3D |
| `net` | `multiplayer` | majra relay, state sync, snapshots, deltas |
| `script` | `scripting` | kavach WASM execution, message passing |
| `ai` | `ai` | daimon/hoosh client |
| `reload` | core | Scene + shader hot reload, file watcher |
| `profiler` | core | Frame timing, per-system cost, slow frame detection |
| `asset` | core | Asset registry, typed handles, hot reload integration |

## Quick Start

```rust
use kiran::{World, GameClock, Scheduler, FnSystem, SystemStage};
use kiran::scene::{load_scene, spawn_scene};

let scene = load_scene(include_str!("level.toml")).unwrap();
let mut world = World::new();
spawn_scene(&mut world, &scene).unwrap();

let mut scheduler = Scheduler::new();
scheduler.add_system(Box::new(FnSystem::new("tick", SystemStage::Input, |world| {
    let clock = world.get_resource_mut::<GameClock>().unwrap();
    clock.tick(1.0 / 60.0);
})));

loop {
    scheduler.run(&mut world);
}
```

## Feature Flags

| Feature | Dependency | Description |
|---------|-----------|-------------|
| `audio` | dhvani | Spatial audio, sound triggers |
| `physics` | impetus | Rigid bodies, collision, raycasting |
| `physics-3d` | impetus (3D) | 3D physics backend |
| `rendering` | soorat | GPU rendering (PBR, shadows, animation, terrain, text, UI) |
| `scripting` | kavach, tokio | WASM script execution via wasmtime |
| `multiplayer` | majra | Networked state sync, relay messaging |
| `ai` | reqwest, tokio | Daimon/hoosh AI integration |
| `full` | all above | Everything |

## Examples

```sh
cargo run --example scene_loader    # Load TOML scene, walk hierarchy
cargo run --example game_loop       # Scheduler, input, profiling
```

## Building

```sh
cargo build
cargo test
cargo bench --features rendering
```

## License

GPL-3.0-only — see [LICENSE](LICENSE).
