# Design: Architecture Overview

- Status: Draft
- Date: 2026-06-20

A lightweight map of the project. For the *why* behind each choice, see the
[ADRs](../adr/).

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

Data-driven command loop working: `core` models tick progression (`Progress`,
`TICKS_PER_SECOND = 1000`) plus a data-driven domain — a `Catalog` template pool
(`TargetTemplate` / `ActionTemplate` keyed by string ids; `builtin()` seeds
Hero/Adventurer/Farmer + Forest Exploration) and a `GameState` of live
`TargetInstance`s, each holding multiple concurrent `quests`, with `spawn_target`
/ `unlock_action` / `assign_action` / `advance` (see ADR 0007). `advance` returns
`GameEvent`s and removes finished quests. `core` is split into `time` / `id` /
`catalog` / `state` modules, with the public API re-exported flat from the crate
root (see ADR 0009). The `tui` is split into `app` (live state + behaviour),
`input` (event translation), and `render` (the five-region projection) modules,
with `main` keeping only the loop and terminal lifecycle (see ADR 0010); it runs
one non-blocking, frame-paced loop and renders the fixed five-region full-screen
layout from
`docs/terminal-ui-layout.md` — Title / Progress / User Choices / Information Log /
Global Menu, ASCII separators, an 80x24 minimum-size guard (see ADR 0008). The
Progress region shows one `[===>---] NN%` bar per active action; the player
selects a target then an action by number in the three-column choices panel
(`1) Hero ----->`); completions surface in the log. `balance-sim` runs the same
model headless. Build/run/test/lint/fmt all pass in the dev container.

## Next steps

- Add JSON save/load for `GameState` (the catalog/state split and string ids
  were built for it).
- Award resources on quest completion (`advance` already emits `GameEvent`s; add
  reward variants and an inventory).
- Expand the `Catalog`: more targets/actions, per-target stats; wire the reserved
  Times / Sub Action columns once those features exist.
- Flesh out tools (balance sim, save inspector, content validator).

## Out of scope (for now)

Networking/multiplayer and a content authoring pipeline (loading the `Catalog`
from external data files) — the in-memory data-driven model and JSON save/load
land first; external authoring comes later.
