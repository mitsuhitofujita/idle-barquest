//! Live, serializable-friendly game state and deterministic task progression.

use crate::catalog::{Catalog, RecipeOutput, RecipeTemplate, Reward};
use crate::id::{ActionId, LocationId, RecipeId, ResourceId, SettlementId, TargetId};
use crate::random::RandomSource;
use crate::time::Progress;

/// The single running task assigned to a target.
#[derive(Debug, Clone)]
pub struct Quest {
    /// Where the action is being performed.
    pub location: LocationId,
    /// Which action is running.
    pub action: ActionId,
    /// Recipe being crafted, or `None` for an ordinary location action.
    pub recipe: Option<RecipeId>,
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
        /// Ordered, aggregated resources awarded by the matching reward table.
        rewards: Vec<Reward>,
    },
    /// A target completed a crafting recipe and was freed.
    CraftCompleted {
        /// Instance that performed the craft.
        target: TargetId,
        /// Location where crafting took place.
        location: LocationId,
        /// Recipe that completed.
        recipe: RecipeId,
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
#[derive(Debug, Clone)]
pub struct GameState {
    /// Settlement currently serving as the player's development base.
    pub current_settlement: SettlementId,
    /// Every live target, in stable display order.
    pub targets: Vec<TargetInstance>,
    /// Discovered or unlocked locations, in menu order.
    pub unlocked_locations: Vec<LocationId>,
    /// Unlocked actions, in menu order.
    pub unlocked_actions: Vec<ActionId>,
    /// Materials and stackable items held, in first-acquisition order.
    pub inventory: Vec<ResourceStack>,
    /// Permanent facilities completed at the current settlement.
    pub built_facilities: Vec<RecipeId>,
}

impl GameState {
    /// An empty world rooted at one current settlement.
    pub fn new(current_settlement: impl Into<SettlementId>) -> Self {
        Self {
            current_settlement: current_settlement.into(),
            targets: Vec::new(),
            unlocked_locations: Vec::new(),
            unlocked_actions: Vec::new(),
            inventory: Vec::new(),
            built_facilities: Vec::new(),
        }
    }

    /// Seeds one instance per target template and unlocks shipped locations and actions.
    pub fn seeded(catalog: &Catalog) -> Self {
        let current_settlement = catalog
            .settlements()
            .next()
            .expect("seeded catalog has a settlement")
            .id
            .clone();
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
            current_settlement,
            targets,
            unlocked_locations,
            unlocked_actions,
            inventory: Vec::new(),
            built_facilities: Vec::new(),
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

    /// Iterates recipes for one Location and Action, omitting facilities that
    /// are already built or currently under construction.
    pub fn available_recipes<'a>(
        &'a self,
        catalog: &'a Catalog,
        location: &'a LocationId,
        action: &'a ActionId,
    ) -> impl Iterator<Item = &'a RecipeTemplate> {
        catalog.recipes().filter(move |recipe| {
            &recipe.location == location
                && &recipe.action == action
                && (!matches!(recipe.output, RecipeOutput::Facility)
                    || (!self.has_facility(&recipe.id)
                        && !self.facility_is_in_progress(&recipe.id)))
        })
    }

    /// Whether a recipe can start now, including target, compatibility,
    /// facility, and material checks.
    pub fn can_craft_recipe(
        &self,
        catalog: &Catalog,
        instance: &TargetId,
        location: &LocationId,
        action: &ActionId,
        recipe: &RecipeId,
    ) -> bool {
        let Some(target) = self.targets.iter().find(|target| &target.id == instance) else {
            return false;
        };
        let Some(target_template) = catalog.target(&target.template_id) else {
            return false;
        };
        let Some(location_template) = catalog.location(location) else {
            return false;
        };
        let Some(recipe_template) = catalog.recipe(recipe) else {
            return false;
        };
        if target.quest.is_some()
            || !self.unlocked_locations.contains(location)
            || !self.unlocked_actions.contains(action)
            || !target_template.supports(action)
            || !location_template.supports(action)
            || &recipe_template.location != location
            || &recipe_template.action != action
            || !valid_recipe(catalog, recipe_template)
            || recipe_template
                .required_facilities
                .iter()
                .any(|facility| !self.has_facility(facility))
            || recipe_template
                .ingredients
                .iter()
                .any(|ingredient| self.resource_count(&ingredient.resource) < ingredient.amount)
        {
            return false;
        }
        !matches!(recipe_template.output, RecipeOutput::Facility)
            || (!self.has_facility(recipe) && !self.facility_is_in_progress(recipe))
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
            recipe: None,
            progress: Progress::new(action_template.goal_ticks),
        });
        true
    }

    /// Starts one craft and consumes all ingredients immediately.
    pub fn assign_recipe(
        &mut self,
        catalog: &Catalog,
        instance: &TargetId,
        location: &LocationId,
        action: &ActionId,
        recipe: &RecipeId,
    ) -> bool {
        if !self.can_craft_recipe(catalog, instance, location, action, recipe) {
            return false;
        }
        let recipe_template = catalog.recipe(recipe).expect("validated recipe exists");
        for ingredient in &recipe_template.ingredients {
            let stack = self
                .inventory
                .iter_mut()
                .find(|stack| stack.resource == ingredient.resource)
                .expect("validated ingredient stack exists");
            stack.amount -= ingredient.amount;
        }
        let target = self
            .targets
            .iter_mut()
            .find(|target| &target.id == instance)
            .expect("validated target exists");
        target.quest = Some(Quest {
            location: location.clone(),
            action: action.clone(),
            recipe: Some(recipe.clone()),
            progress: Progress::new(recipe_template.goal_ticks),
        });
        true
    }

    /// Advances every active task, applies its reward or craft output, emits a
    /// completion, and frees the finished target.
    pub fn advance(
        &mut self,
        catalog: &Catalog,
        ticks: u64,
        random: &mut impl RandomSource,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let (targets, inventory, built_facilities) = (
            &mut self.targets,
            &mut self.inventory,
            &mut self.built_facilities,
        );
        for target in targets {
            let completed = target.quest.as_mut().is_some_and(|quest| {
                quest.progress.advance(ticks);
                quest.progress.is_complete()
            });
            if completed {
                let quest = target.quest.take().expect("completed task exists");
                if let Some(recipe_id) = quest.recipe {
                    let recipe = catalog
                        .recipe(&recipe_id)
                        .expect("assigned craft has a recipe");
                    match &recipe.output {
                        RecipeOutput::Facility => built_facilities.push(recipe_id.clone()),
                        RecipeOutput::Item { resource, amount } => {
                            add_resource(inventory, resource, *amount);
                        }
                    }
                    events.push(GameEvent::CraftCompleted {
                        target: target.id.clone(),
                        location: quest.location,
                        recipe: recipe_id,
                    });
                    continue;
                }
                let rewards = catalog
                    .reward_table(&quest.location, &quest.action)
                    .expect("assigned task has a reward table")
                    .roll(random);
                for reward in &rewards {
                    add_resource(inventory, &reward.resource, reward.amount);
                }
                events.push(GameEvent::QuestCompleted {
                    target: target.id.clone(),
                    location: quest.location,
                    action: quest.action,
                    rewards,
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

    /// Whether one permanent facility has been completed.
    pub fn has_facility(&self, facility: &RecipeId) -> bool {
        self.built_facilities.contains(facility)
    }

    /// Iterates acquired, known resources in Catalog registration order.
    ///
    /// Stack presence records acquisition independently of quantity, so a
    /// zero-quantity stack remains in this iterator.
    pub fn acquired_resources<'a>(
        &'a self,
        catalog: &'a Catalog,
    ) -> impl Iterator<Item = (&'a crate::ResourceTemplate, &'a ResourceStack)> {
        catalog.resources().filter_map(|resource| {
            self.inventory
                .iter()
                .find(|stack| stack.resource == resource.id)
                .map(|stack| (resource, stack))
        })
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

    fn facility_is_in_progress(&self, facility: &RecipeId) -> bool {
        self.targets.iter().any(|target| {
            target
                .quest
                .as_ref()
                .and_then(|quest| quest.recipe.as_ref())
                == Some(facility)
        })
    }
}

fn valid_reward_table(catalog: &Catalog, location: &LocationId, action: &ActionId) -> bool {
    catalog.reward_table(location, action).is_some_and(|table| {
        table.entries().next().is_some()
            && table.entries().all(|entry| {
                (1..=100).contains(&entry.chance)
                    && entry.amount > 0
                    && catalog.resource(&entry.resource).is_some()
            })
    })
}

fn valid_recipe(catalog: &Catalog, recipe: &RecipeTemplate) -> bool {
    let ingredients_are_valid = !recipe.ingredients.is_empty()
        && recipe
            .ingredients
            .iter()
            .enumerate()
            .all(|(index, ingredient)| {
                ingredient.amount > 0
                    && catalog.resource(&ingredient.resource).is_some()
                    && !recipe.ingredients[..index]
                        .iter()
                        .any(|earlier| earlier.resource == ingredient.resource)
            });
    let facilities_are_valid = recipe.required_facilities.iter().all(|facility| {
        catalog
            .recipe(facility)
            .is_some_and(|required| matches!(required.output, RecipeOutput::Facility))
    });
    let output_is_valid = match &recipe.output {
        RecipeOutput::Facility => true,
        RecipeOutput::Item { resource, amount } => {
            *amount > 0 && catalog.resource(resource).is_some()
        }
    };
    recipe.goal_ticks > 0
        && catalog.location(&recipe.location).is_some()
        && catalog.action(&recipe.action).is_some()
        && ingredients_are_valid
        && facilities_are_valid
        && output_is_valid
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
    use crate::catalog::{
        ActionTemplate, LocationTemplate, ResourceTemplate, RewardTable, SettlementTemplate,
        TargetTemplate,
    };
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
        assert_eq!(
            state.current_settlement,
            SettlementId::new("awakening_shore")
        );
        assert_eq!(
            catalog.settlement(&state.current_settlement).unwrap().label,
            "Awakening Shore"
        );
        assert_eq!(state.targets.len(), 1);
        assert_eq!(state.targets[0].template_id, TargetId::new("hero"));
        assert!(state.targets[0].quest.is_none());
        assert_eq!(
            state.unlocked_locations,
            ["first_shore", "nearby_woods", "nearby_hill", "base"].map(LocationId::new)
        );
        assert_eq!(
            state.unlocked_actions,
            ["gather", "fish", "hunt", "craft"].map(ActionId::new)
        );
        assert!(state.inventory.is_empty());
        assert!(state.built_facilities.is_empty());
    }

    #[test]
    fn unlock_location_and_action_are_validated_and_idempotent() {
        let catalog = Catalog::builtin();
        let mut state = GameState::new("awakening_shore");
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
    fn completion_contains_ordered_rewards_and_frees_target() {
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
                rewards: vec![
                    Reward {
                        resource: ResourceId::new("seaweed_fragment"),
                        amount: 1,
                    },
                    Reward {
                        resource: ResourceId::new("pebble"),
                        amount: 1,
                    },
                ],
            }]
        );
        assert_eq!(
            state.resource_count(&ResourceId::new("seaweed_fragment")),
            1
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
        assert_eq!(
            state.resource_count(&ResourceId::new("seaweed_fragment")),
            2
        );
        assert_eq!(state.inventory.len(), 2);
    }

    #[test]
    fn crafting_consumes_materials_at_start_and_builds_a_unique_facility() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let base = LocationId::new("base");
        let craft = ActionId::new("craft");
        let table = RecipeId::new("stone_table");
        add_resource(&mut state.inventory, &ResourceId::new("pebble"), 20);

        assert!(state.can_craft_recipe(&catalog, &hero, &base, &craft, &table));
        assert!(state.assign_recipe(&catalog, &hero, &base, &craft, &table));
        assert_eq!(state.resource_count(&ResourceId::new("pebble")), 0);
        assert!(!state.has_facility(&table));
        assert!(
            state
                .available_recipes(&catalog, &base, &craft)
                .all(|recipe| recipe.id != table),
            "a unique facility should disappear while being built"
        );

        let mut random = FixedRandom(0);
        assert!(
            state
                .advance(&catalog, seconds_to_ticks(19), &mut random)
                .is_empty()
        );
        assert_eq!(
            state.advance(&catalog, seconds_to_ticks(1), &mut random),
            [GameEvent::CraftCompleted {
                target: hero.clone(),
                location: base.clone(),
                recipe: table.clone(),
            }]
        );
        assert!(state.has_facility(&table));
        assert!(!state.can_craft_recipe(&catalog, &hero, &base, &craft, &table));
    }

    #[test]
    fn unavailable_craft_does_not_consume_any_materials() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        add_resource(&mut state.inventory, &ResourceId::new("twig"), 50);
        add_resource(&mut state.inventory, &ResourceId::new("pebble"), 49);
        add_resource(&mut state.inventory, &ResourceId::new("grass"), 50);

        assert!(!state.assign_recipe(
            &catalog,
            &TargetId::new("hero"),
            &LocationId::new("base"),
            &ActionId::new("craft"),
            &RecipeId::new("crude_bed"),
        ));
        assert_eq!(state.resource_count(&ResourceId::new("twig")), 50);
        assert_eq!(state.resource_count(&ResourceId::new("pebble")), 49);
        assert_eq!(state.resource_count(&ResourceId::new("grass")), 50);
        assert!(state.targets[0].quest.is_none());
    }

    #[test]
    fn fishing_rod_requires_the_table_and_remains_repeatable() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let base = LocationId::new("base");
        let craft = ActionId::new("craft");
        let rod = RecipeId::new("primitive_fishing_rod");
        for (resource, amount) in [("twig", 10), ("vine", 10), ("small_fang", 6)] {
            add_resource(&mut state.inventory, &ResourceId::new(resource), amount);
        }

        assert!(!state.can_craft_recipe(&catalog, &hero, &base, &craft, &rod));
        state.built_facilities.push(RecipeId::new("stone_table"));
        let mut random = FixedRandom(0);
        for expected in 1..=2 {
            assert!(state.assign_recipe(&catalog, &hero, &base, &craft, &rod));
            state.advance(&catalog, seconds_to_ticks(20), &mut random);
            assert_eq!(
                state.resource_count(&ResourceId::new("primitive_fishing_rod")),
                expected
            );
            assert!(
                state
                    .available_recipes(&catalog, &base, &craft)
                    .any(|recipe| recipe.id == rod)
            );
        }
    }

    #[test]
    fn facility_cannot_be_started_by_two_targets() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let first = TargetId::new("hero");
        let second = state.spawn_target(&catalog, &first).unwrap();
        let base = LocationId::new("base");
        let craft = ActionId::new("craft");
        let table = RecipeId::new("stone_table");
        add_resource(&mut state.inventory, &ResourceId::new("pebble"), 40);

        assert!(state.assign_recipe(&catalog, &first, &base, &craft, &table));
        assert!(!state.assign_recipe(&catalog, &second, &base, &craft, &table));
        assert_eq!(state.resource_count(&ResourceId::new("pebble")), 20);
    }

    #[test]
    fn acquired_resources_use_catalog_order_and_keep_zero_stacks() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        state.inventory = vec![
            ResourceStack {
                resource: ResourceId::new("vine"),
                amount: 0,
            },
            ResourceStack {
                resource: ResourceId::new("pebble"),
                amount: 34,
            },
        ];

        let acquired: Vec<(&str, u64)> = state
            .acquired_resources(&catalog)
            .map(|(resource, stack)| (resource.id.as_str(), stack.amount))
            .collect();

        assert_eq!(acquired, [("pebble", 34), ("vine", 0)]);
    }

    #[test]
    fn empty_reward_result_does_not_change_inventory() {
        let mut catalog = Catalog::new();
        catalog.register_target(TargetTemplate::new("hero", "Hero").with_action("fish"));
        catalog.register_settlement(SettlementTemplate::new(
            "awakening_shore",
            "Awakening Shore",
        ));
        catalog.register_location(
            LocationTemplate::new("first_shore", "First Shore").with_action("fish"),
        );
        catalog.register_action(ActionTemplate::new("fish", "Fish", seconds_to_ticks(10)));
        catalog.register_resource(ResourceTemplate::new("small_fish", "Small Fish"));
        catalog.register_reward_table(RewardTable::new("first_shore", "fish").with_resource(
            "small_fish",
            1,
            30,
        ));
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let shore = LocationId::new("first_shore");
        let fish = ActionId::new("fish");
        let mut random = FixedRandom(30);
        assert!(state.assign_action(&catalog, &hero, &shore, &fish));

        assert_eq!(
            state.advance(&catalog, seconds_to_ticks(10), &mut random),
            [GameEvent::QuestCompleted {
                target: hero,
                location: shore,
                action: fish,
                rewards: vec![],
            }]
        );
        assert!(state.inventory.is_empty());
    }

    #[test]
    fn assignment_accepts_duplicate_resources_and_no_certain_entry() {
        let mut catalog = Catalog::new();
        catalog.register_target(TargetTemplate::new("hero", "Hero").with_action("gather"));
        catalog.register_settlement(SettlementTemplate::new("camp", "Camp"));
        catalog.register_location(LocationTemplate::new("field", "Field").with_action("gather"));
        catalog.register_action(ActionTemplate::new("gather", "Gather", 100));
        catalog.register_resource(ResourceTemplate::new("grass", "Grass"));
        catalog.register_reward_table(
            RewardTable::new("field", "gather")
                .with_resource("grass", 1, 40)
                .with_resource("grass", 2, 60),
        );
        let mut state = GameState::seeded(&catalog);

        assert!(state.assign_action(
            &catalog,
            &TargetId::new("hero"),
            &LocationId::new("field"),
            &ActionId::new("gather"),
        ));
    }

    #[test]
    fn custom_content_enforces_target_compatibility() {
        let mut catalog = Catalog::new();
        catalog.register_target(TargetTemplate::new("fisher", "Fisher").with_action("fish"));
        catalog.register_settlement(SettlementTemplate::new("camp", "Camp"));
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
