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

## Priority — Next Work

### P0 — Blocking for any real game

- [ ] **ECS Query system** — iterate entities by component tuple (`Query<(&Position, &Velocity)>`), filter support, world iteration without manual `get_component` per entity
- [ ] **Transform component** — replace `Position(Vec3)` with `Transform { position, rotation, scale }`, add `GlobalTransform` component, propagation system for hierarchy
- [ ] **Wire 3D mesh through Renderer** — `SooratRenderer::submit(Mesh)` currently no-ops. Bridge soorat's MeshPipeline through the Renderer trait
- [ ] **Mouse delta tracking** — `InputState::mouse_delta()` for FPS camera control
- [ ] **Gamepad input** — `GamepadButton`, `GamepadAxis`, joystick deadzone, gilrs or winit integration
- [ ] **Action mapping** — abstract layer mapping physical inputs to logical actions ("jump" → Space/A-button/Tap)

### P1 — Blocking for shipping quality

- [ ] **Entity commands buffer** — deferred spawn/despawn/insert from systems without `&mut World`, applied between stages
- [ ] **Component change detection** — `Changed<T>`, `Added<T>` filters on components (resources already have it)
- [ ] **Client-side prediction** — rollback/resimulation for multiplayer latency compensation
- [ ] **Entity interpolation** — smooth position updates between network ticks instead of snapping
- [ ] **Frustum culling** — visibility testing before submitting draw calls (hisab AABB + camera frustum)
- [ ] **Audio mix buses** — SFX/music/ambient/dialogue volume groups with independent control
- [ ] **Debug gizmos API** — expose `draw_box()`, `draw_sphere()`, `draw_ray()` from game code through kiran
- [ ] **In-game debug overlay** — render profiler data (FPS, system timings, entity count) on screen
- [ ] **Component-generic replication** — sync rotation, velocity, health, animation state — not just position
- [ ] **Orthographic camera** — 2D games need ortho projection, not just perspective

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
