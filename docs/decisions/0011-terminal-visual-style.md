# ADR 0011: Terminal visual style

- Status: Accepted
- Date: 2026-08-29
- Discussions: [terminal visual style](../discussions/0003-terminal-visual-style/proposal.md)
- Worklog: [terminal visual style](../worklogs/0002-terminal-visual-style/plan.md)

## Decision

Render the title as a centered, fixed-width, three-line ASCII artwork and use
its bottom row as the boundary above the Progress region. Keep the remaining
full-width separators.

Render unreached progress-bar cells as spaces while retaining `=` for completed
cells, `>` for the current position, brackets for the bar boundary, and a
percentage for the exact value. Keep the minimum terminal size at `80x24`,
accepting one fewer Information Log row at that size.

## Rationale

The artwork gives the game a more distinctive identity, while blank unreached
cells reduce visual density. Fixed-width ASCII keeps the title stable across
terminal widths, and the brackets plus percentage preserve non-color progress
cues. Retaining `80x24` avoids narrowing the set of supported terminals.

## Related

- Refines the visual presentation within the five-region layout established by
  [ADR 0008](0008-full-screen-tui-layout-and-events.md).
