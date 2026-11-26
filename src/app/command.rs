//! The `:` command line (architecture.md, "`:q` quits, as in Neovim"): one line in the footer. `q`, `q!`, `qa`, `qa!`, and
//! `quit` quit on Enter; Esc or an empty Enter cancels; anything else is a notice.

use fmc::keys::Key;

use super::Model;
use crate::ui::footer::Message;

pub fn open(m: &mut Model) {
    m.command = Some(String::new());
}

/// A plain key while the line is open: typing edits it, Enter runs it. Returns true when the command quits.
pub fn prompt_key(m: &mut Model, key: Key<'_>) -> bool {
    match key.name {
        "escape" => m.command = None,
        "return" => {
            let text = m.command.take().unwrap_or_default();
            return run(m, &text);
        }
        "backspace" => {
            if let Some(text) = m.command.as_mut() {
                text.pop();
            }
        }
        _ if !key.text.is_empty() && !key.text.chars().any(char::is_control) => {
            if let Some(text) = m.command.as_mut() {
                text.push_str(key.text);
            }
        }
        _ => {}
    }
    false
}

/// Runs a command line; true when it quits. An empty line cancels, as Neovim's does.
fn run(m: &mut Model, text: &str) -> bool {
    match text.trim() {
        "" => false,
        "q" | "q!" | "qa" | "qa!" | "quit" => true,
        other => {
            m.message = Some(Message { text: format!("Not a command: {other}"), error: true });
            false
        }
    }
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
