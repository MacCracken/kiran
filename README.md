# Kiran

**AI-native game engine** — part of the AGNOS ecosystem.

*Kiran* (Sanskrit/Hindi: ray of light) is a modular game engine built in Rust,
designed from the ground up for AI-driven game development.

## Crates

| Crate | Description |
|-------|-------------|
| `kiran-core` | ECS world, entity allocator, game clock, event bus |
| `kiran-scene` | TOML scene format, loading, entity spawning |
| `kiran-input` | Keyboard, mouse, and gamepad input handling |
| `kiran-render` | Rendering abstraction with headless (null) backend |
| `kiran-ai` | AGNOS integration — daimon agent + hoosh LLM inference |

## CLI

```sh
# Validate a scene file
kiran check examples/level.toml

# Load and run a scene
kiran run examples/level.toml
```

## Building

```sh
cargo build --workspace
cargo test --workspace
```

## License

GPL-3.0 — see [LICENSE](LICENSE).
