//! Testable front-end state and behavior for the progressive choices flow.

use barquest_core::{
    ActionId, Catalog, GameEvent, GameState, LocationId, RecipeId, ResourceId, SeededRandom,
    TargetId,
};

use crate::input::Input;
use crate::materials;

pub(crate) const TICKS_PER_FRAME: u64 = 100;
const LOG_CAPACITY: usize = 200;
pub(crate) const INVENTORY_PREFIX_WIDTH: u16 = 12;

/// The current stage of progressive selection and its final confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Menu {
    SelectTarget,
    SelectLocation {
        target: TargetId,
    },
    SelectAction {
        target: TargetId,
        location: LocationId,
    },
    ConfirmAction {
        target: TargetId,
        location: LocationId,
        action: ActionId,
    },
    SelectRecipe {
        target: TargetId,
        location: LocationId,
        action: ActionId,
    },
    ConfirmRecipe {
        target: TargetId,
        location: LocationId,
        action: ActionId,
        recipe: RecipeId,
    },
}

pub(crate) struct App {
    pub(crate) catalog: Catalog,
    pub(crate) state: GameState,
    pub(crate) menu: Menu,
    pub(crate) log: Vec<String>,
    pub(crate) material_start: Option<ResourceId>,
    random: SeededRandom,
}

impl App {
    pub(crate) fn new(random_seed: u64) -> Self {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);
        Self {
            catalog,
            state,
            menu: Menu::SelectTarget,
            log: Vec::new(),
            material_start: None,
            random: SeededRandom::new(random_seed),
        }
    }

    /// Applies one translated input and returns whether the game should quit.
    pub(crate) fn update(&mut self, input: Input, material_width: u16) -> bool {
        match input {
            Input::Quit => return true,
            Input::Back => self.back(),
            Input::Select(index) => self.select(index),
            Input::Confirm => self.confirm(),
            Input::PreviousMaterials => self.move_materials(material_width, false),
            Input::NextMaterials => self.move_materials(material_width, true),
            Input::Ignored => {}
        }
        false
    }

    fn move_materials(&mut self, width: u16, forward: bool) {
        let viewport = materials::viewport(
            &self.catalog,
            &self.state,
            self.material_start.as_ref(),
            width.saturating_sub(INVENTORY_PREFIX_WIDTH),
        );
        let destination = if forward {
            viewport.next_start
        } else {
            viewport.previous_start
        };
        if let Some(destination) = destination {
            self.material_start = Some(destination);
        }
    }

    fn select(&mut self, index: usize) {
        match &self.menu {
            Menu::SelectTarget => {
                // Target letters refer to fixed display slots. Busy slots remain
                // visible but cannot be selected.
                if let Some(target) = self
                    .state
                    .targets
                    .get(index)
                    .filter(|target| target.quest.is_none())
                    .map(|target| target.id.clone())
                {
                    self.menu = Menu::SelectLocation { target };
                }
            }
            Menu::SelectLocation { target } => {
                let target = target.clone();
                if let Some(location) = self
                    .state
                    .available_locations(&self.catalog)
                    .nth(index)
                    .cloned()
                {
                    self.menu = Menu::SelectAction { target, location };
                }
            }
            Menu::SelectAction { target, location } => {
                let target = target.clone();
                let location = location.clone();
                let action = self
                    .state
                    .available_actions(&self.catalog, &target, &location)
                    .nth(index)
                    .cloned();
                if let Some(action) = action {
                    if self
                        .state
                        .available_recipes(&self.catalog, &location, &action)
                        .next()
                        .is_some()
                    {
                        self.menu = Menu::SelectRecipe {
                            target,
                            location,
                            action,
                        };
                    } else {
                        self.menu = Menu::ConfirmAction {
                            target,
                            location,
                            action,
                        };
                    }
                }
            }
            Menu::SelectRecipe {
                target,
                location,
                action,
            } => {
                let recipe = self
                    .state
                    .available_recipes(&self.catalog, location, action)
                    .nth(index)
                    .map(|recipe| recipe.id.clone());
                if let Some(recipe) = recipe {
                    self.menu = Menu::ConfirmRecipe {
                        target: target.clone(),
                        location: location.clone(),
                        action: action.clone(),
                        recipe,
                    };
                }
            }
            Menu::ConfirmAction { .. } | Menu::ConfirmRecipe { .. } => {}
        }
    }

    fn confirm(&mut self) {
        match &self.menu {
            Menu::ConfirmAction {
                target,
                location,
                action,
            } => {
                if self
                    .state
                    .assign_action(&self.catalog, target, location, action)
                {
                    self.menu = Menu::SelectTarget;
                }
            }
            Menu::ConfirmRecipe {
                target,
                location,
                action,
                recipe,
            } => {
                if self
                    .state
                    .assign_recipe(&self.catalog, target, location, action, recipe)
                {
                    self.menu = Menu::SelectTarget;
                }
            }
            Menu::SelectTarget
            | Menu::SelectLocation { .. }
            | Menu::SelectAction { .. }
            | Menu::SelectRecipe { .. } => {}
        }
    }

    pub(crate) fn can_confirm(&self) -> bool {
        match &self.menu {
            Menu::ConfirmAction { .. } => true,
            Menu::ConfirmRecipe {
                target,
                location,
                action,
                recipe,
            } => self
                .state
                .can_craft_recipe(&self.catalog, target, location, action, recipe),
            _ => false,
        }
    }

    fn back(&mut self) {
        self.menu = match &self.menu {
            Menu::SelectTarget => return,
            Menu::SelectLocation { .. } => Menu::SelectTarget,
            Menu::SelectAction { target, .. } => Menu::SelectLocation {
                target: target.clone(),
            },
            Menu::ConfirmAction {
                target, location, ..
            } => Menu::SelectAction {
                target: target.clone(),
                location: location.clone(),
            },
            Menu::SelectRecipe {
                target, location, ..
            } => Menu::SelectAction {
                target: target.clone(),
                location: location.clone(),
            },
            Menu::ConfirmRecipe {
                target,
                location,
                action,
                ..
            } => Menu::SelectRecipe {
                target: target.clone(),
                location: location.clone(),
                action: action.clone(),
            },
        };
    }

    pub(crate) fn advance(&mut self, ticks: u64) {
        for event in self.state.advance(&self.catalog, ticks, &mut self.random) {
            push_event(&mut self.log, &self.catalog, &event);
        }
    }
}

