# ADR 0006: Concurrent multi-target progress loop

- Status: Superseded by [ADR 0008](0008-full-screen-tui-layout-and-events.md)
- Date: 2026-06-20

## Context

ADR 0005 modelled a single modal command: pick a target, pick an action, watch
one bordered full-screen gauge run, then return to the menu. This is an idle RPG
where several targets (hero, adventurer, farmer) should work in parallel, so a
one-quest-at-a-time loop blocked the core fantasy. We also wanted the screen to
read like an `apt`/`mise` update: a stack of inline bars, not a framed gauge.

## Decision

`core` now exposes three targets (`Hero`, `Adventurer`, `Farmer`) sharing the
existing action. The TUI keeps one `Slot { target, quest: Option<Quest> }` per
target and runs a single non-blocking, frame-paced loop: each frame it advances
every active quest by `TICKS_PER_FRAME`, polls input until the next frame
boundary, then redraws. Layout is progress bars at the top — one left-aligned
`target  action  [===>---] NN%` row each, an `apt`/`mise`-style text bar with no
frame — and the command menu at the bottom. The menu is a two-state machine
(`SelectTarget` → `SelectAction`) that assigns/restarts a target's quest. The
first-letter hotkeys and `q`/`Esc`/`Ctrl-C` quit from ADR 0005 are kept.

## Consequences

- Targets run in parallel; idle targets show `—` and an empty bar.
- Input is non-blocking (`event::poll`), so bars keep filling during menu nav.
- Rendering is a pure projection, verified with `ratatui::TestBackend` tests.
- Completion no longer holds a 100% frame then returns to a menu; finished bars
  rest at 100% until reassigned.
- Resource/reward on completion is still deferred (see ADR 0005).

## Related

- Supersedes [ADR 0005](0005-command-menu-input-model.md) — modal single-quest loop.
- [ADR 0004](0004-tick-time-model.md) — `Action::goal_ticks()` seeds each `Progress`.
- [ADR 0002](0002-ratatui-crossterm-for-tui.md) — ratatui/crossterm rendering.
