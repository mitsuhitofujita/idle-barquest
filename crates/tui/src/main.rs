//! idle-barquest terminal front-end.
//!
//! Drives the command loop and renders the game state from `barquest-core`.
//! The player picks a target (`H)ero`) and an action (`F)orest Exploration`)
//! via first-letter hotkeys; the chosen action then runs as a full-screen
//! progress gauge. When it completes the command is done and control returns to
//! the target menu, so another command can be issued. `q` / `Esc` / `Ctrl-C`
//! quits from any screen.

use std::io;
use std::time::{Duration, Instant};

use barquest_core::{Action, Progress, TICKS_PER_SECOND, Target};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Gauge, Paragraph};
use ratatui::{DefaultTerminal, Frame};

/// Real-time cadence of one render/update frame.
const FRAME: Duration = Duration::from_millis(100);
/// Game-time advanced per frame. `FRAME` (100 ms) == 100 ticks at 1000 ticks/s.
const TICKS_PER_FRAME: u64 = 100;
/// How long the finished 100% frame is held before returning to the menu.
const COMPLETE_HOLD: Duration = Duration::from_millis(800);

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

/// Runs the command loop: select target, select action, run the quest, repeat.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    loop {
        let Some(target) = select(terminal, "Select target", Target::ALL)? else {
            return Ok(()); // quit
        };
        let Some(action) = select(terminal, "Select action", Action::ALL)? else {
            return Ok(()); // quit
        };
        if run_quest(terminal, target, action)? {
            return Ok(()); // quit requested during the quest
        }
        // Command complete: fall through and loop back to target selection.
    }
}

/// A choosable menu entry. Both core `Target` and `Action` implement it so a
/// single [`select`] handles both menus.
trait MenuItem: Copy {
    fn label(self) -> &'static str;
    fn hotkey(self) -> char;
}

impl MenuItem for Target {
    fn label(self) -> &'static str {
        Target::label(self)
    }
    fn hotkey(self) -> char {
        Target::hotkey(self)
    }
}

impl MenuItem for Action {
    fn label(self) -> &'static str {
        Action::label(self)
    }
    fn hotkey(self) -> char {
        Action::hotkey(self)
    }
}

/// Draws a hotkey menu and blocks until the player picks an item or quits.
///
/// Returns `Some(item)` for the matching first-letter key, or `None` if the
/// player asked to quit.
fn select<T: MenuItem>(
    terminal: &mut DefaultTerminal,
    title: &str,
    items: &[T],
) -> io::Result<Option<T>> {
    loop {
        terminal.draw(|frame| render_menu(frame, title, items))?;

        let event = event::read()?;
        if is_quit(&event) {
            return Ok(None);
        }
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if let KeyCode::Char(c) = key.code {
                let pressed = c.to_ascii_lowercase();
                if let Some(&item) = items.iter().find(|item| item.hotkey() == pressed) {
                    return Ok(Some(item));
                }
            }
        }
    }
}

/// Runs the action's progress bar to completion, redrawing once per frame.
///
/// Returns `true` if the player asked to quit mid-quest.
fn run_quest(terminal: &mut DefaultTerminal, target: Target, action: Action) -> io::Result<bool> {
    let mut quest = Progress::new(action.goal_ticks());
    let title = format!("{} — {}", target.label(), action.label());
    let start = Instant::now();
    let mut frame_idx: u32 = 0;

    loop {
        terminal.draw(|frame| render(frame, &title, &quest))?;

        if quest.is_complete() {
            // Hold the finished 100% frame briefly so it's visible.
            wait_until(Instant::now() + COMPLETE_HOLD)?;
            return Ok(false);
        }

        // Pin each frame to the wall clock so the gauge fills in real time,
        // independent of how long rendering takes.
        frame_idx += 1;
        if wait_until(start + FRAME * frame_idx)? {
            return Ok(true); // quit requested
        }
        quest.advance(TICKS_PER_FRAME);
    }
}

/// Draws a centered list of hotkey choices like `H)ero`, one per line.
fn render_menu<T: MenuItem>(frame: &mut Frame, title: &str, items: &[T]) {
    let [_, title_row, list_row, hint_row, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(items.len() as u16),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(title).alignment(Alignment::Center),
        centered(title_row),
    );

    let list = items
        .iter()
        .map(|item| hotkey_label(item.label()))
        .collect::<Vec<_>>()
        .join("\n");
    frame.render_widget(
        Paragraph::new(list).alignment(Alignment::Center),
        centered(list_row),
    );

    frame.render_widget(
        Paragraph::new("頭文字のキーで選択   —   q / Esc / Ctrl-C で終了")
            .alignment(Alignment::Center),
        centered(hint_row),
    );
}

/// Formats a label as a hotkey choice: `"Hero"` -> `"H)ero"`.
fn hotkey_label(label: &str) -> String {
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => format!("{first}){}", chars.as_str()),
        None => String::new(),
    }
}

/// Draws the centered progress gauge plus a tick/seconds detail line.
fn render(frame: &mut Frame, title: &str, quest: &Progress) {
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
        .block(Block::bordered().title(title))
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

/// Reserves horizontal margins so content isn't edge-to-edge.
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
        if event::poll(deadline - now)? && is_quit(&event::read()?) {
            return Ok(true);
        }
    }
}

/// Whether an event is a quit request: `q`, `Esc`, or `Ctrl-C`.
fn is_quit(event: &Event) -> bool {
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
