---
date: 2026-08-30
status: in-progress
discussions: [0006-three-stage-user-choices]
decisions: []
pr: null
---

# Three-stage user choices

## Plan

- Audit the current catalog, live state, assignment rules, TUI state machine,
  input translation, and renderer against the accepted proposal.
- Introduce stable Location content and unlocked Location state, and make each
  running task a single `Target + Location + Action` assignment.
- Implement progressive `Target -> Location -> Action` selection, Backspace
  navigation, fixed Target slots, and unavailable running Targets.
- Replace the Progress region's reserved columns with Target, Location, Action,
  and Progress Bar, and include Location in completion logs.
- Add focused core, app, input, and renderer tests, then run the repository's
  CI-equivalent checks.

## Progress

- Resolved discussion `0006-three-stage-user-choices` and read its proposal,
  discovery, minutes, and the relevant architecture, terminal layout, and TUI
  test-policy documents.
- Confirmed that the work spans the core catalog/state model and all three TUI
  layers: app behavior, input translation, and rendering.
- Noted pre-existing uncommitted edits to the work skill and this discussion's
  proposal; they are outside the implementation and will be preserved.
- Added stable Location ids and templates, unlocked Location live state, and the
  proposal's three starting Locations and three Actions. The starting Target is
  now Hero only.
- Replaced each Target's quest list with one optional task containing Location,
  Action, and Progress. Assignment now rejects busy Targets and validates every
  Target/Location/Action id, unlock, and compatibility constraint.
- Added Location to completion events and changed completion to clear the task,
  making the Target available again without automatic repetition.
- Implemented the three-stage app flow, Backspace navigation, fixed Target
  selection slots, busy-Target rejection, four-column Progress rows, and
  Location-bearing completion logs.
- Added focused coverage across the core rules, app state transitions, input
  translation, and `80x24` renderer projection, including the invariant that a
  Target after a busy slot keeps its original letter.
- Updated the architecture overview, terminal layout, and TUI test policy to
  describe the implemented three-dimensional task model and interaction flow.
- Final verification passes: `just check` completes formatting, Clippy with
  warnings denied, 50 workspace unit tests, and core documentation tests. The
  headless balance simulator completes at 10,000 ticks, and `git diff --check`
  reports no whitespace errors.
- The commit-time audit caught that the proposal's busy-Target example had been
  refined from `--` to `-`; the renderer, focused assertions, and living design
  documents were aligned with the accepted `| -  Hero` form before commit.

## Outcome

- User Choices now progresses through Target, Location, and Action, adding only
  the columns reached by the player. Backspace returns exactly one stage, while
  `Esc` retains its global quit behavior.
- The shipped world starts with Hero; First Shore, Nearby Woods, and Nearby
  Hill; and the proposed Gather, Fish, and Hunt compatibility matrix.
- Core owns the complete assignment contract. Locations and Actions must be
  known and unlocked, both the Target and Location must support the Action, and
  a Target already running a task rejects another assignment.
- A task and its completion event preserve Location as game data. Completion
  clears the Target's sole task, does not repeat automatically, and logs the
  Target, Action, and Location.
- Progress now renders Target, Location, Action, and Progress Bar at
  20/20/20/40 percent widths. Busy Targets stay visible as `-`, and later
  Targets retain their fixed positional keys.

## Difficulties and deviations

The proposal intentionally leaves duration and balance values unspecified, but
the runnable model still requires a nonzero duration. The existing ten-second
development duration was retained uniformly for Gather, Fish, and Hunt; this is
an implementation default rather than a balance decision.

The initial renderer assertions assumed a particular number of padding spaces
before the selected marker. The actual invariant is that `<` is immediately
adjacent to the following separator; assertions were corrected to test that
boundary without changing the rendering.

The discussion minutes still preserve the earlier `--` notation, while the
proposal records the later `-` refinement. As historical discussion text must
not be rewritten after the fact, implementation follows the proposal and the
worklog records the difference instead of altering the minutes.

The documentation-sync skill refers to legacy `docs/design-docs` and `docs/adr`
paths. This repository's `docs/README.md` defines `docs/design` as the living
documentation layer and reserves `docs/decisions` for the explicit `$decide`
workflow, so the three current design documents were updated and no premature
decision was created. No structural difficulty emerged that warrants a new
discovery.
