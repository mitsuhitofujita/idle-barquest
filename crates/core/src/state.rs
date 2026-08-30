//! Live, serializable-friendly game state and deterministic task progression.

use crate::catalog::{Catalog, RewardOutcome};
use crate::id::{ActionId, LocationId, ResourceId, TargetId};
use crate::random::RandomSource;
use crate::time::Progress;

/// The single running task assigned to a target.
#[derive(Debug, Clone)]
pub struct Quest {
    /// Where the action is being performed.
    pub location: LocationId,
    /// Which action is running.
    pub action: ActionId,
    /// How far the action has progressed.
    pub progress: Progress,
}

/// Something that happened during an [`advance`](GameState::advance) step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    /// A target finished its task, drew a reward, and was freed.
    QuestCompleted {
        /// Instance that performed the action.
        target: TargetId,
        /// Location where the action was performed.
        location: LocationId,
        /// Action that completed.
        action: ActionId,
        /// Exclusive result selected from the matching reward table.
        outcome: RewardOutcome,
    },
}

/// One accumulated resource quantity in the live inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceStack {
    /// Resource template whose units are held.
    pub resource: ResourceId,
    /// Total units currently held.
    pub amount: u64,
}

/// One live person or organization that can perform at most one task.
#[derive(Debug, Clone)]
pub struct TargetInstance {
    /// Unique id for this specific target instance.
    pub id: TargetId,
    /// Template that defines this instance's label and action compatibility.
    pub template_id: TargetId,
    /// The currently assigned task, or `None` while the target is available.
    pub quest: Option<Quest>,
}

/// The live game world.
#[derive(Debug, Clone, Default)]
pub struct GameState {
    /// Every live target, in stable display order.
    pub targets: Vec<TargetInstance>,
    /// Discovered or unlocked locations, in menu order.
    pub unlocked_locations: Vec<LocationId>,
    /// Unlocked actions, in menu order.
    pub unlocked_actions: Vec<ActionId>,
    /// Resources earned during this game, in first-acquisition order.
    pub inventory: Vec<ResourceStack>,
}

impl GameState {
    /// An empty world with no targets or unlocks.
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds one instance per target template and unlocks shipped locations and actions.
    pub fn seeded(catalog: &Catalog) -> Self {
        let targets = catalog
            .targets()
            .map(|template| TargetInstance {
                id: template.id.clone(),
                template_id: template.id.clone(),
                quest: None,
            })
            .collect();
        let unlocked_locations = catalog
            .locations()
            .map(|location| location.id.clone())
            .collect();
        let unlocked_actions = catalog.actions().map(|action| action.id.clone()).collect();
        Self {
            targets,
            unlocked_locations,
            unlocked_actions,
            inventory: Vec::new(),
        }
    }

    /// Spawns a target instance from known content.
    pub fn spawn_target(&mut self, catalog: &Catalog, template: &TargetId) -> Option<TargetId> {
        catalog.target(template)?;
        let id = self.unique_instance_id(template);
        self.targets.push(TargetInstance {
            id: id.clone(),
            template_id: template.clone(),
            quest: None,
        });
        Some(id)
    }

    /// Unlocks a known location. The operation is idempotent.
    pub fn unlock_location(&mut self, catalog: &Catalog, location: &LocationId) -> bool {
        if catalog.location(location).is_none() {
            return false;
        }
        if !self.unlocked_locations.contains(location) {
            self.unlocked_locations.push(location.clone());
        }
        true
    }

    /// Unlocks a known action. The operation is idempotent.
    pub fn unlock_action(&mut self, catalog: &Catalog, action: &ActionId) -> bool {
        if catalog.action(action).is_none() {
            return false;
        }
        if !self.unlocked_actions.contains(action) {
            self.unlocked_actions.push(action.clone());
        }
        true
    }

