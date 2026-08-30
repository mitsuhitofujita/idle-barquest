# Terminal UI Layout

- Status: Current
- Date: 2026-06-21
- Updated: 2026-08-30

This document defines the terminal UI layout requirements for `IDLE BARQUEST`.
It is written as an implementation-oriented reference for AI agents and developers.

## Scope

The game uses a full-screen terminal UI. The screen should be redrawn as a stable layout, similar to a vi-style terminal application, rather than rendered as a continuously scrolling terminal log.

The same type of information should appear in the same screen region across redraws.

## Terminal Requirements

The minimum supported terminal size is `80x24`.

If the terminal width is less than 80 columns, or the terminal height is less than 24 rows, the game must show a warning and refuse to start. Do not attempt a compact or degraded layout below `80x24`.

The layout should stretch horizontally when the terminal is wider than 80 columns.

## Character And Color Constraints

Use standard single-width terminal characters for the UI.

Allowed layout characters include:

- `+`
- `-`
- `|`
- ASCII letters, digits, and punctuation

Do not use double-width characters such as Japanese text in the in-game terminal UI. This avoids layout instability caused by terminal width differences.

Color may be used to improve readability, but color must not be the only way to communicate important game state. Important information such as selection, success, failure, and progress must also be represented by text, symbols, position, or other non-color cues.

## Screen Regions

The screen is divided vertically into the following regions, in this fixed order:

1. Title
2. Information Log
3. User Choices
4. Progress
5. Global Menu

The title region is always 3 rows.

The global menu region is always 1 row.

The user choices region is always 7 rows: one heading row and up to 6 choice
rows.

The progress region is always 6 rows and displays up to 6 active actions.

The information log region is at least 4 rows. Its first row is always blank,
leaving visual space below the title. The remaining vertical space is assigned
to this region, so terminals taller than 24 rows show more log history without
moving or stretching the lower fixed-height regions.

At the minimum supported height, the rows are allocated as follows:

| Region | Rows |
| --- | ---: |
| Title | 3 |
| Information Log | 4 |
| Information Log / User Choices separator | 1 |
| User Choices | 7 |
| User Choices / Progress separator | 1 |
| Progress | 6 |
| Progress / Global Menu separator | 1 |
| Global Menu | 1 |
| **Total** | **24** |

## Horizontal Sizing

The minimum layout width is 80 columns.

When the terminal is wider than 80 columns, columns inside each panel may expand according to their configured proportions.

Text must not wrap inside fixed-height regions. If a label or value is too long for its column, truncate it within that column.

## Separators

Do not add a full-width separator between Title and Information Log. The first
row of Information Log remains blank to provide visual separation.

Place a separator row between each of the remaining region pairs: Information
Log and User Choices, User Choices and Progress, and Progress and Global Menu.

Separators should be built from standard ASCII characters. They should include visual marks at the left side, center, and right side.

At 80 columns, the separator should look like this:

```text
+------------------------------------+----+------------------------------------+
```

The separator is primarily a visual boundary between vertical regions. The center `+----+` segment acts as a light visual anchor for the left and right halves of the screen.

## Padding And Alignment

Each column should reserve 1 leading space before its text.

Most column text should be left-aligned.

The title artwork is an exception and should be centered as a fixed-width block.

## Title Region

Display the following three-line game title centered as a 38-column block. Do
not stretch its decoration when the terminal grows wider:

```text
.@~\::::::::::::::::::::::::::::::/@~.
(  {        IDLE BARQUEST         }  )
'@~/::::::::::::::::::::::::::::::\~@'
```

The title is a fixed header. It should not change based on the current selection, progress, or game state.

## Progress Region

The progress region is 6 rows high. It summarizes running actions with one row
per active action and displays at most 6 actions.

Use the following column proportions:

| Column | Width | Purpose |
| --- | ---: | --- |
| Target | 20% | Person or organization performing the task |
| Location | 20% | Place or facility where the task runs |
| Action | 20% | Current work |
| Progress Bar | 40% | Main action progress |

All progress-region columns must fit on one row. Do not wrap long labels or values.

The progress bar should use a familiar character-based style when possible:

```text
[===>   ] 50%
```

Progress bar characters:

- `=` means completed progress.
- `>` means the current progress position.
- A space means remaining progress.
- The percentage should appear to the right of the bar.

At the endpoints and midway through a seven-cell bar, the output is:

```text
[       ]   0%
[===>   ]  50%
[=======] 100%
```

