//! Core game logic for idle-barquest.
//!
//! This crate holds the pure, terminal-agnostic game state and simulation so
//! that both the TUI front-end (`barquest-tui`) and the tools (`barquest-tools`)
//! can share and unit-test the same logic. Time is modelled as discrete
//! **ticks** here; wall-clock pacing and rendering live in the front-ends.
//!
//! Content is **data-driven**: a [`Catalog`] owns the pool of [`TargetTemplate`]
//! and [`ActionTemplate`] definitions, and a [`GameState`] holds the live world
//! — a growable list of [`TargetInstance`]s spawned from those templates plus
//! the set of unlocked actions. Targets/actions reference templates by string id
//! ([`TargetId`] / [`ActionId`]) so new content can be added at runtime without
//! touching the type system, and the state stays trivially serialisable later
//! (JSON save/load is intentionally not implemented yet). It currently all lives
//! in one module; split into submodules once it outgrows a single file.

/// Game-time granularity: `1000` ticks make up one second.
///
/// A single tick is the smallest unit of in-game time (≈1 ms). The front-end
/// loop advances the simulation by some number of ticks each render frame.
pub const TICKS_PER_SECOND: u64 = 1000;

/// Converts whole seconds of game time to ticks.
pub const fn seconds_to_ticks(secs: u64) -> u64 {
    secs * TICKS_PER_SECOND
}

/// A single quest whose progress advances in ticks until it reaches its goal.
///
/// The type is pure: it never touches the clock or the terminal. Callers decide
/// how many ticks elapse per [`advance`](Progress::advance), so the same model
/// drives the real-time TUI and headless tool simulations identically.
#[derive(Debug, Clone)]
pub struct Progress {
    elapsed: u64,
    goal: u64,
}

impl Progress {
    /// Creates a quest that completes after `goal_ticks` ticks.
    ///
    /// The goal is clamped to at least `1` so [`ratio`](Progress::ratio) can
    /// never divide by zero.
    pub fn new(goal_ticks: u64) -> Self {
        Self {
            elapsed: 0,
            goal: goal_ticks.max(1),
        }
    }

    /// Advances progress by `ticks`, saturating at the goal.
    pub fn advance(&mut self, ticks: u64) {
        self.elapsed = self.elapsed.saturating_add(ticks).min(self.goal);
    }

    /// Ticks elapsed so far (never exceeds the goal).
    pub fn elapsed(&self) -> u64 {
        self.elapsed
    }

    /// Total ticks required to complete the quest.
    pub fn goal(&self) -> u64 {
        self.goal
    }

    /// Completion ratio in the range `0.0..=1.0`.
    pub fn ratio(&self) -> f64 {
        self.elapsed as f64 / self.goal as f64
    }

    /// Whether the quest has reached its goal.
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.goal
    }
}

/// Stable string id for a target template (and the instances spawned from it),
/// e.g. `"hero"`. Used as the join key between [`Catalog`] and [`GameState`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(pub String);

