//! Progressive `Target -> Location -> Action -> Recipe` choices rendering.

use barquest_core::{ActionId, Catalog, GameState, LocationId, RecipeTemplate, TargetId};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use super::fit;
use crate::app::Menu;

pub(super) fn render_choices(
    frame: &mut Frame,
    area: Rect,
    catalog: &Catalog,
    state: &GameState,
    menu: &Menu,
) {
    let height = area.height as usize;
    let (chosen_target, chosen_location, chosen_action) = match menu {
        Menu::SelectTarget => (None, None, None),
        Menu::SelectLocation { target } => (Some(target), None, None),
        Menu::SelectAction { target, location } => (Some(target), Some(location), None),
        Menu::SelectRecipe {
            target,
            location,
            action,
        } => (Some(target), Some(location), Some(action)),
    };

    let target_active = matches!(menu, Menu::SelectTarget);
    let mut targets = vec![format!("|{}", header("Target:", target_active))];
    for (index, instance) in state.targets.iter().enumerate() {
        let label = catalog
            .target(&instance.template_id)
            .map_or("?", |target| target.label.as_str());
        let key = if target_active {
            if instance.quest.is_some() {
                "- ".to_string()
            } else {
                format!("{})", choice_key(index))
            }
        } else {
            "  ".to_string()
        };
        targets.push(format!("| {key} {label}"));
    }
    let target_row = chosen_target.and_then(|id| target_row(state, id));

    let mut locations = Vec::new();
    if chosen_target.is_some() {
        locations.push(header(
            "Location:",
            matches!(menu, Menu::SelectLocation { .. }),
        ));
        for (index, id) in state.available_locations(catalog).enumerate() {
            let label = catalog
                .location(id)
                .map_or("?", |location| location.label.as_str());
            let key = if matches!(menu, Menu::SelectLocation { .. }) {
                format!("{})", choice_key(index))
            } else {
                "  ".to_string()
            };
            locations.push(format!(" {key} {label}"));
        }
    }
    let location_row = chosen_location.and_then(|id| location_row(state, catalog, id));

    let mut actions = Vec::new();
    if let (Some(target), Some(location)) = (chosen_target, chosen_location) {
        let action_active = matches!(menu, Menu::SelectAction { .. });
        actions.push(header("Action:", action_active));
        for (index, id) in state
            .available_actions(catalog, target, location)
            .enumerate()
        {
            let label = catalog
                .action(id)
                .map_or("?", |action| action.label.as_str());
            let key = if action_active {
                format!("{})", choice_key(index))
            } else {
                "  ".to_string()
            };
            actions.push(format!(" {key} {label}"));
        }
    }
    let action_row = match (chosen_target, chosen_location, chosen_action) {
        (Some(target), Some(location), Some(action)) => {
            action_row(state, catalog, target, location, action)
        }
        _ => None,
    };

    let mut recipes = Vec::new();
    if let Menu::SelectRecipe {
        target,
        location,
        action,
    } = menu
    {
        recipes.push(header("Recipe:", true));
        for (index, recipe) in state
            .available_recipes(catalog, location, action)
            .enumerate()
        {
            let craftable = state.can_craft_recipe(catalog, target, location, action, &recipe.id);
            let key = if craftable {
                format!("{})", choice_key(index))
            } else {
                "- ".to_string()
            };
            let reason = recipe_unavailable_reason(catalog, state, recipe);
            recipes.push(if reason.is_empty() {
                format!(" {key} {}", recipe.label)
            } else {
                format!(" {key} {reason}: {}", recipe.label)
            });
        }
    }

    let rows = if chosen_target.is_none() {
        single_column(height, area.width as usize, &targets)
    } else if chosen_location.is_none() {
        two_columns(
            height,
            area.width as usize,
            &targets,
            &locations,
            target_row,
        )
    } else if chosen_action.is_none() {
        three_columns(
            height,
            area.width as usize,
            &targets,
            &locations,
            &actions,
            target_row,
            location_row,
        )
    } else {
        four_columns(
            height,
            area.width as usize,
            &targets,
            &locations,
            &actions,
            &recipes,
            target_row,
            location_row,
            action_row,
        )
    };

    frame.render_widget(Paragraph::new(rows.join("\n")), area);
}

#[allow(clippy::too_many_arguments)]
fn four_columns(
    height: usize,
    width: usize,
    first: &[String],
    second: &[String],
    third: &[String],
    fourth: &[String],
    selected_first: Option<usize>,
    selected_second: Option<usize>,
    selected_third: Option<usize>,
) -> Vec<String> {
    let [first_width, second_width, third_width, fourth_width] =
        split_columns(width, &[first, second, third], 20);
    (0..height)
        .map(|row| {
            format!(
                "{}|{}|{}|{}",
                selected_cell(
                    first.get(row).map_or("|", String::as_str),
                    first_width,
                    selected_first == Some(row),
                ),
                selected_cell(
                    second.get(row).map_or("", String::as_str),
                    second_width,
                    selected_second == Some(row),
                ),
                selected_cell(
                    third.get(row).map_or("", String::as_str),
                    third_width,
                    selected_third == Some(row),
                ),
                fit(fourth.get(row).map_or("", String::as_str), fourth_width),
            )
        })
        .collect()
}

