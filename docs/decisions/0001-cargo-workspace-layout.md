# ADR 0001: Cargo workspace split into core / tui / tools

- Status: Accepted
- Date: 2026-06-20

## Context

idle-barquest is a TUI idle RPG. Beyond the game itself, we expect supporting
tools (balance simulator, save inspector, content validators) and dev
automation. A single binary crate would not scale to that.

## Decision

Use a Cargo workspace with three crates under `crates/`:

- `barquest-core` (lib) — pure game logic, no terminal I/O.
- `barquest-tui` (bin `barquest`) — the game front-end.
- `barquest-tools` (bins under `src/bin/`) — game-data & dev tools.

`core` is the shared dependency of `tui` and `tools`. Shared metadata and
dependency versions are centralized in `[workspace.package]` /
`[workspace.dependencies]`.

## Consequences

- Game logic is unit-testable without a terminal and reused by every tool.
- One place to manage versions; new tools are just another `src/bin/*.rs`.
- Slightly more manifest boilerplate; the game binary is named explicitly
  (`barquest`) so the package name and the executable can differ.

## Related

- [ADR 0002](./0002-ratatui-crossterm-for-tui.md)
- [ADR 0003](./0003-toolchain-pinning-and-just.md)
