//! Core game logic for idle-barquest.
//!
//! This crate holds the pure, terminal-agnostic game state and simulation so
//! that both the TUI front-end (`barquest-tui`) and the tools (`barquest-tools`)
//! can share and unit-test the same logic.

/// Returns the game's greeting banner.
pub fn greeting() -> &'static str {
    "Hello, World."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_is_hello_world() {
        assert_eq!(greeting(), "Hello, World.");
    }
}
