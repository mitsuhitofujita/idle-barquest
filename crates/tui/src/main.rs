//! idle-barquest terminal front-end.
//!
//! Drives the command loop and renders the game state from `barquest-core`.
//! Each target (`Hero`, `Adventurer`, `Farmer`) has its own progress bar shown
//! at the top of the screen; the player picks a target (`H)ero`) and an action
//! (`F)orest Exploration`) via first-letter hotkeys in the menu at the bottom,
//! and the bars fill **concurrently** like an `apt` / `mise` update. `q` / `Esc`
//! / `Ctrl-C` quits from any screen.

use std::io;
use std::time::{Duration, Instant};

use barquest_core::{Action, Progress, Target};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

/// Real-time cadence of one render/update frame.
const FRAME: Duration = Duration::from_millis(100);
/// Game-time advanced per frame. `FRAME` (100 ms) == 100 ticks at 1000 ticks/s.
const TICKS_PER_FRAME: u64 = 100;
/// Inner cell count of the `[===>---]` progress bar (brackets excluded).
const BAR_WIDTH: usize = 20;
/// Lines reserved for the bottom menu block: title, choices, hint.
const MENU_HEIGHT: u16 = 3;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

/// A running (or just-finished) action assigned to a target.
struct Quest {
    action: Action,
    progress: Progress,
}

/// One target's row: the target plus its current quest, if any.
struct Slot {
    target: Target,
    quest: Option<Quest>,
}

/// Which menu the bottom of the screen is currently showing.
enum Menu {
    /// Choose which target to command next.
    SelectTarget,
    /// Choose an action for the already-chosen `target`.
    SelectAction { target: Target },
}

/// Runs the unified loop: advance every target's quest each frame while the
/// player assigns actions via the bottom menu. Returns when the player quits.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut slots: Vec<Slot> = Target::ALL
        .iter()
        .map(|&target| Slot {
            target,
            quest: None,
        })
        .collect();
    let mut menu = Menu::SelectTarget;
    let start = Instant::now();
    let mut frame_idx: u32 = 0;

    loop {
        terminal.draw(|frame| render(frame, &slots, &menu))?;

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
                if is_quit(&event) {
                    return Ok(());
                }
                handle_key(&event, &mut slots, &mut menu);
                // Redraw immediately so menu navigation feels responsive.
                terminal.draw(|frame| render(frame, &slots, &menu))?;
            }
        }

        for slot in &mut slots {
            if let Some(quest) = &mut slot.quest
                && !quest.progress.is_complete()
            {
                quest.progress.advance(TICKS_PER_FRAME);
            }
        }
    }
}

/// Applies a key press to the menu/slot state: target selection then action
/// assignment, which (re)starts that target's quest.
fn handle_key(event: &Event, slots: &mut [Slot], menu: &mut Menu) {
    let Event::Key(key) = event else {
        return;
    };
    if key.kind != KeyEventKind::Press {
        return;
    }
    let KeyCode::Char(c) = key.code else {
        return;
    };
    let pressed = c.to_ascii_lowercase();

    match *menu {
        Menu::SelectTarget => {
            if let Some(&target) = Target::ALL.iter().find(|t| t.hotkey() == pressed) {
                *menu = Menu::SelectAction { target };
            }
        }
        Menu::SelectAction { target } => {
            if let Some(&action) = Action::ALL.iter().find(|a| a.hotkey() == pressed) {
                if let Some(slot) = slots.iter_mut().find(|s| s.target == target) {
                    slot.quest = Some(Quest {
                        action,
                        progress: Progress::new(action.goal_ticks()),
                    });
                }
                *menu = Menu::SelectTarget;
            }
        }
    }
}

/// A choosable menu entry. Both core `Target` and `Action` implement it so a
/// single [`render_menu`] can list either menu's choices.
trait MenuItem: Copy {
    fn label(self) -> &'static str;
}

impl MenuItem for Target {
    fn label(self) -> &'static str {
        Target::label(self)
    }
}

impl MenuItem for Action {
    fn label(self) -> &'static str {
        Action::label(self)
    }
}

/// Lays out the screen: progress bars at the top, the command menu at the
/// bottom, separated by a flexible spacer.
fn render(frame: &mut Frame, slots: &[Slot], menu: &Menu) {
    let [top, _spacer, bottom] = Layout::vertical([
        Constraint::Length(slots.len() as u16),
        Constraint::Fill(1),
        Constraint::Length(MENU_HEIGHT),
    ])
    .areas(frame.area());

    render_progress(frame, top, slots);

    match *menu {
        Menu::SelectTarget => render_menu(frame, bottom, "Select target", Target::ALL),
        Menu::SelectAction { target } => {
            let title = format!("{} — select action", target.label());
            render_menu(frame, bottom, &title, Action::ALL);
        }
    }
}

