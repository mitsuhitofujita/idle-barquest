//! idle-barquest terminal front-end.
//!
//! Drives the command loop and renders the game state from `barquest-core`.
//! The world is data-driven: a [`Catalog`] supplies the target/action templates
//! and a [`GameState`] holds the live target instances. The screen is the fixed
//! five-region layout from `docs/terminal-ui-layout.md` (Title, Progress, User
//! Choices, Information Log, and Global Menu), separated by ASCII rules, and is
//! hidden behind a warning below the supported `80x24` terminal size. Progress
//! bars fill **concurrently** like an `apt` / `mise` update. The player picks a
//! target by number, then an action by number, in the three-column user-choices
//! panel (`>` marks the active column; the chosen target shows a ` ----->`
//! arrow); `q` / `Esc` / `Ctrl-C` quits from any screen.

use std::io;
use std::time::{Duration, Instant};

use barquest_core::{Catalog, GameEvent, GameState, TargetId};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

/// Real-time cadence of one render/update frame.
const FRAME: Duration = Duration::from_millis(100);
/// Game-time advanced per frame. `FRAME` (100 ms) == 100 ticks at 1000 ticks/s.
const TICKS_PER_FRAME: u64 = 100;
/// Minimum supported terminal size. Below this the game shows a warning and
/// refuses to draw its UI (see `docs/terminal-ui-layout.md`).
const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;
/// Fixed heights (rows) of the title, user-choices, and global-menu regions.
/// The separators between regions are one row each; the information log takes
/// whatever vertical space is left.
const TITLE_H: u16 = 1;
const CHOICES_H: u16 = 10;
const MENU_H: u16 = 1;
/// How many information-log lines to retain; older lines are dropped. Only the
/// last few (one per visible row) are ever shown, so this just bounds memory.
const LOG_CAPACITY: usize = 200;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

/// Which menu the bottom of the screen is currently showing.
enum Menu {
    /// Choose which target to command next.
    SelectTarget,
    /// Choose an action for the already-chosen target instance.
    SelectAction { target: TargetId },
}

/// Runs the unified loop: advance every target's quest each frame while the
/// player assigns actions via the bottom menu. Returns when the player quits.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let catalog = Catalog::builtin();
    let mut state = GameState::seeded(&catalog);
    let mut menu = Menu::SelectTarget;
    let mut log: Vec<String> = Vec::new();
    let start = Instant::now();
    let mut frame_idx: u32 = 0;

    loop {
        terminal.draw(|frame| render(frame, &catalog, &state, &menu, &log))?;

        // Drain input until the next frame boundary, pinned to the wall clock so
        // every bar fills in real time regardless of how long rendering takes.
        frame_idx += 1;
        let deadline = start + FRAME * frame_idx;
        loop {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            if event::poll(deadline - now)? {
                let event = event::read()?;
                if is_quit(&event) {
                    return Ok(());
                }
                handle_key(&event, &catalog, &mut state, &mut menu);
                // Redraw immediately so menu navigation feels responsive.
                terminal.draw(|frame| render(frame, &catalog, &state, &menu, &log))?;
            }
        }

        for event in state.advance(TICKS_PER_FRAME) {
            push_event(&mut log, &catalog, &event);
        }
    }
}

/// Appends one ASCII log line for a game event, resolving ids to labels via the
/// catalog, and caps the buffer at [`LOG_CAPACITY`] by dropping the oldest line.
fn push_event(log: &mut Vec<String>, catalog: &Catalog, event: &GameEvent) {
    let line = match event {
        GameEvent::QuestCompleted { target, action } => {
            let target = catalog.target(target).map_or("?", |t| t.label.as_str());
            let action = catalog.action(action).map_or("?", |a| a.label.as_str());
            format!("{target} completed {action}")
        }
    };
    log.push(line);
    if log.len() > LOG_CAPACITY {
        log.remove(0);
    }
}

