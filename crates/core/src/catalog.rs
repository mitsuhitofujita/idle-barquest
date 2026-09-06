//! Data-driven content: immutable Target, Settlement, Location, Action, and
//! Resource templates plus
//! the [`Catalog`] pool that owns them.
//!
//! Content is kept separate from the live [`GameState`](crate::GameState):
//! templates are *instantiated* from the catalog into the state, and registering
//! a new template makes content available without touching live state.

use crate::id::{ActionId, LocationId, ResourceId, SettlementId, TargetId};
use crate::random::RandomSource;
use crate::time::seconds_to_ticks;

/// Immutable definition of a kind of target — content data, not live state.
#[derive(Debug, Clone)]
pub struct TargetTemplate {
    /// Stable id, e.g. `"hero"`.
    pub id: TargetId,
    /// English display name, e.g. `"Hero"`.
    pub label: String,
    /// Actions this kind of target can perform. Runtime availability is the
    /// intersection of this list and the live state's unlocked actions.
    pub actions: Vec<ActionId>,
}

impl TargetTemplate {
    /// Builds a template from an id and a display label.
    pub fn new(id: impl Into<TargetId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            actions: Vec::new(),
        }
    }

    /// Adds one compatible action to this target definition.
    pub fn with_action(mut self, action: impl Into<ActionId>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Whether this target kind can perform `action`, independently of whether
    /// it has been unlocked in a particular saved game.
    pub fn supports(&self, action: &ActionId) -> bool {
        self.actions.contains(action)
    }
}

/// Immutable definition of a player settlement.
#[derive(Debug, Clone)]
pub struct SettlementTemplate {
    /// Stable id, e.g. `"awakening_shore"`.
    pub id: SettlementId,
    /// English display name, e.g. `"Awakening Shore"`.
    pub label: String,
}

impl SettlementTemplate {
    /// Builds a settlement from an id and display label.
    pub fn new(id: impl Into<SettlementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Immutable definition of a place where actions can be performed.
#[derive(Debug, Clone)]
pub struct LocationTemplate {
    /// Stable id, e.g. `"nearby_woods"`.
    pub id: LocationId,
    /// English display name, e.g. `"Nearby Woods"`.
    pub label: String,
    /// Actions available at this location.
    pub actions: Vec<ActionId>,
}

impl LocationTemplate {
    /// Builds a location from an id and display label.
    pub fn new(id: impl Into<LocationId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            actions: Vec::new(),
        }
    }

    /// Adds one compatible action to this location definition.
    pub fn with_action(mut self, action: impl Into<ActionId>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Whether this location supports `action`.
    pub fn supports(&self, action: &ActionId) -> bool {
        self.actions.contains(action)
    }
}

/// Immutable definition of an action: its label and how long it runs (ticks).
#[derive(Debug, Clone)]
pub struct ActionTemplate {
    /// Stable id, e.g. `"gather"`.
    pub id: ActionId,
    /// English display name, e.g. `"Gather"`.
    pub label: String,
    /// Ticks this action takes to complete; used to seed a
    /// [`Progress`](crate::Progress).
    pub goal_ticks: u64,
}

impl ActionTemplate {
    /// Builds a template from an id, a display label, and a duration in ticks.
    pub fn new(id: impl Into<ActionId>, label: impl Into<String>, goal_ticks: u64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            goal_ticks,
        }
    }
}

/// Immutable definition of a collectible resource.
#[derive(Debug, Clone)]
pub struct ResourceTemplate {
    /// Stable id, e.g. `"seaweed_fragment"`.
    pub id: ResourceId,
    /// English display name, e.g. `"Seaweed Fragment"`.
    pub label: String,
}

impl ResourceTemplate {
    /// Builds a resource from an id and display label.
    pub fn new(id: impl Into<ResourceId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// One aggregated resource awarded by a completed action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reward {
    /// Resource template to award.
    pub resource: ResourceId,
    /// Number of units awarded.
    pub amount: u64,
}

/// One independently rolled resource in a [`RewardTable`].
#[derive(Debug, Clone)]
pub struct RewardEntry {
    /// Resource template to award when this entry succeeds.
    pub resource: ResourceId,
    /// Number of units awarded when this entry succeeds.
    pub amount: u64,
    /// Integer percentage chance in `1..=100`.
    pub chance: u32,
}

/// Ordered independent reward rolls for one Location and Action combination.
#[derive(Debug, Clone)]
pub struct RewardTable {
    /// Location whose completion uses this table.
    pub location: LocationId,
    /// Action whose completion uses this table.
    pub action: ActionId,
    entries: Vec<RewardEntry>,
}

impl RewardTable {
    /// Starts an empty reward table for a Location and Action combination.
    pub fn new(location: impl Into<LocationId>, action: impl Into<ActionId>) -> Self {
        Self {
            location: location.into(),
            action: action.into(),
            entries: Vec::new(),
        }
    }

