# ADR 0016: Independent multi-material rewards

- Status: Accepted
- Date: 2026-09-06
- Discussions: [multi-material gathering rewards](../discussions/0009-multi-material-gathering-rewards/proposal.md)
- Worklog: [multi-material rewards](../worklogs/0008-multi-material-rewards/plan.md)

## Decision

Model each Location and Action reward table as an ordered, non-empty list of
independent Resource checks. Each entry has a positive amount and a chance from
1% through 100%; table chances need not total 100%, guaranteed entries are
optional, and duplicate Resource ids are allowed.

Core rolls uncertain entries in definition order, treats 100% entries as
successful without consuming randomness, and aggregates successful duplicate
Resources in first-success order. It applies that ordered reward list to
inventory and includes it in the completion event. An empty list means
`Nothing`; the TUI formats all results on the existing one-line completion log.

Keep all six shipped actions at ten seconds, use the accepted reward chances,
and append Tiny Magic Stone to the Resource Catalog.

## Rationale

Independent checks express guaranteed base materials and optional additional
materials in one completion while also supporting future actions where every
check may fail. Centralizing rolls and aggregation in core keeps the TUI and
tools deterministic consumers of one authoritative result.

## Related

- Extends [ADR 0007](0007-data-driven-content-model.md) with ordered reward data.
- Uses the completion-event and one-line log boundaries from
  [ADR 0008](0008-full-screen-tui-layout-and-events.md).
- Preserves the inventory and Catalog-order display semantics from
  [ADR 0015](0015-settlement-and-materials-display.md).
