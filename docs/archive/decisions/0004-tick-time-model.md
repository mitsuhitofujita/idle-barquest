# ADR 0004: Tick-based time model

- Status: Accepted
- Date: 2026-06-20

## Context

Quests advance over time whether or not the player watches. We need a unit of
game time that `core` can simulate without touching the clock, plus a render
cadence for the TUI — and the two are easily conflated under the name "tick".

## Decision

`tick` is the atomic unit of *game time*: `TICKS_PER_SECOND = 1000` (1 tick ≈ 1
ms). `core::Progress` stores elapsed/goal in ticks and exposes `advance(ticks)`
— pure, no clock. The render/update `frame` is a *separate* concept: the TUI
loops every 100 ms and advances a fixed `TICKS_PER_FRAME = 100` per frame, but
pins each frame boundary to the wall clock so N frames ≈ N×100 ms of real time.
Naming game-time (`tick`) and loop cadence (`frame`) distinctly is deliberate.

## Consequences

- `core` stays pure and unit-testable; the same model drives the TUI and headless
  tools (`balance-sim`) identically.
- Front-ends own wall-clock pacing — changing the frame rate or sim speed never
  touches `core`.
- Tick state isn't persisted yet; save/load is future work.

## Related

- [ADR 0002](0002-ratatui-crossterm-for-tui.md) — the gauge that renders this state.
