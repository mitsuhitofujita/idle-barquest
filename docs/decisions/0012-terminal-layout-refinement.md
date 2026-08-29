# ADR 0012: Terminal layout refinement

- Status: Accepted
- Date: 2026-08-29
- Discussions: [terminal layout refinement](../discussions/0004-terminal-layout-refinement/proposal.md)
- Worklog: [terminal layout refinement](../worklogs/0003-terminal-layout-refinement/plan.md)

## Decision

Render the five terminal regions as Title / Information Log / User Choices /
Progress / Global Menu. At the `80x24` minimum, allocate 3 rows to Title, 4 to
Information Log, 7 to User Choices, 6 to Progress, and 1 to Global Menu, with a
one-row separator between each region below Information Log.

Reserve the first Information Log row as a title gap and bottom-align events in
the remaining rows. Keep User Choices and Progress fixed-height; assign all
additional terminal height to Information Log.

## Rationale

Putting events before available choices and running actions creates a natural
top-to-bottom reading order while fixed lower regions keep controls and progress
position-predictable. Including the title gap inside the minimum log allocation
preserves `80x24` support, and directing extra height to the log exposes more
history without destabilizing the interactive regions.

## Related

- Refines the region ordering and sizing established by
  [ADR 0008](0008-full-screen-tui-layout-and-events.md).
- Preserves the title artwork and progress-bar style from
  [ADR 0011](0011-terminal-visual-style.md) while replacing its direct
  Title-to-Progress boundary.
