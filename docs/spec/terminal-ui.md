# Terminal UI

## Application and input

The TUI keeps mutable front-end state in `App`: the catalog and game state, the
current choice stage, a bounded information log, an inventory viewport start id,
and the seeded random source. Raw Crossterm events are translated into a small
input enum before application behavior is applied.

Task assignment normally follows a progressive Target -> Location -> Action
flow. Selecting Craft at Base adds a Recipe stage. The active column uses
case-insensitive positional ASCII letters: `a` selects the first entry, `b` the
second, and so on. Busy targets keep their fixed letter slot but cannot be
selected. Backspace moves back one stage. Numeric and unknown input is ignored;
`q`, Escape, and Ctrl-C quit from any stage.

Comma and period move the inventory viewport one acquired entry backward or
forward. These controls are global and do not alter the current assignment
stage. Movement is a no-op at a boundary or when every remaining resource fits.

## Screen

The game UI is ASCII-only and requires at least `80x24`; smaller terminals show
only a centered size warning. At supported sizes it renders these regions from
top to bottom:

1. a centered, fixed-width, three-row title;
2. an information log that receives all extra height;
3. a full-width separator;
4. a one-row Settlement display;
5. a six-row User Choices display;
6. a full-width separator;
7. a five-row Progress display;
8. a one-row Inventory display;
9. a full-width separator;
10. a one-row global menu.

The first Information Log row is always blank. Visible events are bottom-aligned,
long lines are clipped to the terminal width, and older entries are discarded
from the screen when the region is full. The in-memory log retains at most 200
entries.

## Choices and progress

User Choices initially shows only Target. Selecting a target adds Location, and
selecting a location adds Action. Selecting Craft at Base adds Recipe as a
fourth column. Completed columns lose their letter keys and show `<` at the
selected row; the active column is marked with `>`. Completed columns use their
content width while leaving at least 20 columns for the active stage. The region
displays its heading and at most five entries.

Unbuilt facility recipes and repeatable item recipes remain visible when they
cannot start. Their choice key is replaced with a dash, and the first unmet
facility or material requirement is displayed before the recipe name. A unique
facility recipe is omitted while under construction and after completion.

Each active task occupies one Progress row, with Target, Location, Action, and
Progress Bar columns using 20%, 20%, 20%, and 40% of the width. A crafting row
shows its recipe name in the Action column. The text bar uses `=`, `>`, spaces,
brackets, and a percentage. At most five active tasks are visible.

## Settlement, inventory, and events

Settlement resolves the current settlement id through the catalog and appends
all completed facilities, or `None` when there are none. Inventory shows
acquired material and item stacks in catalog order as `Label: amount`, separated
by ` | `. Four columns are reserved for boundary arrows. Complete entries are
fitted to the available width; if the first entry is too wide, only its label is
shortened so its quantity remains visible. The viewport stores the first visible
resource id, so quantity-width and acquisition changes do not change its logical
position.

A completion event adds one line in the form
`Target completed Action at Location: rewards`. Multiple rewards are comma
separated; an empty reward list is rendered as `Nothing`. Craft completion adds
`Target crafted Recipe at Location`.
