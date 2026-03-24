# Kiran Roadmap

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

All planned features through V1.0 have been implemented. This document serves as a reference for what was built and where future work may go.

## Completed

### V0.1 — Engine Core (2026-03-22)
ECS world (Vec arena O(1)), generational entities, game clock, event bus, TOML scene format, input handling, renderer trait, NullRenderer, daimon/hoosh AI client, impetus physics bridge, CLI, benchmarks + CI

### V0.2 — Systems & Scene (2026-03-23)
System trait + Scheduler (Input→Physics→GameLogic→Render), parent/children hierarchy, prefabs/templates, materials, OrbitController/FlyController/FollowController, resource change detection

### V0.3 — Audio & Physics Polish (2026-03-23)
dhvani audio (AudioEngine, SoundSource, AudioListener, SoundTrigger), TOML-driven physics spawning, raycasting, debug shapes, particle spawning, collider reverse map

### V0.4 — Scripting & Hot Reload (2026-03-23)
Script engine + kavach WASM execution, SceneReloader + ShaderReloader, apply_scene_diff

### V0.5 — Editor (2026-03-23)
salai scaffolded (EditorApp, inspector, hierarchy, viewport, expression evaluator)

### V0.6 — Multiplayer (2026-03-23)
majra integration (NetState, Relay, snapshots, deltas, input replication, peer management)

### Rendering Integration (2026-03-23)
soorat GPU backend — full re-export of sprites, meshes, PBR, shadows, post-processing, skeletal animation, debug lines, terrain, text, UI, render targets, lights

### Impetus 3D (2026-03-23)
physics-3d feature, segment/convex hull colliders, 3D gravity verification

### V1.0 — Production (2026-03-23)
FrameProfiler (per-system timing, EMA averages, slow frame detection), AssetRegistry (typed handles, auto type inference, hot reload integration), examples (scene_loader, game_loop)

## Future Considerations

- [ ] VR/XR support
- [ ] Procedural world generation via hoosh LLM
- [ ] joshua integration (NPC AI, headless simulation mode)
- [ ] bhava integration (NPC emotions, personality-driven behavior)
- [ ] Async asset loading
- [ ] Doc tests (`cargo test --doc`)
- [ ] Architecture overview document
- [ ] Getting-started tutorial

### Science Crate Integration
- Optics (prakash ray tracing) → realistic lighting, caustics
- Fluid dynamics (SPH) → water/smoke/fire
- Electromagnetism → physics puzzles, sci-fi mechanics
- Thermodynamics → environmental simulation

## Dependency Map

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── soorat       — rendering (wgpu, PBR, shadows, animation, terrain, text, UI)
  ├── prakash      — optics (spectral color, PBR math)
  ├── dhvani       — audio (spatial audio, DSP, mixing)
  ├── majra        — multiplayer (pub/sub, relay)
  ├── kavach       — scripting sandbox (WASM)
  ├── bhava        — emotion/personality
  ├── libro        — audit trail
  └── t-ron        — NPC tool call security
```

## Stats

- **Source:** ~8,500 lines across 15 modules
- **Tests:** ~240 (all features), 45 benchmarks, 28 runs
- **Features:** `audio`, `physics`, `physics-3d`, `rendering`, `scripting`, `multiplayer`, `ai`
- **Ecosystem:** 8 AGNOS crates integrated
- **Examples:** scene_loader, game_loop
