# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

#### V0.2 — System Scheduling, Scene Hierarchy, Camera Controllers
- `System` trait with `run(&mut World)`, `stage()`, `name()` for ordered update pipeline.
- `SystemStage` enum: Input → Physics → GameLogic → Render.
- `Scheduler` — collects systems, sorts by stage, runs in order.
- `FnSystem` closure wrapper for quick system creation.
- `Parent`/`Children` components with `set_parent()` and `remove_parent()` helpers.
- Recursive child entity spawning from TOML (`[[entities.children]]`).
- `PrefabDef` templates: `[[prefabs]]` section, entities reference via `prefab = "name"`.
- `Material` component (color, texture path) in scene TOML.
- `OrbitController`, `FlyController`, `FollowController` camera controllers.
- `PartialEq` on `SceneDefinition`, `EntityDef`, `RenderConfig`.
- `skip_serializing_if` on optional/empty scene fields for clean TOML output.
- Resource change detection: `is_resource_changed<T>()`, `clear_resource_changed<T>()`, `increment_tick()`.

#### V0.3 — Audio & Physics Polish
- `audio` feature with dhvani integration.
- `AudioEngine` resource wrapping dhvani graph + clock.
- `SoundSource` component with builder API (source, volume, spatial, looping, max_distance).
- `AudioListener` component for spatial audio positioning.
- `SoundTrigger` component linking collision/action events to sounds.
- `process_sound_triggers()` system with EventBus integration.
- Spatial gain and pan calculation helpers.
- `SoundDef` in scene TOML (`[entities.sound]`).
- `PhysicsDef`/`ColliderDef` in scene TOML (`[entities.physics]`).
- Full TOML-driven physics spawning (auto-creates RigidBody, Collider, PhysicsPosition, Velocity and registers with PhysicsEngine).
- `PhysicsEngine::raycast()` returning `RaycastHit` with entity mapping.
- `PhysicsEngine::spawn_particle()` for impetus particle integration.
- `PhysicsEngine::entity_count()`.
- `PhysicsEngine::debug_shapes()` — generates `DebugShape` wireframes (Circle/Box/Capsule) for all registered colliders.
- Collider-to-entity reverse HashMap for O(1) collision event lookup.

#### V0.4 — Scripting & Hot Reload
- `Script` component (WASM source path, enabled, JSON state).
- `ScriptMessage` for engine ↔ script communication.
- `ScriptEngine` resource with inbox/outbox messaging and frame counters.
- `run_scripts()` system delivering messages to entity scripts.
- `FileWatcher` for polling-based file change detection.
- `SceneReloader` for load-and-watch with auto-despawn/respawn on change.
- `apply_scene_diff()` for in-place scene updates (add/remove/update by name).

### Changed
- **Breaking:** Flattened from workspace (6 crates) to single crate with feature flags.
- **Breaking:** Component storage changed from `HashMap<u64, Box<dyn Any>>` to `Vec<Option<Box<dyn Any>>>` (Vec arena, O(1) access by entity index).
- `EntityAllocator::alive_count()` is now O(1) via cached counter (was O(n) scan).
- Mouse buttons now have edge-triggered queries (`is_mouse_button_just_pressed/released`).
- `Entity::from_id(u64)` added for safe reconstruction from raw ids.
- `World::has_component<T>()` added.

### Fixed
- `spatial_gain` overflow — negative distance produced gain > 1.0 (clamped to 0.0–1.0).
- `SoundTrigger.source` was ignored by trigger processor — now sets source path on playback.
- Entity generation bug in `run_scripts` — hardcoded generation 0, broke recycled entity messaging.
- `run_scripts` never called `record_exec()` — exec_count was always 0.
- `is_resource_changed` returned false for newly inserted resources at tick 0.
- Redundant `LightComponent` double-lookup in CLI `run_scene`.

### Performance
- `get_component`: 33ns → 14ns (2.4x faster, Vec arena).
- `insert_component`: 50ns → 30ns (1.7x faster).
- `spawn_100_entities`: 11.3µs → 5.0µs (2.3x faster).
- `iterate_components` (1000 entities): 39µs → 3.5µs (11x faster).
- `entity_count`: O(n) → O(1) (0.24ns).

## [0.1.0] - 2026-03-22

### Added
- ECS world with generational entity allocator, typed component storage,
  singleton resources, game clock with fixed timestep, typed event bus.
- TOML scene format with entity definitions, position, light, and tag
  components; load and spawn helpers.
- Keyboard (full key code set), mouse, and scroll input state tracking with
  edge-triggered (just-pressed/released) queries.
- Renderer trait, camera with glam view/projection matrices, sprite and mesh
  descriptors, NullRenderer for headless testing.
- AGNOS daimon client for agent registration/heartbeat, hoosh LLM inference.
- Impetus physics bridge with RigidBody, Collider, Velocity, PhysicsPosition
  components and `physics_step()` system.
- CLI: `kiran run <scene>` and `kiran check <scene>` commands.
- Criterion benchmarks with CSV history tracking.
- CI pipeline (GitHub Actions), Makefile, deny.toml, codecov.yml.
