# Design: Architecture Overview

- Status: Current
- Date: 2026-06-20
- Updated: 2026-08-30

A lightweight map of the project. For the *why* behind each choice, see the
[decisions](../decisions/).

## Goal

A terminal idle RPG where every quest is a progress bar: state advances on a
tick whether or not the player is watching, and the TUI renders that state.

## Components

| Crate            | Path           | Kind | Role                                   |
| ---------------- | -------------- | ---- | -------------------------------------- |
| `barquest-core`  | `crates/core`  | lib  | Game state + idle simulation; no I/O   |
| `barquest-tui`   | `crates/tui`   | bin  | Game front-end (`barquest`)            |
| `barquest-tools` | `crates/tools` | bins | Game-data & dev tools (`src/bin/`)     |

## Dependency direction

```
barquest-tui ──┐
               ├──> barquest-core   (core depends on nothing in-repo)
barquest-tools ┘
```

`core` is the single source of truth for game logic. The TUI renders it; tools
inspect/simulate it. Nothing depends back on `tui` or `tools`.

## Run it

```sh
just run                  # play the game
just tool balance-sim     # run a tool binary
just check                # fmt-check + clippy + test
```

## Status

The data-driven command loop models tick progression (`Progress`,
`TICKS_PER_SECOND = 1000`) and a `Catalog` of `TargetTemplate`,
`LocationTemplate`, and `ActionTemplate` content keyed by stable string ids.
`GameState` stores live Targets, unlocked Locations and Actions, and at most one
task per Target. A task and its completion event both carry Target, Location,
and Action ids. Assignment rejects unknown or locked content, incompatible
Target/Action or Location/Action pairs, and busy Targets. Completion removes the
task and makes the Target available again without automatic repetition.

The built-in starting world contains Hero; First Shore, Nearby Woods, and Nearby
Hill; and Gather, Fish, and Hunt with Location-specific compatibility. The
`core` remains split into `time`, `id`, `catalog`, and `state` modules with a flat
public API. The `tui` remains split into `app`, `input`, and `render`, while
`main` owns only pacing and terminal lifecycle. User Choices progressively adds
Location and Action columns after their preceding selections. Backspace returns
one stage, and busy Target rows retain their fixed letter slot while showing
`-` instead of a key. Progress uses Target / Location / Action / Progress Bar,
and completion logs identify all three task dimensions. `balance-sim` runs the
same model headless. Build, tests, lint, and formatting checks pass.

## Next steps

- Add JSON save/load for `GameState` (the catalog/state split and string ids
  were built for it).
- Award resources on quest completion (`advance` already emits `GameEvent`s; add
  reward variants and an inventory).
- Expand the `Catalog` with more Targets, Locations, Actions, and per-target
  stats; add later selection stages only when their gameplay exists.
- Flesh out tools (balance sim, save inspector, content validator).

## Out of scope (for now)

Networking/multiplayer and a content authoring pipeline (loading the `Catalog`
from external data files) — the in-memory data-driven model and JSON save/load
land first; external authoring comes later.

## Related decisions

- [Decision 0001: Cargo workspace layout](../decisions/0001-cargo-workspace-layout.md)
- [Decision 0004: Tick-based time model](../decisions/0004-tick-time-model.md)
- [Decision 0007: Data-driven content model](../decisions/0007-data-driven-content-model.md)
- [Decision 0008: Full-screen TUI layout and events](../decisions/0008-full-screen-tui-layout-and-events.md)
- [Decision 0009: Core module structure](../decisions/0009-core-module-structure.md)
- [Decision 0010: TUI module structure](../decisions/0010-tui-module-structure.md)
- [Decision 0011: Terminal visual style](../decisions/0011-terminal-visual-style.md)
- [Decision 0012: Terminal layout refinement](../decisions/0012-terminal-layout-refinement.md)
- [Decision 0013: Contextual action selection](../decisions/0013-contextual-action-selection.md)
