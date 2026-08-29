//! Data-driven content: immutable [`TargetTemplate`] / [`ActionTemplate`]
//! definitions and the [`Catalog`] pool that owns them.
//!
//! Content is kept separate from the live [`GameState`](crate::GameState):
//! templates are *instantiated* from the catalog into the state, and registering
//! a new template makes content available without touching live state.

use crate::id::{ActionId, TargetId};
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

/// Immutable definition of an action: its label and how long it runs (ticks).
#[derive(Debug, Clone)]
pub struct ActionTemplate {
    /// Stable id, e.g. `"forest_exploration"`.
    pub id: ActionId,
    /// English display name, e.g. `"Forest Exploration"`.
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
    actions: Vec<ActionTemplate>,
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
        catalog
            .register_target(TargetTemplate::new("hero", "Hero").with_action("forest_exploration"));
        catalog.register_target(
            TargetTemplate::new("adventurer", "Adventurer").with_action("forest_exploration"),
        );
        catalog.register_target(
            TargetTemplate::new("farmer", "Farmer").with_action("forest_exploration"),
        );
        catalog.register_action(ActionTemplate::new(
            "forest_exploration",
            "Forest Exploration",
            seconds_to_ticks(10),
        ));
        catalog
    }

    /// Adds a target template to the pool. Duplicate ids are a content bug and
    /// are not deduplicated here.
    pub fn register_target(&mut self, template: TargetTemplate) {
        self.targets.push(template);
    }

    /// Adds an action template to the pool. Duplicate ids are a content bug and
    /// are not deduplicated here.
    pub fn register_action(&mut self, template: ActionTemplate) {
        self.actions.push(template);
    }

    /// Looks up a target template by id, or `None` for unknown content.
    pub fn target(&self, id: &TargetId) -> Option<&TargetTemplate> {
        self.targets.iter().find(|t| &t.id == id)
    }

    /// Looks up an action template by id, or `None` for unknown content.
    pub fn action(&self, id: &ActionId) -> Option<&ActionTemplate> {
        self.actions.iter().find(|a| &a.id == id)
    }

    /// Iterates target templates in registration (= menu) order.
    pub fn targets(&self) -> impl Iterator<Item = &TargetTemplate> {
        self.targets.iter()
    }

    /// Iterates action templates in registration (= menu) order.
    pub fn actions(&self) -> impl Iterator<Item = &ActionTemplate> {
        self.actions.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_lists_are_non_empty() {
        let catalog = Catalog::builtin();
        assert!(catalog.targets().next().is_some());
        assert!(catalog.actions().next().is_some());
    }

    #[test]
    fn builtin_seeds_expected_content() {
        let catalog = Catalog::builtin();

        let target_ids: Vec<&str> = catalog.targets().map(|t| t.id.as_str()).collect();
        assert_eq!(target_ids, ["hero", "adventurer", "farmer"]);
        let target_labels: Vec<&str> = catalog.targets().map(|t| t.label.as_str()).collect();
        assert_eq!(target_labels, ["Hero", "Adventurer", "Farmer"]);

        let action_ids: Vec<&str> = catalog.actions().map(|a| a.id.as_str()).collect();
        assert_eq!(action_ids, ["forest_exploration"]);
        assert_eq!(
            catalog
                .action(&ActionId::new("forest_exploration"))
                .unwrap()
                .label,
            "Forest Exploration"
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
                .action(&ActionId::new("forest_exploration"))
                .is_some()
        );
        assert!(catalog.action(&ActionId::new("fishing")).is_none());
    }

    #[test]
    fn builtin_targets_support_the_shipped_action() {
        let catalog = Catalog::builtin();
        let forest = ActionId::new("forest_exploration");
        for target in catalog.targets() {
            assert!(target.supports(&forest));
        }
    }

    #[test]
    fn action_goal_seeds_a_valid_progress() {
        let catalog = Catalog::builtin();
        let goal = catalog
            .action(&ActionId::new("forest_exploration"))
            .unwrap()
            .goal_ticks;
        assert_eq!(goal, seconds_to_ticks(10));
        assert!(goal > 0);
        assert_eq!(crate::Progress::new(goal).goal(), goal);
    }
}
