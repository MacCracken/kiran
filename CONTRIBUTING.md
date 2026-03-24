# Contributing to Kiran

Thank you for your interest in contributing to Kiran.

## Development Setup

```sh
git clone https://github.com/MacCracken/kiran
cd kiran
cargo build
cargo test
```

## Workflow

1. Fork and create a feature branch
2. Write tests for new functionality
3. Run `make check` (fmt + clippy + test + audit)
4. Submit a pull request

## Code Style

- `cargo fmt` — all code must be formatted
- `cargo clippy --all-features --all-targets -- -D warnings` — zero warnings
- Tests: inline `#[cfg(test)] mod tests` in each module
- Integration tests in `tests/integration.rs`

## Testing

- `cargo test` — run core tests
- `cargo test --features multiplayer` — include multiplayer tests
- `cargo test --lib --tests` — skip bench compilation
- `cargo bench --features rendering` — run benchmarks

## Architecture

See [docs/architecture/overview.md](docs/architecture/overview.md).
