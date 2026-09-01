//! Width-aware projection and navigation for the acquired-material viewport.

use barquest_core::{Catalog, GameState, ResourceId};

const ARROW_COLUMNS: usize = 4;
const SEPARATOR: &str = " | ";

/// One rendered material row and the adjacent starts it can navigate to.
pub(crate) struct MaterialViewport {
    pub(crate) line: String,
    pub(crate) previous_start: Option<ResourceId>,
    pub(crate) next_start: Option<ResourceId>,
}

/// Builds the material line at `width`, keeping the requested first ResourceId
/// when it is still acquired and otherwise falling back to the first item.
pub(crate) fn viewport(
    catalog: &Catalog,
    state: &GameState,
    requested_start: Option<&ResourceId>,
    width: u16,
) -> MaterialViewport {
    let acquired: Vec<_> = state.acquired_resources(catalog).collect();
    if acquired.is_empty() {
        return MaterialViewport {
            line: String::new(),
            previous_start: None,
            next_start: None,
        };
    }

    let start = requested_start
        .and_then(|requested| {
            acquired
                .iter()
                .position(|(resource, _)| &resource.id == requested)
        })
        .unwrap_or(0);
    let content_width = usize::from(width).saturating_sub(ARROW_COLUMNS);
    let mut body = String::new();
    let mut visible = 0;

    for (resource, stack) in &acquired[start..] {
        let item = format!("{}: {}", resource.label, stack.amount);
        let required = item.chars().count()
            + if visible == 0 {
                0
            } else {
                SEPARATOR.chars().count()
            };
        if required <= content_width.saturating_sub(body.chars().count()) {
            if visible > 0 {
                body.push_str(SEPARATOR);
            }
            body.push_str(&item);
            visible += 1;
        } else if visible == 0 {
            body = truncate_item(&resource.label, stack.amount, content_width);
            visible = 1;
            break;
        } else {
            break;
        }
    }

    let previous_start = start
        .checked_sub(1)
        .map(|index| acquired[index].0.id.clone());
    let next_start = (start + visible < acquired.len()).then(|| acquired[start + 1].0.id.clone());
    let left = if previous_start.is_some() { '<' } else { ' ' };
    let right = if next_start.is_some() { '>' } else { ' ' };
    let body = fit(&body, content_width);
    let line = if width >= ARROW_COLUMNS as u16 {
        format!("{left} {body} {right}")
    } else {
        fit(&body, usize::from(width))
    };

    MaterialViewport {
        line,
        previous_start,
        next_start,
    }
}

/// Only the label is shortened; the quantity suffix remains intact at every
/// supported terminal width.
fn truncate_item(label: &str, amount: u64, width: usize) -> String {
    let suffix = format!(": {amount}");
    let suffix_width = suffix.chars().count();
    if suffix_width >= width {
        return suffix
            .chars()
            .rev()
            .take(width)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
    }
    let label_width = width - suffix_width;
    format!(
        "{}{suffix}",
        label.chars().take(label_width).collect::<String>()
    )
}

fn fit(text: &str, width: usize) -> String {
    let mut result: String = text.chars().take(width).collect();
    result.extend(std::iter::repeat_n(
        ' ',
        width.saturating_sub(result.chars().count()),
    ));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use barquest_core::{ResourceStack, ResourceTemplate};

    fn state_with(stacks: &[(&str, u64)]) -> (Catalog, GameState) {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        state.inventory = stacks
            .iter()
            .map(|(resource, amount)| ResourceStack {
                resource: ResourceId::new(*resource),
                amount: *amount,
            })
            .collect();
        (catalog, state)
    }

    #[test]
    fn no_acquired_materials_produce_an_empty_row() {
        let (catalog, state) = state_with(&[]);
        let result = viewport(&catalog, &state, None, 80);
        assert!(result.line.is_empty());
        assert!(result.previous_start.is_none());
        assert!(result.next_start.is_none());
    }

    #[test]
    fn materials_follow_catalog_order_and_include_zero_stacks() {
        let (catalog, state) = state_with(&[("vine", 0), ("pebble", 34), ("twig", 10)]);
        let result = viewport(&catalog, &state, None, 80);
        assert!(result.line.contains("Pebble: 34 | Twig: 10 | Vine: 0"));
    }

    #[test]
    fn arrows_only_show_for_hidden_materials_and_keep_body_column_fixed() {
        let (catalog, state) = state_with(&[("pebble", 1), ("twig", 2), ("grass", 3), ("vine", 4)]);
        let first = viewport(&catalog, &state, None, 24);
        let second = viewport(&catalog, &state, first.next_start.as_ref(), 24);

        assert_eq!(first.line.chars().next(), Some(' '));
        assert_eq!(first.line.chars().nth(2), Some('P'));
        assert_eq!(first.line.chars().last(), Some('>'));
        assert_eq!(second.line.chars().next(), Some('<'));
        assert_eq!(second.line.chars().nth(2), Some('T'));
        assert_eq!(first.line.chars().count(), 24);
        assert_eq!(second.line.chars().count(), 24);
    }

    #[test]
    fn terminal_width_and_quantity_digits_change_visible_count_not_start() {
        let (catalog, mut state) =
            state_with(&[("pebble", 1), ("twig", 2), ("grass", 3), ("vine", 4)]);
        let start = ResourceId::new("pebble");

        let narrow = viewport(&catalog, &state, Some(&start), 24);
        let wide = viewport(&catalog, &state, Some(&start), 34);
        assert!(narrow.line.contains("Pebble: 1 | Twig: 2"));
        assert!(!narrow.line.contains("Grass: 3"));
        assert!(wide.line.contains("Pebble: 1 | Twig: 2 | Grass: 3"));

        state.inventory[0].amount = 1_000_000;
        let more_digits = viewport(&catalog, &state, Some(&start), 24);
        assert!(more_digits.line.contains("Pebble: 1000000"));
        assert!(!more_digits.line.contains("Twig: 2"));
        assert_eq!(more_digits.next_start, Some(ResourceId::new("twig")));
    }

    #[test]
    fn oversized_single_label_is_truncated_but_quantity_is_preserved() {
        let mut catalog = Catalog::new();
        catalog.register_resource(ResourceTemplate::new(
            "long",
            "Extraordinarily Long Material Label",
        ));
        let mut state = GameState::new("awakening_shore");
        state.inventory.push(ResourceStack {
            resource: ResourceId::new("long"),
            amount: 123,
        });

        let result = viewport(&catalog, &state, None, 20);

        assert_eq!(result.line.chars().count(), 20);
        assert!(result.line.contains(": 123"));
        assert!(!result.line.contains("Extraordinarily"));
    }
}