/// Applies a key press to the menu/state: pick a target by number, then pick an
/// action by number, which (re)starts that target's quest. The digit indexes the
/// numbered list shown in the active choices column. Out-of-range or non-digit
/// keys are ignored.
fn handle_key(event: &Event, catalog: &Catalog, state: &mut GameState, menu: &mut Menu) {
    let Event::Key(key) = event else {
        return;
    };
    if key.kind != KeyEventKind::Press {
        return;
    }
    let KeyCode::Char(c) = key.code else {
        return;
    };
    let Some(idx) = digit_index(c) else {
        return;
    };

    match menu {
        Menu::SelectTarget => {
            if let Some(target) = state.targets.get(idx).map(|inst| inst.id.clone()) {
                *menu = Menu::SelectAction { target };
            }
        }
        Menu::SelectAction { target } => {
            // Clone the chosen instance id out so we can reassign `menu` below.
            let target = target.clone();
            if let Some(action) = state.unlocked_actions.get(idx).cloned() {
                state.assign_action(catalog, &target, &action);
                *menu = Menu::SelectTarget;
            }
        }
    }
}

/// Maps a digit key to a 0-based choice index: `'1'..='9'` -> `0..=8`, `'0'` ->
/// `9` (the tenth). Any other character is not a selection.
fn digit_index(c: char) -> Option<usize> {
    match c {
        '1'..='9' => Some(c as usize - '1' as usize),
        '0' => Some(9),
        _ => None,
    }
}

/// Lays out the screen as the fixed five-region stack from
/// `docs/terminal-ui-layout.md`: Title / Progress / User Choices / Information
/// Log / Global Menu, with an ASCII separator row between each region. Below the
/// minimum terminal size it draws only a warning and leaves the game UI hidden.
fn render(frame: &mut Frame, catalog: &Catalog, state: &GameState, menu: &Menu, log: &[String]) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area);
        return;
    }

    // One progress row per active action; keep a 1-row reserve when idle so the
    // region (and the separators below it) stay put.
    let progress_h = (state.active_quests().count() as u16).max(1);

    let [
        title,
        sep1,
        progress,
        sep2,
        choices,
        sep3,
        log_area,
        sep4,
        menu_area,
    ] = Layout::vertical([
        Constraint::Length(TITLE_H),
        Constraint::Length(1),
        Constraint::Length(progress_h),
        Constraint::Length(1),
        Constraint::Length(CHOICES_H),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(MENU_H),
    ])
    .areas(area);

    render_title(frame, title);
    render_progress(frame, progress, catalog, state);
    render_choices(frame, choices, catalog, state, menu);
    render_log(frame, log_area, log);
    render_menu(frame, menu_area);
    for sep in [sep1, sep2, sep3, sep4] {
        render_separator(frame, sep);
    }
}

/// Draws the fixed, centered game title. It never reflects game state.
fn render_title(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("IDLE BARQUEST").alignment(Alignment::Center),
        area,
    );
}

/// Draws the always-visible global menu: commands valid from any UI state.
fn render_menu(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new(" ESC) Quit"), area);
}

/// Draws the information log: game events newest at the bottom, older above,
/// with anything beyond the region height scrolled off the top.
fn render_log(frame: &mut Frame, area: Rect, log: &[String]) {
    let lines = log_lines(log, area.width, area.height).join("\n");
    frame.render_widget(Paragraph::new(lines), area);
}

/// The last `height` log lines (oldest dropped off the top), each truncated to
/// `width` chars. Order is preserved, so the newest line is last (it renders on
/// the bottom row of the top-aligned block).
fn log_lines(log: &[String], width: u16, height: u16) -> Vec<String> {
    let width = width as usize;
    let take = height as usize;
    let start = log.len().saturating_sub(take);
    log[start..]
        .iter()
        .map(|line| line.chars().take(width).collect())
        .collect()
}

/// Centered warning shown when the terminal is smaller than `80x24`.
fn render_too_small(frame: &mut Frame, area: Rect) {
    let msg = format!(
        "Terminal too small: need {MIN_WIDTH}x{MIN_HEIGHT}, have {}x{}",
        area.width, area.height
    );
    frame.render_widget(Paragraph::new(msg).alignment(Alignment::Center), area);
}

