.PHONY: build test check clippy fmt clean doc

build:
	cargo build --workspace

test:
	cargo test --workspace

check:
	cargo check --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clean:
	cargo clean

doc:
	cargo doc --workspace --no-deps

all: fmt-check clippy test
