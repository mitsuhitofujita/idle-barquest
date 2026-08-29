---
date: 2026-08-29
status: merged
discussions: [0005-contextual-action-selection]
decisions: [0013-contextual-action-selection]
pr: null
---

# Contextual action selection

## Plan

- Audit the core action-availability and assignment rules against the proposal,
  including unknown, locked, and target-incompatible actions.
- Complete the progressive Target-to-Action app and input flow with positional
  ASCII letter keys, case-insensitive input, and ignored numeric input.
- Match the discovery's User Choices character layout, dynamically sizing the
  Target column while reserving at least 20 columns for Action choices.
- Add focused core, app, input, and renderer coverage for the rule intersection,
  state transitions, and exact representative rows.
- Update the current design documents to describe the implemented behavior and
  run the repository's CI-equivalent checks.

## Progress

- Confirmed that `TargetTemplate` already stores compatible action ids and that
  `GameState::available_actions` computes the intersection with unlocked action
  ids in unlock order.
- Confirmed that `GameState::assign_action` already rejects unknown targets,
  unknown actions, locked actions, and actions unsupported by the target kind.
- Confirmed that the current input translator maps case-insensitive ASCII
  letters to positional zero-based selection indices and ignores digits.
- Confirmed that the User Choices renderer already creates the Action column
  only after target selection, removes Target keys in that state, and places
  the selected marker immediately before the column separator.
- Confirmed through repository history that these implementation changes were
  already committed as `8256d1c` before this worklog was opened.
- Audited the feature's focused coverage: core tests exercise unlocked and
  target-compatible action intersection and assignment rejection; app tests
  exercise filtered positional selection; input tests exercise uppercase and
  lowercase letters plus ignored digits; renderer tests assert the discovery's
  representative rows and the 20-column minimum Action width.
- Updated the architecture overview, terminal layout, and TUI test policy to
  replace stale descriptions of numeric, permanently three-column choices with
  the implemented progressive letter-selection flow.
- Corrected the TUI crate-level documentation to list the current five-region
  screen order.
- Final verification passes: `just check` completes formatting, Clippy with
  warnings denied, 55 workspace unit tests, and core documentation tests;
  `git diff --check` reports no whitespace errors.

## Outcome

- Target templates declare the action ids their kind supports, and available
  Action choices are the unlock/compatibility intersection in unlock order.
- Core assignment rejects unknown targets, unknown actions, locked actions,
  and target-incompatible actions, so the rule is not confined to the TUI.
- User Choices initially renders only Target. Selecting a Target creates the
  Action column, removes keys from the inactive Target column, and marks the
  selected Target with `<` immediately before the separator.
- Choice keys are positional ASCII letters, accept matching uppercase input,
  and ignore digits. A successful Action selection assigns the quest and
  returns to Target selection.
- The representative `80x24` rows match the discovery character-for-character,
  and dynamic Target sizing preserves at least 20 columns for Action.
- Living design documentation now reflects this behavior. The subsequent
  decision workflow accepted it as ADR 0013.

## Difficulties and deviations

The material deviation from the normal work sequence was historical rather
than technical: commit `8256d1c` already contained the complete feature before
`$work` was invoked, while no corresponding worklog or design synchronization
had been created. Reimplementing the proposal would have duplicated correct
work, so this work audited the commit against every proposal clause, retained
its focused tests, and supplied the missing documentation lifecycle records.

The living design documents still described the superseded numbered,
three-column interaction even though the code and renderer tests had moved to
progressive columns. They were updated in place. During implementation,
existing decision documents were intentionally left unchanged because they are
append-only historical records; the subsequent `$decide` workflow recorded the
accepted refinement as ADR 0013.

The applicable documentation-sync skill names legacy `docs/design-docs` and
`docs/adr` paths. This repository's `docs/README.md` defines `docs/design` as the
living layer and reserves `docs/decisions` for the explicit decision workflow,
so the current repository model took precedence: three existing design files
were synchronized and no decision was created.

Cargo commands emitted a `mise` warning because its cache directory is
read-only in the workspace sandbox. The warning did not affect formatting,
compilation, linting, or tests. No structural design difficulty emerged that
warrants a new discovery.
