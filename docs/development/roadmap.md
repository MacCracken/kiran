# Kiran Roadmap

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS


## 0.24.3

- [ ] Electromagnetism crate integration (not yet scaffolded)
- [ ] Thermodynamics crate integration (not yet scaffolded)
- [ ] Quantum mechanics crate integration (not yet scaffolded)

## V1.0 — Documentation / API Audit

- [ ] Doc comments on all public types and functions
- [ ] `cargo doc --all-features` clean with `-D warnings`
- [ ] Doc tests on key APIs (World, Scheduler, AnimState, NavAgent, FluidSimulation)
- [ ] API review — consistent naming, builder patterns, error types

## Future Features

- [ ] VR/XR support

## Dependency Map

```
kiran (engine orchestration)
  ├── hisab        — math (vectors, geometry, transforms)
  ├── impetus      — physics (rigid bodies, collision, particles)
  ├── soorat       — rendering (wgpu, PBR, shadows, animation, terrain, text, UI)
  ├── dhvani       — audio (spatial audio, DSP, mixing)
  ├── majra        — multiplayer (pub/sub, relay)
  ├── kavach       — scripting sandbox (WASM)
  ├── bhava        — emotion/personality
  ├── raasta       — navigation/pathfinding
  ├── pravash      — fluid dynamics (SPH, shallow water)
  ├── hoosh        — AI inference gateway
  └── prakash      — optics (via soorat)
```

## Stats

- **Source:** ~11,000 lines across 21 modules
- **Tests:** 541 (all features), 72 benchmarks
- **Features:** `ai`, `audio`, `physics`, `physics-3d`, `rendering`, `scripting`, `multiplayer`, `personality`, `navigation`, `fluids`
- **Ecosystem:** 11 AGNOS crates integrated
- **Examples:** scene_loader, game_loop
