# ADR 0015: Settlement and materials display

- Status: Accepted
- Date: 2026-09-01
- Discussions: [location and materials display](../discussions/0008-location-materials-display/proposal.md)
- Worklog: [location and materials display](../worklogs/0007-location-materials-display/plan.md)

## Decision

Model Settlement as Catalog content with a stable id and label, distinct from
the Location where a task runs, and store the current Settlement id in
`GameState`. Seed the starting world with `awakening_shore` (`Awakening Shore`).

Treat inventory stack presence as the acquired marker for a Resource, including
stacks with zero quantity, and project acquired Resources in Catalog order.
Keep Settlement and acquired Materials visible in dedicated rows at the
`80x24` minimum by limiting User Choices and Progress to five entries each.

Render Materials through a width-aware viewport whose first item is stored as a
ResourceId. Move the start one acquired Resource at a time with global `,` and
`.` inputs, reserve stable arrow columns, and truncate only the label when the
first item cannot otherwise fit with its quantity.

## Rationale

Keeping Settlement separate from task Location provides a stable base for
future crafting and technology features. Reusing inventory stack presence
avoids duplicate acquired-state tracking, while Catalog ordering and an
id-based viewport keep the display predictable as quantities and available
width change. The revised fixed-height allocation exposes important world state
without raising the minimum terminal size.

## Related

- Extends [ADR 0007](0007-data-driven-content-model.md) with Settlement content
  and Catalog-ordered acquired-Resource projection.
- Refines [ADR 0012](0012-terminal-layout-refinement.md) with Settlement and
  Materials rows while preserving its `80x24` minimum and fixed lower layout.
- Complements [ADR 0014](0014-three-stage-task-assignment.md) by keeping the
  development Settlement distinct from a task's Location.
- Extends [ADR 0010](0010-tui-module-structure.md) with width-aware global
  material navigation shared by app behavior and rendering.
