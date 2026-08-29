# Terminal UI Layout

- Status: Current
- Date: 2026-06-21
- Updated: 2026-08-29

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
2. Progress
3. User Choices
4. Information Log
5. Global Menu

The title region is always 3 rows.

The global menu region is always 1 row.

The user choices region is normally 10 rows.

The remaining vertical space should be assigned to the information log region.

## Horizontal Sizing

The minimum layout width is 80 columns.

When the terminal is wider than 80 columns, columns inside each panel may expand according to their configured proportions.

Text must not wrap inside fixed-height regions. If a label or value is too long for its column, truncate it within that column.

## Separators

The title artwork's bottom row serves as the visual boundary between the Title
and Progress regions. Do not add a full-width separator below the title.

Place a separator row between each of the remaining region pairs: Progress and
User Choices, User Choices and Information Log, and Information Log and Global
Menu.

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

The progress region summarizes the current target, current action, and action progress.

Use the following column proportions:

| Column | Width | Purpose |
| --- | ---: | --- |
| Target | 20% | Current selected or active target |
| Action | 20% | Current active action |
| Times | 10% | Reserved for a future feature; do not emphasize in the current UI |
| Progress Bar | 30% | Main action progress |
| Sub Action | 10% | Reserved for a future feature; do not emphasize in the current UI |
| Sub Progress Bar | 10% | Reserved for a future feature |

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

The `Times` and `Sub Action` concepts are future-facing. Preserve the column reservations, but avoid making them prominent in the current game UI until the related gameplay features exist.

## User Choices Region

The user choices region lists available player choices.

The expected interaction flow is:

1. The player selects a `Target`.
2. The UI shows the available `Action` choices for that target.

Selection and relationships between choices must not rely on color alone. Use text or ASCII symbols such as arrows.

Example:

```text
Target:        | Action:      | Times:
 1) Hero ----->|  1) Forest   |  1) 1
 2) Explorer   |  2) Sea ---->|  2) 5
```

The user choices region is designed around 10 rows.

For the current implementation, do not handle or display more than 10 choices. Do not show page information in the current UI.

Future paging behavior may use the up and down arrow keys. If paging is added later, headings should include page information such as:

```text
Target: 1/2
```

The page indicator should show the current page and total page count.

## Information Log Region

The information log region displays game events such as gained resources, action results, and state changes.

This region behaves like a terminal log:

- New information appears at the bottom.
- Older information moves upward.
- If the log exceeds the visible region, old lines may disappear off the top.

The current implementation does not need scrollback or history navigation.

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
- The user choices region is limited to 10 rows for now.

## Related decisions

- [Decision 0002: ratatui and crossterm for the TUI](../decisions/0002-ratatui-crossterm-for-tui.md)
- [Decision 0008: Full-screen TUI layout and events](../decisions/0008-full-screen-tui-layout-and-events.md)
- [Decision 0010: TUI module structure](../decisions/0010-tui-module-structure.md)
