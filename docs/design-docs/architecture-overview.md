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
`TargetInstance`s with `spawn_target` / `unlock_action` / `assign_action` /
`advance` (see ADR 0007). The `tui` runs one non-blocking, frame-paced loop that
advances every target's quest in parallel and renders a stack of `apt`/`mise`-style
bars (`target  action  [===>---] NN%`) at the top, with the first-letter hotkey
menu (`H)ero`) at the bottom (see ADR 0006). `balance-sim` runs the same model
headless. Build/run/test/lint/fmt all pass in the dev container.

## Next steps

- Add JSON save/load for `GameState` (the catalog/state split and string ids
  were built for it).
- Award resources on quest completion (the loop still ends with no reward yet).
- Expand the `Catalog`: more targets/actions, per-target stats; surface richer
  per-target state in the TUI.
- Flesh out tools (balance sim, save inspector, content validator).

## Out of scope (for now)

Networking/multiplayer and a content authoring pipeline (loading the `Catalog`
from external data files) — the in-memory data-driven model and JSON save/load
land first; external authoring comes later.
