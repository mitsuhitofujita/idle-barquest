# ADR 0003: Pinned toolchain (Rust 1.96 / edition 2024) and `just` tasks

- Status: Accepted
- Date: 2026-06-20

## Context

Builds must be reproducible across the dev container and contributors, and
common commands (build/run/test/lint/fmt) should be one keystroke. The dev
container already ships Rust 1.96, clippy, rustfmt, and `just`.

## Decision

- `rust-toolchain.toml` pins `1.96.0` with `clippy` and `rustfmt`.
- Edition `2024` with workspace `resolver = "3"`.
- `Justfile` holds the task recipes (`run`, `build`, `test`, `fmt`, `lint`,
  `check`, `tool`); `just` is the entry point for dev automation.
- `Cargo.lock` is committed (this is an application, not a library).

## Consequences

- Everyone compiles with the same toolchain; `just check` mirrors CI locally.
- Upgrading Rust is a deliberate edit to `rust-toolchain.toml`.
- Game-facing tools run via `just tool <name>`; build/release automation grows
  in the same `Justfile` (an `xtask` crate can be added later if needed).
