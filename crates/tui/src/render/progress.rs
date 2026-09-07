//! Four-column active-task progress rendering.

use barquest_core::{Catalog, GameState};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::Paragraph;

use super::fit;

pub(super) fn render_progress(frame: &mut Frame, area: Rect, catalog: &Catalog, state: &GameState) {
    let rows = progress_rows(catalog, state, area.width).join("\n");
    frame.render_widget(Paragraph::new(rows).alignment(Alignment::Left), area);
}

/// Target 20% / Location 20% / Action 20% / Progress Bar 40%.
fn progress_columns(width: u16) -> [usize; 4] {
    let total = width as usize;
    let first = total * 20 / 100;
    let second = total * 20 / 100;
    let third = total * 20 / 100;
    [first, second, third, total - first - second - third]
}

fn progress_rows(catalog: &Catalog, state: &GameState, width: u16) -> Vec<String> {
    let cols = progress_columns(width);
    state
        .active_quests()
        .map(|(target, quest)| {
            let target_label = catalog
                .target(&target.template_id)
                .map_or("?", |template| template.label.as_str());
            let location_label = catalog
                .location(&quest.location)
                .map_or("?", |template| template.label.as_str());
            let action_label = quest
                .recipe
                .as_ref()
                .and_then(|recipe| catalog.recipe(recipe))
                .map(|recipe| recipe.label.as_str())
                .unwrap_or_else(|| {
                    catalog
                        .action(&quest.action)
                        .map_or("?", |template| template.label.as_str())
                });
            [
                column(target_label, cols[0]),
                column(location_label, cols[1]),
                column(action_label, cols[2]),
                column(&progress_cell(quest.progress.ratio(), cols[3]), cols[3]),
            ]
            .concat()
        })
        .collect()
}

fn column(text: &str, width: usize) -> String {
    format!(" {}", fit(text, width.saturating_sub(1)))
}

fn progress_cell(ratio: f64, width: usize) -> String {
    let pct = (ratio * 100.0).round() as u16;
    let suffix = format!(" {pct:>3}%");
    let cells = width.saturating_sub(1 + 2 + suffix.chars().count());
    if cells == 0 {
        return suffix.trim_start().to_string();
    }
    format!("{}{suffix}", progress_bar(ratio, cells))
}

fn progress_bar(ratio: f64, width: usize) -> String {
    let filled = ((ratio * width as f64).round() as i64).clamp(0, width as i64) as usize;
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for index in 0..width {
        let position = index + 1;
        let cell = if position < filled {
            '='
        } else if position == filled && filled < width {
            '>'
        } else if index < filled {
            '='
        } else {
            ' '
        };
        bar.push(cell);
    }
    bar.push(']');
    bar
}

#[cfg(test)]
mod tests {
    use super::*;
    use barquest_core::{
        ActionId, LocationId, RecipeId, ResourceId, ResourceStack, SeededRandom, TargetId,
        seconds_to_ticks,
    };

    #[test]
    fn progress_bar_has_expected_endpoints_and_midpoint() {
        assert_eq!(progress_bar(0.0, 7), "[       ]");
        assert_eq!(progress_bar(0.5, 7), "[===>   ]");
        assert_eq!(progress_bar(1.0, 7), "[=======]");
    }

    #[test]
    fn progress_columns_match_proposal_and_fill_width() {
        assert_eq!(progress_columns(80), [16, 16, 16, 32]);
        assert_eq!(progress_columns(121).iter().sum::<usize>(), 121);
    }

    #[test]
    fn progress_row_contains_all_task_dimensions() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        state.assign_action(
            &catalog,
            &TargetId::new("hero"),
            &LocationId::new("nearby_woods"),
            &ActionId::new("hunt"),
        );
        state.advance(&catalog, seconds_to_ticks(5), &mut SeededRandom::new(0));
        let rows = progress_rows(&catalog, &state, 80);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].chars().count(), 80);
        assert!(rows[0].contains("Hero"));
        assert!(rows[0].contains("Nearby Woods"));
        assert!(rows[0].contains("Hunt"));
        assert!(rows[0].contains("50%"));
    }

    #[test]
    fn crafting_progress_uses_the_recipe_name_and_duration() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        state.inventory.push(ResourceStack {
            resource: ResourceId::new("pebble"),
            amount: 20,
        });
        assert!(state.assign_recipe(
            &catalog,
            &TargetId::new("hero"),
            &LocationId::new("base"),
            &ActionId::new("craft"),
            &RecipeId::new("stone_table"),
        ));
        state.advance(&catalog, seconds_to_ticks(10), &mut SeededRandom::new(0));

        let rows = progress_rows(&catalog, &state, 80);
        assert!(rows[0].contains("Stone Table"));
        assert!(rows[0].contains("50%"));
    }

    #[test]
    fn progress_cell_keeps_percent_at_supported_width() {
        for ratio in [0.0, 0.5, 1.0] {
            assert!(progress_cell(ratio, 32).contains('%'));
        }
    }
}
