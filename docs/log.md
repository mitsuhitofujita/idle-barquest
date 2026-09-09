# Work log

## Cargo workspace (2026-06-20)

- Split the repository into `barquest-core`, `barquest-tui`, and
  `barquest-tools`.
- Centralized shared package metadata and dependencies so game rules remain
  independent of terminal I/O and reusable by tools.

## Terminal rendering stack (2026-06-20)

- Adopted Ratatui with the Crossterm backend for cross-platform terminal
  rendering and input.
- Used immediate-mode redraws to project core state into the terminal.

## Reproducible development commands (2026-06-20)

- Pinned Rust 1.96, edition 2024, Clippy, and rustfmt for the workspace.
- Added `Justfile` commands for build, run, test, formatting, linting, checks,
  and tools, and committed `Cargo.lock` for reproducible application builds.

## Tick-based game time (2026-06-20)

- Defined core game time as 1,000 ticks per second and made progress a pure
  tick-driven value.
- Made the TUI advance 100 ticks per 100 ms frame so front ends control pacing
  while core and headless tools share the same simulation.

## Command menu (2026-06-20)

- Added targets and actions as core concepts and guided the player from target
  choice to action choice and quest execution.
- Used label-derived letter keys for choices and global quit keys.

## Concurrent target progress (2026-06-20)

- Made the game loop non-blocking so multiple targets could progress on the
  same frame schedule while the player used the menu.
- Replaced the single modal gauge with inline action rows.

## Data-driven content (2026-06-20)

- Replaced target and action enums with templates in a registration-ordered
  `Catalog` and live instances in `GameState`, joined by stable string ids.
- Chose data-backed content so the world can grow without adding Rust variants.

## Full-screen TUI and completion events (2026-06-21)

- Replaced the loose terminal output with a fixed full-screen layout containing
  progress, choices, an information log, and global controls.
- Made core emit completion events and remove finished work from active progress
  so the TUI could record completions in the log.

## Core modules (2026-06-21)

- Split core into modules for time, ids, catalog content, and live state while
  preserving a flat public API.
- Kept tests beside the behavior they cover and module dependencies aligned
  with the content-to-state boundary.

## TUI application layers (2026-06-21)

- Separated the terminal crate into input translation, application behavior,
  pure rendering, and a thin terminal loop.
- This made state transitions and screen projection testable without a live
  terminal or wall clock.

## Documentation model migration (2026-08-29)

- Moved existing decisions and current design references into the repository's
  documentation structure at the time.
- Updated repository references and removed the superseded documentation
  directories.

## Terminal visual style (2026-08-29)

- Replaced the title with centered three-line ASCII artwork and removed the
  separator that followed it.
- Changed unreached progress cells from hyphens to spaces while retaining the
  brackets, leading edge, completed cells, and percentage.

## Terminal layout (2026-08-29)

- Reordered the screen to put the information log before choices and active
  progress.
- Fixed the lower region heights, assigned additional height to the log, and
  bottom-aligned events below a permanent blank row.

## Contextual action choices (2026-08-29)

- Made User Choices progressive: choose a target, then see only actions that
  are both unlocked and supported by that target.
- Replaced number keys with positional ASCII letters and enforced the same
  availability rules in core assignment.

## Three-stage task assignment (2026-08-30)

- Added locations as stable catalog content and made each task one Target,
  Location, and Action assignment.
- Core validates unlocks, compatibility, and target availability; the TUI
  exposes all three stages and releases the target when its task completes.

## Material rewards (2026-08-30–2026-09-06)

- Added resources, inventory, reward tables, deterministic randomness, and
  reward-bearing completion events for every shipped Location and Action pair.
- Changed reward tables to ordered independent checks so one completion can
  grant several resources, aggregate repeated resources, or grant nothing.

## Settlement and materials display (2026-09-01)

- Added a catalog-backed current Settlement, seeded as Awakening Shore, and
  fixed Settlement and Materials rows without increasing the `80x24` minimum.
- Displayed acquired resources in catalog order through a width-aware viewport
  that can be navigated globally.

## Base crafting (2026-09-07)

- Added Base -> Craft -> Recipe selection with four twenty-second recipes,
  start-time ingredient consumption, prerequisite checks, and disabled reasons.
- Added permanent unique facilities and a repeatable Primitive Fishing Rod item;
  completed facilities appear beside Settlement and materials/items share the
  renamed Inventory display.
- Kept recipes as catalog content so crafting rules remain terminal-independent
  and existing gathering tasks keep their reward behavior.

## Action confirmation previews (2026-09-09)

- Added reward previews with material amounts and drop chances before ordinary
  actions start, and made Enter the explicit start command.
- Made every available recipe selectable for a requirements preview showing
  held/required materials and facility status; Enter stays disabled until the
  recipe can start.

## Craft start status (2026-09-09)

- Moved the craft start status from the global menu into the Requirements pane.
- The pane shows `ENTER) Start` when the recipe is ready, or its blocking reason
  when it cannot begin, so the state appears beside the requirements.

## Confirmation pane footers (2026-09-09)

- Fixed recipe start states to the final row of the Requirements pane.
- Added `ENTER) Start` to the final row of ordinary-action Rewards panes as well.
