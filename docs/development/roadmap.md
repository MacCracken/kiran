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

## Goonj Integration (acoustics engine)

- [x] **Real-time occlusion queries**: `AcousticsEngine` wraps `OcclusionEngine` for source-listener occlusion with BVH-accelerated wall checks and per-band attenuation
- [x] **Audio propagation**: Distance attenuation, atmospheric absorption, and Doppler shift via `AcousticsEngine` methods
- [x] **Room acoustics**: `RoomAcoustics` component + `generate_ir()` re-export for environment-specific reverb
- [x] **Diffraction**: `edge_diffraction_loss()` and `utd_wedge_diffraction()` re-exported for sound bending around obstacles
- [x] **Portal propagation**: `AcousticPortal` component with `energy_transfer()` for sound through doorways between rooms
- [x] **Coupled rooms**: `CoupledRooms` and `coupled_room_decay()` re-exported for multi-room reverb with double-slope decay
- [x] **Source directivity**: `AcousticSource` component with `DirectivityPattern` for directional sound sources
- [x] **Wall transmission**: `WallTransmission` component wrapping `WallConstruction` for sound through walls
- [x] **Ambisonics**: `encode_bformat()` and `encode_hoa()` re-exported for spatial audio encoding
- [x] **FDN reverb**: `ReverbProcessor` resource wrapping `Fdn` for efficient real-time late reverberation

## Future Features

- [ ] VR/XR support
