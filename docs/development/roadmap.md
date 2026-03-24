# Kiran Roadmap

> **Kiran** is the game engine. Physics lives in [impetus](https://github.com/MacCracken/impetus).
> Simulation and AI NPCs live in [joshua](https://github.com/MacCracken/joshua).
> Higher math lives in [hisab](https://github.com/MacCracken/hisab).
> Emotion/personality lives in [bhava](https://github.com/MacCracken/bhava).

## Scope

Kiran owns the **engine core**: ECS, game loop, scene management, input, rendering integration, and physics integration. It is the thin orchestration layer that composes AGNOS shared crates into a game engine.

Kiran does NOT own:
- **Physics simulation** → impetus
- **Higher math** → hisab (wraps glam)
- **Simulation / AI NPCs** → joshua
- **Emotion / personality** → bhava
- **Audio** → dhvani
- **Rendering backend** → soorat (wgpu)
- **Optics / color science** → prakash
- **Multiplayer transport** → majra
- **Scripting sandbox** → kavach (WASM)

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
- Renderer trait, Camera with hisab view/projection matrices
- SpriteDesc, MeshDesc, DrawCommand, NullRenderer
- Daimon/hoosh AI client (feature-gated)
- Impetus physics bridge (feature-gated)
- CLI: `kiran run` and `kiran check`
- Criterion benchmarks (45) with CSV history tracking
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
- Physics debug rendering (DebugShape with Circle/Box/Capsule/Segment)
- Particle spawning, collider-to-entity reverse map (O(1))

### V0.4 — Scripting & Hot Reload (2026-03-23)

- Script component + ScriptEngine resource with message passing
- kavach WASM execution wired (scripting feature, wasmtime + fuel metering)
- FileWatcher + SceneReloader with live TOML updates
- ShaderReloader — watches .wgsl files, publishes ShaderChanged events
- `apply_scene_diff()` for in-place scene updates

### V0.5 — Editor (2026-03-23)

- salai scaffolded as separate project
- EditorApp with play/pause/step state machine
- Entity inspector, hierarchy builder, viewport with gizmos
- Expression evaluator (abaco) for inspector fields

### V0.6 — Multiplayer (2026-03-23)

- majra integration (multiplayer feature)
- NetState resource with Relay, node identity, peer management
- State snapshots + delta compression (adaptive linear/HashMap)
- Input replication, NetOwner/Replicated components
- 26+ tests, serde roundtrips, 5 benchmarks

### Rendering Integration (2026-03-23)

- soorat GPU rendering backend (rendering feature)
- SooratRenderer implementing kiran Renderer trait
- Full re-export: sprites, meshes, PBR materials, shadows, post-processing, skeletal animation, debug lines, render targets, lights

### Impetus 3D Integration (2026-03-23)

- physics-3d feature, segment/convex hull colliders, 3D gravity tests

## Remaining — V1.0 Production

### API Stabilization

- [ ] Review all public types for consistency (naming, derives, builder patterns)
- [ ] Add `#[non_exhaustive]` to enums that may grow
- [ ] Ensure all public types have `Debug`, appropriate `Clone`/`PartialEq`
- [ ] Review error types — consistent variants across modules

### Documentation

- [ ] Module-level doc comments with usage examples
- [ ] Doc tests (`cargo test --doc` must pass)
- [ ] docs/architecture/overview.md — system diagram, module relationships
- [ ] docs/guides/getting-started.md — tutorial for new users

### Example Games

- [ ] `examples/sprite_demo.rs` — 2D sprite rendering with input
- [ ] `examples/physics_demo.rs` — rigid bodies falling, collisions
- [ ] `examples/scene_loader.rs` — load TOML scene, walk hierarchy
- [ ] `examples/multiplayer_demo.rs` — two-node state sync

### Performance Profiler

- [ ] Frame timing system (per-system cost tracking in Scheduler)
- [ ] Profile resource — stores frame timeline data
- [ ] System to log slow frames (>16ms warning)

### Asset Pipeline

- [ ] Asset registry (path → typed handle)
- [ ] Async asset loading
- [ ] Asset hot reload (integrate with FileWatcher)

## Post-V1

- [ ] VR/XR support
- [ ] Procedural world generation via hoosh LLM
- [ ] joshua integration (NPC AI, headless simulation mode)
- [ ] bhava integration (NPC emotions, personality-driven behavior)

#### Future Science Crate Integration

- Optics (prakash ray tracing) → realistic lighting, caustics
- Fluid dynamics (SPH) → water/smoke/fire
- Electromagnetism → physics puzzles, sci-fi mechanics
- Thermodynamics → environmental simulation

## Dependency Map

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── soorat       — rendering (wgpu, PBR, shadows, animation)
  ├── prakash      — optics (spectral color, PBR math)
  ├── dhvani       — audio (spatial audio, DSP, mixing)
  ├── majra        — multiplayer (pub/sub, relay)
  ├── kavach       — scripting sandbox (WASM)
  ├── bhava        — emotion/personality
  ├── libro        — audit trail
  └── t-ron        — NPC tool call security
```

## Stats

- **Source:** ~7,900 lines across 13 modules
- **Tests:** ~260 (all features), 45 benchmarks, 27 runs
- **Features:** `audio`, `physics`, `physics-3d`, `rendering`, `scripting`, `multiplayer`, `ai`
- **Ecosystem:** 8 AGNOS crates integrated
