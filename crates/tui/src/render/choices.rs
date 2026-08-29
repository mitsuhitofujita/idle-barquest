//! The progressive User Choices region. It shows only `Target` at first, then
//! adds an `Action` column after a target has been selected.

use barquest_core::{Catalog, GameState};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use super::fit;
use crate::app::Menu;

/// Draws the user-choices region in the two states shown by the discovery:
/// Target selection is one full-width column; Action selection adds a
/// content-sized Target column and gives the remainder to Action. Only the
/// active column displays positional letter keys.
pub(super) fn render_choices(
    frame: &mut Frame,
    area: Rect,
    catalog: &Catalog,
    state: &GameState,
    menu: &Menu,
) {
    let height = area.height as usize;
    let selecting_target = matches!(menu, Menu::SelectTarget);
    let chosen = match menu {
        Menu::SelectAction { target } => Some(target),
        Menu::SelectTarget => None,
    };

    // The leading `|`, the two-character selection slot, and the item labels
    // deliberately reproduce the discovery's character positions exactly.
    let mut target_col = vec![format!("|{}", header("Target:", selecting_target))];
    for (i, inst) in state.targets.iter().enumerate() {
        let label = catalog
            .target(&inst.template_id)
            .map(|t| t.label.as_str())
            .unwrap_or("?");
        let row = if selecting_target {
            format!("| {}) {label}", choice_key(i))
        } else {
            format!("|    {label}")
        };
        target_col.push(row);
    }
    let selected_row = chosen.and_then(|target| {
        state
            .targets
            .iter()
            .position(|instance| &instance.id == target)
            .map(|index| index + 1)
    });

    let mut action_col = Vec::new();
    if let Some(target) = chosen {
        action_col.push(header("Action:", true));
        for (i, id) in state.available_actions(catalog, target).enumerate() {
            let label = catalog.action(id).map(|a| a.label.as_str()).unwrap_or("?");
            action_col.push(format!(" {}) {label}", choice_key(i)));
        }
    }

    let rows: Vec<String> = if selecting_target {
        (0..height)
            .map(|row| {
                fit(
                    target_col.get(row).map_or("|", String::as_str),
                    area.width as usize,
                )
            })
            .collect()
    } else {
        let [target_width, action_width] = action_columns(area.width, &target_col);
        (0..height)
            .map(|row| {
                format!(
                    "{}|{}",
                    target_cell(
                        target_col.get(row).map_or("|", String::as_str),
                        target_width,
                        selected_row == Some(row),
                    ),
                    fit(action_col.get(row).map_or("", String::as_str), action_width),
                )
            })
            .collect()
    };

    frame.render_widget(Paragraph::new(rows.join("\n")), area);
}

/// Fits one Target cell and attaches the selection marker to the separator by
/// replacing the cell's final padding character with `<`.
fn target_cell(text: &str, width: usize, selected: bool) -> String {
    let mut cell = fit(text, width);
    if selected && width > 0 {
        cell.pop();
        cell.push('<');
    }
    cell
}

/// Positional menu key for one zero-based entry. The choices region currently
/// shows at most six entries, so this yields `a` through `f` in practice.
fn choice_key(index: usize) -> char {
    u32::try_from(index)
        .ok()
        .and_then(|index| char::from_u32('a' as u32 + index))
        .unwrap_or('?')
}

/// A choices-column header with the discovery's 2-char selection slot: `> `
/// when active and two spaces when inactive.
fn header(name: &str, active: bool) -> String {
    let marker = if active { '>' } else { ' ' };
    format!("{marker} {name}")
}

/// Sizes the two Action-selection columns from the rendered Target content.
/// Two trailing positions separate the widest Target label from `|`; the final
/// one can hold `<`. Action receives
/// the rest, with 20 columns protected when a very long Target label appears.
fn action_columns(width: u16, target_lines: &[String]) -> [usize; 2] {
    const MIN_ACTION_WIDTH: usize = 20;

    let inner = (width as usize).saturating_sub(1); // one `|`
    let natural_target = target_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
        .saturating_add(2);
    let max_target = inner.saturating_sub(MIN_ACTION_WIDTH);
    let target = natural_target.min(max_target);
    [target, inner.saturating_sub(target)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_columns_follow_content_and_fill_the_row() {
        let targets = vec!["|  Target:".to_string(), "|    Adventurer".to_string()];
        let [target, action] = action_columns(80, &targets);

        assert_eq!(target, "|    Adventurer".chars().count() + 2);
        assert_eq!(target + action + 1, 80);
    }

    #[test]
    fn target_marker_occupies_the_cell_before_the_separator() {
        assert_eq!(target_cell("|    Hero", 17, true), "|    Hero       <");
        assert_eq!(target_cell("|    Farmer", 17, false), "|    Farmer      ");
    }

    #[test]
    fn action_columns_protect_action_space_from_long_targets() {
        let targets = vec!["x".repeat(100)];
        let [target, action] = action_columns(80, &targets);

        assert_eq!(action, 20);
        assert_eq!(target + action + 1, 80);
    }
}
