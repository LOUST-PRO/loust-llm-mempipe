# loust-llm-mempipe — local Makefile
# All targets are read-only / local; no push targets (stage-local-first rule).

.PHONY: all build release test clippy fmt fmt-check run info clean help

all: fmt-check clippy test build

help:
	@echo "Targets:"
	@echo "  build       - cargo build (debug)"
	@echo "  release     - cargo build --release"
	@echo "  test        - cargo test"
	@echo "  clippy      - cargo clippy --all-targets -- -D warnings"
	@echo "  fmt         - cargo fmt"
	@echo "  fmt-check   - cargo fmt --check"
	@echo "  run         - cargo run (with --info)"
	@echo "  info        - print build info from release binary"
	@echo "  clean       - cargo clean"

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

run:
	cargo run -- --info

info: release
	./target/release/loust-llm-mempipe --info

clean:
	cargo clean
