# ADR 0009: Split `barquest-core` into modules

- Status: Accepted
- Date: 2026-06-21

## Context

`crates/core/src/lib.rs` had grown to ~760 lines — the tick model, string ids,
the content catalog, and the live game state all in one module with every test
appended at the bottom. ADR 0007's `lib.rs` header already flagged the exit
criterion: *"split into submodules once it outgrows a single file."* It had.

## Decision

Split `core` into four focused modules along the existing content/state seam:

- `time` — `TICKS_PER_SECOND`, `seconds_to_ticks`, `Progress`.
- `id` — `TargetId`, `ActionId` (string-id newtypes).
- `catalog` — `TargetTemplate`, `ActionTemplate`, `Catalog`, and the private
  `first_hotkey` helper (content).
- `state` — `Quest`, `GameEvent`, `TargetInstance`, `GameState` (live world).

`lib.rs` keeps the crate docs, declares the modules, and **re-exports the full
public API flat** (`pub use …`) from the crate root, so consumers still write
`use barquest_core::Catalog;`. Each module owns its own `#[cfg(test)] mod tests`.

## Consequences

- No public API change: `barquest-tui` and `barquest-tools` compile untouched
  (verified by `just check`).
- Module dependencies stay acyclic: `catalog` → `id` + `time`; `state` → `id` +
  `time` + `catalog`.
- Tests now live next to the code they cover; coverage grew while moving
  (incrementing instance ids, concurrent multi-quest completion, empty-label
  hotkey): core unit tests 18 → 22.
- A `serde` derive on `GameState` (future save/load) remains a localized,
  additive step in `state`.

## Related

- Enacts the "split once it outgrows a single file" intent of
  [ADR 0007](0007-data-driven-content-model.md); module seam mirrors its
  content/state split.
- `time` houses the tick model from [ADR 0004](0004-tick-time-model.md).
