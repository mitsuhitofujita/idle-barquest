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
}
