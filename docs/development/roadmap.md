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

- [ ] **Real-time occlusion queries**: Use `goonj::integration::kiran::OcclusionEngine` for source-listener occlusion with BVH-accelerated wall checks and per-band attenuation
- [ ] **Audio propagation**: Use `goonj::propagation` for distance attenuation, atmospheric absorption, and Doppler shift in game audio
- [ ] **Room acoustics**: Use `goonj::impulse::generate_ir()` for environment-specific reverb (cave, corridor, open field)
- [ ] **Diffraction**: Use `goonj::diffraction::edge_diffraction_loss()` and `utd_wedge_diffraction()` for sound bending around obstacles
- [ ] **Portal propagation**: Use `goonj::portal::portal_energy_transfer()` for sound through doorways between rooms
- [ ] **Coupled rooms**: Use `goonj::coupled::coupled_room_decay()` for multi-room reverb with double-slope decay
- [ ] **Source directivity**: Use `goonj::directivity::DirectivityPattern` for directional sound sources (speakers, NPCs)
- [ ] **Wall transmission**: Use `goonj::material::WallConstruction::transmission_coefficient()` for sound through walls
- [ ] **Ambisonics**: Use `goonj::ambisonics::encode_bformat()` for spatial audio encoding in VR/XR scenes
- [ ] **FDN reverb**: Use `goonj::fdn::Fdn` for efficient real-time late reverberation

## Future Features

- [ ] VR/XR support
