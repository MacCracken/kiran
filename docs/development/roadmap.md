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
- **Rendering backend** → soorat (wgpu rendering engine)
- **Optics / color science** → prakash (ray optics, spectral color, PBR)
- **Multiplayer** → majra (pub/sub, relay, datagrams)

## Completed

### V0.1 — Engine Core (2026-03-22)

- ECS world with generational entity allocator (u64: generation + index)
- Vec-arena component storage (O(1) access by entity index)
- Cached entity count (O(1)), `Entity::from_id()`, `has_component<T>()`
- Singleton resources with tick-based change detection
- GameClock with fixed timestep + variable render
- Typed event bus (publish/drain)
- TOML scene format (entities, position, light, tags)
- Scene loading and entity spawning
- Full KeyCode/MouseButton input with edge-triggered queries
- Renderer trait, Camera with glam view/projection matrices
- SpriteDesc, MeshDesc, DrawCommand, NullRenderer
- Daimon/hoosh AI client (feature-gated)
- Impetus physics bridge (feature-gated)
- CLI: `kiran run` and `kiran check`
- Criterion benchmarks (40) with CSV history tracking
- CI pipeline, Makefile, deny.toml, codecov

### V0.2 — System Scheduling, Scene Hierarchy, Camera Controllers (2026-03-23)

- System trait + SystemStage enum (Input → Physics → GameLogic → Render)
- Scheduler with stage-ordered execution, FnSystem closure wrapper
- Parent/Children components with hierarchy helpers
- Recursive child spawning from TOML
- Prefab/template entities with inheritance
- Material definitions in scene TOML
- OrbitController, FlyController, FollowController
- Resource change detection (tick-based dirty flags)

### V0.3 — Audio & Physics Polish (2026-03-23)

- dhvani audio integration (AudioEngine, SoundSource, AudioListener, SoundTrigger)
- Sound definitions in scene TOML, spatial gain/pan calculations
- Full TOML-driven physics spawning (PhysicsDef → RigidBody + Collider + auto-register)
- PhysicsEngine raycasting (RaycastHit with entity mapping)
- Physics debug rendering (DebugShape with Circle/Box/Capsule kinds)
- Particle spawning, collider-to-entity reverse map (O(1))

### V0.4 — Scripting & Hot Reload (2026-03-23)

- Script component + ScriptEngine resource with message passing
- FileWatcher + SceneReloader with live TOML updates
- `apply_scene_diff()` for in-place scene updates

### Rendering Integration (2026-03-23)

- soorat GPU rendering backend integration (`rendering` feature)
- SooratRenderer implementing kiran Renderer trait
- DrawCommand → soorat Sprite/Color translation
- soorat re-exports (Color, Sprite, SpriteBatch, Vertex2D/3D, GpuContext, WindowConfig)
- prakash optics integration via soorat (color temperature, wavelength, PBR)

### Impetus 3D Integration (2026-03-23)

- `physics-3d` feature flag (activates impetus 3D backend)
- Segment and ConvexHull collider factories
- Segment shape in TOML (`shape = "segment"` with point_a/point_b)
- DebugShapeKind::Segment for debug rendering
- 3D position tests (gravity on Z axis, X/Z unchanged)
- Full 3D physics pipeline verified (register → step → sync)

## Remaining

### Scripting

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
  ├── soorat       — rendering (wgpu, sprites, meshes)
  ├── prakash      — optics (ray tracing, spectral color, PBR)
  ├── dhvani       — audio (spatial audio, DSP, mixing)
  ├── ranga        — image processing (textures, GPU compute)
  ├── majra        — multiplayer (pub/sub, relay, QUIC datagrams)
  ├── kavach       — scripting sandbox (WASM)
  ├── bhava        — emotion/personality (mood vectors, trait spectrums)
  ├── libro        — replay audit trail
  └── t-ron        — NPC tool call security
```

## Stats

- **Source:** ~5,800 lines across 12 modules
- **Tests:** 217 (all features), 40 benchmarks, 12 benchmark runs
- **Features:** `audio` (dhvani), `physics` (impetus), `rendering` (soorat), `ai` (reqwest/tokio)
