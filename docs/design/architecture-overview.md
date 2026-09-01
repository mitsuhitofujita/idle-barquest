# Design: Architecture Overview

- Status: Current
- Date: 2026-06-20
- Updated: 2026-09-01

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
`SettlementTemplate`, `LocationTemplate`, `ActionTemplate`, and
`ResourceTemplate` content keyed by stable string ids. The Catalog also owns
exclusive reward tables keyed by Location and Action. Each table totals 100%,
may include an explicit `Nothing` outcome, and is selected by a
caller-controlled random source so tests and headless tools can reproduce exact
reward sequences.

`GameState` stores live Targets, the current Settlement id, unlocked Locations
and Actions, an in-memory resource inventory, and at most one task per Target.
The shipped current Settlement is `awakening_shore` (`Awakening Shore`) and is a
development base distinct from the Location where a task executes. A task
carries Target, Location, and Action ids; its completion event adds the selected
reward outcome. Assignment rejects unknown or locked content, incompatible
Target/Action or Location/Action pairs, busy Targets, and missing or invalid
reward content. Completion accumulates awarded resources, removes the task, and
makes the Target available again without automatic repetition. Inventory stack
presence records that a Resource was acquired even when its quantity is zero,
and acquired Resources can be projected in Catalog order.

The built-in starting world contains Hero; First Shore, Nearby Woods, and Nearby
Hill; and Gather, Fish, and Hunt with Location-specific compatibility. Their
six supported Location and Action combinations take ten seconds and award from
the shipped Pebble, Twig, Grass, Vine, Small Fish, Seaweed Fragment, Small Fang,
and Awful Meat resource set. The `core` is split into `time`, `id`, `catalog`,
`random`, and `state` modules with a flat public API. The `tui` remains split
into `app`, `input`, `materials`, and `render`, while `main` owns pacing,
terminal lifecycle, and production random seeding. User Choices progressively adds Location and
Action columns after their preceding selections. Backspace returns one stage,
and busy Target rows retain their fixed letter slot while showing `-` instead
of a key. The lower screen also shows the current Settlement and a width-aware
viewport of acquired materials. The viewport keeps its first ResourceId stable,
moves one Catalog item with `,` and `.`, and indicates hidden content with
reserved-width `<` and `>` slots. Progress uses Target / Location / Action /
Progress Bar, and one-line completion logs identify all three task dimensions
plus the resource result or explicit `Nothing`. `balance-sim` runs the same
model headless with a fixed seed. Build, tests, lint, and formatting checks pass.

## Next steps

- Add JSON save/load for `GameState`, including its inventory (the catalog/state
  split and string ids were built for it).
- Add resource consumers such as crafting, technology, and food processing,
  preserving zero-quantity stacks as acquired-state markers.
- Add Settlement discovery, unlocking, switching, and Settlement-specific
  crafting or technology choices.
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
- [Decision 0014: Three-stage task assignment](../decisions/0014-three-stage-task-assignment.md)
