//! Testable front-end state and behavior for the progressive choices flow.

use barquest_core::{Catalog, GameEvent, GameState, LocationId, TargetId};

use crate::input::Input;

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
}

impl App {
    pub(crate) fn new() -> Self {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);
        Self {
            catalog,
            state,
            menu: Menu::SelectTarget,
            log: Vec::new(),
        }
    }

    /// Applies one translated input and returns whether the game should quit.
    pub(crate) fn update(&mut self, input: Input) -> bool {
        match input {
            Input::Quit => return true,
            Input::Back => self.back(),
            Input::Select(index) => self.select(index),
            Input::Ignored => {}
        }
        false
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
        for event in self.state.advance(ticks) {
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
            format!("{target} completed {action} at {location}")
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
    use barquest_core::{ActionId, seconds_to_ticks};

    #[test]
    fn selection_walks_target_location_action_then_assigns() {
        let mut app = App::new();
        assert!(!app.update(Input::Select(0)));
        assert_eq!(
            app.menu,
            Menu::SelectLocation {
                target: TargetId::new("hero")
            }
        );
        app.update(Input::Select(1));
        assert_eq!(
            app.menu,
            Menu::SelectAction {
                target: TargetId::new("hero"),
                location: LocationId::new("nearby_woods")
            }
        );
        // Nearby Woods exposes Gather then Hunt; `b` chooses Hunt.
        app.update(Input::Select(1));
        assert_eq!(app.menu, Menu::SelectTarget);
        let quest = app.state.targets[0].quest.as_ref().unwrap();
        assert_eq!(quest.location, LocationId::new("nearby_woods"));
        assert_eq!(quest.action, ActionId::new("hunt"));
    }

    #[test]
    fn back_moves_exactly_one_stage_and_does_nothing_at_target() {
        let mut app = App::new();
        app.update(Input::Back);
        assert_eq!(app.menu, Menu::SelectTarget);
        app.update(Input::Select(0));
        app.update(Input::Select(0));
        app.update(Input::Back);
        assert_eq!(
            app.menu,
            Menu::SelectLocation {
                target: TargetId::new("hero")
            }
        );
        app.update(Input::Back);
        assert_eq!(app.menu, Menu::SelectTarget);
    }

    #[test]
    fn busy_target_fixed_slot_is_ignored_and_freed_after_completion() {
        let mut app = App::new();
        app.update(Input::Select(0));
        app.update(Input::Select(0));
        app.update(Input::Select(0));
        assert!(app.state.targets[0].quest.is_some());
        app.update(Input::Select(0));
        assert_eq!(app.menu, Menu::SelectTarget);
        app.advance(seconds_to_ticks(10));
        app.update(Input::Select(0));
        assert!(matches!(app.menu, Menu::SelectLocation { .. }));
    }

    #[test]
    fn target_after_busy_slot_keeps_its_original_letter() {
        let mut app = App::new();
        let hero = TargetId::new("hero");
        let second = app.state.spawn_target(&app.catalog, &hero).unwrap();
        assert!(app.state.assign_action(
            &app.catalog,
            &hero,
            &LocationId::new("first_shore"),
            &ActionId::new("gather"),
        ));

        app.update(Input::Select(0));
        assert_eq!(app.menu, Menu::SelectTarget, "busy `a` slot is invalid");
        app.update(Input::Select(1));
        assert_eq!(app.menu, Menu::SelectLocation { target: second });
    }

    #[test]
    fn out_of_range_and_ignored_inputs_are_noops() {
        let mut app = App::new();
        for input in [Input::Select(8), Input::Ignored] {
            assert!(!app.update(input));
            assert_eq!(app.menu, Menu::SelectTarget);
        }
    }

    #[test]
    fn quit_works_from_every_stage() {
        let mut app = App::new();
        assert!(app.update(Input::Quit));
        app.menu = Menu::SelectLocation {
            target: TargetId::new("hero"),
        };
        assert!(app.update(Input::Quit));
        app.menu = Menu::SelectAction {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
        };
        assert!(app.update(Input::Quit));
    }

    #[test]
    fn completion_log_contains_target_location_and_action() {
        let mut app = App::new();
        app.update(Input::Select(0));
        app.update(Input::Select(1));
        app.update(Input::Select(1));
        app.advance(seconds_to_ticks(10));
        assert_eq!(app.log, ["Hero completed Hunt at Nearby Woods".to_string()]);
        assert!(app.state.targets[0].quest.is_none());
    }

    #[test]
    fn event_log_capacity_drops_the_oldest_line() {
        let catalog = Catalog::builtin();
        let event = GameEvent::QuestCompleted {
            target: TargetId::new("hero"),
            location: LocationId::new("first_shore"),
            action: ActionId::new("gather"),
        };
        let mut log: Vec<String> = (0..LOG_CAPACITY).map(|i| format!("line {i}")).collect();
        push_event(&mut log, &catalog, &event);
        assert_eq!(log.len(), LOG_CAPACITY);
        assert_eq!(log.first().unwrap(), "line 1");
        assert_eq!(log.last().unwrap(), "Hero completed Gather at First Shore");
    }
}
