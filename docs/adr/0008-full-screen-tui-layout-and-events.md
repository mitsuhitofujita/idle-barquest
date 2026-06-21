# ADR 0008: Full-screen TUI layout, numbered selection, per-action rows, events

- Status: Accepted
- Date: 2026-06-21

## Context

ADR 0006 rendered the TUI as a loose `apt`/`mise` stack of inline bars with a
first-letter hotkey menu and finished bars resting at 100%. `docs/terminal-ui-layout.md`
was then added as the source of truth for layout and asks for a stable, vi-style
**full-screen** screen instead: fixed regions, ASCII-only chrome, a minimum size,
and numbered selection. This is an idle RPG where a target — especially a
facility — should run several actions at once, so "one bar per target" no longer
fits either.

## Decision

The `tui` renders a fixed five-region stack from `terminal-ui-layout.md` — Title
/ Progress / User Choices / Information Log / Global Menu — separated by ASCII
`+--+--+` rules, full-width, ASCII-only, hidden behind a warning below `80x24`.
The Progress region shows **one row per active action** (six proportional
columns); `core`'s `TargetInstance` now holds `quests: Vec<Quest>` so a target
can run many at once (ADR 0006's concurrency is kept, re-expressed per action).
Selection is **numbered with arrow cues** (`1) Hero ----->`) in a persistent
three-column panel, replacing first-letter hotkeys. `GameState::advance` returns
`GameEvent`s and **removes a quest when it completes**; the front-end logs the
completion in the Information Log instead of leaving a 100% bar.

## Consequences

- The screen is stable and position-predictable; layout is deterministic and
  unit-tested with `TestBackend`. Times / Sub-Action columns are reserved blanks.
- Numbered selection drops the unique-leading-letter constraint from ADR 0005.
- Completion is observable (events) and the progress region self-cleans; this is
  the hook for future rewards/inventory.
- `core` stays terminal-free; the log buffer and formatting live in `tui`.

## Related

- Supersedes [ADR 0006](0006-concurrent-multi-target-progress-loop.md) (inline-bar
  layout, rest-at-100%) and revisits the input model of
  [ADR 0005](0005-command-menu-input-model.md) (first-letter hotkeys).
- Builds on [ADR 0007](0007-data-driven-content-model.md) (catalog/state split)
  and [ADR 0004](0004-tick-time-model.md) (tick-driven `advance`).
- Implements `docs/terminal-ui-layout.md`.
