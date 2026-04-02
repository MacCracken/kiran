# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-04-01

### Added

#### Ecosystem Integration
- **46 optional dependencies** across 16 cohesive feature gates (up from 11 deps / 10 gates)
- **rendering** — prakash (PBR optics, spectral color, atmosphere) + ranga (pixel buffers, blend modes, filters, compositing)
- **audio** — naad (synthesis), shravan (codecs: WAV/FLAC/Ogg), garjan (environmental sounds), ghurni (mechanical sounds)
- **voice** — new feature gate: svara (formant synthesis), shabda (G2P), prani (creature vocals)
- **dynamics** — new feature gate: bijli (EM fields), dravya (material science), ushma (thermodynamics), pavan (aerodynamics)
- **behavior** — expanded from bhava-only to include bodh (psychology), mastishk (neuroscience), jantu (ethology)
- **biology** — new feature gate: sharira (physiology), jivanu (microbiology), rasayan (biochemistry), vanaspati (botany)
- **chemistry** — new feature gate: kimiya (reactions), khanij (geology), tanmatra (atomic physics), kana (quantum mechanics)
- **astronomy** — new feature gate: falak (orbital mechanics), jyotish (planetary positions), tara (stellar), brahmanda (cosmology), badal (weather)
- **world** — new feature gate: itihas (history), sankhya (calendars), varna (languages), pramana (statistics)

#### ECS Components
- `EnvironmentSound`, `MechanicalSound` — procedural audio components (garjan/ghurni)
- `VoiceSource`, `CreatureVoiceSource` — vocal synthesis components (svara/prani)
- `Cognition`, `NeuralState`, `CreatureBehavior` — behavior components (bodh/mastishk/jantu)
- `EmField`, `MaterialBody`, `ThermalBody`, `AeroSurface` — dynamics components
- `Physiology`, `Microbe`, `MetabolicProfile`, `PlantState` — biology components
- `ChemicalBody`, `GeologicalBody`, `RadioactiveSource` — chemistry components
- `CelestialBody`, `WeatherZone` — astronomy components
- `CultureProfile`, `StochasticSource` — world-building components
- `SpeechRequest`, `VocalizeRequest` — voice event types

#### Documentation
- Doc comments on all 403 public items (types, functions, fields, variants)
- 27 doc tests across core + all wiring modules (up from 2)
- 4 architectural decision records (ECS storage, TOML scenes, WASM sandbox, feature isolation)
- Threat model documenting security surface and mitigations
- Guides: usage patterns, testing strategy, performance tuning
- 6 runnable examples (physics, audio, scripting, multiplayer, behavior, dynamics)

#### Infrastructure
- Fuzz testing: 4 targets (scene loading, world operations, input, asset paths)
- Supply-chain verification via cargo-vet (`supply-chain/` + Makefile `vet` target)
- 4 new benchmark suites (dynamics, biology, science, voice) — 8 total
- README rewritten with badges, full architecture tree, 16-gate feature table
- Architecture overview updated for all 28 modules and 46 deps

### Changed
- **license** — GPL-3.0 → GPL-3.0-only (SPDX-correct) across Cargo.toml, README, CLAUDE.md, deny.toml
- **feature `personality`** → renamed to **`behavior`** (now includes bodh, mastishk, jantu)
- **feature `acoustics`** → folded into **`audio`** (goonj now part of audio gate)
- **deps** — all AGNOS crates updated to 1.x stable (hisab 1.4, impetus 1.3, soorat 1.0, etc.)
- **deps** — criterion 0.5 → 0.8 (migrated `criterion::black_box` → `std::hint::black_box`)
- **deps** — toml 0.8 → 1.1, reqwest 0.12 → 0.13
- **deny.toml** — removed deprecated `GPL-3.0` SPDX, removed stale RUSTSEC-2024-0436 advisory ignore, trimmed unused license allowances

### Fixed
- **audio** — eliminated `unwrap()` in `apply_trigger` (replaced with safe `if let` chain)

### Security
- **scripting** — path validation on WASM loading (canonicalization + cwd boundary check)
- **scripting** — bounded message buffers (MAX_SCRIPT_MESSAGES = 1024)
- **assets** — path traversal protection (reject `..` components, validate canonicalized paths)
- **assets** — file size limit before loading (MAX_ASSET_FILE_SIZE = 256 MB)
- **scene** — TOML input size limit (MAX_SCENE_TOML_SIZE = 10 MB)
- **network** — bounded inbox/outbox (MAX_INBOX_SIZE = MAX_OUTBOX_SIZE = 4096)
- **network** — bounded dedup set with auto-trim (MAX_DEDUP_ENTRIES = 16384)
- **network** — message field validation (node ID length, entity list size, payload size)
- **network** — documented authentication requirements for PlayerJoin/PlayerLeave

### Quality
- 84.03% test coverage (2237/2662 lines) — above 80% target
- 587 unit tests + 27 doc tests — 0 failures
- 8 benchmark suites across all feature domains
- All 16 feature gates compile independently — zero cross-gate leaks
- Clean: fmt, clippy (-D warnings), docs (-W missing_docs), deny, audit

## [0.26.3] - 2026-03-26

### Added
- **acoustics** — Goonj 1.0 integration via new `acoustics` feature flag
  - `AcousticsEngine` resource: BVH-accelerated occlusion queries, distance attenuation, Doppler shift, atmospheric absorption
  - `RoomAcoustics` component: room geometry/materials with cached RT60
  - `AcousticSource` component: directivity patterns (omni, cardioid, supercardioid, tabulated)
  - `AcousticPortal` component: frequency-dependent sound transmission through openings
  - `WallTransmission` component: mass-law transmission loss through walls
  - `ReverbProcessor` resource: FDN-based real-time late reverberation
  - Re-exports: impulse response generation, ambisonics encoding (B-Format/HOA), coupled room decay, diffraction, materials

### Changed
- **deps** — Switched raasta, pravash, soorat from local path deps to crates.io versions
- **deps** — Pinned kavach to 1.0.1 (fixes `ExternalizationGate` compile error without `process` feature)

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
