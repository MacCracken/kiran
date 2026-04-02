# Kiran Architecture

## Overview

Kiran is an AI-native game engine that orchestrates AGNOS shared crates into a unified development framework. It owns the ECS, game loop, scene management, and integration layer — delegating physics, rendering, audio, networking, and scripting to specialized crates.

## Module Structure

```
src/
├── Core (always available)
│   ├── world.rs       — ECS world, Entity, EntityAllocator, GameClock, EventBus, Scheduler
│   ├── scene.rs       — TOML scene format, hierarchy, prefabs, materials, spawning
│   ├── input.rs       — KeyCode, MouseButton, InputState, edge-triggered queries
│   ├── render.rs      — Renderer trait, Camera, controllers (Orbit/Fly/Follow), NullRenderer
│   ├── reload.rs      — FileWatcher, SceneReloader, ShaderReloader, apply_scene_diff
│   ├── profiler.rs    — FrameProfiler (per-system timing, EMA, slow frame detection)
│   ├── asset.rs       — AssetRegistry, AssetHandle, AssetType, hot reload
│   ├── animation.rs   — Animation system
│   ├── archetype.rs   — Archetype storage
│   ├── gizmos.rs      — Debug visualization
│   ├── job.rs         — Job system
│   ├── pool.rs        — Object pooling
│   ├── state.rs       — State management
│   ├── script.rs      — Core scripting types
│   ├── lib.rs         — crate root, feature-gated module declarations, re-exports
│   └── main.rs        — CLI binary (kiran run/check)
│
├── Feature-gated modules
│   ├── gpu.rs         — soorat rendering bridge (feature: rendering)
│   ├── audio.rs       — dhvani audio integration (feature: audio)
│   ├── acoustics.rs   — spatial audio pipeline (feature: audio)
│   ├── voice.rs       — speech synthesis/recognition (feature: voice)
│   ├── physics.rs     — impetus physics bridge (feature: physics)
│   ├── fluids.rs      — fluid simulation (feature: fluids)
│   ├── dynamics.rs    — electromagnetic/thermal/material dynamics (feature: dynamics)
│   ├── net.rs         — majra multiplayer (feature: multiplayer)
│   ├── personality.rs — behavior trees, emotion, neural (feature: behavior)
│   ├── ai.rs          — hoosh AI client (feature: ai)
│   ├── nav.rs         — pathfinding/navigation (feature: navigation)
│   ├── biology.rs     — biological simulation (feature: biology)
│   ├── chemistry.rs   — chemical simulation (feature: chemistry)
│   ├── astronomy.rs   — celestial mechanics (feature: astronomy)
│   └── lore.rs        — world history/culture (feature: world)
│
├── examples/
│   ├── scene_loader.rs
│   └── game_loop.rs
│
├── tests/
│   └── integration.rs
│
└── benches/
    ├── engine.rs      (requires rendering)
    ├── personality.rs (requires behavior)
    ├── dynamics.rs    (requires dynamics)
    ├── biology.rs     (requires biology)
    ├── science.rs     (requires chemistry, astronomy, world)
    └── voice_bench.rs (requires voice)
```

## Data Flow

```
Scene TOML → load_scene() → spawn_scene() → World (entities + components)
                                                ↓
                                           Scheduler.run()
                                                ↓
                        ┌─────────┬──────────┬──────────┬─────────┐
                        │  Input  │ Physics  │  Logic   │ Render  │
                        │ (clear  │ (step,   │ (scripts,│ (submit │
                        │  frame, │  sync,   │  AI,     │  draw   │
                        │  poll)  │  raycast)│  audio)  │  cmds)  │
                        └─────────┴──────────┴──────────┴─────────┘
```

## ECS Design

- **Entity**: u64 packed (upper 32 = generation, lower 32 = index)
- **Components**: `Vec<Option<Box<dyn Any>>>` per TypeId, indexed by entity index — O(1)
- **Resources**: `HashMap<TypeId, ResourceEntry>` with integrated change tracking
- **Events**: typed event bus, publish/drain pattern

## Feature Gates (16 gates, 46 optional deps)

| Feature | Optional Deps | Description |
|---------|---------------|-------------|
| `rendering` | soorat, prakash, ranga | GPU rendering pipeline, PBR, color science |
| `audio` | dhvani, naad, shravan, goonj, garjan, ghurni | Audio engine, synthesis, codecs, spatial audio, reverb |
| `voice` | svara, shabda, prani | Speech synthesis, phonetics, voice character |
| `physics` | impetus | Rigid body physics, collision detection |
| `physics-3d` | impetus (3d feature) | Extends `physics` with 3D-specific support |
| `fluids` | pravash | SPH and shallow-water fluid simulation |
| `dynamics` | bijli, dravya, ushma, pavan | Electromagnetic, material, thermal, wind dynamics |
| `ai` | hoosh, reqwest, tokio | AI inference client with async runtime |
| `behavior` | bhava, bodh, mastishk, jantu | Behavior trees, perception, neural networks, creature AI |
| `scripting` | kavach, tokio | WASM scripting sandbox with async runtime |
| `multiplayer` | majra | Relay networking, state synchronization |
| `navigation` | raasta | Pathfinding, navmesh, steering |
| `biology` | sharira, jivanu, rasayan, vanaspati | Anatomy, microbiology, biochemistry, botany |
| `chemistry` | kimiya, khanij, tanmatra, kana | Reactions, mineralogy, molecular, particle |
| `astronomy` | falak, jyotish, tara, brahmanda, badal | Orbital mechanics, astro-observation, stars, cosmology, clouds |
| `world` | itihas, sankhya, varna, pramana | History, demographics, culture, measurement |
| `full` | (all of the above) | Every feature gate enabled |

## Always-On Dependencies (8)

| Crate | Purpose |
|-------|---------|
| hisab | Math types (Vec3, Mat4, Quat) via glam re-exports |
| serde + serde_json | Serialization |
| toml | Scene file format |
| anyhow + thiserror | Error handling |
| tracing | Structured logging |
| crossbeam-channel | Lock-free message passing |
| clap | CLI argument parsing |
| tracing-subscriber | Log output (CLI binary) |

## Consumers

- **joshua** — simulation layer (depends on kiran)
- **salai** — editor (depends on kiran)