    /// Adds an independently rolled resource with an integer percentage chance.
    pub fn with_resource(
        mut self,
        resource: impl Into<ResourceId>,
        amount: u64,
        chance: u32,
    ) -> Self {
        self.entries.push(RewardEntry {
            resource: resource.into(),
            amount,
            chance,
        });
        self
    }

    /// Iterates rewards in roll order.
    pub fn entries(&self) -> impl Iterator<Item = &RewardEntry> {
        self.entries.iter()
    }

    pub(crate) fn roll(&self, random: &mut impl RandomSource) -> Vec<Reward> {
        assert!(!self.entries.is_empty(), "reward table must not be empty");
        let mut rewards: Vec<Reward> = Vec::new();
        for entry in &self.entries {
            assert!(
                (1..=100).contains(&entry.chance),
                "reward chance must be in 1..=100"
            );
            if entry.chance == 100 || random.below(100) < entry.chance {
                if let Some(reward) = rewards
                    .iter_mut()
                    .find(|reward| reward.resource == entry.resource)
                {
                    reward.amount = reward.amount.saturating_add(entry.amount);
                } else {
                    rewards.push(Reward {
                        resource: entry.resource.clone(),
                        amount: entry.amount,
                    });
                }
            }
        }
        rewards
    }
}

/// The pool of all known content templates, looked up by id.
///
/// Targets and actions are *instantiated* from here into a
/// [`GameState`](crate::GameState); registering a new template makes content
/// available without touching live state. Templates are stored in `Vec`s so
/// iteration order is registration order, which the menus and column layout rely
/// on (a `HashMap` would scramble it).
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    targets: Vec<TargetTemplate>,
    settlements: Vec<SettlementTemplate>,
    locations: Vec<LocationTemplate>,
    actions: Vec<ActionTemplate>,
    resources: Vec<ResourceTemplate>,
    reward_tables: Vec<RewardTable>,
}

impl Catalog {
    /// An empty pool; use [`builtin`](Catalog::builtin) for the shipped content.
    pub fn new() -> Self {
        Self::default()
    }

    /// The content the game ships with today. Registration order is the menu
    /// order, so this reproduces the previous hard-coded behaviour exactly.
    pub fn builtin() -> Self {
        let mut catalog = Self::new();
        catalog.register_target(
            TargetTemplate::new("hero", "Hero")
                .with_action("gather")
                .with_action("fish")
                .with_action("hunt"),
        );
        catalog.register_settlement(SettlementTemplate::new(
            "awakening_shore",
            "Awakening Shore",
        ));
        catalog.register_location(
            LocationTemplate::new("first_shore", "First Shore")
                .with_action("gather")
                .with_action("fish"),
        );
        catalog.register_location(
            LocationTemplate::new("nearby_woods", "Nearby Woods")
                .with_action("gather")
                .with_action("hunt"),
        );
        catalog.register_location(
            LocationTemplate::new("nearby_hill", "Nearby Hill")
                .with_action("gather")
                .with_action("hunt"),
        );
        for (id, label) in [("gather", "Gather"), ("fish", "Fish"), ("hunt", "Hunt")] {
            catalog.register_action(ActionTemplate::new(id, label, seconds_to_ticks(10)));
        }
        for (id, label) in [
            ("pebble", "Pebble"),
            ("twig", "Twig"),
            ("grass", "Grass"),
            ("vine", "Vine"),
            ("small_fish", "Small Fish"),
            ("seaweed_fragment", "Seaweed Fragment"),
            ("small_fang", "Small Fang"),
            ("awful_meat", "Awful Meat"),
            ("tiny_magic_stone", "Tiny Magic Stone"),
        ] {
            catalog.register_resource(ResourceTemplate::new(id, label));
        }
        for table in [
            RewardTable::new("nearby_hill", "gather")
                .with_resource("grass", 1, 100)
                .with_resource("pebble", 1, 50),
            RewardTable::new("nearby_hill", "hunt")
                .with_resource("awful_meat", 1, 100)
                .with_resource("small_fang", 1, 60),
            RewardTable::new("nearby_woods", "gather")
                .with_resource("vine", 1, 60)
                .with_resource("twig", 1, 100),
            RewardTable::new("nearby_woods", "hunt")
                .with_resource("awful_meat", 1, 100)
                .with_resource("tiny_magic_stone", 1, 10),
            RewardTable::new("first_shore", "fish")
                .with_resource("small_fish", 1, 30)
                .with_resource("seaweed_fragment", 1, 100),
            RewardTable::new("first_shore", "gather")
                .with_resource("seaweed_fragment", 1, 60)
                .with_resource("pebble", 1, 100),
        ] {
            catalog.register_reward_table(table);
        }
        catalog
    }

