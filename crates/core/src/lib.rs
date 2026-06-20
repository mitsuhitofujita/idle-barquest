//! Core game logic for idle-barquest.
//!
//! This crate holds the pure, terminal-agnostic game state and simulation so
//! that both the TUI front-end (`barquest-tui`) and the tools (`barquest-tools`)
//! can share and unit-test the same logic. Time is modelled as discrete
//! **ticks** here; wall-clock pacing and rendering live in the front-ends.

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

/// Who or what an action is performed on.
///
/// Several targets exist so the player can run actions on them in parallel —
/// the hero adventures while the adventurer and farmer work on their own tasks.
/// Richer per-target state (stats, multiple of each) is deferred to later work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The player's own avatar.
    Hero,
    /// A guild-managed worker for exploration and hunting.
    Adventurer,
    /// A villager who works the land.
    Farmer,
}

impl Target {
    /// Every target the player can currently choose, in menu order.
    pub const ALL: &'static [Target] = &[Target::Hero, Target::Adventurer, Target::Farmer];

    /// English display name, e.g. `"Hero"`.
    pub fn label(self) -> &'static str {
        match self {
            Target::Hero => "Hero",
            Target::Adventurer => "Adventurer",
            Target::Farmer => "Farmer",
        }
    }

    /// First-letter selection key (the label's first char, ASCII-lowercased).
    pub fn hotkey(self) -> char {
        first_hotkey(self.label())
    }
}

/// What the player does with the chosen target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Send the target to explore the forest.
    ForestExploration,
}

impl Action {
    /// Every action the player can currently choose, in menu order.
    pub const ALL: &'static [Action] = &[Action::ForestExploration];

    /// English display name, e.g. `"Forest Exploration"`.
    pub fn label(self) -> &'static str {
        match self {
            Action::ForestExploration => "Forest Exploration",
        }
    }

    /// First-letter selection key (the label's first char, ASCII-lowercased).
    pub fn hotkey(self) -> char {
        first_hotkey(self.label())
    }

    /// Ticks this action takes to complete; used to seed a [`Progress`].
    pub fn goal_ticks(self) -> u64 {
        match self {
            Action::ForestExploration => seconds_to_ticks(10),
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
        assert!(!Target::ALL.is_empty());
        assert!(!Action::ALL.is_empty());
    }

    #[test]
    fn target_hotkeys_are_unique() {
        // First-letter hotkeys must not collide within the target menu
        // (ADR 0005): Hero/Adventurer/Farmer -> h/a/f.
        let mut keys: Vec<char> = Target::ALL.iter().map(|t| t.hotkey()).collect();
        keys.sort_unstable();
        let count = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), count, "target hotkeys collide: {keys:?}");
    }

    #[test]
    fn hotkeys_are_derived_from_labels() {
        assert_eq!(Target::Hero.hotkey(), 'h');
        assert_eq!(Action::ForestExploration.hotkey(), 'f');
        for &target in Target::ALL {
            let first = target
                .label()
                .chars()
                .next()
                .map(|c| c.to_ascii_lowercase());
            assert_eq!(Some(target.hotkey()), first);
        }
        for &action in Action::ALL {
            let first = action
                .label()
                .chars()
                .next()
                .map(|c| c.to_ascii_lowercase());
            assert_eq!(Some(action.hotkey()), first);
        }
    }

    #[test]
    fn action_goal_seeds_a_valid_progress() {
        let goal = Action::ForestExploration.goal_ticks();
        assert_eq!(goal, seconds_to_ticks(10));
        assert!(goal > 0);
        assert_eq!(Progress::new(goal).goal(), goal);
    }
}
