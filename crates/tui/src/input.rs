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
    /// The player picked the 0-based entry represented by a letter in the
    /// active choices column (`a` is 0, `b` is 1, and so on).
    Select(usize),
    /// Return one stage in the choices flow (`Backspace`).
    Back,
    /// Move the acquired-material viewport one item toward Catalog start (`,`).
    PreviousMaterials,
    /// Move the acquired-material viewport one item toward Catalog end (`.`).
    NextMaterials,
    /// Nothing the game reacts to.
    Ignored,
}

/// Reduces a raw terminal event to an [`Input`]. Only key *presses* count: `q` /
/// `Esc` / `Ctrl-C` quit, Backspace returns one stage, a letter selects an
/// entry, and anything else is ignored.
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
        KeyCode::Backspace => Input::Back,
        KeyCode::Char(',') => Input::PreviousMaterials,
        KeyCode::Char('.') => Input::NextMaterials,
        KeyCode::Char(c) => selection_index(c).map_or(Input::Ignored, Input::Select),
        _ => Input::Ignored,
    }
}

/// Maps an ASCII letter to a 0-based choice index. Case is ignored; digits and
/// punctuation are not selections. `q` remains reserved for quitting above.
fn selection_index(c: char) -> Option<usize> {
    let c = c.to_ascii_lowercase();
    if c.is_ascii_lowercase() {
        Some(c as usize - 'a' as usize)
    } else {
        None
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
    fn selection_index_maps_letters_and_rejects_digits() {
        assert_eq!(selection_index('a'), Some(0));
        assert_eq!(selection_index('f'), Some(5));
        assert_eq!(selection_index('A'), Some(0));
        assert_eq!(selection_index('1'), None);
        assert_eq!(selection_index(')'), None);
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
    fn letters_translate_to_a_zero_based_select() {
        assert_eq!(translate(&press('a')), Input::Select(0));
        assert_eq!(translate(&press('b')), Input::Select(1));
        assert_eq!(translate(&press('F')), Input::Select(5));
        assert_eq!(translate(&press('1')), Input::Ignored);
    }

    #[test]
    fn backspace_translates_to_back() {
        let backspace = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(translate(&backspace), Input::Back);
    }

    #[test]
    fn punctuation_translates_to_material_navigation() {
        assert_eq!(translate(&press(',')), Input::PreviousMaterials);
        assert_eq!(translate(&press('.')), Input::NextMaterials);
    }

    #[test]
    fn plain_c_selects_while_unknown_keys_are_ignored() {
        assert_eq!(translate(&press('c')), Input::Select(2));
        assert_eq!(translate(&press('!')), Input::Ignored);
    }

    #[test]
    fn non_press_events_are_ignored() {
        // A key *release* for a letter must not select anything.
        let release = Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        ));
        assert_eq!(translate(&release), Input::Ignored);
    }
}
