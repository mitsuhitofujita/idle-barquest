//! Placeholder game-balance simulator.
//!
//! Tools live as separate binaries under `src/bin/` and reuse `barquest-core`,
//! the same logic the game runs on — here run headless (no terminal) to prove
//! the tick model advances to completion through the real data-driven model
//! (`Catalog` + `GameState`). Replace with a real simulation later.

use barquest_core::{
    ActionId, Catalog, GameEvent, GameState, LocationId, TICKS_PER_SECOND, TargetId,
};

fn main() {
    let catalog = Catalog::builtin();
    let mut state = GameState::seeded(&catalog);
    let hero = TargetId::new("hero");
    let shore = LocationId::new("first_shore");
    let gather = ActionId::new("gather");
    state.assign_action(&catalog, &hero, &shore, &gather);

    // Advance frame-by-frame (100 ticks) until the hero's quest completes. The
    // quest is removed on completion, so we watch the returned events.
    let mut ticks = 0u64;
    loop {
        let events = state.advance(100);
        ticks += 100;
        let done = events.iter().any(|event| {
            matches!(
                event,
                GameEvent::QuestCompleted { target, location, action }
                    if *target == hero && *location == shore && *action == gather
            )
        });
        if done {
            println!(
                "barquest balance-sim: complete at {ticks} ticks ({} s)",
                ticks / TICKS_PER_SECOND,
            );
            break;
        }
    }
}
