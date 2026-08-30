//! Rendering: the pure projection from [`App`] state onto a terminal buffer.
//!
//! This module owns the fixed five-region stack from
//! `docs/design/terminal-ui-layout.md` (Title / Information Log / User Choices /
//! Progress / Global Menu) and the ASCII chrome separating the lower regions. The
//! two fixed data regions live in submodules: [`progress`] and [`choices`]. Renderers never
//! mutate game state or read input, so they can be exercised with a
//! `TestBackend` (see `docs/design/tui-test-policy.md`).

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::widgets::Paragraph;

use crate::app::App;

mod choices;
mod progress;

/// Minimum supported terminal size. Below this the game shows a warning and
/// refuses to draw its UI (see `docs/design/terminal-ui-layout.md`).
const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;
/// Fixed heights (rows) of the title, user-choices, progress, and global-menu
/// regions. The three separators below the information log are one row each;
/// the information log takes whatever vertical space is left.
const TITLE_H: u16 = 3;
const CHOICES_H: u16 = 7;
const PROGRESS_H: u16 = 6;
const MENU_H: u16 = 1;
/// The first log-region row is always blank to separate the title from content.
const LOG_GAP_H: u16 = 1;

/// Fixed-width title artwork. Every line is 38 single-width ASCII characters.
const TITLE: &str = concat!(
    r".@~\::::::::::::::::::::::::::::::/@~.",
    "\n",
    r"(  {        IDLE BARQUEST         }  )",
    "\n",
    r"'@~/::::::::::::::::::::::::::::::\~@'",
);

/// Lays out the screen as the fixed five-region stack from
/// `docs/design/terminal-ui-layout.md`: Title / Information Log / User Choices /
/// Progress / Global Menu. The first log row leaves a gap below the title; ASCII
/// separator rows divide every later region. Below the minimum terminal size it
/// draws only a warning and leaves the game UI hidden.
pub(crate) fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area);
        return;
    }

    let [
        title,
        log_area,
        sep1,
        choices,
        sep2,
        progress,
        sep3,
        menu_area,
    ] = Layout::vertical([
        Constraint::Length(TITLE_H),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(CHOICES_H),
        Constraint::Length(1),
        Constraint::Length(PROGRESS_H),
        Constraint::Length(1),
        Constraint::Length(MENU_H),
    ])
    .areas(area);

    render_title(frame, title);
    progress::render_progress(frame, progress, &app.catalog, &app.state);
    choices::render_choices(frame, choices, &app.catalog, &app.state, &app.menu);
    render_log(frame, log_area, &app.log);
    render_menu(frame, menu_area);
    for sep in [sep1, sep2, sep3] {
        render_separator(frame, sep);
    }
}

/// Draws the fixed-width, three-row game title centered in its region. It never
/// reflects game state or stretches with terminal width.
fn render_title(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new(TITLE).alignment(Alignment::Center), area);
}

/// Draws the always-visible global menu: commands valid from any UI state.
fn render_menu(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new(" ESC) Quit"), area);
}

/// Draws the information log with a permanent blank first row. Game events are
/// bottom-aligned below it, newest at the bottom and older above, with anything
/// beyond the available content height scrolled off the top.
fn render_log(frame: &mut Frame, area: Rect, log: &[String]) {
    let lines = log_lines(log, area.width, area.height).join("\n");
    frame.render_widget(Paragraph::new(lines), area);
}

