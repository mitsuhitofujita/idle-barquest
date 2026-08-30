# ADR 0014: Three-stage task assignment

- Status: Accepted
- Date: 2026-08-30
- Discussions: [three-stage user choices](../discussions/0006-three-stage-user-choices/proposal.md)
- Worklog: [three-stage user choices](../worklogs/0005-three-stage-user-choices/plan.md)

## Decision

Represent each running task as a Target, Location, and Action. Locations are
catalog content with stable ids, labels, and supported Actions; live state
tracks which Locations and Actions are unlocked. Core assignment validates all
ids, unlocks, Target compatibility, Location compatibility, and Target
availability. Each Target may hold at most one task.

Present assignment as progressive Target, Location, and Action columns. Keep
busy Targets in their fixed slots with `-` instead of a key, use Backspace to
return one stage, and keep `Esc` as quit. Include Location in progress rows and
completion events, and release the Target after completion without repetition.

## Rationale

Separating actor, place, and work supports geographic areas and facilities
without treating facilities as autonomous actors. Enforcing the same
three-dimensional constraints in core keeps every front end consistent, while
progressive columns expose only relevant choices and remain usable at `80x24`.

## Related

- Refines [ADR 0013](0013-contextual-action-selection.md) by inserting Location
  into the progressive selection and compatibility model.
- Extends [ADR 0007](0007-data-driven-content-model.md) with Location content
  and unlocked Location state.
- Revises [ADR 0008](0008-full-screen-tui-layout-and-events.md)'s per-action
  concurrency while preserving its event-driven completion and
  [ADR 0012](0012-terminal-layout-refinement.md)'s fixed regions.
