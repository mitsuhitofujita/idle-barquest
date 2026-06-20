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

Interactive command loop working: `core` models tick progression (`Progress`,
`TICKS_PER_SECOND = 1000`) plus the choosable domain (`Target::Hero`,
`Action::ForestExploration`). The `tui` runs `select target → select action →
run quest → back to menu`, picking each choice by its first-letter hotkey
(`H)ero`); the chosen action drives the gauge. `balance-sim` runs the same model
headless. Build/run/test/lint/fmt all pass in the dev container.

## Next steps

- Award resources on quest completion (the current loop ends with no reward yet).
- Grow the idle domain in `core`: more targets/actions, multiple concurrent
  quests, save/load, content data.
- Render multiple quests and player stats in the TUI.
- Flesh out tools (balance sim, save inspector, content validator).

## Out of scope (for now)

Networking/multiplayer, persistence format details, and content authoring
pipeline — revisit once the core tick loop and TUI exist.
