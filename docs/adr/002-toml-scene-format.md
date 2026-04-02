# 002 — TOML for Scene Serialization

## Status: Accepted

## Context

Kiran needs a human-readable scene file format that developers edit by hand
during development. Scenes define entity hierarchies, components (position,
light, physics, sound, material), and prefab templates. The format must
support hot-reload: save a file, see the change in-engine within one frame.

Candidates evaluated:

| Format | Pros | Cons |
|--------|------|------|
| JSON   | Universal tooling | No comments, verbose braces |
| YAML   | Compact, comments | Whitespace-sensitive, security footguns |
| TOML   | Comments, clear tables, good errors | No inline comments in serialized output |
| RON    | Rust-native, enums | Niche tooling, unfamiliar to non-Rust users |

## Decision

Use TOML as the scene format, parsed with the `toml` crate and mapped to
Rust types via serde derive (`Serialize` + `Deserialize`).

The scene schema is defined in `scene.rs`:

- `SceneDefinition` — top-level: name, description, prefabs, entities.
- `EntityDef` — per-entity: name, position, tags, material, sound, physics,
  children, prefab reference.
- Nested types (`PhysicsDef`, `ColliderDef`, `Material`, `SoundDef`) keep
  the TOML tables readable.

Hot-reload integration: the `FileWatcher` in `reload.rs` detects `.toml`
changes and triggers a scene re-parse and entity re-spawn.

## Consequences

**Positive**

- Developers can add comments directly in scene files (critical for team
  workflows and version control diffs).
- Serde derive keeps the Rust types and file format in lock-step; adding a
  field to the struct automatically extends the format.
- TOML parse errors include line/column, making typos easy to locate.
- Hot-reload round-trip is fast: parse is sub-millisecond for typical scenes.

**Negative**

- Re-serialized output (`toml::to_string`) strips comments from the
  original file. Round-tripping through code loses developer annotations.
- Deeply nested entity hierarchies (`children` of `children`) become verbose
  in TOML's table syntax.
- Binary assets (meshes, textures) are referenced by path string, not
  embedded; the asset pipeline must resolve and validate those paths
  separately (see `asset.rs` path traversal validation).
