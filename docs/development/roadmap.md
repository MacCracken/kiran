# Kiran Roadmap

> **Kiran** (Sanskrit: किरण — ray of light) — AI-native game engine for AGNOS

## Current State

**Version**: 0.26.3 → next release will be V1.0

- 46 optional deps across 16 feature gates
- 587 unit tests + 27 doc tests, 84% coverage, 8 benchmark suites
- Security review complete (scripting, assets, network)
- Full documentation (403 doc items, 4 ADRs, threat model, 3 guides, 8 examples)
- Fuzz testing + supply-chain verification in place

See [CHANGELOG.md](../../CHANGELOG.md) for detailed history.

## V1.0 Release Criteria

All items below must be complete before tagging 1.0.0:

- [ ] Version bump (0.26.3 → 1.0.0) across Cargo.toml, VERSION, CLAUDE.md
- [ ] Final `cargo audit` / `cargo deny` pass on release day
- [ ] Publish to crates.io
- [ ] Tag + GitHub release with CHANGELOG excerpt
- [ ] Notify consumers (joshua, salai) of stable API

## Future Features (demand-gated)

These ship post-1.0 only when a consumer (joshua, salai, or community) requests them:

- [ ] VR/XR support
- [ ] Deterministic replay / rollback netcode
- [ ] GPU compute pipeline (particle systems, cloth, hair)
- [ ] Procedural generation framework
- [ ] Editor protocol (for salai)
