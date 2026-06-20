# ADR 0007: Data-driven targets and actions

- Status: Accepted
- Date: 2026-06-20

## Context

ADR 0005 modelled the domain as `Copy` enums (`Target::Hero`,
`Action::ForestExploration`) with hard-coded `label()` / `hotkey()` /
`goal_ticks()` / `ALL`, and the TUI built its `Slot`s straight from
`Target::ALL`. To grow content as the game progresses (new targets/actions,
multiple of a kind) and to prepare for JSON save/load, the choosable domain
needs to be *data*, not variants — and live state must be separable from content.

## Decision

Split content from state. A `Catalog` owns the template pool — `TargetTemplate`
/ `ActionTemplate` keyed by string ids (`TargetId("hero")`, `ActionId(...)`),
iterated in registration (= menu) order; `Catalog::builtin()` seeds today's
content. A separate `GameState` holds the live world: a growable
`Vec<TargetInstance>` (each with its own id, a `template_id`, and an optional
`Quest`) plus `unlocked_actions`, with `spawn_target` / `unlock_action` /
`assign_action` / `advance`. Instances reference templates *by id*, so state owns
no borrows, stays cheaply `Clone`, and is trivially serialisable later. The TUI
and tools hold a `Catalog` + `GameState` and resolve labels/hotkeys/durations
through the catalog. JSON persistence is deliberately not implemented yet.

## Consequences

- Adding content is data (`register_*` / `spawn_target` / `unlock_action`), not a
  new enum variant plus match arms.
- The concurrent loop, inline bars, and two-state menu (ADR 0006) are unchanged;
  only the domain representation underneath them changed.
- Hotkey uniqueness drops from a compile-time guarantee to a property of seeded
  content (unit-tested on `builtin()`); runtime-content collisions are future work.
- `GameState` is the future save payload; `serde` derives are a later, additive step.

## Related

- Evolves the `Copy`-enum domain from [ADR 0005](0005-command-menu-input-model.md).
- Builds on [ADR 0006](0006-concurrent-multi-target-progress-loop.md) — same loop, data-driven domain.
- [ADR 0004](0004-tick-time-model.md) — `ActionTemplate.goal_ticks` seeds each `Progress`.