fn single_column(height: usize, width: usize, column: &[String]) -> Vec<String> {
    (0..height)
        .map(|row| fit(column.get(row).map_or("|", String::as_str), width))
        .collect()
}

fn two_columns(
    height: usize,
    width: usize,
    left: &[String],
    right: &[String],
    selected_left: Option<usize>,
) -> Vec<String> {
    let [left_width, right_width] = split_columns(width, &[left], 20);
    (0..height)
        .map(|row| {
            format!(
                "{}|{}",
                selected_cell(
                    left.get(row).map_or("|", String::as_str),
                    left_width,
                    selected_left == Some(row),
                ),
                fit(right.get(row).map_or("", String::as_str), right_width),
            )
        })
        .collect()
}

fn three_columns(
    height: usize,
    width: usize,
    first: &[String],
    second: &[String],
    third: &[String],
    selected_first: Option<usize>,
    selected_second: Option<usize>,
) -> Vec<String> {
    let [first_width, second_width, third_width] = split_columns(width, &[first, second], 20);
    (0..height)
        .map(|row| {
            format!(
                "{}|{}|{}",
                selected_cell(
                    first.get(row).map_or("|", String::as_str),
                    first_width,
                    selected_first == Some(row),
                ),
                selected_cell(
                    second.get(row).map_or("", String::as_str),
                    second_width,
                    selected_second == Some(row),
                ),
                fit(third.get(row).map_or("", String::as_str), third_width),
            )
        })
        .collect()
}

/// Natural widths for completed columns, preserving `minimum_active` cells for
/// the current stage. The final returned width receives all remaining space.
fn split_columns<const N: usize>(
    width: usize,
    completed: &[&[String]],
    minimum_active: usize,
) -> [usize; N] {
    let separators = completed.len();
    let available_completed = width.saturating_sub(separators + minimum_active);
    let naturals: Vec<usize> = completed
        .iter()
        .map(|column| {
            column
                .iter()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or_default()
                .saturating_add(2)
        })
        .collect();
    let natural_total: usize = naturals.iter().sum();
    let mut result = [0; N];
    let mut used = 0;
    for (index, natural) in naturals.into_iter().enumerate() {
        let value = if natural_total <= available_completed {
            natural
        } else {
            available_completed * natural / natural_total
        };
        result[index] = value;
        used += value;
    }
    result[N - 1] = width.saturating_sub(separators + used);
    result
}

fn selected_cell(text: &str, width: usize, selected: bool) -> String {
    let mut cell = fit(text, width);
    if selected && width > 0 {
        cell.pop();
        cell.push('<');
    }
    cell
}

fn target_row(state: &GameState, target: &TargetId) -> Option<usize> {
    state
        .targets
        .iter()
        .position(|instance| &instance.id == target)
        .map(|index| index + 1)
}

fn location_row(state: &GameState, catalog: &Catalog, location: &LocationId) -> Option<usize> {
    state
        .available_locations(catalog)
        .position(|id| id == location)
        .map(|index| index + 1)
}

fn action_row(
    state: &GameState,
    catalog: &Catalog,
    target: &TargetId,
    location: &LocationId,
    action: &ActionId,
) -> Option<usize> {
    state
        .available_actions(catalog, target, location)
        .position(|id| id == action)
        .map(|index| index + 1)
}

fn recipe_unavailable_reason(
    catalog: &Catalog,
    state: &GameState,
    recipe: &RecipeTemplate,
) -> String {
    if let Some(required) = recipe
        .required_facilities
        .iter()
        .find(|facility| !state.has_facility(facility))
    {
        let label = catalog
            .recipe(required)
            .map_or("?", |facility| facility.label.as_str());
        return format!("needs {label}");
    }
    if let Some(ingredient) = recipe
        .ingredients
        .iter()
        .find(|ingredient| state.resource_count(&ingredient.resource) < ingredient.amount)
    {
        let label = catalog
            .resource(&ingredient.resource)
            .map_or("?", |resource| resource.label.as_str());
        let held = state.resource_count(&ingredient.resource);
        return format!("needs {label} {held}/{}", ingredient.amount);
    }
    String::new()
}

fn choice_key(index: usize) -> char {
    u32::try_from(index)
        .ok()
        .and_then(|index| char::from_u32('a' as u32 + index))
        .unwrap_or('?')
}

fn header(name: &str, active: bool) -> String {
    format!("{} {name}", if active { '>' } else { ' ' })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_marker_touches_separator() {
        assert_eq!(selected_cell("|    Hero", 14, true), "|    Hero    <");
    }

    #[test]
    fn split_preserves_active_space_and_fills_width() {
        let target = vec!["|  Target:".to_string(), "|    Hero".to_string()];
        let location = vec!["  Location:".to_string(), "    Nearby Woods".to_string()];
        let widths: [usize; 3] = split_columns(80, &[&target, &location], 20);
        assert_eq!(widths.iter().sum::<usize>() + 2, 80);
        assert!(widths[2] >= 20);
    }
}
