# Kiran Roadmap

> **Kiran** is the game engine. Physics lives in [impetus](https://github.com/MacCracken/impetus).
> Simulation and AI NPCs live in [joshua](https://github.com/MacCracken/joshua).
> Higher math lives in [hisab](https://github.com/MacCracken/hisab).
> Emotion/personality lives in [bhava](https://github.com/MacCracken/bhava).

## Scope

Kiran owns the **engine core**: ECS, game loop, scene management, input, rendering integration, and physics integration. It is the thin orchestration layer that composes AGNOS shared crates into a game engine.

Kiran does NOT own:
- **Physics simulation** → impetus (rigid bodies, collision, particles, spatial hash)
- **Higher math** → hisab (vectors, geometry, calculus, numerical methods, spatial structures)
- **Simulation / AI NPCs** → joshua (headless sim, NPC agents, deterministic replay)
- **Emotion / personality** → bhava (mood vectors, trait spectrums, sentiment)
- **Audio** → dhvani (spatial audio, DSP, mixing)
- **Rendering backend** → aethersafta (wgpu scene graph, compositing)
- **Multiplayer** → majra (pub/sub, relay, datagrams)

## Completed

### V0.1 — Engine Core (2026-03-22)

- ECS world with generational entity allocator (u64: generation + index)
- Vec-arena component storage (O(1) access by entity index)
- Cached entity count (O(1))
- `Entity::from_id()` for safe reconstruction from raw ids
- `has_component<T>()` query
- Singleton resources
- GameClock with fixed timestep + variable render
- Typed event bus (publish/drain)
- TOML scene format (entities, position, light, tags)
- Scene loading and entity spawning
- Full KeyCode enum, MouseButton, scroll, mouse position tracking
- Edge-triggered queries for keys and mouse buttons (just_pressed/released)
- Renderer trait, Camera with glam view/projection matrices
- SpriteDesc, MeshDesc, DrawCommand
- NullRenderer for headless testing
- Daimon/hoosh AI client (feature-gated)
- Impetus physics bridge (feature-gated)
- CLI: `kiran run` and `kiran check`
- Criterion benchmarks with CSV history tracking
- CI pipeline, Makefile, deny.toml, codecov

### V0.2 — System Scheduling, Scene Hierarchy, Camera Controllers (2026-03-23)

- System trait with `run(&mut World)`, `stage()`, `name()`
- SystemStage enum: Input → Physics → GameLogic → Render
- Scheduler: collects systems, sorts by stage, runs in order
- FnSystem closure wrapper
- Parent/Children components with hierarchy helpers (set_parent, remove_parent)
- Recursive child spawning from TOML (`[[entities.children]]`)
- Prefab/template entities (`[[prefabs]]` + `prefab = "name"`)
- Material definitions in scene TOML (color, texture path)
- OrbitController, FlyController, FollowController camera controllers
- `skip_serializing_if` for clean TOML output
- PartialEq on SceneDefinition, EntityDef, RenderConfig

### V0.3 — Audio & Physics Polish (2026-03-23)

- dhvani audio integration (AudioEngine resource, graph + clock)
- SoundSource component (source, volume, spatial, looping, max_distance)
- AudioListener component
- SoundTrigger component (collision/action → sound)
- `process_sound_triggers()` system with event bus integration
- Spatial gain/pan calculations
- Sound definition in scene TOML (`[entities.sound]`)
- Physics definition in scene TOML (`[entities.physics]` + collider)
- PhysicsEngine raycasting API (RaycastHit with entity mapping)
- Particle spawning through PhysicsEngine
- PhysicsEngine entity_count()
- Collider-to-entity reverse HashMap (O(1) lookup)

### V0.4 — Scripting & Hot Reload (2026-03-23)

- Script component (WASM source, enabled, JSON state)
- ScriptMessage for engine ↔ script communication (sender/target/kind/payload)
- ScriptEngine resource (inbox/outbox, frame counter, config)
- `run_scripts()` system with message delivery
- FileWatcher (polling-based mtime change detection)
- SceneReloader (load + watch, auto-despawn/respawn on file change)
- `apply_scene_diff()` (update in place by name, add new, remove missing)

## Remaining

### V0.2 — Rendering Integration (blocked on aethersafta wiring)

- [ ] aethersafta backend (wgpu scene graph integration)
- [ ] Sprite rendering pipeline (2D)
- [ ] Basic 3D mesh rendering (glTF loading via aethersafta)
- [ ] Window management (winit integration)
- [ ] Debug wireframe overlay (collider shapes from impetus)
- [ ] Resource change detection (dirty flags)

### V0.3 — Physics Polish

- [ ] Full TOML-driven physics spawning (parse PhysicsDef → register with PhysicsEngine)
- [ ] Full impetus 3D integration
- [ ] Physics debug rendering (wireframe colliders)

### V0.4 — Scripting

- [ ] kavach WASM backend wiring (blocked on kavach wasm feature build)
- [ ] Hot reload for shaders

### V0.5 — Editor

- [ ] egui-based visual editor
- [ ] Entity inspector (view/edit components)
- [ ] Scene hierarchy tree view
- [ ] Viewport with gizmos (translate, rotate, scale)
- [ ] Play/pause/step controls
- [ ] Scene save/load from editor
- [ ] Component drag-and-drop (add physics, add AI, add sound)

### V0.6 — Multiplayer

- [ ] majra integration for networked game state
- [ ] Client-server architecture (authoritative server)
- [ ] State snapshot + delta compression
- [ ] Input prediction and reconciliation
- [ ] Lobby / matchmaking via daimon
- [ ] QUIC transport (when majra QUIC lands)

### V0.7 — Advanced Rendering

- [ ] PBR materials (metallic-roughness workflow)
- [ ] Shadow mapping (directional, point, spot)
- [ ] Post-processing pipeline (bloom, tone mapping, SSAO)
- [ ] Skeletal animation (glTF skinned meshes)
- [ ] Terrain rendering (heightmap or procedural)
- [ ] Particle visual effects (GPU particles via ranga)
- [ ] UI system (in-game HUD, menus)

### V1.0 — Production Ready

- [ ] API stabilization
- [ ] Comprehensive documentation with tutorials
- [ ] Example games (2D platformer, 3D exploration, NPC sandbox)
- [ ] Performance profiler (frame timeline, per-system cost)
- [ ] Asset pipeline (import, convert, cache, hot reload)
- [ ] WebGPU export target (run in browser)
- [ ] Publish to crates.io

### Post-V1

- [ ] VR/XR support
- [ ] Procedural world generation via hoosh LLM
- [ ] joshua integration (NPC AI, headless simulation mode)
- [ ] bhava integration (NPC emotions, personality-driven behavior)

#### Future Science Crate Integration

As AGNOS science simulation crates come online, kiran gains new capabilities without engine changes — they plug in via the same impetus/hisab foundation:

- Optics (ray tracing, refraction) → realistic lighting, laser puzzles
- Fluid dynamics (SPH, Navier-Stokes) → water, smoke, fire
- Electromagnetism (fields, charges) → physics puzzles, sci-fi mechanics
- Thermodynamics (heat transfer) → environmental simulation
- Quantum mechanics (state vectors) → joshua quantum simulation backend

See [shared-crates.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/shared-crates.md) for the full science crate roadmap.

## Dependency Map

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms, spatial structures)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── aethersafta  — rendering (wgpu scene graph, compositing)
  ├── dhvani       — audio (spatial audio, DSP, mixing)
  ├── ranga        — image processing (textures, GPU compute)
  ├── majra        — multiplayer (pub/sub, relay, QUIC datagrams)
  ├── kavach       — scripting sandbox (WASM)
  ├── bhava        — emotion/personality (mood vectors, trait spectrums)
  ├── libro        — replay audit trail
  └── t-ron        — NPC tool call security
```

## Stats

- **Source:** 5,500 lines across 10 modules
- **Tests:** 191 (all features), 40 benchmarks with CSV history
- **Features:** `audio` (dhvani), `physics` (impetus), `ai` (reqwest/tokio)
