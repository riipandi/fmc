//! The `:` command line, where `:q` quits as in Neovim: one line in the footer. `q`, `q!`, `qa`, `qa!`, and
//! `quit` quit on Enter; Esc or an empty Enter cancels; in the hex view any other line is an offset to go to, and
//! elsewhere it is a notice.

use fmc::keys::Key;
use fmc::line_edit::{Line, edit_of, is_text};

use super::Model;
use crate::ui::footer::Message;

pub fn open(m: &mut Model) {
    m.command = Some(Line::default());
}

/// A key while the line is open: the field edits and the typed text work at its caret, and Enter runs it. Returns true
/// when the command quits.
pub fn prompt_key(m: &mut Model, key: Key<'_>) -> bool {
    if let Some(edit) = edit_of(key) {
        if let Some(line) = m.command.as_mut() {
            line.edit(edit);
        }
        return false;
    }
    match key.name {
        "escape" => m.command = None,
        "return" => {
            let text = m.command.take().map(|line| line.text().to_owned()).unwrap_or_default();
            return run(m, &text);
        }
        "left" | "right" => {
            if let Some(line) = m.command.as_mut() {
                if key.name == "left" { line.left() } else { line.right() }
            }
        }
        _ if is_text(key.text) => {
            if let Some(line) = m.command.as_mut() {
                line.insert(key.text);
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
        // In the hex view any other line is an offset to go to.
        other if super::hex::go_to(m, other) => false,
        other => {
            m.message = Some(Message { text: format!("Not a command: {other}"), error: true });
            false
        }
    }
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