    /// Adds a target template to the pool. Duplicate ids are a content bug and
    /// are not deduplicated here.
    pub fn register_target(&mut self, template: TargetTemplate) {
        self.targets.push(template);
    }

    /// Adds a settlement template to the pool.
    pub fn register_settlement(&mut self, template: SettlementTemplate) {
        self.settlements.push(template);
    }

    /// Adds a location template to the pool.
    pub fn register_location(&mut self, template: LocationTemplate) {
        self.locations.push(template);
    }

    /// Adds an action template to the pool. Duplicate ids are a content bug and
    /// are not deduplicated here.
    pub fn register_action(&mut self, template: ActionTemplate) {
        self.actions.push(template);
    }

    /// Adds a resource template to the pool.
    pub fn register_resource(&mut self, template: ResourceTemplate) {
        self.resources.push(template);
    }

    /// Adds one Location and Action reward table.
    pub fn register_reward_table(&mut self, table: RewardTable) {
        self.reward_tables.push(table);
    }

    /// Looks up a target template by id, or `None` for unknown content.
    pub fn target(&self, id: &TargetId) -> Option<&TargetTemplate> {
        self.targets.iter().find(|t| &t.id == id)
    }

    /// Looks up a settlement template by id, or `None` for unknown content.
    pub fn settlement(&self, id: &SettlementId) -> Option<&SettlementTemplate> {
        self.settlements
            .iter()
            .find(|settlement| &settlement.id == id)
    }

    /// Looks up an action template by id, or `None` for unknown content.
    pub fn action(&self, id: &ActionId) -> Option<&ActionTemplate> {
        self.actions.iter().find(|a| &a.id == id)
    }

    /// Looks up a location template by id, or `None` for unknown content.
    pub fn location(&self, id: &LocationId) -> Option<&LocationTemplate> {
        self.locations.iter().find(|location| &location.id == id)
    }

    /// Looks up a resource template by id, or `None` for unknown content.
    pub fn resource(&self, id: &ResourceId) -> Option<&ResourceTemplate> {
        self.resources.iter().find(|resource| &resource.id == id)
    }

    /// Looks up the reward table for a Location and Action combination.
    pub fn reward_table(&self, location: &LocationId, action: &ActionId) -> Option<&RewardTable> {
        self.reward_tables
            .iter()
            .find(|table| &table.location == location && &table.action == action)
    }

    /// Iterates target templates in registration (= menu) order.
    pub fn targets(&self) -> impl Iterator<Item = &TargetTemplate> {
        self.targets.iter()
    }

    /// Iterates settlement templates in registration order.
    pub fn settlements(&self) -> impl Iterator<Item = &SettlementTemplate> {
        self.settlements.iter()
    }

    /// Iterates location templates in registration (= menu) order.
    pub fn locations(&self) -> impl Iterator<Item = &LocationTemplate> {
        self.locations.iter()
    }

    /// Iterates action templates in registration (= menu) order.
    pub fn actions(&self) -> impl Iterator<Item = &ActionTemplate> {
        self.actions.iter()
    }

    /// Iterates resource templates in registration order.
    pub fn resources(&self) -> impl Iterator<Item = &ResourceTemplate> {
        self.resources.iter()
    }

