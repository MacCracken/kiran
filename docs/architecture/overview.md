# Kiran Architecture

## Overview

Kiran is an AI-native game engine that orchestrates AGNOS shared crates into a unified development framework. It owns the ECS, game loop, scene management, and integration layer — delegating physics, rendering, audio, networking, and scripting to specialized crates.

## Module Structure

```
src/
├── Core
│   ├── world.rs       — ECS world, Entity, EntityAllocator, GameClock, EventBus, Scheduler
│   ├── scene.rs       — TOML scene format, hierarchy, prefabs, materials, spawning
│   ├── input.rs       — KeyCode, MouseButton, InputState, edge-triggered queries
│   ├── render.rs      — Renderer trait, Camera, controllers (Orbit/Fly/Follow), NullRenderer
│   ├── reload.rs      — FileWatcher, SceneReloader, ShaderReloader, apply_scene_diff
│   ├── profiler.rs    — FrameProfiler (per-system timing, EMA, slow frame detection)
│   ├── asset.rs       — AssetRegistry, AssetHandle, AssetType, hot reload
│   ├── lib.rs         — crate root, feature-gated module declarations, re-exports
│   └── main.rs        — CLI binary (kiran run/check)
│
├── Feature-gated
│   ├── gpu.rs         — soorat rendering bridge (feature: rendering)
│   ├── audio.rs       — dhvani audio integration (feature: audio)
│   ├── physics.rs     — impetus physics bridge (feature: physics)
│   ├── net.rs         — majra multiplayer (feature: multiplayer)
│   ├── script.rs      — kavach WASM scripting (feature: scripting)
│   └── ai.rs          — daimon/hoosh AI client (feature: ai)
│
├── examples/
│   ├── scene_loader.rs
│   └── game_loop.rs
│
├── tests/
│   └── integration.rs
│
└── benches/
    └── engine.rs      (requires rendering feature)
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
- **Components**: `Vec<Option<Box<dyn Any>>>` per TypeId, indexed by entity index → O(1)
- **Resources**: `HashMap<TypeId, ResourceEntry>` with integrated change tracking
- **Events**: typed event bus, publish/drain pattern

## Ecosystem Integration

| Crate | Feature | Integration Point |
|-------|---------|-------------------|
| hisab | always | Math types (Vec3, Mat4, Quat) |
| impetus | `physics` | PhysicsEngine resource, RigidBody/Collider components |
| soorat | `rendering` | SooratRenderer, re-exports full GPU pipeline |
| dhvani | `audio` | AudioEngine resource, SoundSource/Trigger components |
| majra | `multiplayer` | NetState resource, Relay, state sync |
| kavach | `scripting` | WasmBackend, Script component, exec_wasm() |
| prakash | via soorat | Color temperature, spectral color, PBR math |
