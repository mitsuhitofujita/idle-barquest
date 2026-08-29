//! The front-end's live state and the behaviour layer that mutates it.
//!
//! [`App`] owns the [`Catalog`], the live [`GameState`], the bottom [`Menu`],
//! and the information log. [`App::update`] applies one translated [`Input`] and
//! [`App::advance`] steps the simulation, turning completion events into log
//! lines. Neither reads the wall clock nor the terminal, so both can be driven
//! directly from tests (see `docs/design/tui-test-policy.md`).

use barquest_core::{Catalog, GameEvent, GameState, TargetId};

use crate::input::Input;

/// Game-time advanced per frame. The TUI frame is 100 ms (see `FRAME` in
/// `main.rs`), which is 100 ticks at 1000 ticks/s.
pub(crate) const TICKS_PER_FRAME: u64 = 100;
/// How many information-log lines to retain; older lines are dropped. Only the
/// last few (one per visible row) are ever shown, so this just bounds memory.
const LOG_CAPACITY: usize = 200;

/// Which menu the bottom of the screen is currently showing.
pub(crate) enum Menu {
    /// Choose which target to command next.
    SelectTarget,
    /// Choose an action for the already-chosen target instance.
    SelectAction { target: TargetId },
}

/// All mutable front-end state: the content catalog, the live world, the current
/// menu, and the rolling information log.
pub(crate) struct App {
    pub(crate) catalog: Catalog,
    pub(crate) state: GameState,
    pub(crate) menu: Menu,
    pub(crate) log: Vec<String>,
}

impl App {
    /// Builds the starting world from the built-in catalog.
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

    /// Applies one translated input. Returns `true` when the player asked to
    /// quit; the caller then tears down the terminal and exits.
    pub(crate) fn update(&mut self, input: Input) -> bool {
        match input {
            Input::Quit => return true,
            Input::Select(idx) => self.select(idx),
            Input::Ignored => {}
        }
        false
    }

    /// Pick a target by number, then an action by number, which (re)starts that
    /// target's quest. The digit indexes the numbered list shown in the active
    /// choices column. Out-of-range indices are ignored.
    fn select(&mut self, idx: usize) {
        match &self.menu {
            Menu::SelectTarget => {
                if let Some(target) = self.state.targets.get(idx).map(|inst| inst.id.clone()) {
                    self.menu = Menu::SelectAction { target };
                }
            }
            Menu::SelectAction { target } => {
                // Clone the chosen instance id out so we can reassign `menu` below.
                let target = target.clone();
                if let Some(action) = self.state.unlocked_actions.get(idx).cloned() {
                    self.state.assign_action(&self.catalog, &target, &action);
                    self.menu = Menu::SelectTarget;
                }
            }
        }
    }

    /// Advances the world by `ticks` and logs any completion events.
    pub(crate) fn advance(&mut self, ticks: u64) {
        for event in self.state.advance(ticks) {
            push_event(&mut self.log, &self.catalog, &event);
        }
    }
}

/// Appends one ASCII log line for a game event, resolving ids to labels via the
/// catalog, and caps the buffer at [`LOG_CAPACITY`] by dropping the oldest line.
fn push_event(log: &mut Vec<String>, catalog: &Catalog, event: &GameEvent) {
    let line = match event {
        GameEvent::QuestCompleted { target, action } => {
            let target = catalog.target(target).map_or("?", |t| t.label.as_str());
            let action = catalog.action(action).map_or("?", |a| a.label.as_str());
            format!("{target} completed {action}")
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
    fn selecting_a_target_number_opens_its_action_menu() {
        let mut app = App::new();

        // Select(1) picks the second target (Adventurer).
        assert!(!app.update(Input::Select(1)));
        match &app.menu {
            Menu::SelectAction { target } => assert_eq!(*target, TargetId::new("adventurer")),
            Menu::SelectTarget => panic!("expected SelectAction"),
        }
    }

    #[test]
    fn selecting_an_action_number_assigns_and_returns() {
        let mut app = App::new();
        let hero = TargetId::new("hero");
        app.menu = Menu::SelectAction {
            target: hero.clone(),
        };

        // Select(0) picks the first unlocked action (Forest Exploration).
        app.update(Input::Select(0));

        assert!(
            matches!(app.menu, Menu::SelectTarget),
            "returns to target menu"
        );
        let target = app.state.targets.iter().find(|t| t.id == hero).unwrap();
        assert_eq!(target.quests.len(), 1);
        assert_eq!(target.quests[0].action, ActionId::new("forest_exploration"));
    }

    #[test]
    fn out_of_range_and_ignored_inputs_change_nothing() {
        let mut app = App::new();

        // Only three targets, so index 8 selects nothing; Ignored is a no-op.
        for input in [Input::Select(8), Input::Ignored] {
            assert!(!app.update(input));
            assert!(matches!(app.menu, Menu::SelectTarget));
        }
        assert!(app.state.targets.iter().all(|t| t.quests.is_empty()));
    }

    #[test]
    fn quit_input_reports_quit() {
        let mut app = App::new();
        assert!(app.update(Input::Quit), "Quit must ask the loop to stop");
    }

    #[test]
    fn advancing_past_the_goal_logs_completion_and_clears_the_row() {
        let mut app = App::new();
        // Assign Forest Exploration (10s goal) to the hero, then run it out.
        app.menu = Menu::SelectAction {
            target: TargetId::new("hero"),
        };
        app.update(Input::Select(0));
        assert_eq!(app.state.active_quests().count(), 1, "quest is running");

        app.advance(seconds_to_ticks(10)); // reach the goal in one step
        assert_eq!(
            app.state.active_quests().count(),
            0,
            "completed quest is removed from the progress region"
        );
        assert_eq!(
            app.log,
            vec!["Hero completed Forest Exploration".to_string()]
        );
    }

    #[test]
    fn push_event_formats_completion_and_caps_the_buffer() {
        let catalog = Catalog::builtin();
        let event = GameEvent::QuestCompleted {
            target: TargetId::new("hero"),
            action: ActionId::new("forest_exploration"),
        };

        let mut log = Vec::new();
        push_event(&mut log, &catalog, &event);
        assert_eq!(log, vec!["Hero completed Forest Exploration".to_string()]);

        // The buffer never grows past LOG_CAPACITY; the oldest line drops first.
        let mut full: Vec<String> = (0..LOG_CAPACITY).map(|i| format!("line {i}")).collect();
        push_event(&mut full, &catalog, &event);
        assert_eq!(full.len(), LOG_CAPACITY);
        assert_eq!(full.first().unwrap(), "line 1"); // "line 0" was dropped
        assert_eq!(full.last().unwrap(), "Hero completed Forest Exploration");
    }
}
