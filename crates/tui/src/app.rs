//! Testable front-end state and behavior for the progressive choices flow.

use barquest_core::{
    Catalog, GameEvent, GameState, LocationId, ResourceId, SeededRandom, TargetId,
};

use crate::input::Input;
use crate::materials;

pub(crate) const TICKS_PER_FRAME: u64 = 100;
const LOG_CAPACITY: usize = 200;

/// The current stage of `Target -> Location -> Action` selection.
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
            width,
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
                if let Some(action) = action
                    && self
                        .state
                        .assign_action(&self.catalog, &target, &location, &action)
                {
                    self.menu = Menu::SelectTarget;
                }
            }
        }
    }

    fn back(&mut self) {
        self.menu = match &self.menu {
            Menu::SelectTarget => return,
            Menu::SelectLocation { .. } => Menu::SelectTarget,
            Menu::SelectAction { target, .. } => Menu::SelectLocation {
                target: target.clone(),
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
    };
    log.push(line);
    if log.len() > LOG_CAPACITY {
        log.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barquest_core::{ActionId, ResourceId, ResourceStack, Reward, seconds_to_ticks};

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
    fn selection_walks_target_location_action_then_assigns() {
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
        app.update(Input::Select(0), 80);
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
    fn busy_target_fixed_slot_is_ignored_and_freed_after_completion() {
        let mut app = App::new(0);
        app.update(Input::Select(0), 80);
        app.update(Input::Select(0), 80);
        app.update(Input::Select(0), 80);
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