If the progress column is narrow, shorten the bar before removing the percentage. If the content still does not fit, truncate it like other column content.

## User Choices Region

The user choices region presents one selection stage at a time. It does not
reserve empty columns for stages the player has not reached.

The expected interaction flow is:

1. Show only `Target` choices.
2. After the player selects a Target, create a `Location` column to its right.
3. After the player selects a Location, create an `Action` column to its right.
4. After the player selects an Action, assign it and return to Target selection.

Target selection at `80x24` begins like this:

```text
|> Target:
| a) Hero
```

After selecting Hero and First Shore, representative rows are:

```text
|  Target:  |  Location:      |> Action:
|    Hero <|    First Shore <| a) Gather
|          |    Nearby Woods  | b) Fish
|          |    Nearby Hill   |
```

Each active column assigns positional ASCII lowercase keys in display order:
`a)`, `b)`, `c)`, and so on. The corresponding uppercase letter selects the
same item. Digits do not select choices, and inactive columns do not display
selection keys. Target keys correspond to fixed display slots. A busy Target
remains visible with `-` in place of its key, and later Targets keep their
original letters:

```text
| -  Hero
| b) Adventurer
```

Location choices include only discovered or unlocked Locations. Action choices
are the intersection of unlocked Actions and those supported by both the
selected Target and Location. Core validates the same constraints and rejects
unknown ids, locked content, incompatible combinations, and busy Targets.

During later stages, `<` marks the chosen Target and Location immediately before
the following separator. This makes the relationship visible without relying
on color.

Completed columns derive their width from their widest heading or label plus
space for the selected marker. The active column receives the remainder, with
at least 20 columns reserved for it. Truncate completed cells if necessary.
During Target selection, do not render or reserve later columns.

The user choices region is fixed at 7 rows: one column-heading row followed by
up to 6 choices.

For the current implementation, do not handle or display more than 6 choices.
Do not show page information in the current UI.

Backspace returns exactly one stage: Action to Location, Location to Target, and
no change from Target. `Esc` remains a global quit command in every stage.

Future paging behavior may use the up and down arrow keys. If paging is added
later, headings should include page information such as:

```text
Target: 1/2
```

The page indicator should show the current page and total page count.

## Information Log Region

The information log region displays game events such as gained resources, action results, and state changes.

This region behaves like a terminal log:

- Its first row is always blank, providing the gap below the title.
- Its minimum height is 4 rows, leaving 3 event rows at `80x24`.
- New information appears at the bottom.
- Older information moves upward.
- If the log exceeds the visible region, old lines may disappear off the top.
- If fewer events exist than fit, they are bottom-aligned.
- All terminal height above 24 rows is assigned to this region while its blank
  first row remains exactly one row high.

The current implementation does not need scrollback or history navigation.

Action completion and its exclusive reward result use one line so the three
event rows available at `80x24` remain useful:

```text
Hero completed Gather at Nearby Hill: Pebble x1
Hero completed Fish at First Shore: Nothing
```

The inventory total is intentionally not rendered yet.

## Global Menu Region

The global menu is always visible at the bottom of the screen.

It contains commands that are valid from any UI state.

The current global menu only includes quit:

```text
ESC) Quit
```

## Implementation Notes For Agents

Treat this file as the source of truth for terminal layout behavior.

Prefer deterministic layout calculations over ad hoc string assembly.

When implementing or modifying the UI, preserve these invariants:

- The game does not start below `80x24`.
- The screen has stable full-screen regions.
- Text does not wrap inside fixed-height regions.
- Important state is not color-only.
- The title and global menu remain fixed.
- The user choices region has one heading plus at most 6 entries.
- The progress region displays at most 6 active actions.
- Extra terminal height expands only the information log.

## Related decisions

- [Decision 0002: ratatui and crossterm for the TUI](../decisions/0002-ratatui-crossterm-for-tui.md)
- [Decision 0008: Full-screen TUI layout and events](../decisions/0008-full-screen-tui-layout-and-events.md)
- [Decision 0010: TUI module structure](../decisions/0010-tui-module-structure.md)
- [Decision 0011: Terminal visual style](../decisions/0011-terminal-visual-style.md)
- [Decision 0012: Terminal layout refinement](../decisions/0012-terminal-layout-refinement.md)
- [Decision 0013: Contextual action selection](../decisions/0013-contextual-action-selection.md)
- [Decision 0014: Three-stage task assignment](../decisions/0014-three-stage-task-assignment.md)