fn push_event(log: &mut Vec<String>, catalog: &Catalog, event: &GameEvent) {
    let line = match event {
        GameEvent::QuestCompleted {
            target,
            location,
            action,
            rewards,
        } => {
            let target = catalog
                .target(target)
                .map_or("?", |value| value.label.as_str());
            let location = catalog
                .location(location)
                .map_or("?", |value| value.label.as_str());
            let action = catalog
                .action(action)
                .map_or("?", |value| value.label.as_str());
            let rewards = if rewards.is_empty() {
                "Nothing".to_string()
            } else {
                rewards
                    .iter()
                    .map(|reward| {
                        let resource = catalog
                            .resource(&reward.resource)
                            .map_or("?", |value| value.label.as_str());
                        format!("{resource} x{}", reward.amount)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            format!("{target} completed {action} at {location}: {rewards}")
        }
        GameEvent::CraftCompleted {
            target,
            location,
            recipe,
        } => {
            let target = catalog
                .target(target)
                .map_or("?", |value| value.label.as_str());
            let location = catalog
                .location(location)
                .map_or("?", |value| value.label.as_str());
            let recipe = catalog
                .recipe(recipe)
                .map_or("?", |value| value.label.as_str());
            format!("{target} crafted {recipe} at {location}")
        }
    };
    log.push(line);
    if log.len() > LOG_CAPACITY {
        log.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barquest_core::{ActionId, RecipeId, ResourceId, ResourceStack, Reward, seconds_to_ticks};

    fn acquire_all_materials(app: &mut App) {
        app.state.inventory = app
            .catalog
            .resources()
            .map(|resource| ResourceStack {
                resource: resource.id.clone(),
                amount: 1,
            })
            .collect();
    }

    #[test]
    fn selection_walks_to_action_confirmation_then_enter_assigns() {
        let mut app = App::new(0);
        assert!(!app.update(Input::Select(0), 80));
        assert_eq!(
            app.menu,
            Menu::SelectLocation {
                target: TargetId::new("hero")
            }
        );
        app.update(Input::Select(1), 80);
        assert_eq!(
            app.menu,
            Menu::SelectAction {
                target: TargetId::new("hero"),
                location: LocationId::new("nearby_woods")
            }
        );
        // Nearby Woods exposes Gather then Hunt; `b` chooses Hunt.
        app.update(Input::Select(1), 80);
        assert_eq!(
            app.menu,
            Menu::ConfirmAction {
                target: TargetId::new("hero"),
                location: LocationId::new("nearby_woods"),
                action: ActionId::new("hunt"),
            }
        );
        assert!(app.state.targets[0].quest.is_none());

        app.update(Input::Confirm, 80);
        assert_eq!(app.menu, Menu::SelectTarget);
        let quest = app.state.targets[0].quest.as_ref().unwrap();
        assert_eq!(quest.location, LocationId::new("nearby_woods"));
        assert_eq!(quest.action, ActionId::new("hunt"));
    }

    #[test]
    fn back_moves_exactly_one_stage_and_does_nothing_at_target() {
        let mut app = App::new(0);
        app.update(Input::Back, 80);
        assert_eq!(app.menu, Menu::SelectTarget);

        app.update(Input::Select(0), 80);
        app.update(Input::Select(3), 80);
        app.update(Input::Select(0), 80);
        assert!(matches!(app.menu, Menu::SelectRecipe { .. }));
        app.update(Input::Back, 80);
        assert_eq!(
            app.menu,
            Menu::SelectAction {
                target: TargetId::new("hero"),
                location: LocationId::new("base"),
            }
        );
        app.update(Input::Back, 80);
        assert_eq!(
            app.menu,
            Menu::SelectLocation {
                target: TargetId::new("hero")
            }
        );
        app.update(Input::Back, 80);
        assert_eq!(app.menu, Menu::SelectTarget);
    }

    #[test]
    fn back_returns_from_confirmations_to_their_choice_stage() {
        let mut app = App::new(0);
        app.menu = Menu::ConfirmAction {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
            action: ActionId::new("gather"),
        };
        app.update(Input::Back, 80);
        assert_eq!(
            app.menu,
            Menu::SelectAction {
                target: TargetId::new("hero"),
                location: LocationId::new("first_shore"),
            }
        );

        app.menu = Menu::ConfirmRecipe {
            target: TargetId::new("hero"),
            location: LocationId::new("base"),
            action: ActionId::new("craft"),
            recipe: RecipeId::new("stone_table"),
        };
        app.update(Input::Back, 80);
        assert_eq!(
            app.menu,
            Menu::SelectRecipe {
                target: TargetId::new("hero"),
                location: LocationId::new("base"),
                action: ActionId::new("craft"),
            }
        );
    }

    #[test]
    fn busy_target_fixed_slot_is_ignored_and_freed_after_completion() {
        let mut app = App::new(0);
        app.update(Input::Select(0), 80);
        app.update(Input::Select(0), 80);
        app.update(Input::Select(0), 80);
        app.update(Input::Confirm, 80);
        assert!(app.state.targets[0].quest.is_some());
        app.update(Input::Select(0), 80);
        assert_eq!(app.menu, Menu::SelectTarget);
        app.advance(seconds_to_ticks(10));
        app.update(Input::Select(0), 80);
        assert!(matches!(app.menu, Menu::SelectLocation { .. }));
    }

    #[test]
    fn target_after_busy_slot_keeps_its_original_letter() {
        let mut app = App::new(0);
        let hero = TargetId::new("hero");
        let second = app.state.spawn_target(&app.catalog, &hero).unwrap();
        assert!(app.state.assign_action(
            &app.catalog,
            &hero,
            &LocationId::new("first_shore"),
            &ActionId::new("gather"),
        ));

        app.update(Input::Select(0), 80);
        assert_eq!(app.menu, Menu::SelectTarget, "busy `a` slot is invalid");
        app.update(Input::Select(1), 80);
        assert_eq!(app.menu, Menu::SelectLocation { target: second });
    }

    #[test]
    fn out_of_range_and_ignored_inputs_are_noops() {
        let mut app = App::new(0);
        for input in [Input::Select(8), Input::Ignored] {
            assert!(!app.update(input, 80));
            assert_eq!(app.menu, Menu::SelectTarget);
        }
    }

    #[test]
    fn quit_works_from_every_stage() {
        let mut app = App::new(0);
        assert!(app.update(Input::Quit, 80));
        app.menu = Menu::SelectLocation {
            target: TargetId::new("hero"),
        };
        assert!(app.update(Input::Quit, 80));
        app.menu = Menu::SelectAction {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
        };
        assert!(app.update(Input::Quit, 80));
        app.menu = Menu::SelectRecipe {
            target: TargetId::new("hero"),
            location: LocationId::new("base"),
            action: ActionId::new("craft"),
        };
        assert!(app.update(Input::Quit, 80));
        app.menu = Menu::ConfirmAction {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
            action: ActionId::new("gather"),
        };
        assert!(app.update(Input::Quit, 80));
        app.menu = Menu::ConfirmRecipe {
            target: TargetId::new("hero"),
            location: LocationId::new("base"),
            action: ActionId::new("craft"),
            recipe: RecipeId::new("stone_table"),
        };
        assert!(app.update(Input::Quit, 80));
    }

    #[test]
    fn disabled_recipe_can_be_previewed_but_only_starts_when_affordable() {
        let mut app = App::new(0);
        app.update(Input::Select(0), 80);
        app.update(Input::Select(3), 80);
        app.update(Input::Select(0), 80);
        assert_eq!(
            app.menu,
            Menu::SelectRecipe {
                target: TargetId::new("hero"),
                location: LocationId::new("base"),
                action: ActionId::new("craft"),
            }
        );

        app.update(Input::Select(0), 80);
        assert_eq!(
            app.menu,
            Menu::ConfirmRecipe {
                target: TargetId::new("hero"),
                location: LocationId::new("base"),
                action: ActionId::new("craft"),
                recipe: RecipeId::new("stone_table"),
            }
        );
        assert!(!app.can_confirm());
        app.update(Input::Confirm, 80);
        assert!(matches!(app.menu, Menu::ConfirmRecipe { .. }));
        assert!(app.state.targets[0].quest.is_none());

        app.state.inventory.push(ResourceStack {
            resource: ResourceId::new("pebble"),
            amount: 20,
        });
        assert!(app.can_confirm());
        app.update(Input::Confirm, 80);

        assert_eq!(app.menu, Menu::SelectTarget);
        assert_eq!(app.state.resource_count(&ResourceId::new("pebble")), 0);
        let quest = app.state.targets[0].quest.as_ref().unwrap();
        assert_eq!(quest.recipe, Some(RecipeId::new("stone_table")));
        assert_eq!(quest.progress.goal(), seconds_to_ticks(20));
    }

    #[test]
    fn craft_completion_is_logged_and_builds_the_facility() {
        let mut app = App::new(0);
        app.state.inventory.push(ResourceStack {
            resource: ResourceId::new("pebble"),
            amount: 20,
        });
        assert!(app.state.assign_recipe(
            &app.catalog,
            &TargetId::new("hero"),
            &LocationId::new("base"),
            &ActionId::new("craft"),
            &RecipeId::new("stone_table"),
        ));

        app.advance(seconds_to_ticks(20));

        assert_eq!(app.log, ["Hero crafted Stone Table at Base"]);
        assert!(app.state.has_facility(&RecipeId::new("stone_table")));
    }

    #[test]
    fn material_navigation_is_global_and_moves_one_catalog_item() {
        let mut app = App::new(0);
        acquire_all_materials(&mut app);
        app.menu = Menu::SelectAction {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
        };

        app.update(Input::NextMaterials, 40);
        assert_eq!(app.material_start, Some(ResourceId::new("twig")));
        assert!(matches!(app.menu, Menu::SelectAction { .. }));

        app.update(Input::PreviousMaterials, 40);
        assert_eq!(app.material_start, Some(ResourceId::new("pebble")));
        app.update(Input::PreviousMaterials, 40);
        assert_eq!(app.material_start, Some(ResourceId::new("pebble")));
    }

    #[test]
    fn next_material_is_a_noop_when_every_remaining_item_fits() {
        let mut app = App::new(0);
        acquire_all_materials(&mut app);

        app.update(Input::NextMaterials, 200);

        assert!(app.material_start.is_none());
    }

    #[test]
    fn material_start_id_survives_amount_width_and_acquisition_changes() {
        let mut app = App::new(0);
        app.state.inventory.push(ResourceStack {
            resource: ResourceId::new("twig"),
            amount: 9,
        });
        app.material_start = Some(ResourceId::new("twig"));

        app.state.inventory[0].amount = 10_000;
        app.state.inventory.push(ResourceStack {
            resource: ResourceId::new("pebble"),
            amount: 1,
        });

        assert_eq!(app.material_start, Some(ResourceId::new("twig")));
        assert!(
            materials::viewport(&app.catalog, &app.state, app.material_start.as_ref(), 30,)
                .line
                .contains("Twig: 10000")
        );
    }

    #[test]
    fn completion_log_contains_target_location_action_and_resource() {
        let mut app = App::new(0);
        app.update(Input::Select(0), 80);
        app.update(Input::Select(1), 80);
        app.update(Input::Select(1), 80);
        app.update(Input::Confirm, 80);
        app.advance(seconds_to_ticks(10));
        assert_eq!(
            app.log,
            ["Hero completed Hunt at Nearby Woods: Awful Meat x1".to_string()]
        );
        assert!(app.state.targets[0].quest.is_none());
    }

    #[test]
    fn completion_log_reports_nothing_for_an_empty_reward_list() {
        let catalog = Catalog::builtin();
        let event = GameEvent::QuestCompleted {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
            action: ActionId::new("fish"),
            rewards: vec![],
        };
        let mut log = Vec::new();

        push_event(&mut log, &catalog, &event);

        assert_eq!(log, ["Hero completed Fish at First Shore: Nothing"]);
    }

    #[test]
    fn completion_log_formats_multiple_and_aggregated_rewards() {
        let catalog = Catalog::builtin();
        let event = GameEvent::QuestCompleted {
            target: TargetId::new("hero"),
            location: LocationId::new("nearby_hill"),
            action: ActionId::new("gather"),
            rewards: vec![
                Reward {
                    resource: ResourceId::new("grass"),
                    amount: 1,
                },
                Reward {
                    resource: ResourceId::new("pebble"),
                    amount: 2,
                },
            ],
        };
        let mut log = Vec::new();

        push_event(&mut log, &catalog, &event);

        assert_eq!(
            log,
            ["Hero completed Gather at Nearby Hill: Grass x1, Pebble x2"]
        );
    }

    #[test]
    fn event_log_capacity_drops_the_oldest_line() {
        let catalog = Catalog::builtin();
        let event = GameEvent::QuestCompleted {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
            action: ActionId::new("gather"),
            rewards: vec![Reward {
                resource: ResourceId::new("pebble"),
                amount: 1,
            }],
        };
        let mut log: Vec<String> = (0..LOG_CAPACITY).map(|i| format!("line {i}")).collect();
        push_event(&mut log, &catalog, &event);
        assert_eq!(log.len(), LOG_CAPACITY);
        assert_eq!(log.first().unwrap(), "line 1");
        assert_eq!(
            log.last().unwrap(),
            "Hero completed Gather at First Shore: Pebble x1"
        );
    }
}