/// Draws one left-aligned row per target: `target  action  [===>---]  NN%`.
fn render_progress(frame: &mut Frame, area: Rect, slots: &[Slot]) {
    let tw = column_width(Target::ALL.iter().map(|t| t.label()));
    let aw = column_width(Action::ALL.iter().map(|a| a.label()).chain(["—"]));

    let rows = slots
        .iter()
        .map(|slot| {
            let (action, ratio) = match &slot.quest {
                Some(quest) => (quest.action.label(), quest.progress.ratio()),
                None => ("—", 0.0),
            };
            let target = slot.target.label();
            let bar = progress_bar(ratio, BAR_WIDTH);
            let pct = (ratio * 100.0).round() as u16;
            format!("{target:<tw$}  {action:<aw$}  {bar} {pct:>3}%")
        })
        .collect::<Vec<_>>()
        .join("\n");

    frame.render_widget(
        Paragraph::new(rows).alignment(Alignment::Left),
        centered(area),
    );
}

/// Draws the bottom menu block: a title, a single line of `H)ero`-style
/// choices, and the shared quit hint.
fn render_menu<T: MenuItem>(frame: &mut Frame, area: Rect, title: &str, items: &[T]) {
    let [title_row, list_row, hint_row] = Layout::vertical([Constraint::Length(1); 3]).areas(area);

    frame.render_widget(
        Paragraph::new(title).alignment(Alignment::Center),
        centered(title_row),
    );

    let list = items
        .iter()
        .map(|item| hotkey_label(item.label()))
        .collect::<Vec<_>>()
        .join("   ");
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

/// Widest label (in chars) across an iterator, used to align text columns.
fn column_width<'a>(labels: impl Iterator<Item = &'a str>) -> usize {
    labels.map(|l| l.chars().count()).max().unwrap_or(0)
}

/// Builds an `apt`/`mise`-style bar like `[===>---]` for `ratio` (0.0..=1.0).
///
/// Filled cells are `=`, the leading edge is `>` while in progress, and the
/// remainder is `-`. `ratio` is clamped so out-of-range values stay valid.
fn progress_bar(ratio: f64, width: usize) -> String {
    let filled = ((ratio * width as f64).round() as i64).clamp(0, width as i64) as usize;
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for i in 0..width {
        let pos = i + 1;
        let cell = if pos < filled {
            '=' // fully behind the leading edge
        } else if pos == filled && filled < width {
            '>' // the moving leading edge
        } else if i < filled {
            '=' // last cell of a completed (full) bar
        } else {
            '-' // not yet reached
        };
        bar.push(cell);
    }
    bar.push(']');
    bar
}

/// Formats a label as a hotkey choice: `"Hero"` -> `"H)ero"`.
fn hotkey_label(label: &str) -> String {
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => format!("{first}){}", chars.as_str()),
        None => String::new(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_empty_is_all_dashes() {
        assert_eq!(progress_bar(0.0, 7), "[-------]");
    }

    #[test]
    fn progress_bar_full_is_all_equals() {
        assert_eq!(progress_bar(1.0, 7), "[=======]");
    }

    #[test]
    fn progress_bar_half_shows_leading_edge() {
        assert_eq!(progress_bar(0.5, 7), "[===>---]");
    }

    #[test]
    fn progress_bar_clamps_out_of_range() {
        assert_eq!(progress_bar(-1.0, 5), "[-----]");
        assert_eq!(progress_bar(2.0, 5), "[=====]");
    }

    #[test]
    fn column_width_picks_the_widest_label() {
        assert_eq!(
            column_width(["Hero", "Adventurer", "Farmer"].into_iter()),
            10
        );
        assert_eq!(column_width(std::iter::empty()), 0);
    }

    /// Flattens a full `render` pass to a single string of cell symbols so we
    /// can assert the screen contains the expected pieces, position-agnostic.
    fn rendered(slots: &[Slot], menu: &Menu) -> String {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut terminal = Terminal::new(TestBackend::new(100, 14)).unwrap();
        terminal.draw(|frame| render(frame, slots, menu)).unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn render_shows_a_bar_row_per_target_and_the_menu() {
        let mut running = Progress::new(Action::ForestExploration.goal_ticks());
        running.advance(running.goal() / 2); // 50%
        let slots = vec![
            Slot {
                target: Target::Hero,
                quest: None,
            },
            Slot {
                target: Target::Adventurer,
                quest: Some(Quest {
                    action: Action::ForestExploration,
                    progress: running,
                }),
            },
            Slot {
                target: Target::Farmer,
                quest: None,
            },
        ];

        let screen = rendered(&slots, &Menu::SelectTarget);

        // A row per target with the apt/mise-style bar and percent.
        for target in ["Hero", "Adventurer", "Farmer"] {
            assert!(screen.contains(target), "missing target row: {target}");
        }
        assert!(screen.contains("Forest Exploration"));
        assert!(screen.contains('[') && screen.contains(']'));
        assert!(screen.contains("50%"), "running quest should show 50%");
        // The command menu sits at the bottom.
        assert!(screen.contains("Select target"));
        assert!(screen.contains("H)ero"));
    }

    #[test]
    fn action_menu_titles_the_chosen_target() {
        let slots: Vec<Slot> = Target::ALL
            .iter()
            .map(|&target| Slot {
                target,
                quest: None,
            })
            .collect();

        let screen = rendered(
            &slots,
            &Menu::SelectAction {
                target: Target::Hero,
            },
        );

        assert!(screen.contains("select action"));
        assert!(screen.contains("F)orest Exploration"));
    }
}
