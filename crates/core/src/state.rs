//! The live game world: [`GameState`] and the [`TargetInstance`]s / [`Quest`]s
//! it holds, plus the [`GameEvent`]s that [`advance`](GameState::advance)
//! emits.
//!
//! Kept separate from the [`Catalog`] (content): the state references templates
//! by id, owns no borrows, stays cheaply `Clone`, and can gain a `serde` derive
//! later without dragging the content pool in.

use crate::catalog::Catalog;
use crate::id::{ActionId, TargetId};
use crate::time::Progress;

/// A running action assigned to a target instance. A quest exists only while it
/// is in progress; [`GameState::advance`] removes it the moment it completes and
/// reports that as a [`GameEvent::QuestCompleted`].
#[derive(Debug, Clone)]
pub struct Quest {
    /// Which action is running; resolve its label/duration via the
    /// [`Catalog`].
    pub action: ActionId,
    /// How far the action has progressed.
    pub progress: Progress,
}

/// Something that happened during an [`advance`](GameState::advance) step. The
/// front-end turns these into log lines; ids are resolved to labels via the
/// [`Catalog`]. More variants (rewards, discoveries, depletion) land later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    /// A target finished an action this step; the quest has been removed.
    QuestCompleted {
        /// Instance that was running the action.
        target: TargetId,
        /// Action that just completed.
        action: ActionId,
    },
}

/// One live target in the world: its own instance id, the template it was
/// spawned from, and the actions it is currently running.
///
/// `id` is unique per instance while `template_id` says what kind it is; for
/// today's one-of-each world they happen to be equal. Label lookups must
/// always go through `template_id`. A dedicated `InstanceId` type is the natural
/// upgrade once duplicates become common.
///
/// A target may run several actions at once — a facility, for example, can keep
/// multiple productions going — so `quests` is a list with at most one entry per
/// action (see [`GameState::assign_action`]).
#[derive(Debug, Clone)]
pub struct TargetInstance {
    /// Unique id for this specific target instance.
    pub id: TargetId,
    /// The template this instance was spawned from.
    pub template_id: TargetId,
    /// The actions currently running on this target, one quest each.
    pub quests: Vec<Quest>,
}

/// The live game world: which target instances exist, which actions are
/// unlocked, and each target's running quest.
///
/// Kept separate from the [`Catalog`] (content) so that this — the mutable save
/// data — references templates by id, owns no borrows, stays cheaply `Clone`,
/// and can gain a `serde` derive later without dragging the content pool in.
#[derive(Debug, Clone, Default)]
pub struct GameState {
    /// Every live target, in display order.
    pub targets: Vec<TargetInstance>,
    /// Actions the player may assign, in menu order.
    pub unlocked_actions: Vec<ActionId>,
}

impl GameState {
    /// An empty world with no targets and nothing unlocked.
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds the starting world from a catalog: one instance per target
    /// template and every action unlocked. Reproduces the previous launch state
    /// (three idle targets, one action) for the built-in catalog.
    pub fn seeded(catalog: &Catalog) -> Self {
        let mut state = Self::new();
        for template in catalog.targets() {
            state.targets.push(TargetInstance {
                id: template.id.clone(),
                template_id: template.id.clone(),
                quests: Vec::new(),
            });
        }
        for template in catalog.actions() {
            state.unlocked_actions.push(template.id.clone());
        }
        state
    }

    /// Spawns a new live target from a template, returning its instance id, or
    /// `None` if the template is unknown to `catalog`. The instance id is the
    /// template id when free, else a `"<id>#N"` suffix so duplicates stay
    /// distinct.
    pub fn spawn_target(&mut self, catalog: &Catalog, template: &TargetId) -> Option<TargetId> {
        catalog.target(template)?;
        let id = self.unique_instance_id(template);
        self.targets.push(TargetInstance {
            id: id.clone(),
            template_id: template.clone(),
            quests: Vec::new(),
        });
        Some(id)
    }

    /// Unlocks an action so it appears in the action menu. Idempotent: returns
    /// `true` while the action is known to `catalog` (whether newly added or
    /// already unlocked) and `false` for unknown content.
    pub fn unlock_action(&mut self, catalog: &Catalog, action: &ActionId) -> bool {
        if catalog.action(action).is_none() {
            return false;
        }
        if !self.unlocked_actions.contains(action) {
            self.unlocked_actions.push(action.clone());
        }
        true
    }

