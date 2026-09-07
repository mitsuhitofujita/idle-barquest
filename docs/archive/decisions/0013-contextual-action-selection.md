# ADR 0013: Contextual action selection

- Status: Accepted
- Date: 2026-08-29
- Discussions: [contextual action selection](../discussions/0005-contextual-action-selection/proposal.md)
- Worklog: [contextual action selection](../worklogs/0004-contextual-action-selection/plan.md)

## Decision

Present User Choices as progressive stages. Show only Target initially, then
create an Action column after a target is selected. List only actions that are
both unlocked and supported by the selected target kind, and enforce the same
constraints in core assignment.

Use case-insensitive positional ASCII letters (`a`, `b`, ...) for the active
column and ignore numeric input. During Action selection, hide Target keys and
mark the selected Target immediately before the column separator. Size the
Target column from its content while preserving at least 20 Action columns;
defer Times until it becomes a real selection stage.

## Rationale

Generating each stage from the current selection makes the Target–Action
relationship explicit and avoids presenting invalid combinations. Enforcing
compatibility in core keeps all front ends consistent, while positional letters
avoid label-based key collisions and deterministic sizing preserves the
interaction at the minimum terminal width.

## Related

- Refines the persistent numbered User Choices model from
  [ADR 0008](0008-full-screen-tui-layout-and-events.md).
- Extends the target and action templates established by
  [ADR 0007](0007-data-driven-content-model.md).
- Preserves the fixed User Choices region established by
  [ADR 0012](0012-terminal-layout-refinement.md).
