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

### V1.0 — Documentation / API Audit

- [x] Doc comments on all public types, functions, fields, and variants (403 items)
- [x] `RUSTDOCFLAGS="-D warnings -W missing_docs" cargo doc --all-features --no-deps` clean
- [x] Doc tests on core engine APIs — World, Transform, AnimState, InputState, Camera, AssetRegistry, StateMachine
- [x] Doc tests on all wiring modules — dynamics, biology, chemistry, astronomy, lore, voice, behavior
- [x] 27 doc tests passing
- [x] All integration modules wired (rendering, audio, voice, dynamics, behavior, biology, chemistry, astronomy, world)
- [x] Tracing instrumentation on all modules
- [x] Zero unwrap() in library code (only Mutex::lock in job pool)
- [x] `#[non_exhaustive]` on all public enums
- [x] Consistent API (naming, builders, error types)

### V1.0 — Hardening

- [x] 84.03% test coverage (2237/2662 lines) — above 80% target
- [x] 8 benchmark suites (engine, personality, dynamics, biology, science, voice)
- [x] 587 unit tests + 27 doc tests — 0 failures
- [x] `cargo audit` / `cargo deny` clean
- [x] All 16 feature gates compile independently — zero cross-gate leaks

### V1.0 — Security Review

- [x] Scripting sandbox: path validation on WASM loading, bounded message buffers, documented memory limit upstream gap
- [x] Asset loading: path traversal protection (canonicalization + cwd boundary), file size limits (256 MB), TOML input size limits (10 MB), symlink-aware validation
- [x] Network protocol: bounded inbox/outbox (4096), bounded dedup set (16384 + auto-trim), message field validation (node ID, entity lists, payload size), documented authentication requirements
- [x] Zero unsafe blocks in security-critical code

## Future Features (demand-gated)

- [ ] VR/XR support
- [ ] Deterministic replay / rollback netcode
- [ ] GPU compute pipeline (particle systems, cloth, hair)
- [ ] Procedural generation framework
- [ ] Editor protocol (for salai)
