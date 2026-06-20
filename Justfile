default:
    @just --list

# Run the game (TUI front-end).
run:
    cargo run -p barquest-tui

# Build the whole workspace.
build:
    cargo build --workspace

# Run all tests.
test:
    cargo test --workspace

# Format all crates.
fmt:
    cargo fmt --all

# Lint with clippy, treating warnings as errors.
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Verify formatting without modifying files.
fmt-check:
    cargo fmt --all --check

# CI-equivalent: formatting, lints, and tests.
check: fmt-check lint test

# Run a tool binary, e.g. `just tool balance-sim -- --help`.
tool name *args:
    cargo run -p barquest-tools --bin {{name}} -- {{args}}
