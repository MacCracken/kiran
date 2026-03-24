# Changelog

All notable changes to this project will be documented in this file.

## [0.23.3] - 2026-03-23

### Added

#### Engine Core
- ECS world with generational entity allocator (u64: generation + index).
- Vec-arena component storage — O(1) access by entity index (replaced HashMap).
- Cached entity count (O(1)), `Entity::from_id()`, `has_component<T>()`.
- Singleton resources with tick-based change detection (`is_resource_changed`, `clear_resource_changed`).
- `GameClock` with fixed timestep + variable render, accumulator pattern.
- Typed `EventBus` (publish/drain by type).
- `System` trait + `SystemStage` enum (Input → Physics → GameLogic → Render).
- `Scheduler` — stage-ordered system execution, `FnSystem` closure wrapper.

#### Scene
- TOML scene format with entities, position, light, tags, materials, sound, physics.
- `Parent`/`Children` components with hierarchy helpers (`set_parent`, `remove_parent`).
- Recursive child spawning from TOML (`[[entities.children]]`).
- `PrefabDef` templates with inheritance (`[[prefabs]]` + `prefab = "name"`).
- `Material` component (color, texture path).
- `PhysicsDef`/`ColliderDef` for TOML-driven physics spawning (ball/box/capsule/segment).
- `SoundDef` for TOML-driven audio sources.
- `skip_serializing_if` on optional/empty fields for clean TOML output.
- `PartialEq` on `SceneDefinition`, `EntityDef`, `RenderConfig`.

#### Input
- Full `KeyCode` enum (A-Z, digits, arrows, modifiers, F-keys, punctuation).
- `MouseButton` with edge-triggered queries (`is_mouse_button_just_pressed/released`).
- Scroll accumulation, mouse position tracking.
- Serde roundtrip for all input event variants.

#### Rendering
- `Renderer` trait (init, begin_frame, submit, end_frame, shutdown).
- `Camera` with view/projection matrices, `OrbitController`, `FlyController`, `FollowController`.
- `NullRenderer` for headless testing.
- `SooratRenderer` — GPU-accelerated renderer via soorat (`rendering` feature).
- Full soorat re-export: sprites, meshes, PBR materials, shadows, post-processing, skeletal animation, debug lines, terrain, text, UI, render targets, lights.

#### Audio (`audio` feature)
- dhvani `AudioEngine` resource with graph + clock.
- `SoundSource` component with builder API.
- `AudioListener` component, spatial gain/pan calculations.
- `SoundTrigger` component with collision/action → sound mapping.
- `process_sound_triggers()` system with EventBus integration.

#### Physics (`physics` feature)
- impetus `PhysicsEngine` resource with body/collider registration.
- `RigidBody`, `Collider`, `Velocity`, `PhysicsPosition` components.
- `physics_step()` system — step simulation, sync positions, publish collision events.
- `raycast()` returning `RaycastHit` with entity mapping.
- `debug_shapes()` — wireframe DebugShape generation (Circle/Box/Capsule/Segment).
- `spawn_particle()` for impetus particle integration.
- `physics-3d` feature — impetus 3D backend, segment/convex hull colliders.
- Collider-to-entity reverse HashMap (O(1) collision event lookup).

#### Scripting (`scripting` feature)
- `Script` component (WASM source, enabled, JSON state).
- `ScriptEngine` resource with inbox/outbox messaging and frame counters.
- kavach WASM execution — wasmtime + fuel metering + timeout.
- `run_scripts()` — auto-detects .wasm sources, kavach execution, fallback to message state.

#### Multiplayer (`multiplayer` feature)
- majra `NetState` resource with Relay, node identity, peer management.
- `NetRole` (Server/Client), `NetOwner` component, `Replicated` marker.
- `NetMessage` enum (StateSnapshot, StateDelta, InputReplication, PlayerJoin/Leave).
- `build_snapshot()`, `build_delta()` (adaptive linear/HashMap), `apply_snapshot()`, `apply_delta()`.
- `InputMessage` for input replication.

#### AI (`ai` feature)
- `DaimonClient` for agent registration, heartbeat, hoosh LLM inference.
- `DaimonConfig` with configurable endpoints.

#### Hot Reload
- `FileWatcher` — polling-based mtime change detection.
- `SceneReloader` — load + watch, auto-despawn/respawn on file change.
- `ShaderReloader` — watches .wgsl files, publishes `ShaderChanged` events.
- `apply_scene_diff()` — in-place scene updates (add/remove/update by name).

#### Profiler
- `FrameProfiler` — per-system timing, EMA averages, slow frame detection.
- `time_system()` closure-based timing, frame history, FPS calculation.

#### Asset Pipeline
- `AssetRegistry` — path → typed `AssetHandle` mapping.
- `AssetType` auto-inference from file extension (Texture/Sound/Scene/Shader/Model/Script).
- Hot reload integration via `FileWatcher`, dirty tracking.

#### Examples
- `examples/scene_loader.rs` — load TOML scene, walk hierarchy, print entity tree.
- `examples/game_loop.rs` — scheduler, input, profiling, player movement.

### Changed
- **Breaking:** Flattened from workspace (6 crates) to single crate with feature flags.
- **Breaking:** Component storage: HashMap → Vec arena (O(1) access by entity index).
- **Breaking:** `glam` replaced with `hisab` (hisab re-exports glam types).
- Resource storage consolidated: 3 HashMaps → 1 (ResourceEntry with integrated change tracking).
- Mouse buttons now have edge-triggered queries.
- `Entity::from_id(u64)` added for safe reconstruction.
- `World::has_component<T>()` added.
- Benchmark target requires `rendering` feature (`required-features`).

### Fixed
- `spatial_gain` overflow — negative distance produced gain > 1.0.
- `SoundTrigger.source` ignored by trigger processor.
- Entity generation bug in `run_scripts` (hardcoded generation 0).
- `run_scripts` never called `record_exec()`.
- `is_resource_changed` false for newly inserted resources at tick 0.
- Redundant `LightComponent` double-lookup in CLI.
- `set_parent` self-parenting now rejected with error.
- `watch_directory` skips bad entries instead of failing.
- `render_target.rs` — removed panicking `unwrap()` in GPU readback.
- `apply_trigger` — compare kind before cloning (avoid unnecessary String allocation).
- `apply_scene_diff` — uses `spawn_entity_def` directly (avoids temporary SceneDefinition allocation).
- Tokio runtime reused across `exec_wasm` calls (was creating per-call).

### Performance
- `get_component`: 33ns → 14ns (2.4x faster, Vec arena).
- `insert_component`: 50ns → 30ns (1.7x faster).
- `spawn_100_entities`: 11.3µs → 5.0µs (2.3x faster).
- `iterate_components` (1000 entities): 39µs → 3.5µs (11x faster).
- `entity_count`: O(n) → O(1) (0.24ns).
- `get_resource_mut`: 27ns → 14ns (consolidated ResourceEntry).

## [0.1.0] - 2026-03-22

### Added
- Initial scaffold with workspace architecture (6 crates).
- ECS world, generational entity allocator, typed component storage.
- TOML scene format, input handling, renderer trait.
- Impetus physics bridge, daimon AI client, CLI.