    /// Iterates reward tables in registration order.
    pub fn reward_tables(&self) -> impl Iterator<Item = &RewardTable> {
        self.reward_tables.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SequenceRandom {
        values: Vec<u32>,
        consumed: usize,
    }

    impl SequenceRandom {
        fn new(values: impl IntoIterator<Item = u32>) -> Self {
            Self {
                values: values.into_iter().collect(),
                consumed: 0,
            }
        }
    }

    impl RandomSource for SequenceRandom {
        fn below(&mut self, upper_exclusive: u32) -> u32 {
            let value = self.values[self.consumed];
            self.consumed += 1;
            assert!(value < upper_exclusive);
            value
        }
    }

    #[test]
    fn menu_lists_are_non_empty() {
        let catalog = Catalog::builtin();
        assert!(catalog.targets().next().is_some());
        assert!(catalog.settlements().next().is_some());
        assert!(catalog.locations().next().is_some());
        assert!(catalog.actions().next().is_some());
    }

    #[test]
    fn builtin_seeds_expected_content() {
        let catalog = Catalog::builtin();

        let target_ids: Vec<&str> = catalog.targets().map(|t| t.id.as_str()).collect();
        assert_eq!(target_ids, ["hero"]);
        let target_labels: Vec<&str> = catalog.targets().map(|t| t.label.as_str()).collect();
        assert_eq!(target_labels, ["Hero"]);

        let settlements: Vec<(&str, &str)> = catalog
            .settlements()
            .map(|settlement| (settlement.id.as_str(), settlement.label.as_str()))
            .collect();
        assert_eq!(settlements, [("awakening_shore", "Awakening Shore")]);

        let location_ids: Vec<&str> = catalog.locations().map(|l| l.id.as_str()).collect();
        assert_eq!(location_ids, ["first_shore", "nearby_woods", "nearby_hill"]);
        let location_labels: Vec<&str> = catalog.locations().map(|l| l.label.as_str()).collect();
        assert_eq!(
            location_labels,
            ["First Shore", "Nearby Woods", "Nearby Hill"]
        );

        let action_ids: Vec<&str> = catalog.actions().map(|a| a.id.as_str()).collect();
        assert_eq!(action_ids, ["gather", "fish", "hunt"]);
        assert_eq!(
            catalog.action(&ActionId::new("gather")).unwrap().label,
            "Gather"
        );

        // No duplicate target ids in the shipped pool.
        let mut ids = target_ids.clone();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "duplicate target ids: {target_ids:?}");
    }

    #[test]
    fn catalog_lookup_hits_and_misses() {
        let catalog = Catalog::builtin();
        assert!(catalog.target(&TargetId::new("hero")).is_some());
        assert!(catalog.target(&TargetId::new("dragon")).is_none());
        assert!(
            catalog
                .settlement(&SettlementId::new("awakening_shore"))
                .is_some()
        );
        assert!(catalog.settlement(&SettlementId::new("camp")).is_none());
        assert!(catalog.location(&LocationId::new("first_shore")).is_some());
        assert!(catalog.location(&LocationId::new("volcano")).is_none());
        assert!(catalog.action(&ActionId::new("gather")).is_some());
        assert!(catalog.action(&ActionId::new("mining")).is_none());
    }

    #[test]
    fn builtin_compatibility_matches_starting_content() {
        let catalog = Catalog::builtin();
        let hero = catalog.target(&TargetId::new("hero")).unwrap();
        assert!(hero.supports(&ActionId::new("gather")));
        assert!(hero.supports(&ActionId::new("fish")));
        assert!(hero.supports(&ActionId::new("hunt")));
        let shore = catalog.location(&LocationId::new("first_shore")).unwrap();
        assert!(shore.supports(&ActionId::new("gather")));
        assert!(shore.supports(&ActionId::new("fish")));
        assert!(!shore.supports(&ActionId::new("hunt")));
    }

    #[test]
    fn every_builtin_action_takes_ten_seconds() {
        let catalog = Catalog::builtin();
        for action in catalog.actions() {
            assert_eq!(action.goal_ticks, seconds_to_ticks(10));
            assert_eq!(
                crate::Progress::new(action.goal_ticks).goal(),
                action.goal_ticks
            );
        }
    }

