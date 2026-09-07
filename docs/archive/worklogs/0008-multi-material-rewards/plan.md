---
date: 2026-09-06
status: merged
discussions: [0009-multi-material-gathering-rewards]
decisions: [0016-independent-multi-material-rewards]
pr: null
---

# Multi-Material Rewards

## Objective

Implement discussion 0009 by replacing mutually exclusive reward outcomes with
ordered, independently rolled resource entries. A completed action may award
multiple resources, aggregate repeated resource entries, or award nothing.

## Plan

1. Replace the catalog reward outcome model and shipped reward tables, adding
   Tiny Magic Stone at the end of the Resource Catalog.
2. Update core completion processing to roll, aggregate, inventory-credit, and
   emit an ordered reward list.
3. Update the TUI event formatting and ensure tools continue consuming core
   completion events without implementing reward logic.
4. Add deterministic catalog, core, TUI, and tool-facing verification, then run
   the repository's full check suite.

## Progress

- Resolved discussion `0009-multi-material-gathering-rewards` and reviewed its
  discovery, minutes, proposal, and the current architecture, terminal layout,
  and TUI testing policy.
- Confirmed that the existing implementation uses a single `RewardOutcome`, an
  explicit `Nothing` entry, and a total-chance-equals-100 invariant across the
  catalog, game event, state completion, and TUI formatter.
- Replaced `RewardOutcome` with a resource-only `Reward` value and changed
  completion events to carry an ordered `Vec<Reward>`.
- Changed `RewardTable` to roll each entry independently, skip random-number
  consumption for 100% entries, aggregate duplicate resources with saturating
  addition, preserve the first successful entry order, and return an empty list
  when every uncertain entry fails.
- Updated runtime reward-table validation to require at least one entry, known
  Resource ids, positive amounts, and chances in `1..=100`, while deliberately
  allowing duplicate Resource ids, arbitrary chance totals, and tables without
  a guaranteed reward.
- Replaced all six shipped reward tables with the proposed values and appended
  Tiny Magic Stone to the Resource Catalog without disturbing existing order.
- Updated the TUI to format the core-provided reward list as one comma-separated
  completion line and to render an empty list as `Nothing`.
- Updated the architecture overview, terminal layout, and TUI test policy to
  describe the implemented multi-reward behavior. A decision document remains
  intentionally deferred to the explicit decision workflow.

## Verification

- `just check` passed: formatting, Clippy with warnings denied, 29 core tests,
  45 TUI tests, the tools test target, and core documentation tests.
- Deterministic tests cover simultaneous and partial successes, empty results,
  duplicate aggregation, first-success ordering, no random draw for guaranteed
  entries, inventory accumulation, completion events, target release, all six
  ten-second shipped actions, and single/multiple/aggregated/empty TUI logs.
- `just tool balance-sim` completed its fixed-seed core simulation at 10,000
  ticks (10 seconds).
- `git diff --check` passed.

## Outcome

Actions can now award zero, one, or multiple resource types from independent
percentage checks. Core owns rolling and aggregation, applies every aggregated
reward to inventory, and emits the same ordered list consumed by the TUI and
tools. The shipped content uses the requested probabilities and includes Tiny
Magic Stone.

The work is implemented and verified. Its status remains `in-progress` until
the explicit decision workflow records and accepts the resulting decision.

## Difficulties

The prior single-outcome assumption was embedded in three related shapes: the
catalog's enum (including explicit `Nothing`), the completion event's singular
field, and deterministic tests whose random stub returned one reusable value.
Changing only the draw algorithm would therefore have left downstream code
capable of representing only one reward. The event and tests were migrated at
the same time, and catalog tests now use a sequence random source so they can
assert both draw order and the exact number of consumed random values.

Removing the total-100 invariant also required preserving validation at the
entry level. The new assignment validation rejects empty tables, unknown
resources, zero amounts, and chances outside `1..=100`, but does not accidentally
reintroduce a total-chance or guaranteed-entry requirement.

These were localized consequences of replacing the old model rather than signs
of a new structural flaw.

## Differences from the Proposal

No material deviations. The implementation names each aggregated result
`Reward` and represents the proposal's zero-or-more results as `Vec<Reward>`;
an empty vector is the specified `Nothing` result rather than a catalog entry or
enum variant. Saturating addition is retained for both duplicate aggregation
and inventory accumulation, matching the existing inventory overflow behavior.
