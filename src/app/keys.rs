//! Turns an OpenTUI key event into the core's `Key`, named the way OpenTUI Core names keys (Scouter reads those).

use opentui::input::KeyEventKind;
use opentui::prelude::*;

/// The key's name and text, owned; `Key` borrows them.
pub struct Named {
    pub name: String,
    pub text: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Named {
    pub fn key(&self) -> fmc::keys::Key<'_> {
        fmc::keys::Key { name: &self.name, text: &self.text, ctrl: self.ctrl, alt: self.alt, shift: self.shift }
    }
}

/// `None` for a key release.
pub fn name(event: &KeyEvent) -> Option<Named> {
    if event.kind == KeyEventKind::Release {
        return None;
    }
    let m = event.modifiers;
    let mut shift = m.contains(Modifiers::SHIFT);
    let (name, text) = match event.code {
        KeyCode::Char(' ') => ("space".to_string(), " ".to_string()),
        KeyCode::Char(c) => {
            shift |= c.is_uppercase();
            (c.to_lowercase().collect(), c.to_string())
        }
        KeyCode::Enter => ("return".into(), "\r".into()),
        KeyCode::LineFeed => ("linefeed".into(), "\n".into()),
        KeyCode::Tab => ("tab".into(), "\t".into()),
        KeyCode::BackTab => {
            shift = true;
            ("tab".into(), String::new())
        }
        KeyCode::Backspace => ("backspace".into(), String::new()),
        KeyCode::Escape => ("escape".into(), "\x1b".into()),
        KeyCode::Up => ("up".into(), String::new()),
        KeyCode::Down => ("down".into(), String::new()),
        KeyCode::Left => ("left".into(), String::new()),
        KeyCode::Right => ("right".into(), String::new()),
        KeyCode::Home => ("home".into(), String::new()),
        KeyCode::End => ("end".into(), String::new()),
        KeyCode::PageUp => ("pageup".into(), String::new()),
        KeyCode::PageDown => ("pagedown".into(), String::new()),
        KeyCode::Delete => ("delete".into(), String::new()),
        _ => (String::new(), String::new()),
    };
    let alt = m.contains(Modifiers::ALT) || m.contains(Modifiers::META);
    Some(Named { name, text, ctrl: m.contains(Modifiers::CTRL), alt, shift })
}