/// Draws one full-width ASCII separator row between two vertical regions.
fn render_separator(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new(separator(area.width)), area);
}

/// Builds a full-width separator like
/// `+------+----+------+`: a `+` at each end, a fixed `+----+` anchor in the
/// middle, and `-` fill on both sides. At 80 columns each side is 36 dashes,
/// reproducing the example in `docs/terminal-ui-layout.md`.
fn separator(width: u16) -> String {
    let w = width as usize;
    // The frame (`+`...`+----+`...`+`) needs 8 non-fill columns; below that just
    // fill with dashes so narrow widths stay valid.
    if w < 8 {
        return "-".repeat(w);
    }
    let fill = w - 8;
    let left = fill.div_ceil(2);
    let right = fill - left;
    format!("+{}+----+{}+", "-".repeat(left), "-".repeat(right))
}

/// Draws one row per **active action** (not per target): a facility running two
/// productions shows two rows. Each row is the six proportional columns from
/// `docs/terminal-ui-layout.md`. Idle targets contribute no row.
fn render_progress(frame: &mut Frame, area: Rect, catalog: &Catalog, state: &GameState) {
    let rows = progress_rows(catalog, state, area.width).join("\n");
    frame.render_widget(Paragraph::new(rows).alignment(Alignment::Left), area);
}

/// Splits a full row width into the six progress columns from
/// `docs/terminal-ui-layout.md` — Target 20% / Action 20% / Times 10% /
/// Progress Bar 30% / Sub Action 10% / Sub Progress Bar 10% — as integer cell
/// counts. The remainder from rounding is given to the last column so the parts
/// always sum back to `width`.
fn progress_columns(width: u16) -> [usize; 6] {
    let total = width as usize;
    let pcts = [20usize, 20, 10, 30, 10, 10];
    let mut cols = [0usize; 6];
    let mut used = 0;
    for i in 0..5 {
        cols[i] = total * pcts[i] / 100;
        used += cols[i];
    }
    cols[5] = total.saturating_sub(used);
    cols
}

/// Builds the progress rows for the current world, one per active quest, each
/// exactly `width` chars wide.
///
/// Per column the text gets a 1-space lead and is padded/truncated to fill the
/// column ([`fit`]). The progress-bar column shrinks its bar to whatever space
/// is left after the brackets and ` NNN%` suffix. `Times`, `Sub Action`, and
/// `Sub Progress Bar` are reserved for later features and render blank.
fn progress_rows(catalog: &Catalog, state: &GameState, width: u16) -> Vec<String> {
    let cols = progress_columns(width);
    state
        .active_quests()
        .map(|(target, quest)| {
            let target_label = catalog
                .target(&target.template_id)
                .map(|t| t.label.as_str())
                .unwrap_or("?");
            let action_label = catalog
                .action(&quest.action)
                .map(|a| a.label.as_str())
                .unwrap_or("?");
            let cells = [
                column(target_label, cols[0]),
                column(action_label, cols[1]),
                column("", cols[2]),
                column(&progress_cell(quest.progress.ratio(), cols[3]), cols[3]),
                column("", cols[4]),
                column("", cols[5]),
            ];
            cells.concat()
        })
        .collect()
}

/// Renders one column cell: a leading space then `text` fitted to the remaining
/// width, so the whole cell is exactly `width` chars.
fn column(text: &str, width: usize) -> String {
    format!(" {}", fit(text, width.saturating_sub(1)))
}

/// The bar-plus-percent content for the progress column, sized to fit `width`
/// (the leading space is added by [`column`]). The bar shrinks before the `%`
/// is dropped; [`fit`] truncates if it still does not fit.
fn progress_cell(ratio: f64, width: usize) -> String {
    let pct = (ratio * 100.0).round() as u16;
    let suffix = format!(" {pct:>3}%"); // e.g. "  50%" / " 100%"
    // Account for the leading space `column` adds, the `[]` brackets, and suffix.
    let cells = width.saturating_sub(1 + 2 + suffix.chars().count());
    if cells == 0 {
        return suffix.trim_start().to_string();
    }
    format!("{}{suffix}", progress_bar(ratio, cells))
}

