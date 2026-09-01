# Terminal UI Layout

- Status: Current
- Date: 2026-06-21
- Updated: 2026-09-01

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
3. Settlement
4. User Choices
5. Progress
6. Materials
7. Global Menu

The title region is always 3 rows.

The global menu region is always 1 row.

The Settlement and Materials regions are always 1 row each.

The user choices region is always 6 rows: one heading row and up to 5 choice
rows.

The progress region is always 5 rows and displays up to 5 active actions.

The information log region is at least 4 rows. Its first row is always blank,
leaving visual space below the title. The remaining vertical space is assigned
to this region, so terminals taller than 24 rows show more log history without
moving or stretching the lower fixed-height regions.

At the minimum supported height, the rows are allocated as follows:

| Region | Rows |
| --- | ---: |
| Title | 3 |
| Information Log | 4 |
| Information Log / Settlement and User Choices separator | 1 |
| Settlement | 1 |
| User Choices | 6 |
| User Choices / Progress separator | 1 |
| Progress | 5 |
| Materials | 1 |
| Progress and Materials / Global Menu separator | 1 |
| Global Menu | 1 |
| **Total** | **24** |

## Horizontal Sizing

The minimum layout width is 80 columns.

When the terminal is wider than 80 columns, columns inside each panel may expand according to their configured proportions.

Text must not wrap inside fixed-height regions. If a label or value is too long for its column, truncate it within that column.

## Separators

Do not add a full-width separator between Title and Information Log. The first
row of Information Log remains blank to provide visual separation.

Keep three separator rows: after Information Log, after User Choices, and after
Materials. Do not add a separator between Settlement and User Choices or
between Progress and Materials.

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

The progress region is 5 rows high. It summarizes running actions with one row
per active action and displays at most 5 actions.

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

The user choices region is fixed at 6 rows: one column-heading row followed by
up to 5 choices.

For the current implementation, do not handle or display more than 5 choices.
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

The acquired-resource totals are rendered in the Materials region described
below.

## Settlement Region

Settlement is the player's development base, not the Location where a Target
performs an Action. Resolve the current Settlement id from `GameState` through
the Catalog and render its English label on one line:

```text
 Settlement: Awakening Shore
```

The shipped starting Settlement is `awakening_shore` (`Awakening Shore`). The
current UI does not switch, discover, or unlock Settlements.

## Materials Region

The Materials row displays only Resources whose inventory stack exists. Stack
presence means the Resource has been acquired, so a stack remains visible when
its quantity is zero. Display them in Resource Catalog registration order,
independently of inventory acquisition order. Leave the entire row blank while
no known Resource stack exists.

Format each item as `Label: amount` and separate adjacent items with ` | `.
Reserve one arrow column plus one space on each side of the full-width content
area:

```text
< Pebble: 34 | Twig: 10 >
```

Fill the content area from the current first Resource until another complete
item plus separator no longer fits. Do not use a fixed item count. Normally,
do not truncate labels or quantities; show fewer items instead. If the first
item alone cannot fit, truncate only its label and preserve `: amount`.

The TUI stores the first visible ResourceId rather than an array index or page
number. Quantity digit changes and newly acquired Resources must not move that
start while the ResourceId still exists. `,` moves the start one acquired
Catalog item earlier. `.` moves it one item later only when the current viewport
has hidden content on the right. Both commands are global and are no-ops at
their boundaries or while the row is empty.

Show `<` only when an earlier acquired Resource exists, and `>` only when
content is hidden on the right. Replace either unavailable arrow with a space so
the content begins in the same column when arrows appear or disappear.

## Global Menu Region

The global menu is always visible at the bottom of the screen.

It contains commands that are valid from any UI state.

The global menu always includes material navigation, one-stage Backspace, and
quit, even when an operation is currently a no-op:

```text
 ,) Previous Materials  .) Next Materials  BACKSPACE) Back  ESC) Quit
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
- Settlement and acquired Materials remain visible in their fixed rows.
- The user choices region has one heading plus at most 5 entries.
- The progress region displays at most 5 active actions.
- Extra terminal height expands only the information log.

## Related decisions

- [Decision 0002: ratatui and crossterm for the TUI](../decisions/0002-ratatui-crossterm-for-tui.md)
- [Decision 0008: Full-screen TUI layout and events](../decisions/0008-full-screen-tui-layout-and-events.md)
- [Decision 0010: TUI module structure](../decisions/0010-tui-module-structure.md)
- [Decision 0011: Terminal visual style](../decisions/0011-terminal-visual-style.md)
- [Decision 0012: Terminal layout refinement](../decisions/0012-terminal-layout-refinement.md)
- [Decision 0013: Contextual action selection](../decisions/0013-contextual-action-selection.md)
- [Decision 0014: Three-stage task assignment](../decisions/0014-three-stage-task-assignment.md)
- [Decision 0015: Settlement and materials display](../decisions/0015-settlement-and-materials-display.md)
