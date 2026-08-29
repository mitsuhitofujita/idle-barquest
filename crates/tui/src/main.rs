//! idle-barquest terminal front-end.
//!
//! Drives the command loop and renders the game state from `barquest-core`.
//! The world is data-driven: a `Catalog` supplies the target/action templates
//! and a `GameState` holds the live target instances. The screen is the fixed
//! five-region layout from `docs/design/terminal-ui-layout.md` (Title, Information
//! Log, User Choices, Progress, and Global Menu), separated by ASCII rules, and is
//! hidden behind a warning below the supported `80x24` terminal size. Progress
//! bars fill **concurrently** like an `apt` / `mise` update. The player picks a
//! target by letter, then an action by letter, in a progressive user-choices
//! panel (`>` marks the active column; `<` marks the chosen target); `q` / `Esc`
//! / `Ctrl-C` quits from any screen.
//!
//! The binary is split into three concerns: [`app`] owns the mutable game state
//! and its behaviour, [`input`] translates terminal events into intent, and
//! [`render`] projects state onto the terminal. `main`/`run` keep only the
//! wall-clock frame loop and terminal lifecycle.

mod app;
mod input;
mod render;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event;
use ratatui::DefaultTerminal;

use app::{App, TICKS_PER_FRAME};
use render::render;

/// Real-time cadence of one render/update frame. `FRAME` (100 ms) advances
/// `TICKS_PER_FRAME` ticks of game time (1000 ticks/s).
const FRAME: Duration = Duration::from_millis(100);

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

/// Runs the unified loop: advance every target's quest each frame while the
/// player assigns actions via the bottom menu. Returns when the player quits.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();
    let start = Instant::now();
    let mut frame_idx: u32 = 0;

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        // Drain input until the next frame boundary, pinned to the wall clock so
        // every bar fills in real time regardless of how long rendering takes.
        frame_idx += 1;
        let deadline = start + FRAME * frame_idx;
        loop {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            if event::poll(deadline - now)? {
                let event = event::read()?;
                if app.update(input::translate(&event)) {
                    return Ok(());
                }
                // Redraw immediately so menu navigation feels responsive.
                terminal.draw(|frame| render(frame, &app))?;
            }
        }

        app.advance(TICKS_PER_FRAME);
    }
}
