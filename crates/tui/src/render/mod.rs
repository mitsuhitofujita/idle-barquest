//! Rendering: the pure projection from [`App`] state onto a terminal buffer.
//!
//! This module owns the fixed five-region stack from
//! `docs/design/terminal-ui-layout.md` (Title / Progress / User Choices / Information
//! Log / Global Menu) and the ASCII chrome separating the lower regions. The two larger
//! regions live in submodules: [`progress`] and [`choices`]. Renderers never
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
/// Fixed heights (rows) of the title, user-choices, and global-menu regions.
/// The three separators below the progress region are one row each; the
/// information log takes whatever vertical space is left.
const TITLE_H: u16 = 3;
const CHOICES_H: u16 = 10;
const MENU_H: u16 = 1;

/// Fixed-width title artwork. Every line is 38 single-width ASCII characters.
const TITLE: &str = concat!(
    r".@~\::::::::::::::::::::::::::::::/@~.",
    "\n",
    r"(  {        IDLE BARQUEST         }  )",
    "\n",
    r"'@~/::::::::::::::::::::::::::::::\~@'",
);

/// Lays out the screen as the fixed five-region stack from
/// `docs/design/terminal-ui-layout.md`: Title / Progress / User Choices / Information
/// Log / Global Menu. The title artwork provides its own lower boundary; ASCII
/// separator rows divide the remaining regions. Below the minimum terminal size
/// it draws only a warning and leaves the game UI hidden.
pub(crate) fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area);
        return;
    }

    // One progress row per active action; keep a 1-row reserve when idle so the
    // region (and the separators below it) stay put.
    let progress_h = (app.state.active_quests().count() as u16).max(1);

    let [
        title,
        progress,
        sep1,
        choices,
        sep2,
        log_area,
        sep3,
        menu_area,
    ] = Layout::vertical([
        Constraint::Length(TITLE_H),
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
    use barquest_core::{ActionId, TargetId, seconds_to_ticks};

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
        // Put the adventurer halfway through forest exploration.
        app.state.assign_action(
            &app.catalog,
            &TargetId::new("adventurer"),
            &ActionId::new("forest_exploration"),
        );
        app.state.advance(seconds_to_ticks(5)); // 50% of the 10s goal

        let screen = rendered(&app);

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
        let mut app = App::new();
        app.menu = Menu::SelectAction {
            target: TargetId::new("hero"),
        };

        let screen = rendered(&app);

        // Action becomes the active column; the chosen target points an arrow.
        assert!(screen.contains(">Action:"));
        assert!(screen.contains("1) Hero ----->"));
        assert!(screen.contains("1) Forest Exploration"));
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
        let mut app = App::new();
        app.log = vec!["Hero completed Forest Exploration".to_string()];

        let screen = rendered(&app);
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
        let app = App::new();
        let rows = screen_rows_at(MIN_WIDTH, MIN_HEIGHT, &app);
        let screen = rows.concat();

        let side_padding = " ".repeat(21);
        for (row, title_line) in rows.iter().take(TITLE_H as usize).zip(TITLE.lines()) {
            assert_eq!(row, &format!("{side_padding}{title_line}{side_padding}"));
        }
        assert!(
            !rows[TITLE_H as usize].contains("+----+"),
            "title should lead directly into progress"
        );
        assert!(
            rows[TITLE_H as usize + 1].contains("+----+"),
            "progress separator should remain"
        );
        assert!(screen.contains("ESC) Quit"), "missing global menu");
        assert!(screen.contains("+----+"), "missing separators");
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
