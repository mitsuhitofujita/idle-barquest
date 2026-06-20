//! Placeholder game-balance simulator.
//!
//! Tools live as separate binaries under `src/bin/` and reuse `barquest-core`,
//! the same logic the game runs on — here run headless (no terminal) to prove
//! the tick model advances to completion. Replace with a real simulation later.

use barquest_core::{Progress, TICKS_PER_SECOND, seconds_to_ticks};

fn main() {
    let mut quest = Progress::new(seconds_to_ticks(10));
    while !quest.is_complete() {
        quest.advance(100);
    }
    println!(
        "barquest balance-sim: complete at {} ticks ({} s)",
        quest.elapsed(),
        quest.elapsed() / TICKS_PER_SECOND,
    );
}
