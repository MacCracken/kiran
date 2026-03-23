# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-03-22

### Added
- `kiran-core`: ECS world with generational entity allocator, typed component
  storage, singleton resources, game clock with fixed timestep, typed event bus.
- `kiran-scene`: TOML scene format with entity definitions, position, light, and
  tag components; load and spawn helpers.
- `kiran-input`: Keyboard (full key code set), mouse, and scroll input state
  tracking with edge-triggered (just-pressed/released) queries.
- `kiran-render`: Renderer trait, camera with glam view/projection matrices,
  sprite and mesh descriptors, NullRenderer for headless testing.
- `kiran-ai`: AGNOS daimon client for agent registration/heartbeat, hoosh LLM
  inference routing.
- CLI: `kiran run <scene>` and `kiran check <scene>` commands.