    /// Iterates unlocked, known locations in live-state order.
    pub fn available_locations<'a>(
        &'a self,
        catalog: &'a Catalog,
    ) -> impl Iterator<Item = &'a LocationId> {
        self.unlocked_locations
            .iter()
            .filter(|location| catalog.location(location).is_some())
    }

    /// Iterates actions unlocked globally and supported by both target and location.
    pub fn available_actions<'a>(
        &'a self,
        catalog: &'a Catalog,
        instance: &TargetId,
        location: &LocationId,
    ) -> impl Iterator<Item = &'a ActionId> {
        let target_template = self
            .targets
            .iter()
            .find(|target| &target.id == instance)
            .and_then(|target| catalog.target(&target.template_id));
        let location_template = self
            .unlocked_locations
            .contains(location)
            .then(|| catalog.location(location))
            .flatten();
        self.unlocked_actions.iter().filter(move |action| {
            target_template.is_some_and(|target| target.supports(action))
                && location_template.is_some_and(|location| location.supports(action))
        })
    }

    /// Assigns a new task when every id and compatibility constraint is valid.
    ///
    /// A busy target rejects all assignments; tasks cannot be restarted or replaced.
    pub fn assign_action(
        &mut self,
        catalog: &Catalog,
        instance: &TargetId,
        location: &LocationId,
        action: &ActionId,
    ) -> bool {
        let Some(action_template) = catalog.action(action) else {
            return false;
        };
        let Some(location_template) = catalog.location(location) else {
            return false;
        };
        if !self.unlocked_locations.contains(location)
            || !self.unlocked_actions.contains(action)
            || !location_template.supports(action)
            || !valid_reward_table(catalog, location, action)
        {
            return false;
        }
        let Some(target) = self
            .targets
            .iter_mut()
            .find(|target| &target.id == instance)
        else {
            return false;
        };
        let Some(target_template) = catalog.target(&target.template_id) else {
            return false;
        };
        if target.quest.is_some() || !target_template.supports(action) {
            return false;
        }

        target.quest = Some(Quest {
            location: location.clone(),
            action: action.clone(),
            progress: Progress::new(action_template.goal_ticks),
        });
        true
    }

    /// Advances every active task, draws rewards, emits completions, and frees
    /// finished targets.
    pub fn advance(
        &mut self,
        catalog: &Catalog,
        ticks: u64,
        random: &mut impl RandomSource,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let (targets, inventory) = (&mut self.targets, &mut self.inventory);
        for target in targets {
            let completed = target.quest.as_mut().is_some_and(|quest| {
                quest.progress.advance(ticks);
                quest.progress.is_complete()
            });
            if completed {
                let quest = target.quest.take().expect("completed task exists");
                let outcome = catalog
                    .reward_table(&quest.location, &quest.action)
                    .expect("assigned task has a reward table")
                    .draw(random);
                if let RewardOutcome::Resource { resource, amount } = &outcome {
                    add_resource(inventory, resource, *amount);
                }
                events.push(GameEvent::QuestCompleted {
                    target: target.id.clone(),
                    location: quest.location,
                    action: quest.action,
                    outcome,
                });
            }
        }
        events
    }

    /// Iterates active tasks in stable target order.
    pub fn active_quests(&self) -> impl Iterator<Item = (&TargetInstance, &Quest)> {
        self.targets
            .iter()
            .filter_map(|target| target.quest.as_ref().map(|quest| (target, quest)))
    }

    /// Returns the currently held amount of one resource.
    pub fn resource_count(&self, resource: &ResourceId) -> u64 {
        self.inventory
            .iter()
            .find(|stack| &stack.resource == resource)
            .map_or(0, |stack| stack.amount)
    }

    fn unique_instance_id(&self, template: &TargetId) -> TargetId {
        if !self.targets.iter().any(|target| &target.id == template) {
            return template.clone();
        }
        let mut n = 2;
        loop {
            let candidate = TargetId::new(format!("{}#{n}", template.as_str()));
            if !self.targets.iter().any(|target| target.id == candidate) {
                return candidate;
            }
            n += 1;
        }
    }
}

