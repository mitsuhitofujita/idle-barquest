//! Core game logic for idle-barquest.
//!
//! This crate holds the pure, terminal-agnostic game state and simulation so
//! that both the TUI front-end (`barquest-tui`) and the tools (`barquest-tools`)
//! can share and unit-test the same logic. Time is modelled as discrete
//! **ticks** here; wall-clock pacing and rendering live in the front-ends.
//!
//! Content is **data-driven**: a [`Catalog`] owns the pool of [`TargetTemplate`],
//! [`LocationTemplate`], and [`ActionTemplate`] definitions, and a [`GameState`] holds the live world
//! — a growable list of [`TargetInstance`]s spawned from those templates plus
//! the sets of unlocked locations and actions. Content references templates by string id
//! ([`TargetId`] / [`LocationId`] / [`ActionId`]) so new content can be added at runtime without
//! touching the type system, and the state stays trivially serialisable later
//! (JSON save/load is intentionally not implemented yet).
//!
//! The logic is split across small modules — [`mod@time`] (tick model and
//! [`Progress`]), [`mod@id`] (string ids), [`mod@catalog`] (content templates),
//! [`mod@random`] (caller-controlled reward randomness), and [`mod@state`] (the
//! live world) — and the full public API is re-exported flat from the crate
//! root, so consumers `use barquest_core::Catalog;` and friends regardless of
//! where each type lives.

mod catalog;
mod id;
mod random;
mod state;
mod time;

pub use catalog::{
    ActionTemplate, Catalog, LocationTemplate, ResourceTemplate, RewardEntry, RewardOutcome,
    RewardTable, TargetTemplate,
};
pub use id::{ActionId, LocationId, ResourceId, TargetId};
pub use random::{RandomSource, SeededRandom};
pub use state::{GameEvent, GameState, Quest, ResourceStack, TargetInstance};
pub use time::{Progress, TICKS_PER_SECOND, seconds_to_ticks};
