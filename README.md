# Kiran

[![Crates.io](https://img.shields.io/crates/v/kiran.svg)](https://crates.io/crates/kiran)
[![docs.rs](https://docs.rs/kiran/badge.svg)](https://docs.rs/kiran)
[![CI](https://github.com/MacCracken/kiran/actions/workflows/ci.yml/badge.svg)](https://github.com/MacCracken/kiran/actions/workflows/ci.yml)
[![License: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](LICENSE)

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

Modular game engine built in Rust, designed for AI-driven game development.
46 optional dependencies across 16 feature gates — pull only what you need.

For AI NPCs and headless simulation, see [joshua](https://github.com/MacCracken/joshua).
For the visual editor, see [salai](https://github.com/MacCracken/salai).

## Architecture

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, matrices, geometry, transforms)
  │
  ├── rendering:   soorat (GPU/wgpu), prakash (PBR/optics), ranga (image processing)
  ├── audio:       dhvani (engine), naad (synthesis), shravan (codecs),
  │                goonj (acoustics), garjan (environmental), ghurni (mechanical)
  ├── voice:       svara (formant), shabda (G2P), prani (creature vocals)
  ├── physics:     impetus (rigid body, collision, raycasting)
  ├── fluids:      pravash (SPH, shallow water)
  ├── dynamics:    bijli (EM), dravya (materials), ushma (thermo), pavan (aero)
  │
  ├── ai:          hoosh (LLM inference)
  ├── behavior:    bhava (personality), bodh (psychology), mastishk (neuro), jantu (ethology)
  ├── scripting:   kavach (WASM sandbox)
  ├── multiplayer: majra (pub/sub relay)
  ├── navigation:  raasta (pathfinding, navmesh)
  │
  ├── biology:     sharira (physiology), jivanu (micro), rasayan (biochem), vanaspati (botany)
  ├── chemistry:   kimiya (reactions), khanij (geology), tanmatra (atomic), kana (quantum)
  ├── astronomy:   falak (orbits), jyotish (planetary), tara (stellar), brahmanda (cosmo), badal (weather)
  └── world:       itihas (history), sankhya (calendars), varna (language), pramana (statistics)
```

## Consumers

| Crate | Role |
|-------|------|
| [joshua](https://github.com/MacCracken/joshua) | Simulation layer — headless AI-driven game worlds |
| [salai](https://github.com/MacCracken/salai) | Visual editor — scene editing, asset preview |

## Feature Flags

### Engine Core

| Feature | Dependencies | Description |
|---------|-------------|-------------|
| `rendering` | soorat, prakash, ranga | GPU rendering, PBR optics, image processing |
| `audio` | dhvani, naad, shravan, goonj, garjan, ghurni | Audio engine, synthesis, codecs, acoustics, environmental/mechanical sounds |
| `voice` | svara, shabda, prani | Formant synthesis, grapheme-to-phoneme, creature vocals |
| `physics` | impetus | Rigid bodies, collision detection, raycasting |
| `physics-3d` | impetus (3D) | 3D physics backend |
| `fluids` | pravash | SPH and shallow water simulation |
| `dynamics` | bijli, dravya, ushma, pavan | Electromagnetism, materials, thermodynamics, aerodynamics |
| `ai` | hoosh, reqwest, tokio | LLM inference integration |
| `behavior` | bhava, bodh, mastishk, jantu | Personality, psychology, neuroscience, ethology |
| `scripting` | kavach, tokio | WASM script sandbox via wasmtime |
| `multiplayer` | majra | Networked state sync, relay messaging |
| `navigation` | raasta | Pathfinding, navmesh, steering |

### Science

| Feature | Dependencies | Description |
|---------|-------------|-------------|
| `biology` | sharira, jivanu, rasayan, vanaspati | Physiology, microbiology, biochemistry, botany |
| `chemistry` | kimiya, khanij, tanmatra, kana | Chemistry, geology, atomic physics, quantum mechanics |
| `astronomy` | falak, jyotish, tara, brahmanda, badal | Orbital mechanics, planetary positions, stellar physics, cosmology, weather |
| `world` | itihas, sankhya, varna, pramana | History, calendars, languages, statistics |

### Meta

| Feature | Description |
|---------|-------------|
| `full` | All of the above |

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

## Examples

```sh
cargo run --example game_loop                          # Scheduler, input, profiling
cargo run --example scene_loader                       # Load TOML scene, walk hierarchy
cargo run --example physics_demo --features physics    # Rigid bodies, collision
cargo run --example audio_demo --features audio        # Spatial audio, mix buses
cargo run --example behavior_demo --features behavior  # NPC personality, emotions
cargo run --example dynamics_demo --features dynamics  # Thermal bodies, materials
```

## Building

```sh
cargo build                              # Core only
cargo build --all-features               # Everything
cargo test --all-features                # 587 unit tests + 27 doc tests
cargo bench --all-features               # 8 benchmark suites
make check                               # fmt + clippy + test + audit
```

## License

GPL-3.0-only — see [LICENSE](LICENSE).
