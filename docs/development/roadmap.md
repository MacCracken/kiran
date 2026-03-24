# Kiran Roadmap

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

## Completed (0.23.3)

### V0.1 — Engine Core
ECS world (Vec arena O(1)), generational entities, game clock, event bus, TOML scenes, input, renderer trait, NullRenderer, AI client (hoosh), physics bridge, CLI, benchmarks + CI

### V0.2 — Systems & Scene
System trait + Scheduler (topological ordering, before/after), parent/children hierarchy, prefabs/templates, materials (PBR: metallic/roughness), camera controllers, resource change detection

### V0.3 — Audio & Physics Polish
dhvani audio integration (mix buses, pitch, pooling, fade), TOML-driven physics spawning, raycasting, debug shapes, particles

### V0.4 — Scripting & Hot Reload
kavach WASM execution, scene + shader hot reload, apply_scene_diff

### V0.5 — Editor
salai scaffolded (EditorApp, inspector, hierarchy, viewport, expression evaluator, animation panel, terrain panel)

### V0.6 — Multiplayer
majra integration (NetState, Relay, snapshots, deltas, interpolation, prediction, reliable channels, interest management, clock sync, component replication)

### V0.7 — Rendering Integration
soorat GPU backend — sprites, meshes, PBR, shadows, animation, terrain, text, UI, instancing, GPU particles, render graph, MeshDrawParams, SpriteBatchDrawParams

### V0.8 — Game Systems
ECS queries, transforms (Quat + GlobalTransform), gamepad/touch/text/cursor input, action mapping, commands buffer, change tracking, ortho camera, gizmos, AABB + frustum culling, game state machine, animation state machine, bundles, scene save/load/instancing

### V0.9 — Navigation & AI
raasta integration (NavAgent, grid/navmesh/flow fields/steering), bhava personality (mood stimuli, decay, compose_prompt)

### V1.0 — Production
FrameProfiler, AssetRegistry (async loading, preprocessing pipeline), object pooling (Pool, FrameArena), SIMD layouts (SimdVec, Soa2d, Soa3d), fluid dynamics (pravash bridge), examples, documentation, first-party standards compliance

## Priority — Next Work

### P3 — Future

- [x] ~~Archetype-based SOA component storage~~ — `ArchetypeStore` with type-signature grouping, cache-friendly query/query2
- [x] ~~Job system / task parallelism~~ — `JobPool` thread pool with `scope()` for barrier sync, crossbeam channels
- [ ] VR/XR support
- [ ] Electromagnetism crate integration (TBD — not yet scaffolded)
- [ ] Thermodynamics crate integration (TBD — not yet scaffolded)
- [ ] Quantum mechanics crate integration (TBD — not yet scaffolded)

Note: prakash (optics) integrated via soorat, pravash (fluids) integrated via `fluids` feature. Remaining science crates are not yet scaffolded — see [shared-crates.md](../../agnosticos/docs/development/applications/shared-crates.md).

## Dependency Map

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── soorat       — rendering (wgpu, PBR, shadows, animation, terrain, text, UI)
  ├── dhvani       — audio (spatial audio, DSP, mixing)
  ├── majra        — multiplayer (pub/sub, relay)
  ├── kavach       — scripting sandbox (WASM)
  ├── bhava        — emotion/personality
  ├── raasta       — navigation/pathfinding
  ├── pravash      — fluid dynamics (SPH, shallow water)
  ├── hoosh        — AI inference gateway
  └── prakash      — optics (via soorat)
```

## Stats

- **Source:** ~10,000 lines across 19 modules
- **Tests:** 516 (all features), 66 benchmarks
- **Features:** `ai`, `audio`, `physics`, `physics-3d`, `rendering`, `scripting`, `multiplayer`, `personality`, `navigation`, `fluids`
- **Ecosystem:** 11 AGNOS crates integrated
- **Examples:** scene_loader, game_loop
