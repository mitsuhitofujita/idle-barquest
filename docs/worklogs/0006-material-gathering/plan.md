---
date: 2026-08-30
status: in-progress
discussions: [0007-material-gathering]
decisions: []
pr: null
---

# Material Gathering

## Objective

Connect the shipped Location and Action combinations to exclusive material
reward tables, accumulate awarded resources in the live game state, and report
each completion and outcome as one Information Log entry.

## Plan

1. Extend the content catalog with resource templates and validated reward
   tables for the shipped gathering, fishing, and hunting combinations.
2. Add an in-memory resource inventory and deterministic, caller-controlled
   reward selection to task completion in core.
3. Extend completion events and the TUI log projection with the selected
   resource or explicit `Nothing` outcome.
4. Update the headless simulator and add boundary, inventory, event, duration,
   content-integrity, and TUI log tests.
5. Run formatting, linting, and workspace tests, then record the outcome and
   any deviations from the proposal.

## Progress

- Resolved discussion `0007-material-gathering` and reviewed its discovery,
  minutes, proposal, architecture overview, terminal layout, and TUI test
  policy.
- Confirmed the existing task model already uses ten-second actions and emits
  completion events, but has no resource ids, reward tables, inventory, or
  controllable random input.
- Added the eight proposed Resource templates and all six shipped reward tables,
  including the explicit 90% `Nothing` fishing outcome.
- Added caller-controlled random draws, in-memory inventory stacks, reward-table
  validation at assignment, reward-bearing completion events, and accumulation
  of repeated Resource awards in core.
- Seeded production randomness at the TUI's wall-clock edge while giving tests
  and `balance-sim` explicit deterministic seeds or exact fixed rolls.
- Extended Information Log projection to include `Resource xN` or `Nothing` on
  the same line as Target, Location, and Action completion.
- Added core coverage for shipped content integrity, ten-second durations,
  exclusive draw boundaries, inventory accumulation, `Nothing`, event payloads,
  task removal, and target reuse. Added TUI coverage for both reward log forms.
- Updated current-state design documents for the resource model, completion
  flow, Information Log format, and test policy.

## Outcome

Implemented the material-gathering proposal end to end. Every shipped Action
still takes ten seconds, and every supported Location and Action combination now
draws exactly one result from its proposed 100% reward table. Resource outcomes
accumulate in the live inventory; explicit `Nothing` outcomes leave it unchanged.
Completion events and Information Log lines retain Target, Location, and Action
while adding the selected outcome.

Verification completed successfully:

- `just check` passed formatting, Clippy with warnings denied, and all 56
  workspace tests.
- `git diff --check` found no whitespace errors.
- `cargo run -p barquest-tools --bin balance-sim` completed deterministically at
  10,000 ticks (10 seconds) through the same reward-aware core path.

The worklog remains `in-progress` until the explicit decision workflow is run,
as required by the repository documentation model.

## Difficulties and Deviations

- The proposal required a controllable random source but did not prescribe an
  implementation. A small SplitMix64 generator now lives in core and accepts an
  explicit seed. This avoids a global random dependency and lets the TUI seed
  randomness at its I/O boundary while headless consumers stay reproducible.
- Assignment now rejects a Location and Action combination whose reward table is
  missing, does not total 100%, has a zero-chance entry, or references an unknown
  Resource. This makes malformed content fail before a ten-second task begins
  instead of failing only at completion.
- The generic `docs-sync` instructions refer to `docs/design-docs` and
  `docs/adr`, but this repository uses `docs/design` and reserves
  `docs/decisions` creation for the explicit decision workflow. Current-state
  design docs were updated, but no decision document was created during work.
