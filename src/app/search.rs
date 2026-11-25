//! In-file search on the UI thread (Scouter `src/reader/document-search.ts`, `handleSearchKeyInReader`, `startSearch`,
//! `escape`): the prompt takes the plain keys while it is open, `n` and `N` step through the hits, and the footer shows
//! the query and the hit count.

use fmc::keys::Key;
use fmc::reader::search::Search;

use super::reader::Shown;
use super::{Model, Ui};
use crate::ui::footer::Message;

/// The footer's text when `n` or `N` comes before any search (Scouter `step`).
const NO_SEARCH: &str = "Press / to search this file.";

fn show(m: &mut Model, text: String) {
    m.message = Some(Message { text, error: false });
}

/// The text the search reads: the document's lines, as the code view shows them.
fn text(m: &Model) -> String {
    match &m.shown {
        Shown::Text { document, .. } => document.content().to_owned(),
        _ => String::new(),
    }
}

/// Moves the view to `line`, with half a view above it (Scouter `scrollToLine`).
fn go(m: &Model, ui: &Ui, line: Option<usize>) {
    if let Some(line) = line {
        ui.viewer.scroll_to(0.0, fmc::reader::motion::line_top(line, m.viewport) as f64);
    }
}

/// `/` in the reader: opens the prompt on an empty query (Scouter `startSearch`).
pub fn start(m: &mut Model) {
    m.search = Some(Search::start());
    show(m, "/".into());
}

/// A key while the prompt is open: the query edits, Enter and the arrows step, Esc closes (Scouter `handleKey`).
/// Only plain keys reach here; Ctrl keys keep their commands.
pub fn prompt_key(m: &mut Model, ui: &Ui, key: Key<'_>) {
    let Some(mut search) = m.search.take() else { return };
    let line = match key.name {
        "escape" => {
            if search.query.is_empty() {
                m.message = None;
                return;
            }
            search.editing = false;
            None
        }
        "return" => search.step(!key.shift),
        "down" => search.step(true),
        "up" => search.step(false),
        "backspace" => {
            let mut query = search.query.clone();
            query.pop();
            search.retype(&text(m), query, m.top.floor() as usize)
        }
        _ => match key.text.chars().collect::<Vec<char>>().as_slice() {
            [typed] if !typed.is_control() => {
                let query = format!("{}{typed}", search.query);
                search.retype(&text(m), query, m.top.floor() as usize)
            }
            _ => None,
        },
    };
    show(m, search.message());
    m.search = Some(search);
    go(m, ui, line);
}

/// `n` (forward) and `N` or `p` (back): the next hit, wrapping (Scouter `step`).
pub fn step(m: &mut Model, ui: &Ui, forward: bool) {
    let Some(search) = m.search.as_mut() else {
        show(m, NO_SEARCH.into());
        return;
    };
    let line = search.step(forward);
    let message = search.message();
    show(m, message);
    go(m, ui, line);
}

/// Esc with a kept search: clears it (Scouter `dismiss`). Returns whether there was one to clear.
pub fn dismiss(m: &mut Model) -> bool {
    match m.search.as_ref() {
        Some(search) if !search.editing => {
            m.search = None;
            m.message = None;
            true
        }
        _ => false,
    }
}

/// A text that was shown again keeps the search, with its hits recomputed (Scouter `refresh`).
pub fn follow(m: &mut Model) {
    let content = text(m);
    if let Some(search) = m.search.as_mut() {
        search.refresh(&content);
    }
}