fn valid_reward_table(catalog: &Catalog, location: &LocationId, action: &ActionId) -> bool {
    catalog.reward_table(location, action).is_some_and(|table| {
        table.total_chance() == 100
            && table.entries().all(|entry| {
                entry.chance > 0
                    && match &entry.outcome {
                        RewardOutcome::Resource { resource, amount } => {
                            *amount > 0 && catalog.resource(resource).is_some()
                        }
                        RewardOutcome::Nothing => true,
                    }
            })
    })
}

fn add_resource(inventory: &mut Vec<ResourceStack>, resource: &ResourceId, amount: u64) {
    if let Some(stack) = inventory
        .iter_mut()
        .find(|stack| &stack.resource == resource)
    {
        stack.amount = stack.amount.saturating_add(amount);
    } else {
        inventory.push(ResourceStack {
            resource: resource.clone(),
            amount,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{ActionTemplate, LocationTemplate, TargetTemplate};
    use crate::time::seconds_to_ticks;

    struct FixedRandom(u32);

    impl RandomSource for FixedRandom {
        fn below(&mut self, upper_exclusive: u32) -> u32 {
            assert!(self.0 < upper_exclusive);
            self.0
        }
    }

    fn ids() -> (TargetId, LocationId, ActionId) {
        (
            TargetId::new("hero"),
            LocationId::new("first_shore"),
            ActionId::new("gather"),
        )
    }

    #[test]
    fn seeded_state_matches_starting_content() {
        let catalog = Catalog::builtin();
        let state = GameState::seeded(&catalog);
        assert_eq!(state.targets.len(), 1);
        assert_eq!(state.targets[0].template_id, TargetId::new("hero"));
        assert!(state.targets[0].quest.is_none());
        assert_eq!(
            state.unlocked_locations,
            ["first_shore", "nearby_woods", "nearby_hill"].map(LocationId::new)
        );
        assert_eq!(
            state.unlocked_actions,
            ["gather", "fish", "hunt"].map(ActionId::new)
        );
        assert!(state.inventory.is_empty());
    }

    #[test]
    fn unlock_location_and_action_are_validated_and_idempotent() {
        let catalog = Catalog::builtin();
        let mut state = GameState::new();
        let shore = LocationId::new("first_shore");
        let gather = ActionId::new("gather");
        assert!(state.unlock_location(&catalog, &shore));
        assert!(state.unlock_location(&catalog, &shore));
        assert_eq!(state.unlocked_locations, [shore]);
        assert!(!state.unlock_location(&catalog, &LocationId::new("volcano")));
        assert!(state.unlock_action(&catalog, &gather));
        assert!(state.unlock_action(&catalog, &gather));
        assert_eq!(state.unlocked_actions, [gather]);
        assert!(!state.unlock_action(&catalog, &ActionId::new("mine")));
    }

    #[test]
    fn available_actions_intersect_unlock_target_and_location_support() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let shore = LocationId::new("first_shore");
        let woods = LocationId::new("nearby_woods");
        assert_eq!(
            state
                .available_actions(&catalog, &hero, &shore)
                .map(ActionId::as_str)
                .collect::<Vec<_>>(),
            ["gather", "fish"]
        );
        assert_eq!(
            state
                .available_actions(&catalog, &hero, &woods)
                .map(ActionId::as_str)
                .collect::<Vec<_>>(),
            ["gather", "hunt"]
        );
        state
            .unlocked_actions
            .retain(|action| action.as_str() == "hunt");
        assert!(
            state
                .available_actions(&catalog, &hero, &shore)
                .next()
                .is_none()
        );
    }

    #[test]
    fn assignment_validates_every_constraint() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let (hero, shore, gather) = ids();
        state.unlocked_locations.clear();
        assert!(!state.assign_action(&catalog, &hero, &shore, &gather));
        state.unlock_location(&catalog, &shore);
        state.unlocked_actions.clear();
        assert!(!state.assign_action(&catalog, &hero, &shore, &gather));
        state.unlock_action(&catalog, &gather);
        assert!(!state.assign_action(&catalog, &hero, &shore, &ActionId::new("hunt")));
        assert!(!state.assign_action(&catalog, &TargetId::new("ghost"), &shore, &gather));
        assert!(!state.assign_action(&catalog, &hero, &LocationId::new("volcano"), &gather));
        assert!(state.assign_action(&catalog, &hero, &shore, &gather));
    }

    #[test]
    fn busy_target_rejects_a_second_task() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let (hero, shore, gather) = ids();
        assert!(state.assign_action(&catalog, &hero, &shore, &gather));
        assert!(!state.assign_action(
            &catalog,
            &hero,
            &LocationId::new("nearby_woods"),
            &ActionId::new("hunt")
        ));
        let quest = state.targets[0].quest.as_ref().unwrap();
        assert_eq!(quest.location, shore);
        assert_eq!(quest.action, gather);
    }

    #[test]
    fn completion_contains_location_and_frees_target() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let (hero, shore, gather) = ids();
        let mut random = FixedRandom(20);
        assert!(state.assign_action(&catalog, &hero, &shore, &gather));
        assert!(
            state
                .advance(&catalog, seconds_to_ticks(5), &mut random)
                .is_empty()
        );
        assert_eq!(
            state.targets[0].quest.as_ref().unwrap().progress.ratio(),
            0.5
        );
        assert_eq!(
            state.advance(&catalog, seconds_to_ticks(5), &mut random),
            [GameEvent::QuestCompleted {
                target: hero.clone(),
                location: shore.clone(),
                action: gather.clone(),
                outcome: RewardOutcome::Resource {
                    resource: ResourceId::new("pebble"),
                    amount: 1,
                },
            }]
        );
        assert_eq!(state.resource_count(&ResourceId::new("pebble")), 1);
        assert!(state.targets[0].quest.is_none());
        assert!(state.assign_action(&catalog, &hero, &shore, &gather));
    }

    #[test]
    fn repeated_resource_rewards_accumulate_in_one_stack() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let (hero, shore, gather) = ids();
        let mut random = FixedRandom(20);

        for _ in 0..2 {
            assert!(state.assign_action(&catalog, &hero, &shore, &gather));
            state.advance(&catalog, seconds_to_ticks(10), &mut random);
        }

        assert_eq!(state.resource_count(&ResourceId::new("pebble")), 2);
        assert_eq!(state.inventory.len(), 1);
    }

    #[test]
    fn nothing_reward_does_not_change_inventory() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let shore = LocationId::new("first_shore");
        let fish = ActionId::new("fish");
        let mut random = FixedRandom(10);
        assert!(state.assign_action(&catalog, &hero, &shore, &fish));

        assert_eq!(
            state.advance(&catalog, seconds_to_ticks(10), &mut random),
            [GameEvent::QuestCompleted {
                target: hero,
                location: shore,
                action: fish,
                outcome: RewardOutcome::Nothing,
            }]
        );
        assert!(state.inventory.is_empty());
    }

    #[test]
    fn custom_content_enforces_target_compatibility() {
        let mut catalog = Catalog::new();
        catalog.register_target(TargetTemplate::new("fisher", "Fisher").with_action("fish"));
        catalog.register_location(LocationTemplate::new("woods", "Woods").with_action("hunt"));
        catalog.register_action(ActionTemplate::new("fish", "Fish", 100));
        catalog.register_action(ActionTemplate::new("hunt", "Hunt", 100));
        let mut state = GameState::seeded(&catalog);
        assert!(
            state
                .available_actions(
                    &catalog,
                    &TargetId::new("fisher"),
                    &LocationId::new("woods")
                )
                .next()
                .is_none()
        );
        assert!(!state.assign_action(
            &catalog,
            &TargetId::new("fisher"),
            &LocationId::new("woods"),
            &ActionId::new("hunt"),
        ));
    }

    #[test]
    fn duplicate_spawns_receive_stable_unique_ids() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        assert_eq!(
            state.spawn_target(&catalog, &hero),
            Some(TargetId::new("hero#2"))
        );
        assert_eq!(
            state.spawn_target(&catalog, &hero),
            Some(TargetId::new("hero#3"))
        );
        assert_eq!(state.spawn_target(&catalog, &TargetId::new("ghost")), None);
    }
}
