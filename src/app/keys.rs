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
    pub cmd: bool,
}

impl Named {
    pub fn key(&self) -> fmc::keys::Key<'_> {
        fmc::keys::Key {
            name: &self.name,
            text: &self.text,
            ctrl: self.ctrl,
            alt: self.alt,
            shift: self.shift,
            cmd: self.cmd,
        }
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
    Some(Named { name, text, ctrl: m.contains(Modifiers::CTRL), alt, shift, cmd: m.contains(Modifiers::SUPER) })
}

/// The presses a tap on a footer hint sends, named by the hint's key (plan-00004, P2): the hint's own key, or its first
/// where it names several. `None` for a hint with no key to press; `^q` stays on the keyboard.
pub fn hint_presses(key: &str) -> Option<Vec<Named>> {
    let press = |name: &str, text: &str, ctrl: bool| Named {
        name: name.into(),
        text: text.into(),
        ctrl,
        alt: false,
        shift: false,
        cmd: false,
    };
    Some(match key {
        "Enter" => vec![press("return", "\r", false)],
        "Tab" => vec![press("tab", "\t", false)],
        "Esc" => vec![press("escape", "\x1b", false)],
        "↑ ↓" => vec![press("up", "", false)],
        "j k" => vec![press("j", "j", false)],
        "n N" => vec![press("n", "n", false)],
        "gg G" => vec![press("g", "g", false), press("g", "g", false)],
        "z z" => vec![press("z", "z", false), press("z", "z", false)],
        "^d ^u" => vec![press("d", "d", true)],
        "^o alt+i" => vec![press("o", "o", true)],
        "^/" => vec![press("/", "/", true)],
        single if single.chars().count() == 1 => vec![press(single, single, false)],
        _ => return None,
    })
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod tests;
