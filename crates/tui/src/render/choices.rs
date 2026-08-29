//! The User Choices region: the three `Target: | Action: | Times:` columns from
//! `docs/design/terminal-ui-layout.md`. Entries are numbered for digit selection.

use barquest_core::{Catalog, GameState};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use super::fit;
use crate::app::Menu;

/// Draws the user-choices region: the three `Target: | Action: | Times:` columns
/// from `docs/design/terminal-ui-layout.md`. Entries are numbered; the player presses a
/// digit to pick within the active column (`>` marks which one), and the chosen
/// target carries a ` ----->` arrow into the action column.
pub(super) fn render_choices(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices_columns_account_for_the_separators() {
        let [t, a, ti] = choices_columns(80);
        assert_eq!(t + a + ti + 2, 80, "three columns plus two '|' fill 80");
        assert_eq!([t, a], [31, 31]); // 40% of (80 - 2)

        let [t, a, ti] = choices_columns(120);
        assert_eq!(t + a + ti + 2, 120);
    }
}
