# Architecture

idle-barquest is a Rust 1.96 Cargo workspace with three crates:

- `barquest-core` contains terminal-independent content, state, time, reward,
  and simulation logic.
- `barquest-tui` is the `barquest` executable. It owns terminal lifecycle,
  wall-clock pacing, input behavior, and rendering.
- `barquest-tools` contains headless binaries that consume the same core API.

Both front-end crates depend on core; core has no dependency on either of them.
Workspace package metadata and dependency versions are centralized in the root
manifest. Core's public types are implemented in focused modules and re-exported
from the crate root.

## Runtime flow

Core models time as ticks, with 1,000 ticks per second. Callers advance a
`GameState` explicitly and supply a `RandomSource`, so simulation contains no
clock or terminal access. An advance updates every active task, applies rewards
for completed tasks, frees their targets, and returns `GameEvent` values.

The TUI redraws and advances the world by 100 ticks on a 100 ms wall-clock frame.
It seeds core's deterministic random generator from system time. Input may cause
an immediate redraw between simulation frames.

The `balance-sim` tool constructs the built-in catalog and seeded state, assigns
a real task, and advances it in 100-tick steps with a fixed random seed until it
observes the completion event.

## Development checks

The root `Justfile` exposes workspace build, run, test, format, and lint commands.
`just check` verifies formatting, runs Clippy with warnings denied, and executes
all workspace tests. Core tests exercise simulation without a front end; TUI
behavior and buffer rendering are tested without an interactive terminal.