/// Exactly `height` rows for the log region. The first row is reserved as the
/// title gap. The remaining rows are padded above the visible entries so the
/// last log line is always on the region's bottom row. Entries are truncated to
/// `width`, and older entries beyond the available rows are dropped.
fn log_lines(log: &[String], width: u16, height: u16) -> Vec<String> {
    let width = width as usize;
    let height = height as usize;
    let take = height.saturating_sub(LOG_GAP_H as usize);
    let start = log.len().saturating_sub(take);
    let mut rows = vec![String::new(); height.saturating_sub(log.len() - start)];
    rows.extend(
        log[start..]
            .iter()
            .map(|line| line.chars().take(width).collect()),
    );
    rows
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
/// reproducing the example in `docs/design/terminal-ui-layout.md`.
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

/// Left-aligns `text` to exactly `width` chars: padded with spaces if short,
/// truncated (by char) if long. Shared by the [`progress`] and [`choices`]
/// region renderers.
fn fit(text: &str, width: usize) -> String {
    let mut out: String = text.chars().take(width).collect();
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Menu};
    use barquest_core::{ActionId, LocationId, TargetId, seconds_to_ticks};

    /// Renders at `w x h` and returns one full-width string per terminal row.
    fn screen_rows_at(w: u16, h: u16, app: &App) -> Vec<String> {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .chunks(w as usize)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect())
            .collect()
    }

    /// Flattens a `render` pass so tests can assert content position-agnostically.
    fn screen_at(w: u16, h: u16, app: &App) -> String {
        screen_rows_at(w, h, app).concat()
    }

    /// Renders at the minimum supported size.
    fn rendered(app: &App) -> String {
        screen_at(MIN_WIDTH, MIN_HEIGHT, app)
    }

    #[test]
    fn render_shows_the_running_action_row_and_the_menu() {
        let mut app = App::new();
        // Put the hero halfway through hunting in the woods.
        app.state.assign_action(
            &app.catalog,
            &TargetId::new("hero"),
            &LocationId::new("nearby_woods"),
            &ActionId::new("hunt"),
        );
        app.state.spawn_target(&app.catalog, &TargetId::new("hero"));
        app.state.advance(seconds_to_ticks(5)); // 50% of the 10s goal

        let screen = rendered(&app);

        assert!(screen.contains("Hero"));
        assert!(screen.contains("Nearby Woods"));
        assert!(screen.contains("Hunt"));
        assert!(screen.contains('[') && screen.contains(']'));
        assert!(screen.contains("50%"), "running quest should show 50%");
        // The user-choices region letters every selectable target; Target is the
        // active column (`>`) while choosing one.
        assert!(screen.contains("|> Target:"));
        assert!(screen.contains("-  Hero"));
        assert!(screen.contains("b) Hero"));
        assert!(!screen.contains("Action:"));
        assert!(!screen.contains("Times:"));

        let rows = screen_rows_at(MIN_WIDTH, MIN_HEIGHT, &app);
        assert!(rows[8].starts_with("|> Target:"));
        assert!(rows[9].starts_with("| -  Hero"));
        assert!(rows[10].starts_with("| b) Hero"));
        assert!(
            rows[8..15]
                .iter()
                .all(|row| row.chars().filter(|&c| c == '|').count() == 1),
            "Target selection should render only its left boundary"
        );
    }

    #[test]
    fn location_menu_marks_target_and_letters_locations() {
        let mut app = App::new();
        app.menu = Menu::SelectLocation {
            target: TargetId::new("hero"),
        };

        let screen = rendered(&app);

        assert!(screen.contains("> Location:"));
        assert!(screen.contains("Hero  <|"));
        assert!(!screen.contains("a) Hero"));
        assert!(screen.contains("a) First Shore"));
        assert!(screen.contains("c) Nearby Hill"));

        let rows = screen_rows_at(MIN_WIDTH, MIN_HEIGHT, &app);
        assert!(
            rows[8..15]
                .iter()
                .all(|row| row.chars().filter(|&c| c == '|').count() == 2),
            "Location selection should render a left boundary and one separator"
        );
    }

    #[test]
    fn action_menu_marks_target_and_location_and_filters_actions() {
        let mut app = App::new();
        app.menu = Menu::SelectAction {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
        };
        let screen = rendered(&app);
        assert!(screen.contains("> Action:"));
        assert!(screen.contains("Hero  <|"));
        assert!(screen.contains("First Shore  <|"));
        assert!(screen.contains("a) Gather"));
        assert!(screen.contains("b) Fish"));
        assert!(!screen.contains("c) Hunt"));
        let rows = screen_rows_at(MIN_WIDTH, MIN_HEIGHT, &app);
        assert!(
            rows[8..15]
                .iter()
                .all(|row| row.chars().filter(|&c| c == '|').count() == 3)
        );
    }

    #[test]
    fn log_lines_keep_the_newest_and_truncate() {
        let log: Vec<String> = (0..5).map(|i| format!("event number {i}")).collect();

        // One row stays blank below the title, so only the last three events fit.
        let lines = log_lines(&log, 80, 4);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines.first().unwrap(), "");
        assert_eq!(lines[1], "event number 2"); // older scrolled off
        assert_eq!(lines.last().unwrap(), "event number 4"); // newest is last

        // Long lines are cut to the width.
        let wide = log_lines(&log, 5, 4);
        assert!(wide.iter().all(|l| l.chars().count() <= 5));
        assert_eq!(wide.last().unwrap(), "event");
    }

    #[test]
    fn log_lines_bottom_align_short_logs_below_the_title_gap() {
        let lines = log_lines(&["latest".to_string()], 80, 5);

        assert_eq!(lines, vec!["", "", "", "", "latest"]);
    }

    #[test]
    fn render_shows_log_lines_in_the_log_region() {
        let mut app = App::new();
        app.log = vec!["Hero completed Gather at First Shore".to_string()];

        let screen = rendered(&app);
        assert!(screen.contains("Hero completed Gather at First Shore"));
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
        let app = App::new();
        let rows = screen_rows_at(MIN_WIDTH, MIN_HEIGHT, &app);
        let screen = rows.concat();

        let side_padding = " ".repeat(21);
        for (row, title_line) in rows.iter().take(TITLE_H as usize).zip(TITLE.lines()) {
            assert_eq!(row, &format!("{side_padding}{title_line}{side_padding}"));
        }
        assert!(
            !rows[TITLE_H as usize].contains("+----+"),
            "title should lead directly into the blank log row"
        );
        assert!(
            rows[7].contains("+----+"),
            "log separator should follow the four-row minimum log region"
        );
        assert!(rows[3].trim().is_empty(), "first log row should be blank");
        assert!(rows[15].contains("+----+"), "choices separator moved");
        assert!(rows[22].contains("+----+"), "progress separator moved");
        assert!(screen.contains("ESC) Quit"), "missing global menu");
        assert!(screen.contains("+----+"), "missing separators");
    }

    #[test]
    fn extra_terminal_height_is_assigned_only_to_the_log() {
        let mut app = App::new();
        app.log = vec!["latest event".to_string()];

        let rows = screen_rows_at(MIN_WIDTH, MIN_HEIGHT + 5, &app);

        assert!(rows[3].trim().is_empty(), "title gap must remain one row");
        assert_eq!(rows[11].trim(), "latest event");
        assert!(rows[12].contains("+----+"), "log should receive five rows");
        assert!(rows[20].contains("+----+"), "choices must stay seven rows");
        assert!(rows[27].contains("+----+"), "progress must stay six rows");
        assert!(rows[28].contains("ESC) Quit"), "menu must remain last");
    }

    #[test]
    fn render_chrome_is_ascii_only() {
        let app = App::new();
        let screen = rendered(&app);
        assert!(
            screen.is_ascii(),
            "in-game UI must be ASCII (no double-width chars)"
        );
    }

    #[test]
    fn refuses_to_draw_below_minimum_size() {
        let app = App::new();

        for (w, h) in [(MIN_WIDTH - 1, MIN_HEIGHT), (MIN_WIDTH, MIN_HEIGHT - 1)] {
            let screen = screen_at(w, h, &app);
            assert!(screen.contains("too small"), "no warning at {w}x{h}");
            assert!(
                !screen.contains("IDLE BARQUEST"),
                "game UI shown at {w}x{h}"
            );
        }
    }
}
