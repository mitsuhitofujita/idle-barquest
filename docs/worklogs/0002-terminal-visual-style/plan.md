---
date: 2026-08-29
status: in-progress
discussions: [0003-terminal-visual-style]
decisions: []
pr: null
---

# Terminal visual style

## Plan

- Replace the single-line title with the agreed fixed-width, centered
  three-line ASCII artwork.
- Remove only the separator immediately below the title while preserving the
  remaining region separators and the `80x24` minimum terminal size.
- Render unreached progress-bar cells as spaces instead of hyphens.
- Update focused renderer tests and the current design documents.
- Run the repository's CI-equivalent checks.

## Progress

- Confirmed that the current renderer uses a one-row title followed by four
  full-width separator rows.
- Confirmed that progress-bar generation and its endpoint/partial-state tests
  are isolated in `crates/tui/src/render/progress.rs`.
- Confirmed that the proposed layout consumes one additional row overall: the
  title grows by two rows and its following separator is removed.
- Implemented the three-row, 38-column title as a fixed renderer constant and
  changed the vertical layout from four separator slots to three.
- Changed unreached progress cells from hyphens to spaces while retaining the
  brackets, completed cells, leading edge, and percentage.
- Added renderer coverage for exact title rows at `80x24`, direct adjacency of
  title and progress, and preservation of the progress separator.
- Updated the terminal layout and architecture design documents to describe the
  implemented state.

## Outcome

- The TUI now renders the agreed three-line title artwork as a centered,
  non-stretching 38-column block.
- The title's lower artwork row now borders the progress region directly. The
  full-width separators between Progress, User Choices, Information Log, and
  Global Menu remain in place.
- Progress bars use spaces for unreached cells and retain `=`, `>`, brackets,
  and the right-aligned percentage at the same widths and values as before.
- The minimum supported size remains `80x24`; at that size the information log
  has one fewer row, as anticipated by the proposal.
- `just check` passes: formatting, Clippy with warnings denied, 51 workspace
  unit tests, and the core documentation tests all succeed.

## Difficulties and deviations

There were no implementation deviations from the proposal. The existing
vertical layout represented every boundary as a separate constraint, so
removing the title separator required changing both the constraint tuple and
its rendered separator list. Row-level `TestBackend` assertions now cover that
relationship explicitly, including the exact 21-column padding on each side of
the 38-column artwork at the minimum width.

The first full check rejected an unnecessary temporary binding in the new test
helper under Clippy's `let_and_return` lint. Returning the collected rows
directly resolved it; the complete check then passed. Each Cargo invocation also
printed a warning because `mise` could not write a cache file outside the
workspace sandbox, but this did not affect any build or verification result.
