# Kiran Roadmap

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS


## Completed

### 0.26 — Ecosystem Integration

- [x] Full AGNOS crate stack updated to 1.x stable releases
- [x] 46 optional dependencies across 16 cohesive feature gates
- [x] Goonj acoustics integration (occlusion, propagation, room acoustics, diffraction, portals, coupled rooms, directivity, wall transmission, ambisonics, FDN reverb)
- [x] Electromagnetism integration (bijli)
- [x] Thermodynamics integration (ushma)
- [x] Quantum mechanics integration (kana)
- [x] License hardened to GPL-3.0-only (SPDX-correct)

### Feature gate layout (0.26)

**Engine core:** rendering, audio, voice, physics, physics-3d, fluids, dynamics, ai, behavior, scripting, multiplayer, navigation

**Science:** biology, chemistry, astronomy, world

## V1.0

### Documentation / API Audit

- [ ] Doc comments on all public types, functions, fields, and variants
- [ ] `RUSTDOCFLAGS="-D warnings -W missing_docs" cargo doc --all-features --no-deps` clean
- [ ] Doc tests on key APIs (World, Scheduler, AnimState, NavAgent, FluidSimulation)
- [ ] API review — consistent naming, builder patterns, error types

### Integration Modules

- [ ] Wire new science crates into ECS (biology, chemistry, astronomy, world modules)
- [ ] Wire dynamics crates into physics pipeline (bijli, dravya, ushma, pavan)
- [ ] Wire voice crates into audio pipeline (svara, shabda, prani)
- [ ] Wire rendering additions (prakash optics, ranga image processing)

### Hardening

- [ ] 80%+ test coverage
- [ ] Benchmark coverage for all hot paths
- [ ] `cargo audit` / `cargo deny` clean
- [ ] Security review (scripting sandbox, network protocol, asset loading)

## Future Features (demand-gated)

- [ ] VR/XR support
- [ ] Deterministic replay / rollback netcode
- [ ] GPU compute pipeline (particle systems, cloth, hair)
- [ ] Procedural generation framework
- [ ] Editor protocol (for salai)