    /// Iterates the unlocked actions supported by one target instance, keeping
    /// the global unlock order used by the menu.
    pub fn available_actions<'a>(
        &'a self,
        catalog: &'a Catalog,
        instance: &TargetId,
    ) -> impl Iterator<Item = &'a ActionId> {
        let target = self.targets.iter().find(|target| &target.id == instance);
        let template = target.and_then(|target| catalog.target(&target.template_id));
        self.unlocked_actions
            .iter()
            .filter(move |action| template.is_some_and(|target| target.supports(action)))
    }

    /// Assigns (or restarts) an action on a target instance, seeding a fresh
    /// [`Progress`] from the action template's `goal_ticks`. Returns `false` if
    /// the action or the instance id is unknown, the action is locked, or the
    /// target kind does not support it.
    ///
    /// A target may run several actions at once, but never the same action
    /// twice: if it is already running, its progress is reset (a restart);
    /// otherwise a new quest is appended.
    pub fn assign_action(
        &mut self,
        catalog: &Catalog,
        instance: &TargetId,
        action: &ActionId,
    ) -> bool {
        let Some(action_template) = catalog.action(action) else {
            return false;
        };
        if !self.unlocked_actions.contains(action) {
            return false;
        }
        let Some(target_index) = self
            .targets
            .iter()
            .position(|target| &target.id == instance)
        else {
            return false;
        };
        let Some(target_template) = catalog.target(&self.targets[target_index].template_id) else {
            return false;
        };
        if !target_template.supports(action) {
            return false;
        }

        let goal = action_template.goal_ticks;
        let target = &mut self.targets[target_index];
        match target.quests.iter_mut().find(|q| &q.action == action) {
            Some(quest) => quest.progress = Progress::new(goal),
            None => target.quests.push(Quest {
                action: action.clone(),
                progress: Progress::new(goal),
            }),
        }
        true
    }

    /// Advances every running quest by `ticks` (the loop step) and returns the
    /// events that fired. A quest that reaches its goal this step is reported as
    /// a [`GameEvent::QuestCompleted`] and removed, so finished work leaves the
    /// progress region and surfaces in the log instead.
    pub fn advance(&mut self, ticks: u64) -> Vec<GameEvent> {
        let mut events = Vec::new();
        for target in &mut self.targets {
            for quest in &mut target.quests {
                quest.progress.advance(ticks);
                if quest.progress.is_complete() {
                    events.push(GameEvent::QuestCompleted {
                        target: target.id.clone(),
                        action: quest.action.clone(),
                    });
                }
            }
            target.quests.retain(|quest| !quest.progress.is_complete());
        }
        events
    }

    /// Iterates every running quest paired with the target running it, in
    /// target-then-assignment order. The front-end renders one progress row per
    /// item. Completed quests are not included — [`advance`](Self::advance)
    /// removes them as they finish.
    pub fn active_quests(&self) -> impl Iterator<Item = (&TargetInstance, &Quest)> {
        self.targets
            .iter()
            .flat_map(|target| target.quests.iter().map(move |quest| (target, quest)))
    }

    /// Picks an unused instance id for a new target of `template`: the template
    /// id itself when free, else the first free `"<id>#N"` (N starting at 2).
    fn unique_instance_id(&self, template: &TargetId) -> TargetId {
        if !self.targets.iter().any(|t| &t.id == template) {
            return template.clone();
        }
        let mut n = 2;
        loop {
            let candidate = TargetId::new(format!("{}#{n}", template.as_str()));
            if !self.targets.iter().any(|t| t.id == candidate) {
                return candidate;
            }
            n += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{ActionTemplate, TargetTemplate};
    use crate::time::seconds_to_ticks;

    #[test]
    fn seeded_state_matches_builtin() {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);

        let template_ids: Vec<&str> = state
            .targets
            .iter()
            .map(|t| t.template_id.as_str())
            .collect();
        assert_eq!(template_ids, ["hero", "adventurer", "farmer"]);
        assert!(state.targets.iter().all(|t| t.quests.is_empty()));
        assert_eq!(
            state.unlocked_actions,
            vec![ActionId::new("forest_exploration")]
        );
    }

    #[test]
    fn spawn_target_adds_instance() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let before = state.targets.len();

        // Unknown template is rejected and leaves the world unchanged.
        assert!(
            state
                .spawn_target(&catalog, &TargetId::new("dragon"))
                .is_none()
        );
        assert_eq!(state.targets.len(), before);

        // A second adventurer gets a distinct instance id but the same template.
        let id = state
            .spawn_target(&catalog, &TargetId::new("adventurer"))
            .unwrap();
        assert_eq!(state.targets.len(), before + 1);
        assert_ne!(id, TargetId::new("adventurer"));
        assert_eq!(
            state.targets.last().unwrap().template_id,
            TargetId::new("adventurer")
        );
    }

    #[test]
    fn duplicate_spawns_get_incrementing_instance_ids() {
        // The seeded world already holds one `adventurer`, so further spawns must
        // walk the `#N` suffix upward without collisions.
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let adventurer = TargetId::new("adventurer");

        let second = state.spawn_target(&catalog, &adventurer).unwrap();
        let third = state.spawn_target(&catalog, &adventurer).unwrap();
        let fourth = state.spawn_target(&catalog, &adventurer).unwrap();

        assert_eq!(second, TargetId::new("adventurer#2"));
        assert_eq!(third, TargetId::new("adventurer#3"));
        assert_eq!(fourth, TargetId::new("adventurer#4"));

        // All four (the seeded one plus three spawns) share the template id but
        // carry distinct instance ids.
        let adventurers: Vec<&TargetInstance> = state
            .targets
            .iter()
            .filter(|t| t.template_id == adventurer)
            .collect();
        assert_eq!(adventurers.len(), 4);
        let mut ids: Vec<&str> = adventurers.iter().map(|t| t.id.as_str()).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "instance ids must be unique");
    }

    #[test]
    fn unlock_action_adds_once() {
        let catalog = Catalog::builtin();
        let mut state = GameState::new();
        let forest = ActionId::new("forest_exploration");

        assert!(state.unlock_action(&catalog, &forest));
        assert_eq!(state.unlocked_actions, vec![forest.clone()]);
        // Idempotent: unlocking again does not duplicate.
        assert!(state.unlock_action(&catalog, &forest));
        assert_eq!(state.unlocked_actions, vec![forest]);
        // Unknown action is rejected.
        assert!(!state.unlock_action(&catalog, &ActionId::new("fishing")));
        assert_eq!(state.unlocked_actions.len(), 1);
    }

    #[test]
    fn available_and_assignable_actions_require_unlock_and_target_support() {
        let mut catalog = Catalog::new();
        catalog
            .register_target(TargetTemplate::new("hero", "Hero").with_action("forest_exploration"));
        catalog.register_target(TargetTemplate::new("farm", "Farm").with_action("farming"));
        catalog.register_action(ActionTemplate::new(
            "forest_exploration",
            "Forest Exploration",
            seconds_to_ticks(10),
        ));
        catalog.register_action(ActionTemplate::new(
            "farming",
            "Farming",
            seconds_to_ticks(10),
        ));
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let farm = TargetId::new("farm");
        let forest = ActionId::new("forest_exploration");
        let farming = ActionId::new("farming");

        state.unlocked_actions.retain(|action| action == &forest);
        assert_eq!(
            state.available_actions(&catalog, &hero).collect::<Vec<_>>(),
            vec![&forest]
        );
        assert!(state.available_actions(&catalog, &farm).next().is_none());
        assert!(!state.assign_action(&catalog, &farm, &farming));

        assert!(state.unlock_action(&catalog, &farming));
        assert_eq!(
            state.available_actions(&catalog, &farm).collect::<Vec<_>>(),
            vec![&farming]
        );
        assert!(!state.assign_action(&catalog, &hero, &farming));
        assert!(state.assign_action(&catalog, &farm, &farming));
    }

    #[test]
    fn assign_action_seeds_progress() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let forest = ActionId::new("forest_exploration");

        assert!(state.assign_action(&catalog, &hero, &forest));
        let target = state.targets.iter().find(|t| t.id == hero).unwrap();
        assert_eq!(target.quests.len(), 1);
        let quest = &target.quests[0];
        assert_eq!(quest.action, forest);
        assert_eq!(quest.progress.goal(), seconds_to_ticks(10));
        assert_eq!(quest.progress.ratio(), 0.0);

        // Unknown instance or action are rejected.
        assert!(!state.assign_action(&catalog, &TargetId::new("ghost"), &forest));
        assert!(!state.assign_action(&catalog, &hero, &ActionId::new("fishing")));
    }

    #[test]
    fn assigning_the_same_action_twice_restarts_one_quest() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let forest = ActionId::new("forest_exploration");

        state.assign_action(&catalog, &hero, &forest);
        state.advance(seconds_to_ticks(5));
        // Re-assigning the running action resets its progress, not a second row.
        state.assign_action(&catalog, &hero, &forest);

        let target = state.targets.iter().find(|t| t.id == hero).unwrap();
        assert_eq!(target.quests.len(), 1, "same action must not duplicate");
        assert_eq!(target.quests[0].progress.ratio(), 0.0, "should restart");
    }

    #[test]
    fn one_target_runs_distinct_actions_concurrently() {
        // A facility-style target running two different actions at once.
        let mut catalog = Catalog::new();
        catalog.register_target(
            TargetTemplate::new("farm", "Farm")
                .with_action("farming")
                .with_action("livestock"),
        );
        catalog.register_action(ActionTemplate::new(
            "farming",
            "Farming",
            seconds_to_ticks(10),
        ));
        catalog.register_action(ActionTemplate::new(
            "livestock",
            "Livestock",
            seconds_to_ticks(20),
        ));
        let mut state = GameState::seeded(&catalog);
        let farm = TargetId::new("farm");

        assert!(state.assign_action(&catalog, &farm, &ActionId::new("farming")));
        assert!(state.assign_action(&catalog, &farm, &ActionId::new("livestock")));

        let target = state.targets.iter().find(|t| t.id == farm).unwrap();
        assert_eq!(target.quests.len(), 2);
        assert_eq!(state.active_quests().count(), 2);
    }

    #[test]
    fn advance_completes_multiple_concurrent_quests_in_one_step() {
        // One target runs two actions with the same goal: a single advance past
        // that goal must report both completions and empty the target's quests.
        let mut catalog = Catalog::new();
        catalog.register_target(
            TargetTemplate::new("farm", "Farm")
                .with_action("farming")
                .with_action("livestock"),
        );
        catalog.register_action(ActionTemplate::new(
            "farming",
            "Farming",
            seconds_to_ticks(10),
        ));
        catalog.register_action(ActionTemplate::new(
            "livestock",
            "Livestock",
            seconds_to_ticks(10),
        ));
        let mut state = GameState::seeded(&catalog);
        let farm = TargetId::new("farm");
        let farming = ActionId::new("farming");
        let livestock = ActionId::new("livestock");
        state.assign_action(&catalog, &farm, &farming);
        state.assign_action(&catalog, &farm, &livestock);

        let events = state.advance(seconds_to_ticks(10));
        assert_eq!(
            events,
            vec![
                GameEvent::QuestCompleted {
                    target: farm.clone(),
                    action: farming,
                },
                GameEvent::QuestCompleted {
                    target: farm.clone(),
                    action: livestock,
                },
            ]
        );
        let target = state.targets.iter().find(|t| t.id == farm).unwrap();
        assert!(
            target.quests.is_empty(),
            "both completed quests are removed"
        );
    }

    #[test]
    fn advance_only_completes_the_finished_concurrent_quest() {
        // Two actions with different goals: a step that finishes only the shorter
        // one reports a single event and leaves the longer quest running.
        let mut catalog = Catalog::new();
        catalog.register_target(
            TargetTemplate::new("farm", "Farm")
                .with_action("farming")
                .with_action("livestock"),
        );
        catalog.register_action(ActionTemplate::new(
            "farming",
            "Farming",
            seconds_to_ticks(10),
        ));
        catalog.register_action(ActionTemplate::new(
            "livestock",
            "Livestock",
            seconds_to_ticks(20),
        ));
        let mut state = GameState::seeded(&catalog);
        let farm = TargetId::new("farm");
        let farming = ActionId::new("farming");
        let livestock = ActionId::new("livestock");
        state.assign_action(&catalog, &farm, &farming);
        state.assign_action(&catalog, &farm, &livestock);

        let events = state.advance(seconds_to_ticks(10));
        assert_eq!(
            events,
            vec![GameEvent::QuestCompleted {
                target: farm.clone(),
                action: farming,
            }]
        );
        let target = state.targets.iter().find(|t| t.id == farm).unwrap();
        assert_eq!(target.quests.len(), 1, "the longer quest keeps running");
        assert_eq!(target.quests[0].action, livestock);
        assert_eq!(target.quests[0].progress.ratio(), 0.5);
    }

    #[test]
    fn advance_progresses_then_completes_and_removes_the_quest() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let forest = ActionId::new("forest_exploration");
        state.assign_action(&catalog, &hero, &forest);

        // Half-way: still running, no event yet.
        let events = state.advance(seconds_to_ticks(5));
        assert!(events.is_empty());
        let ratio = state.targets.iter().find(|t| t.id == hero).unwrap().quests[0]
            .progress
            .ratio();
        assert_eq!(ratio, 0.5);

        // Completing advance: one event, and the finished quest is removed.
        let events = state.advance(seconds_to_ticks(100)); // overshoot the goal
        assert_eq!(
            events,
            vec![GameEvent::QuestCompleted {
                target: hero.clone(),
                action: forest.clone(),
            }]
        );
        let hero_target = state.targets.iter().find(|t| t.id == hero).unwrap();
        assert!(hero_target.quests.is_empty(), "completed quest is removed");

        // The completion fires once: a further advance reports nothing.
        assert!(state.advance(seconds_to_ticks(5)).is_empty());

        // Idle targets stayed questless throughout.
        assert!(
            state
                .targets
                .iter()
                .filter(|t| t.id != hero)
                .all(|t| t.quests.is_empty())
        );
    }
}
