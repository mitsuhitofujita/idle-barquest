//! Translation layer: raw `crossterm` events -> the internal [`Input`] the app
//! reacts to. Keeping this separate from behaviour (see [`crate::app`]) lets the
//! menu/state logic be tested without constructing terminal events, and keeps
//! most of the code independent of `crossterm` details (see
//! `docs/design/tui-test-policy.md`).

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

/// A terminal event reduced to the only things the game acts on. Everything the
/// UI ignores (key releases, unknown keys, mouse, resize, ...) becomes
/// [`Input::Ignored`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Input {
    /// The player asked to quit (`q`, `Esc`, or `Ctrl-C`).
    Quit,
    /// The player picked the 0-based entry `n` in the active choices column.
    Select(usize),
    /// Nothing the game reacts to.
    Ignored,
}

/// Reduces a raw terminal event to an [`Input`]. Only key *presses* count: `q` /
/// `Esc` / `Ctrl-C` quit, a digit selects an entry, and anything else is ignored.
pub(crate) fn translate(event: &Event) -> Input {
    let Event::Key(key) = event else {
        return Input::Ignored;
    };
    if key.kind != KeyEventKind::Press {
        return Input::Ignored;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Input::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Input::Quit,
        KeyCode::Char(c) => digit_index(c).map_or(Input::Ignored, Input::Select),
        _ => Input::Ignored,
    }
}

/// Maps a digit key to a 0-based choice index: `'1'..='9'` -> `0..=8`, `'0'` ->
/// `9` (the tenth). Any other character is not a selection.
fn digit_index(c: char) -> Option<usize> {
    match c {
        '1'..='9' => Some(c as usize - '1' as usize),
        '0' => Some(9),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    fn press(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
    }

    #[test]
    fn digit_index_maps_number_keys() {
        assert_eq!(digit_index('1'), Some(0));
        assert_eq!(digit_index('9'), Some(8));
        assert_eq!(digit_index('0'), Some(9));
        assert_eq!(digit_index('a'), None);
        assert_eq!(digit_index(')'), None);
    }

    #[test]
    fn quit_keys_translate_to_quit() {
        assert_eq!(translate(&press('q')), Input::Quit);

        let esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(translate(&esc), Input::Quit);

        let ctrl_c = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(translate(&ctrl_c), Input::Quit);
    }

    #[test]
    fn digits_translate_to_a_zero_based_select() {
        assert_eq!(translate(&press('1')), Input::Select(0));
        assert_eq!(translate(&press('9')), Input::Select(8));
        assert_eq!(translate(&press('0')), Input::Select(9));
    }

    #[test]
    fn plain_c_and_unknown_keys_are_ignored() {
        // 'c' without CONTROL is just an unknown key, not a quit.
        assert_eq!(translate(&press('c')), Input::Ignored);
        assert_eq!(translate(&press('x')), Input::Ignored);
    }

    #[test]
    fn non_press_events_are_ignored() {
        // A key *release* for a digit must not select anything.
        let release = Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('1'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        ));
        assert_eq!(translate(&release), Input::Ignored);
    }
}