    #[test]
    fn builtin_resources_are_unique_and_reward_references_resolve() {
        let catalog = Catalog::builtin();
        let resource_ids: Vec<&str> = catalog
            .resources()
            .map(|resource| resource.id.as_str())
            .collect();
        assert_eq!(
            resource_ids,
            [
                "pebble",
                "twig",
                "grass",
                "vine",
                "small_fish",
                "seaweed_fragment",
                "small_fang",
                "awful_meat",
                "tiny_magic_stone",
            ]
        );
        let mut unique = resource_ids.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), resource_ids.len());

        let mut reward_pairs = Vec::new();
        for table in catalog.reward_tables() {
            assert!(catalog.location(&table.location).is_some());
            assert!(catalog.action(&table.action).is_some());
            assert!(table.entries().next().is_some());
            for entry in table.entries() {
                assert!((1..=100).contains(&entry.chance));
                assert!(catalog.resource(&entry.resource).is_some());
                assert_eq!(entry.amount, 1);
            }
            reward_pairs.push((table.location.as_str(), table.action.as_str()));
        }
        assert_eq!(catalog.reward_tables().count(), 6);
        reward_pairs.sort_unstable();
        reward_pairs.dedup();
        assert_eq!(reward_pairs.len(), 6, "duplicate reward-table key");

        for location in catalog.locations() {
            for action in &location.actions {
                assert!(
                    catalog.reward_table(&location.id, action).is_some(),
                    "missing reward table for {}/{}",
                    location.id.as_str(),
                    action.as_str()
                );
            }
        }
    }

    #[test]
    fn builtin_reward_entries_match_the_shipped_balance() {
        let catalog = Catalog::builtin();
        let expected = [
            ("nearby_hill", "gather", [("grass", 100), ("pebble", 50)]),
            (
                "nearby_hill",
                "hunt",
                [("awful_meat", 100), ("small_fang", 60)],
            ),
            ("nearby_woods", "gather", [("vine", 60), ("twig", 100)]),
            (
                "nearby_woods",
                "hunt",
                [("awful_meat", 100), ("tiny_magic_stone", 10)],
            ),
            (
                "first_shore",
                "fish",
                [("small_fish", 30), ("seaweed_fragment", 100)],
            ),
            (
                "first_shore",
                "gather",
                [("seaweed_fragment", 60), ("pebble", 100)],
            ),
        ];

        for (location, action, entries) in expected {
            let actual: Vec<(&str, u64, u32)> = catalog
                .reward_table(&LocationId::new(location), &ActionId::new(action))
                .unwrap()
                .entries()
                .map(|entry| (entry.resource.as_str(), entry.amount, entry.chance))
                .collect();
            assert_eq!(
                actual,
                entries
                    .map(|(resource, chance)| (resource, 1, chance))
                    .as_slice()
            );
        }
    }

    #[test]
    fn reward_rolls_are_independent_and_preserve_success_order() {
        let table = RewardTable::new("place", "act")
            .with_resource("first", 1, 40)
            .with_resource("certain", 2, 100)
            .with_resource("last", 3, 60);
        let mut random = SequenceRandom::new([40, 59]);

        assert_eq!(
            table.roll(&mut random),
            [
                Reward {
                    resource: ResourceId::new("certain"),
                    amount: 2,
                },
                Reward {
                    resource: ResourceId::new("last"),
                    amount: 3,
                },
            ]
        );
        assert_eq!(random.consumed, 2);
    }

    #[test]
    fn reward_roll_can_return_empty() {
        let table = RewardTable::new("place", "act")
            .with_resource("first", 1, 40)
            .with_resource("second", 1, 60);
        assert!(table.roll(&mut SequenceRandom::new([40, 60])).is_empty());
    }

    #[test]
    fn duplicate_resource_rewards_are_aggregated_at_first_success() {
        let table = RewardTable::new("place", "act")
            .with_resource("pebble", 1, 50)
            .with_resource("grass", 1, 100)
            .with_resource("pebble", 2, 100);

        assert_eq!(
            table.roll(&mut SequenceRandom::new([0])),
            [
                Reward {
                    resource: ResourceId::new("pebble"),
                    amount: 3,
                },
                Reward {
                    resource: ResourceId::new("grass"),
                    amount: 1,
                },
            ]
        );
    }

    #[test]
    fn certain_rewards_do_not_consume_randomness() {
        let table = RewardTable::new("place", "act").with_resource("certain", 1, 100);
        let mut random = SequenceRandom::new([]);

        assert_eq!(table.roll(&mut random).len(), 1);
        assert_eq!(random.consumed, 0);
    }
}
