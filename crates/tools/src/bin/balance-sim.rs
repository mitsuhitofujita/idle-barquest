//! Placeholder game-balance simulator.
//!
//! Tools live as separate binaries under `src/bin/` and reuse `barquest-core`,
//! the same logic the game runs on — here run headless (no terminal) to prove
//! the tick model advances to completion through the real data-driven model
//! (`Catalog` + `GameState`). Replace with a real simulation later.

use barquest_core::{ActionId, Catalog, GameState, TICKS_PER_SECOND, TargetId};

fn main() {
    let catalog = Catalog::builtin();
    let mut state = GameState::seeded(&catalog);
    let hero = TargetId::new("hero");
    state.assign_action(&catalog, &hero, &ActionId::new("forest_exploration"));

    // Advance frame-by-frame (100 ticks) until the hero's quest finishes.
    loop {
        state.advance(100);
        let quest = state
            .targets
            .iter()
            .find(|t| t.id == hero)
            .and_then(|t| t.quest.as_ref())
            .expect("hero has an assigned quest");
        if quest.progress.is_complete() {
            println!(
                "barquest balance-sim: complete at {} ticks ({} s)",
                quest.progress.elapsed(),
                quest.progress.elapsed() / TICKS_PER_SECOND,
            );
            break;
        }
    }
}
