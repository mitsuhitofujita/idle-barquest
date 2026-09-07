# ADR 0005: Command menu and first-letter input model

- Status: Superseded by [ADR 0006](0006-concurrent-multi-target-progress-loop.md)
- Date: 2026-06-20

## Context

The TUI previously ran a single hardcoded quest on launch with no player choice.
The game's core loop is: pick a *target*, pick an *action*, run a progress bar,
then (later) gain resources. We need a place for the target/action domain and an
input model for choosing between them in the terminal.

## Decision

Targets and actions are modelled in `core` as small `Copy` enums (`Target::Hero`,
`Action::ForestExploration`), each exposing `label()`, a derived `hotkey()` (the
label's first char, ASCII-lowercased), an `ALL` list for menu order, and — for
actions — `goal_ticks()` to seed a `Progress`. The TUI owns presentation and
input only: it renders each choice as `H)ero` and selects by pressing the
first-letter key (no arrow navigation). A `MenuItem` trait lets one generic
`select()` drive every menu. The command loop is `select target → select action
→ run quest → back to the target menu`; `q`/`Esc`/`Ctrl-C` quits anywhere.

## Consequences

- The domain stays pure and unit-testable in `core`; the TUI holds no game data.
- Adding a target/action is one enum variant plus its `ALL` entry — menus and
  hotkeys follow automatically.
- First-letter hotkeys are simple but require distinct leading letters per menu;
  revisit if a menu ever needs two same-letter entries.
- Resource/reward on completion is intentionally deferred (next issue).

## Related

- [ADR 0001](0001-cargo-workspace-layout.md) — core is the single source of truth.
- [ADR 0004](0004-tick-time-model.md) — `Action::goal_ticks()` seeds a `Progress`.
