---
date: 2026-08-29
status: merged
discussions: [0004-terminal-layout-refinement]
decisions: [0012-terminal-layout-refinement]
pr: null
---

# Terminal layout refinement

## Plan

- Reorder the five TUI regions to Title / Information Log / User Choices /
  Progress / Global Menu.
- Keep the `80x24` minimum while assigning fixed heights of 3 rows to Title,
  7 rows to User Choices, 6 rows to Progress, and 1 row to Global Menu.
- Give Information Log all remaining height, with a 4-row minimum at `80x24`;
  reserve its first row as the title gap and bottom-align visible log entries in
  the remaining rows.
- Preserve one full-width separator between each region below Information Log.
- Update renderer tests and the current design documents, then run the
  repository's CI-equivalent checks.

## Progress

- Confirmed that the current renderer orders regions as Title / Progress / User
  Choices / Information Log / Global Menu.
- Confirmed that Progress currently tracks the active-action count with a
  one-row minimum, while User Choices has a fixed 10-row height.
- Confirmed that the existing `80x24` guard and the three separators can remain
  unchanged; only region order, fixed heights, and log alignment need to change.
- Reordered the renderer constraints to place Information Log immediately below
  Title, followed by the fixed 7-row User Choices and fixed 6-row Progress
  regions.
- Changed Information Log rendering to reserve its first row, clip old entries
  against the remaining capacity, and pad above short logs so the newest event
  stays on the bottom row.
- Added row-level renderer coverage for all three separator positions at
  `80x24`, the permanent title gap, log clipping and bottom alignment, and the
  rule that only Information Log grows on a taller terminal.
- Updated the terminal layout and architecture design documents to describe the
  implemented region order, allocation, and limits.
- Targeted verification passes: formatting is clean, all 31 `barquest-tui`
  tests pass, and the 11 focused top-level renderer tests pass.

## Outcome

- The five-region screen now renders as Title / Information Log / User Choices
  / Progress / Global Menu.
- At `80x24`, Information Log receives 4 rows, User Choices 7 rows, and Progress
  6 rows. The three separators and one-row Global Menu complete the exact
  24-row allocation.
- Information Log always leaves its first row blank, bottom-aligns shorter logs,
  and retains the newest entries when content exceeds the available rows.
- Additional terminal height expands only Information Log; the title gap and
  all lower region heights remain fixed.
- The `80x24` minimum-size behavior and all existing visual constraints remain
  intact.
- `just check` passes: formatting, Clippy with warnings denied, 53 workspace
  unit tests, and the core documentation tests all succeed.

## Difficulties and deviations

There were no material deviations from the proposal. The main implementation
detail was that the previous log helper returned only event rows and relied on
the paragraph's default top alignment. Supporting both a permanent first blank
row and bottom alignment required it to return a full-height row set: it first
selects only the newest events that fit below the gap, then prepends enough
blank rows to fill the region. Calculating padding from the selected slice keeps
the first row blank even when the log is full and also handles an empty log.

The former Progress height was derived from the number of active actions, so
moving it alone would not have implemented the agreed stable layout. It was
replaced with an explicit 6-row constraint, while the existing render widget
continues to clip content at the region boundary. User Choices similarly uses
its 7-row region to limit the heading plus visible entries without introducing
the deferred paging design.

The documentation synchronization skill refers to legacy `docs/design-docs`
and `docs/adr` paths, but this repository's current documentation model uses
`docs/design` and reserves `docs/decisions` creation for the explicit decision
workflow. The current design documents were therefore updated in place and no
decision document was created during this work step.

Every Cargo command emitted a `mise` warning because its cache directory is
read-only in the workspace sandbox. The warning did not affect formatting,
compilation, linting, or tests. No structural design difficulty emerged that
warrants a new discovery.