/// Left-aligns `text` to exactly `width` chars: padded with spaces if short,
/// truncated (by char) if long.
fn fit(text: &str, width: usize) -> String {
    let mut out: String = text.chars().take(width).collect();
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

/// Draws the user-choices region: the three `Target: | Action: | Times:` columns
/// from `docs/terminal-ui-layout.md`. Entries are numbered; the player presses a
/// digit to pick within the active column (`>` marks which one), and the chosen
/// target carries a ` ----->` arrow into the action column.
fn render_choices(
    frame: &mut Frame,
    area: Rect,
    catalog: &Catalog,
    state: &GameState,
    menu: &Menu,
) {
    let widths = choices_columns(area.width);
    let height = area.height as usize;
    let selecting_target = matches!(menu, Menu::SelectTarget);
    let chosen = match menu {
        Menu::SelectAction { target } => Some(target),
        Menu::SelectTarget => None,
    };

    // Target column: every target, numbered; the chosen one points an arrow at
    // the action column it is about to be assigned from.
    let mut target_col = vec![header("Target:", selecting_target)];
    for (i, inst) in state.targets.iter().enumerate() {
        let label = catalog
            .target(&inst.template_id)
            .map(|t| t.label.as_str())
            .unwrap_or("?");
        let arrow = if chosen == Some(&inst.id) {
            " ----->"
        } else {
            ""
        };
        target_col.push(format!("  {}) {label}{arrow}", i + 1));
    }

    // Action column: the unlocked actions, numbered (active while choosing one).
    let mut action_col = vec![header("Action:", !selecting_target)];
    for (i, id) in state.unlocked_actions.iter().enumerate() {
        let label = catalog.action(id).map(|a| a.label.as_str()).unwrap_or("?");
        action_col.push(format!("  {}) {label}", i + 1));
    }

    // Times column: reserved for a future "repeat N times" feature. Show the
    // implicit default so the column reads, but it is not selectable yet.
    let times_col = vec![header("Times:", false), "  1) 1".to_string()];

    let rows: Vec<String> = (0..height)
        .map(|r| {
            let cell = |col: &[String], w: usize| fit(col.get(r).map_or("", |s| s), w);
            format!(
                "{}|{}|{}",
                cell(&target_col, widths[0]),
                cell(&action_col, widths[1]),
                cell(&times_col, widths[2]),
            )
        })
        .collect();

    frame.render_widget(Paragraph::new(rows.join("\n")), area);
}

/// A choices-column header with a 1-char active marker: `>` when this column is
/// where digit presses currently go, a space otherwise.
fn header(name: &str, active: bool) -> String {
    let marker = if active { '>' } else { ' ' };
    format!("{marker}{name}")
}

/// Splits a row width into the three user-choices columns (`Target | Action |
/// Times`), accounting for the two `|` separators. Target and Action take ~40%
/// each; Times gets the remainder, so the three columns plus separators sum back
/// to `width`.
fn choices_columns(width: u16) -> [usize; 3] {
    let inner = (width as usize).saturating_sub(2); // two '|' separators
    let target = inner * 40 / 100;
    let action = inner * 40 / 100;
    let times = inner - target - action;
    [target, action, times]
}

/// Builds an `apt`/`mise`-style bar like `[===>---]` for `ratio` (0.0..=1.0).
///
/// Filled cells are `=`, the leading edge is `>` while in progress, and the
/// remainder is `-`. `ratio` is clamped so out-of-range values stay valid.
fn progress_bar(ratio: f64, width: usize) -> String {
    let filled = ((ratio * width as f64).round() as i64).clamp(0, width as i64) as usize;
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for i in 0..width {
        let pos = i + 1;
        let cell = if pos < filled {
            '=' // fully behind the leading edge
        } else if pos == filled && filled < width {
            '>' // the moving leading edge
        } else if i < filled {
            '=' // last cell of a completed (full) bar
        } else {
            '-' // not yet reached
        };
        bar.push(cell);
    }
    bar.push(']');
    bar
}

/// Whether an event is a quit request: `q`, `Esc`, or `Ctrl-C`.
fn is_quit(event: &Event) -> bool {
    let Event::Key(key) = event else {
        return false;
    };
    if key.kind != KeyEventKind::Press {
        return false;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barquest_core::{ActionId, ActionTemplate, TargetTemplate, seconds_to_ticks};

    #[test]
    fn progress_bar_empty_is_all_dashes() {
        assert_eq!(progress_bar(0.0, 7), "[-------]");
    }

    #[test]
    fn progress_bar_full_is_all_equals() {
        assert_eq!(progress_bar(1.0, 7), "[=======]");
    }

    #[test]
    fn progress_bar_half_shows_leading_edge() {
        assert_eq!(progress_bar(0.5, 7), "[===>---]");
    }

    #[test]
    fn progress_bar_clamps_out_of_range() {
        assert_eq!(progress_bar(-1.0, 5), "[-----]");
        assert_eq!(progress_bar(2.0, 5), "[=====]");
    }

    #[test]
    fn progress_columns_sum_to_width_and_match_proportions() {
        let cols = progress_columns(80);
        assert_eq!(cols, [16, 16, 8, 24, 8, 8]); // 20/20/10/30/10/10 of 80
        assert_eq!(cols.iter().sum::<usize>(), 80);

        let wide = progress_columns(200);
        assert_eq!(wide.iter().sum::<usize>(), 200);
        assert_eq!(wide[0], 40); // 20% of 200
        assert_eq!(wide[3], 60); // 30% of 200
    }

    #[test]
    fn progress_rows_are_one_per_active_action_and_full_width() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        // Two targets running the same action -> two rows.
        state.assign_action(
            &catalog,
            &TargetId::new("hero"),
            &ActionId::new("forest_exploration"),
        );
        state.assign_action(
            &catalog,
            &TargetId::new("adventurer"),
            &ActionId::new("forest_exploration"),
        );
        state.advance(seconds_to_ticks(5)); // 50%

        // 120 cols gives the 20% action column room for the full label.
        let rows = progress_rows(&catalog, &state, 120);
        assert_eq!(rows.len(), 2, "one row per active quest");
        for row in &rows {
            assert_eq!(row.chars().count(), 120, "row must fill the width");
            assert!(row.is_ascii());
            assert!(row.contains("Forest Exploration"));
            assert!(row.contains("50%"));
        }
        assert!(rows[0].contains("Hero"));
        assert!(rows[1].contains("Adventurer"));
    }

    #[test]
    fn progress_rows_truncate_a_long_action_label() {
        let mut catalog = Catalog::new();
        catalog.register_target(TargetTemplate::new("hero", "Hero"));
        catalog.register_action(ActionTemplate::new(
            "expedition",
            "Extremely Long Dungeon Expedition Name",
            seconds_to_ticks(10),
        ));
        let mut state = GameState::seeded(&catalog);
        state.assign_action(
            &catalog,
            &TargetId::new("hero"),
            &ActionId::new("expedition"),
        );

        let rows = progress_rows(&catalog, &state, 80);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].chars().count(), 80);
        // The label is wider than the 20% action column, so it is cut off.
        assert!(!rows[0].contains("Expedition Name"));
    }

    #[test]
    fn progress_cell_renders_bar_endpoints() {
        assert!(progress_cell(0.0, 24).contains("[--"));
        assert!(progress_cell(0.0, 24).contains("0%"));
        assert!(progress_cell(0.5, 24).contains('>'));
        assert!(progress_cell(0.5, 24).contains("50%"));
        let full = progress_cell(1.0, 24);
        assert!(full.contains("100%"));
        assert!(!full.contains('-'));
    }

    /// Flattens a `render` pass at `w x h` to a single string of cell symbols so
    /// we can assert the screen contains the expected pieces, position-agnostic.
    fn screen_at(
        w: u16,
        h: u16,
        catalog: &Catalog,
        state: &GameState,
        menu: &Menu,
        log: &[String],
    ) -> String {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal
            .draw(|frame| render(frame, catalog, state, menu, log))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    /// Renders at the minimum supported size for the common case (empty log).
    fn rendered(catalog: &Catalog, state: &GameState, menu: &Menu) -> String {
        screen_at(MIN_WIDTH, MIN_HEIGHT, catalog, state, menu, &[])
    }

    #[test]
    fn render_shows_the_running_action_row_and_the_menu() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        // Put the adventurer halfway through forest exploration.
        state.assign_action(
            &catalog,
            &TargetId::new("adventurer"),
            &ActionId::new("forest_exploration"),
        );
        state.advance(seconds_to_ticks(5)); // 50% of the 10s goal

        // Exactly one active action -> exactly one progress row.
        assert_eq!(progress_rows(&catalog, &state, MIN_WIDTH).len(), 1);

        let screen = rendered(&catalog, &state, &Menu::SelectTarget);

        // The running action renders an apt/mise-style bar with its percent. At
        // 80 cols the 20% action column truncates the label, so match a prefix.
        assert!(screen.contains("Adventurer"));
        assert!(screen.contains("Forest Expl"));
        assert!(screen.contains('[') && screen.contains(']'));
        assert!(screen.contains("50%"), "running quest should show 50%");
        // The user-choices region numbers every selectable target; Target is the
        // active column (`>`) while choosing one.
        assert!(screen.contains(">Target:"));
        assert!(screen.contains("1) Hero"));
        assert!(screen.contains("3) Farmer"));
    }

    #[test]
    fn action_menu_marks_the_chosen_target_and_numbers_actions() {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);

        let screen = rendered(
            &catalog,
            &state,
            &Menu::SelectAction {
                target: TargetId::new("hero"),
            },
        );

        // Action becomes the active column; the chosen target points an arrow.
        assert!(screen.contains(">Action:"));
        assert!(screen.contains("1) Hero ----->"));
        assert!(screen.contains("1) Forest Exploration"));
    }

    #[test]
    fn choices_columns_account_for_the_separators() {
        let [t, a, ti] = choices_columns(80);
        assert_eq!(t + a + ti + 2, 80, "three columns plus two '|' fill 80");
        assert_eq!([t, a], [31, 31]); // 40% of (80 - 2)

        let [t, a, ti] = choices_columns(120);
        assert_eq!(t + a + ti + 2, 120);
    }

    #[test]
    fn digit_index_maps_number_keys() {
        assert_eq!(digit_index('1'), Some(0));
        assert_eq!(digit_index('9'), Some(8));
        assert_eq!(digit_index('0'), Some(9));
        assert_eq!(digit_index('a'), None);
        assert_eq!(digit_index(')'), None);
    }

    fn press(c: char) -> Event {
        use crossterm::event::{KeyEvent, KeyModifiers};
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
    }

    #[test]
    fn selecting_a_target_number_opens_its_action_menu() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let mut menu = Menu::SelectTarget;

        // '2' picks the second target (Adventurer).
        handle_key(&press('2'), &catalog, &mut state, &mut menu);
        match &menu {
            Menu::SelectAction { target } => assert_eq!(*target, TargetId::new("adventurer")),
            Menu::SelectTarget => panic!("expected SelectAction"),
        }
    }

    #[test]
    fn selecting_an_action_number_assigns_and_returns() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let mut menu = Menu::SelectAction {
            target: hero.clone(),
        };

        // '1' picks the first unlocked action (Forest Exploration).
        handle_key(&press('1'), &catalog, &mut state, &mut menu);

        assert!(matches!(menu, Menu::SelectTarget), "returns to target menu");
        let target = state.targets.iter().find(|t| t.id == hero).unwrap();
        assert_eq!(target.quests.len(), 1);
        assert_eq!(target.quests[0].action, ActionId::new("forest_exploration"));
    }

    #[test]
    fn out_of_range_and_non_digit_keys_are_ignored() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);

        // Only three targets, so '9' selects nothing and a letter is not a digit.
        for key in ['9', 'x'] {
            let mut menu = Menu::SelectTarget;
            handle_key(&press(key), &catalog, &mut state, &mut menu);
            assert!(
                matches!(menu, Menu::SelectTarget),
                "key {key} changed state"
            );
        }
        assert!(state.targets.iter().all(|t| t.quests.is_empty()));
    }

    #[test]
    fn push_event_formats_completion_and_caps_the_buffer() {
        let catalog = Catalog::builtin();
        let event = GameEvent::QuestCompleted {
            target: TargetId::new("hero"),
            action: ActionId::new("forest_exploration"),
        };

        let mut log = Vec::new();
        push_event(&mut log, &catalog, &event);
        assert_eq!(log, vec!["Hero completed Forest Exploration".to_string()]);

        // The buffer never grows past LOG_CAPACITY; the oldest line drops first.
        let mut full: Vec<String> = (0..LOG_CAPACITY).map(|i| format!("line {i}")).collect();
        push_event(&mut full, &catalog, &event);
        assert_eq!(full.len(), LOG_CAPACITY);
        assert_eq!(full.first().unwrap(), "line 1"); // "line 0" was dropped
        assert_eq!(full.last().unwrap(), "Hero completed Forest Exploration");
    }

    #[test]
    fn log_lines_keep_the_newest_and_truncate() {
        let log: Vec<String> = (0..5).map(|i| format!("event number {i}")).collect();

        // Only the last `height` lines survive, newest last.
        let lines = log_lines(&log, 80, 3);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines.first().unwrap(), "event number 2"); // older scrolled off
        assert_eq!(lines.last().unwrap(), "event number 4"); // newest is last

        // Long lines are cut to the width.
        let wide = log_lines(&log, 5, 3);
        assert!(wide.iter().all(|l| l.chars().count() <= 5));
        assert_eq!(wide.last().unwrap(), "event");
    }

    #[test]
    fn render_shows_log_lines_in_the_log_region() {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);
        let log = vec!["Hero completed Forest Exploration".to_string()];

        let screen = screen_at(
            MIN_WIDTH,
            MIN_HEIGHT,
            &catalog,
            &state,
            &Menu::SelectTarget,
            &log,
        );
        assert!(screen.contains("Hero completed Forest Exploration"));
        assert!(screen.is_ascii());
    }

    #[test]
    fn separator_matches_the_documented_80col_shape() {
        let sep = separator(80);
        assert_eq!(sep.chars().count(), 80);
        assert_eq!(
            sep,
            "+------------------------------------+----+------------------------------------+"
        );
    }

    #[test]
    fn separator_stretches_to_any_width() {
        for w in [80u16, 100, 120, 81] {
            let sep = separator(w);
            assert_eq!(sep.chars().count(), w as usize, "width {w}");
            assert!(sep.starts_with('+') && sep.ends_with('+'), "width {w}");
            assert!(sep.contains("+----+"), "missing center anchor at width {w}");
        }
    }

    #[test]
    fn render_shows_the_five_region_chrome() {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);
        let screen = rendered(&catalog, &state, &Menu::SelectTarget);

        assert!(screen.contains("IDLE BARQUEST"), "missing title");
        assert!(screen.contains("ESC) Quit"), "missing global menu");
        assert!(screen.contains("+----+"), "missing separators");
    }

    #[test]
    fn render_chrome_is_ascii_only() {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);
        let screen = rendered(&catalog, &state, &Menu::SelectTarget);
        assert!(
            screen.is_ascii(),
            "in-game UI must be ASCII (no double-width chars)"
        );
    }

    #[test]
    fn refuses_to_draw_below_minimum_size() {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);

        for (w, h) in [(MIN_WIDTH - 1, MIN_HEIGHT), (MIN_WIDTH, MIN_HEIGHT - 1)] {
            let screen = screen_at(w, h, &catalog, &state, &Menu::SelectTarget, &[]);
            assert!(screen.contains("too small"), "no warning at {w}x{h}");
            assert!(
                !screen.contains("IDLE BARQUEST"),
                "game UI shown at {w}x{h}"
            );
        }
    }
}
