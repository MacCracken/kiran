# 004 — Strict Feature Gate Isolation

## Status: Accepted

## Context

Kiran depends on 46+ optional crates spanning rendering, audio, voice,
physics, fluids, dynamics, AI, behavior, scripting, multiplayer, navigation,
biology, chemistry, astronomy, and world simulation. Consumers (joshua,
salai) pull different subsets. A simulation layer does not need rendering; an
editor does not need multiplayer.

Without discipline, feature gates accumulate hidden cross-dependencies: enabling
`scripting` silently requires `audio`, or `physics` drags in `rendering`
internals. This bloats compile times and binary size for consumers who only
need a slice of the engine.

## Decision

Every feature gate compiles independently with zero cross-gate dependencies:

1. **One gate, one concern.** Each `[features]` entry maps to a single
   domain (e.g., `rendering`, `audio`, `scripting`). The `full` meta-feature
   enables everything.

2. **`cfg` at the use site.** Code that touches an optional crate is wrapped
   in `#[cfg(feature = "...")]` at the import and the function/impl block,
   not at the module level. This keeps the module structure visible even
   when the feature is off.

3. **No cross-gate deps.** `scripting` depends on `kavach` and `tokio`.
   `multiplayer` depends on `majra`. Neither depends on the other, and
   neither depends on `rendering` or `audio`. The `Cargo.toml` dependency
   list makes this explicit.

4. **Consumer opt-in.** `default = []` means a bare `kiran` dependency
   compiles only the ECS core, scene format, and asset pipeline. Consumers
   explicitly enable the features they need.

## Consequences

**Positive**

- Clean dependency trees: `cargo tree --features scripting` shows only
  kavach, tokio, and core deps. No surprise transitive pulls.
- Faster CI: feature-gated test matrices catch breakage per-gate without
  compiling the full engine every time.
- Smaller binaries: a headless simulation server compiles without GPU,
  audio, or rendering code.
- Consumers control their dependency footprint precisely.

**Negative**

- More `#[cfg(feature = "...")]` attributes throughout the codebase,
  increasing visual noise and the risk of dead code behind a gate.
- Cross-feature integration (e.g., physics-driven audio occlusion) requires
  a dedicated combined feature or glue code in the consumer.
- Testing the full matrix (all gate combinations) is combinatorially
  expensive; in practice only `default`, individual gates, and `full` are
  tested in CI.
