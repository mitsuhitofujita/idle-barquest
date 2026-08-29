//! The Progress region: one row per active action, each laid out as the six
//! proportional columns from `docs/design/terminal-ui-layout.md`, with an `apt`/`mise`
//! style text progress bar.

use barquest_core::{Catalog, GameState};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::Paragraph;

use super::fit;

/// Draws one row per **active action** (not per target): a facility running two
/// productions shows two rows. Each row is the six proportional columns from
/// `docs/design/terminal-ui-layout.md`. Idle targets contribute no row.
pub(super) fn render_progress(frame: &mut Frame, area: Rect, catalog: &Catalog, state: &GameState) {
    let rows = progress_rows(catalog, state, area.width).join("\n");
    frame.render_widget(Paragraph::new(rows).alignment(Alignment::Left), area);
}

/// Splits a full row width into the six progress columns from
/// `docs/design/terminal-ui-layout.md` — Target 20% / Action 20% / Times 10% /
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

#[cfg(test)]
mod tests {
    use super::*;
    use barquest_core::{ActionId, ActionTemplate, TargetId, TargetTemplate, seconds_to_ticks};

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
}
