# Kiran Roadmap

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

## Completed (0.23.3)

### V0.1 — Engine Core
ECS world (Vec arena O(1)), generational entities, game clock, event bus, TOML scenes, input, renderer trait, NullRenderer, AI client, physics bridge, CLI, benchmarks + CI

### V0.2 — Systems & Scene
System trait + Scheduler, parent/children hierarchy, prefabs/templates, materials, camera controllers, resource change detection

### V0.3 — Audio & Physics Polish
dhvani audio integration, TOML-driven physics spawning, raycasting, debug shapes, particles

### V0.4 — Scripting & Hot Reload
kavach WASM execution, scene + shader hot reload, apply_scene_diff

### V0.5 — Editor
salai scaffolded (EditorApp, inspector, hierarchy, viewport, expression evaluator)

### V0.6 — Multiplayer
majra integration (NetState, Relay, snapshots, deltas, input replication)

### Rendering Integration
soorat GPU backend — full re-export (sprites, meshes, PBR, shadows, animation, terrain, text, UI)

### V1.0 — Production
FrameProfiler, AssetRegistry, examples, documentation, first-party standards compliance

### P0 — Core Game Features (2026-03-24)
- ECS Query system — `query<A>()`, `query2<A,B>()`, `query3<A,B,C>()` with alive-entity filtering
- Transform component — position + rotation (Quat) + scale, `GlobalTransform`, `propagate_transforms()`
- 3D mesh commands wired through SooratRenderer (mesh_queue + mesh_count)
- Mouse delta tracking — `mouse_delta()` with first-move guard
- Gamepad input — `GamepadButton` (15 buttons), `GamepadAxis` (6 axes), edge-triggered queries
- Action mapping — `ActionMap` with `bind()`, `is_action_pressed()`, `is_action_just_pressed()`, `action_axis()`, key/mouse/gamepad bindings

## Priority — Next Work

### P1 — Shipping Quality (2026-03-24)
- Commands buffer — deferred spawn/despawn/insert, applied between stages
- ChangeTracker — per-component mark_changed/mark_added/is_changed/is_added
- OrthoCamera — from_screen, centered, orthographic projection matrix
- Gizmos resource — line, draw_box, sphere, ray, point commands
- NetInterpolation — smooth lerp with retargeting, step_interpolation system
- PredictionBuffer — ring buffer, server reconciliation check_prediction
- AABB + frustum culling — contains_point, intersects, is_visible (view-projection)
- MixBusVolumes — Master/Music/SFX/Ambient/Dialogue/UI with effective volume
- Debug overlay — FrameProfiler::overlay_text() with FPS, systems, entity count
- Component-generic replication — serialize_component/apply_replicated_component

### P2 — Polish and completeness

- [ ] **Parallel system execution** — concurrent systems within a stage when read/write sets don't overlap
- [ ] **System ordering constraints** — `before`/`after` dependencies between systems within a stage
- [ ] **Component bundles** — insert multiple components atomically (reduce boilerplate)
- [ ] **Scene save** — `save_scene(world) -> String` to serialize world back to TOML
- [ ] **Scene instancing at runtime** — spawn prefab mid-game with a parent entity
- [ ] **Game state machine** — menu → playing → paused transitions with enter/exit hooks
- [ ] **Animation state machine** — blend trees, state transitions, not just raw clips
- [ ] **Navigation / pathfinding** — navmesh generation + A* pathfinding
- [ ] **Pitch control** — `SoundSource` playback speed
- [ ] **Sound pooling** — max concurrent sounds per type
- [ ] **Audio fade in/out** — transition helpers for music
- [ ] **Touch input** — `TouchEvent` with ID, position, phase for mobile
- [ ] **Input contexts** — switch input maps between gameplay, UI, menu
- [ ] **Cursor locking** — hide/confine cursor for FPS games
- [ ] **Text input events** — character composition for UI text fields
- [ ] **Async asset loading** — background loading with completion callbacks
- [ ] **Asset preprocessing** — compress textures, optimize meshes at build time
- [ ] **Instanced rendering** — draw thousands of identical meshes efficiently
- [ ] **GPU particle rendering** — render impetus particles on GPU
- [ ] **Multi-pass rendering** — render graph or multi-pass abstraction for deferred shading
- [ ] **Reliable vs unreliable channels** — state updates lossy, RPCs reliable
- [ ] **Interest management** — spatial filtering for multiplayer
- [ ] **Clock synchronization** — NTP-style time sync between server and clients

### P3 — Future

- [ ] Archetype-based SOA component storage
- [ ] Job system / task parallelism
- [ ] SIMD-friendly data layouts
- [ ] Object pooling / arena allocators
- [ ] VR/XR support
- [ ] Procedural world generation via hoosh LLM
- [ ] joshua integration (NPC AI, headless simulation)
- [ ] bhava integration (NPC emotions, personality)
- [ ] Science crate integration (optics, fluids, electromagnetism, thermodynamics)

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
