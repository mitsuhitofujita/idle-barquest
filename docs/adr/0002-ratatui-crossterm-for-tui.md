# ADR 0002: ratatui + crossterm for the TUI

- Status: Accepted
- Date: 2026-06-20

## Context

The game renders entirely in the terminal — quests shown as progress bars,
updated on a tick. We need a maintained, cross-platform rendering + input
library.

## Decision

Adopt `ratatui` (0.30) with the `crossterm` (0.29) backend. ratatui is the
de-facto standard Rust TUI library (maintained successor to tui-rs) and ships
gauge/progress widgets that fit an idle game. Both are declared in
`[workspace.dependencies]` but not yet wired in — the current Hello, World. is
intentionally dependency-free so the toolchain is validated first.

## Consequences

- Widely used and documented; cross-platform terminal handling via crossterm.
- Immediate-mode redraw model: the UI re-renders from `core` state each frame.
- Centralized version means the rendering loop can be added later without
  touching `core` or `tools`.
