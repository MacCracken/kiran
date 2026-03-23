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

## V0.1 — Engine Core (done, 2026-03-22)

### kiran-core (24 tests)
- ECS world with generational entity allocator (u64: generation + index)
- Typed component storage (HashMap<TypeId, HashMap<Entity, Box<dyn Any>>>)
- Singleton resources
- GameClock with fixed timestep + variable render
- Typed event bus (publish/drain)

### kiran-scene (11 tests)
- TOML scene format (entities, position, light, tags, physics, AI)
- Scene loading and entity spawning into World
- Position, Name, Light components

### kiran-input (10 tests)
- Full KeyCode enum (A-Z, digits, arrows, modifiers, F-keys)
- MouseButton, scroll, mouse position tracking
- Edge-triggered queries (just_pressed, just_released)

### kiran-render (8 tests)
- Renderer trait (init, begin_frame, submit, end_frame, shutdown)
- Camera with glam view/projection matrices
- SpriteDesc, MeshDesc, DrawCommand
- NullRenderer for headless testing

### kiran-ai (2 tests)
- Daimon/hoosh client for agent registration

### kiran-physics
- Impetus integration bridge (added post-scaffold)

### CLI
- `kiran run <scene.toml>` — load and run
- `kiran check <scene.toml>` — validate

## V0.2 — Rendering Integration

### kiran-render
- [ ] aethersafta backend (wgpu scene graph integration)
- [ ] Sprite rendering pipeline (2D)
- [ ] Basic 3D mesh rendering (glTF loading via aethersafta)
- [ ] Window management (winit integration)
- [ ] Viewport / camera controller (orbit, fly, follow)
- [ ] Debug wireframe overlay (collider shapes from impetus)

### kiran-core
- [ ] System trait — ordered update pipeline (input → physics → game logic → render)
- [ ] System scheduling with dependency ordering
- [ ] Resource change detection (dirty flags)

### kiran-scene
- [ ] Material definitions in scene TOML (color, texture path)
- [ ] Prefab / template entities (spawn from template)
- [ ] Scene hierarchy (parent-child entity relationships)

## V0.3 — Audio & Physics Polish

### Audio integration
- [ ] dhvani spatial audio integration
- [ ] Sound component in scene TOML (source, volume, spatial, loop)
- [ ] Audio listener tied to camera
- [ ] Event-driven sound triggers (collision → sound, action → sound)

### kiran-physics
- [ ] Full impetus 2D integration (rigid bodies, colliders, joints from scene TOML)
- [ ] Full impetus 3D integration
- [ ] Physics debug rendering (wireframe colliders)
- [ ] Collision event → ECS event bridge
- [ ] Raycasting API (mouse picking, line-of-sight)
- [ ] Particle system integration (impetus particles)

## V0.4 — Scripting & Hot Reload

- [ ] WASM scripting via kavach (sandboxed game logic)
- [ ] Hot reload for scripts (file watcher → reload without restart)
- [ ] Hot reload for scenes (edit TOML → live update)
- [ ] Hot reload for shaders
- [ ] Script ↔ ECS bridge (scripts can query/modify components)

## V0.5 — Editor

### kiran-editor (new crate)
- [ ] egui-based visual editor
- [ ] Entity inspector (view/edit components)
- [ ] Scene hierarchy tree view
- [ ] Viewport with gizmos (translate, rotate, scale)
- [ ] Play/pause/step controls
- [ ] Scene save/load from editor
- [ ] Component drag-and-drop (add physics, add AI, add sound)

## V0.6 — Multiplayer

- [ ] majra integration for networked game state
- [ ] Client-server architecture (authoritative server)
- [ ] State snapshot + delta compression
- [ ] Input prediction and reconciliation
- [ ] Lobby / matchmaking via daimon
- [ ] QUIC transport (when majra QUIC lands — see network-evolution.md)

## V0.7 — Advanced Rendering

- [ ] PBR materials (metallic-roughness workflow)
- [ ] Shadow mapping (directional, point, spot)
- [ ] Post-processing pipeline (bloom, tone mapping, SSAO)
- [ ] Skeletal animation (glTF skinned meshes)
- [ ] Terrain rendering (heightmap or procedural)
- [ ] Particle visual effects (GPU particles via ranga)
- [ ] UI system (in-game HUD, menus)

## V1.0 — Production Ready

- [ ] API stabilization
- [ ] Comprehensive documentation with tutorials
- [ ] Example games (2D platformer, 3D exploration, NPC sandbox)
- [ ] Performance profiler (frame timeline, per-system cost)
- [ ] Asset pipeline (import, convert, cache, hot reload)
- [ ] WebGPU export target (run in browser)
- [ ] Benchmarks with history tracking
- [ ] Publish to crates.io

## Post-V1

- [ ] VR/XR support
- [ ] Procedural world generation via hoosh LLM
- [ ] joshua integration (NPC AI, headless simulation mode)
- [ ] bhava integration (NPC emotions, personality-driven behavior)

### Future Science Crate Integration

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
