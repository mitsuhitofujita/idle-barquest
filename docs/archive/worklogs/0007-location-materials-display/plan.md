---
date: 2026-09-01
status: merged
discussions: [0008-location-materials-display]
decisions: [0015-settlement-and-materials-display]
pr: null
---

# Location and Materials Display

## Objective

Keep the current Settlement and acquired material quantities visible in the
full-screen TUI without increasing the `80x24` minimum terminal size, and make
the variable-width material list globally navigable without destabilizing its
viewport.

## Plan

1. Add stable Settlement content ids and templates to the core Catalog, and
   seed `awakening_shore` as the current Settlement in GameState.
2. Represent acquired materials from inventory stacks in Resource Catalog
   order while preserving zero-quantity stacks as the acquired-state marker.
3. Add TUI material viewport state keyed by its first ResourceId and global
   previous/next material inputs independent of the choice-selection stage.
4. Revise the fixed-height layout to add Settlement and Materials rows, reduce
   User Choices and Progress to five entries each, and expand the Global Menu.
5. Add core, app/input, and renderer coverage for Settlement resolution,
   acquisition visibility, variable-width rendering, truncation, viewport
   stability, navigation boundaries, and the `80x24` allocation.
6. Update current-state design documentation, run the repository checks, and
   record the outcome, difficulties, and deviations.

## Progress

- Resolved discussion `0008-location-materials-display` and reviewed its
  discovery, minutes, proposal, architecture overview, terminal layout, and TUI
  test policy.
- Confirmed the current core Catalog has stable ids and registration-order
  iteration for Targets, Locations, Actions, and Resources, while GameState has
  no Settlement field and currently appends inventory stacks on first award.
- Confirmed the existing `80x24` layout allocates seven User Choices rows and
  six Progress rows, leaving exactly the two rows that the proposal replaces
  with Settlement and Materials.
- Added `SettlementId` and `SettlementTemplate`, registered
  `awakening_shore` / `Awakening Shore`, and made seeded GameState hold the
  Catalog-backed current Settlement.
- Added a core acquired-resource projection that uses Resource Catalog order
  and treats ResourceStack presence, including a zero-quantity stack, as the
  acquired marker.
- Added a TUI material viewport keyed by its first ResourceId. It computes
  complete-item fit from the actual row width, truncates only an oversized
  first label, reserves stable arrow columns, and exposes adjacent one-item
  starts to both rendering and input behavior.
- Added global `,` and `.` inputs and passed the current terminal width into app
  updates so `.` can remain a no-op when all remaining Resources already fit.
  Backspace and quit behavior remain independent and unchanged.
- Revised the `80x24` renderer to add one-row Settlement and Materials regions,
  reduce User Choices to one heading plus five entries, reduce Progress to five
  entries, preserve all three separator rows, and expand the Global Menu.
- Added core, app, input, viewport, and TestBackend coverage for Settlement
  resolution, Catalog ordering, zero stacks, empty materials, variable widths
  and quantity digits, label-only truncation, stable ResourceId starts, global
  one-item navigation, arrow boundaries, five-entry limits, and exact row
  placement.
- Updated the architecture overview, terminal layout, and TUI test policy to
  describe the implemented Settlement state, material viewport, controls, and
  seven-region layout.
- Completed the full repository gate and a deterministic headless simulator
  smoke test.

## Outcome

Implemented the location and materials display proposal end to end. The seeded
world now resolves its `awakening_shore` current Settlement through core
Catalog content, and the TUI shows `Settlement: Awakening Shore` in its fixed
row. The Materials row remains blank before the first acquisition, then shows
known Resource stacks in Catalog order, including zero quantities.

The material viewport derives its item count from the actual terminal width and
keeps a ResourceId as its first visible item. `,` and `.` move that start by one
acquired Resource from every selection stage, unavailable movement is a no-op,
and reserved arrow cells prevent the body from shifting at boundaries. The
minimum `80x24` screen retains its three separators and four-row Information
Log while User Choices and Progress now display at most five entries each.

Verification completed successfully:

- `just check` passed formatting, Clippy with warnings denied, all 68 workspace
  tests, and core doc tests.
- `cargo run -p barquest-tools --bin balance-sim` completed deterministically at
  10,000 ticks (10 seconds) through the Settlement-aware seeded state.
- `git diff --check` found no whitespace errors.

The worklog remains `in-progress` until the explicit decision workflow is run,
as required by the repository documentation model.

## Difficulties and Deviations Identified During Implementation

- Rightward navigation cannot be decided from inventory state alone: whether
  any Resource is hidden depends on the current terminal width. `App::update`
  therefore accepts the material-row width, and input and rendering share one
  viewport calculation. This was an implementation detail not prescribed by
  the proposal, but it preserves the proposal's no-op behavior exactly across
  terminal sizes.
- Introducing a required current Settlement exposed one custom-Catalog test
  fixture that did not register Settlement content. The fixture now registers
  `camp`; seeded states intentionally require at least one Catalog Settlement so
  their current id is always resolvable.
- The generic `docs-sync` skill refers to legacy `docs/design-docs` and
  `docs/adr` paths. This repository uses `docs/design`, and its explicit
  decision workflow owns additions under `docs/decisions`, so current-state
  design documents were updated without creating a decision record.
- The first full Clippy gate rejected two new test assertions that used
  `.nth(0)` instead of `.next()`. This was limited to test expression style and
  was corrected before the successful final gate; it did not affect the
  viewport implementation or behavior.