impl TargetId {
    /// Builds an id from anything string-like (`"hero"`, `String`, …).
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrows the id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TargetId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for TargetId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Stable string id for an action template, e.g. `"forest_exploration"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId(pub String);

impl ActionId {
    /// Builds an id from anything string-like.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrows the id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ActionId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for ActionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Immutable definition of a kind of target — content data, not live state.
#[derive(Debug, Clone)]
pub struct TargetTemplate {
    /// Stable id, e.g. `"hero"`.
    pub id: TargetId,
    /// English display name, e.g. `"Hero"`.
    pub label: String,
}

impl TargetTemplate {
    /// Builds a template from an id and a display label.
    pub fn new(id: impl Into<TargetId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }

    /// First-letter selection key (the label's first char, ASCII-lowercased).
    pub fn hotkey(&self) -> char {
        first_hotkey(&self.label)
    }
}

/// Immutable definition of an action: its label and how long it runs (ticks).
#[derive(Debug, Clone)]
pub struct ActionTemplate {
    /// Stable id, e.g. `"forest_exploration"`.
    pub id: ActionId,
    /// English display name, e.g. `"Forest Exploration"`.
    pub label: String,
    /// Ticks this action takes to complete; used to seed a [`Progress`].
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

    /// First-letter selection key (the label's first char, ASCII-lowercased).
    pub fn hotkey(&self) -> char {
        first_hotkey(&self.label)
    }
}

/// The pool of all known content templates, looked up by id.
///
/// Targets and actions are *instantiated* from here into a [`GameState`];
/// registering a new template makes content available without touching live
/// state. Templates are stored in `Vec`s so iteration order is registration
/// order, which the menus and column layout rely on (a `HashMap` would scramble
/// it).
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
        catalog.register_target(TargetTemplate::new("hero", "Hero"));
        catalog.register_target(TargetTemplate::new("adventurer", "Adventurer"));
        catalog.register_target(TargetTemplate::new("farmer", "Farmer"));
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

/// A running (or just-finished) action assigned to a target instance.
#[derive(Debug, Clone)]
pub struct Quest {
    /// Which action is running; resolve its label/duration via the [`Catalog`].
    pub action: ActionId,
    /// How far the action has progressed.
    pub progress: Progress,
}

/// One live target in the world: its own instance id, the template it was
/// spawned from, and its current quest (if any).
///
/// `id` is unique per instance while `template_id` says what kind it is; for
/// today's one-of-each world they happen to be equal. Label/hotkey lookups must
/// always go through `template_id`. A dedicated `InstanceId` type is the natural
/// upgrade once duplicates become common.
#[derive(Debug, Clone)]
pub struct TargetInstance {
    /// Unique id for this specific target instance.
    pub id: TargetId,
    /// The template this instance was spawned from.
    pub template_id: TargetId,
    /// The action currently assigned, if any.
    pub quest: Option<Quest>,
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
                quest: None,
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
            quest: None,
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

    /// Assigns (or restarts) an action on a target instance, seeding a fresh
    /// [`Progress`] from the action template's `goal_ticks`. Returns `false` if
    /// the action or the instance id is unknown. The menu guarantees the action
    /// is unlocked, so only catalog membership is validated here.
    pub fn assign_action(
        &mut self,
        catalog: &Catalog,
        instance: &TargetId,
        action: &ActionId,
    ) -> bool {
        let Some(template) = catalog.action(action) else {
            return false;
        };
        let goal = template.goal_ticks;
        let Some(target) = self.targets.iter_mut().find(|t| &t.id == instance) else {
            return false;
        };
        target.quest = Some(Quest {
            action: action.clone(),
            progress: Progress::new(goal),
        });
        true
    }

    /// Advances every active, incomplete quest by `ticks` (the loop step).
    pub fn advance(&mut self, ticks: u64) {
        for target in &mut self.targets {
            if let Some(quest) = &mut target.quest
                && !quest.progress.is_complete()
            {
                quest.progress.advance(ticks);
            }
        }
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

/// Derives a menu hotkey from a label: its first char, ASCII-lowercased.
fn first_hotkey(label: &str) -> char {
    label
        .chars()
        .next()
        .map(|c| c.to_ascii_lowercase())
        .unwrap_or(' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_convert_to_ticks() {
        assert_eq!(seconds_to_ticks(10), 10_000);
        assert_eq!(seconds_to_ticks(0), 0);
    }

    #[test]
    fn advance_accumulates_ticks() {
        let mut quest = Progress::new(1_000);
        quest.advance(100);
        quest.advance(150);
        assert_eq!(quest.elapsed(), 250);
        assert!(!quest.is_complete());
    }

    #[test]
    fn advance_saturates_at_goal() {
        let mut quest = Progress::new(1_000);
        quest.advance(900);
        quest.advance(500);
        assert_eq!(quest.elapsed(), 1_000);
        assert!(quest.is_complete());
    }

    #[test]
    fn ratio_spans_zero_to_one() {
        let mut quest = Progress::new(10_000);
        assert_eq!(quest.ratio(), 0.0);
        quest.advance(5_000);
        assert_eq!(quest.ratio(), 0.5);
        quest.advance(5_000);
        assert_eq!(quest.ratio(), 1.0);
    }

    #[test]
    fn goal_is_clamped_to_at_least_one() {
        let quest = Progress::new(0);
        assert_eq!(quest.goal(), 1);
        assert_eq!(quest.ratio(), 0.0);
    }

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
    fn target_hotkeys_are_unique() {
        // First-letter hotkeys must not collide within the seeded target menu
        // (ADR 0005): Hero/Adventurer/Farmer -> h/a/f. With data-driven content
        // this is no longer a compile-time guarantee but a property of the
        // shipped pool; resolving collisions in runtime-registered content is
        // future work.
        let catalog = Catalog::builtin();
        let mut keys: Vec<char> = catalog.targets().map(|t| t.hotkey()).collect();
        keys.sort_unstable();
        let count = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), count, "target hotkeys collide: {keys:?}");
    }

    #[test]
    fn hotkeys_are_derived_from_labels() {
        let catalog = Catalog::builtin();
        assert_eq!(
            catalog.target(&TargetId::new("hero")).unwrap().hotkey(),
            'h'
        );
        assert_eq!(
            catalog
                .action(&ActionId::new("forest_exploration"))
                .unwrap()
                .hotkey(),
            'f'
        );
        for target in catalog.targets() {
            let first = target.label.chars().next().map(|c| c.to_ascii_lowercase());
            assert_eq!(Some(target.hotkey()), first);
        }
        for action in catalog.actions() {
            let first = action.label.chars().next().map(|c| c.to_ascii_lowercase());
            assert_eq!(Some(action.hotkey()), first);
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
        assert_eq!(Progress::new(goal).goal(), goal);
    }

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
        assert!(state.targets.iter().all(|t| t.quest.is_none()));
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
    fn assign_action_seeds_progress() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let forest = ActionId::new("forest_exploration");

        assert!(state.assign_action(&catalog, &hero, &forest));
        let quest = state
            .targets
            .iter()
            .find(|t| t.id == hero)
            .unwrap()
            .quest
            .as_ref()
            .unwrap();
        assert_eq!(quest.action, forest);
        assert_eq!(quest.progress.goal(), seconds_to_ticks(10));
        assert_eq!(quest.progress.ratio(), 0.0);

        // Unknown instance or action are rejected.
        assert!(!state.assign_action(&catalog, &TargetId::new("ghost"), &forest));
        assert!(!state.assign_action(&catalog, &hero, &ActionId::new("fishing")));
    }

    #[test]
    fn advance_progresses_active_quests() {
        let catalog = Catalog::builtin();
        let mut state = GameState::seeded(&catalog);
        let hero = TargetId::new("hero");
        let forest = ActionId::new("forest_exploration");
        state.assign_action(&catalog, &hero, &forest);

        state.advance(seconds_to_ticks(5)); // half of the 10s goal
        let ratio = state
            .targets
            .iter()
            .find(|t| t.id == hero)
            .unwrap()
            .quest
            .as_ref()
            .unwrap()
            .progress
            .ratio();
        assert_eq!(ratio, 0.5);

        state.advance(seconds_to_ticks(100)); // overshoot saturates at the goal
        let progress = state
            .targets
            .iter()
            .find(|t| t.id == hero)
            .unwrap()
            .quest
            .as_ref()
            .unwrap()
            .progress
            .clone();
        assert!(progress.is_complete());
        assert_eq!(progress.elapsed(), seconds_to_ticks(10));

        // Idle targets stay questless.
        assert!(
            state
                .targets
                .iter()
                .filter(|t| t.id != hero)
                .all(|t| t.quest.is_none())
        );
    }
}
