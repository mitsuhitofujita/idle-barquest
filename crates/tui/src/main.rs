//! idle-barquest terminal front-end.
//!
//! Renders the game state from `barquest-core` every frame as a full-screen
//! progress gauge. One quest runs for [`QUEST_SECONDS`] seconds, then the
//! program restores the terminal and exits.

use std::io;
use std::time::{Duration, Instant};

use barquest_core::{Progress, TICKS_PER_SECOND, seconds_to_ticks};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Gauge, Paragraph};
use ratatui::{DefaultTerminal, Frame};

/// Real-time cadence of one render/update frame.
const FRAME: Duration = Duration::from_millis(100);
/// Game-time advanced per frame. `FRAME` (100 ms) == 100 ticks at 1000 ticks/s.
const TICKS_PER_FRAME: u64 = 100;
/// Length of the demo quest; reaches 100% after this many seconds.
const QUEST_SECONDS: u64 = 10;
/// How long the finished 100% frame is held before exiting.
const COMPLETE_HOLD: Duration = Duration::from_millis(800);

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

/// Drives the quest to completion, redrawing once per frame.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut quest = Progress::new(seconds_to_ticks(QUEST_SECONDS));
    let start = Instant::now();
    let mut frame_idx: u32 = 0;

    loop {
        terminal.draw(|frame| render(frame, &quest))?;

        if quest.is_complete() {
            // Hold the finished 100% frame briefly so it's visible on exit.
            wait_until(Instant::now() + COMPLETE_HOLD)?;
            break;
        }

        // Pin each frame to the wall clock so 100 frames ≈ QUEST_SECONDS,
        // independent of how long rendering takes.
        frame_idx += 1;
        if wait_until(start + FRAME * frame_idx)? {
            break; // quit requested
        }
        quest.advance(TICKS_PER_FRAME);
    }

    Ok(())
}

/// Draws the centered progress gauge plus a tick/seconds detail line.
fn render(frame: &mut Frame, quest: &Progress) {
    let [_, gauge_row, detail_row, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(frame.area());

    let gauge_area = centered(gauge_row);
    let detail_area = centered(detail_row);

    let percent = (quest.ratio() * 100.0).round() as u16;
    let gauge = Gauge::default()
        .block(Block::bordered().title("Quest: The Long Road"))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .ratio(quest.ratio())
        .label(format!("{percent}%"));
    frame.render_widget(gauge, gauge_area);

    let secs = quest.elapsed() as f64 / TICKS_PER_SECOND as f64;
    let total_secs = quest.goal() / TICKS_PER_SECOND;
    let detail = format!(
        "{} / {} ticks   ({secs:.1}s / {total_secs}s)   —   q / Esc / Ctrl-C で中断",
        quest.elapsed(),
        quest.goal(),
    );
    frame.render_widget(
        Paragraph::new(detail).alignment(Alignment::Center),
        detail_area,
    );
}

/// Reserves horizontal margins so the gauge isn't edge-to-edge.
fn centered(row: Rect) -> Rect {
    let [_, middle, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(80),
        Constraint::Fill(1),
    ])
    .areas(row);
    middle
}

/// Polls input until `deadline`, returning `true` if the user asked to quit.
fn wait_until(deadline: Instant) -> io::Result<bool> {
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Ok(false);
        }
        if event::poll(deadline - now)? && is_quit(event::read()?) {
            return Ok(true);
        }
    }
}

/// Whether an event is a quit request: `q`, `Esc`, or `Ctrl-C`.
fn is_quit(event: Event) -> bool {
    let Event::Key(key) = event else {
        return false;
    };
    if key.kind != KeyEventKind::Press {
        return false;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
        _ => false,
    }
}
