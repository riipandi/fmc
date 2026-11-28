//! In-file search on the UI thread (Scouter `src/reader/document-search.ts`, `handleSearchKeyInReader`, `startSearch`,
//! `escape`): the prompt takes the plain keys while it is open, `n` and `N` step through the hits, and the footer shows
//! the query and the hit count. In the rendered Markdown view the hits are occurrences, marked in the view and counted
//! over its rendered text (`reader/rendered_search.rs`); the code view's hits are lines.

use fmc::keys::Key;
use fmc::reader::rendered_search::{Hit, Rendered, row_within};
use fmc::reader::search::Search;

use super::{Model, Ui};
use crate::ui::footer::Message;
use crate::ui::markdown_search::Mark;

/// The footer's text when `n` or `N` comes before any search (Scouter `step`).
const NO_SEARCH: &str = "Press / to search this file.";

fn show(m: &mut Model, text: String) {
    m.message = Some(Message { text, error: false });
}

/// The text the code view's search reads: the document's lines, as the code view shows them.
fn text(m: &Model) -> String {
    m.shown.text().map_or_else(String::new, |(document, ..)| document.content().to_owned())
}

/// The source of the rendered view's index: the text the view was built from, with its final newline.
fn source(m: &Model) -> String {
    m.shown.text().map_or_else(String::new, |(document, ..)| document.text.clone())
}

/// The top row the search starts from: the code view's line at the top, or the rendered view's row.
fn top(m: &Model) -> usize {
    m.top.floor() as usize
}

/// Moves the code view to `line`, with half a view above it (Scouter `scrollToLine`).
fn go(m: &Model, ui: &Ui, line: Option<usize>) {
    if let Some(line) = line {
        ui.viewer.scroll_to(0.0, fmc::reader::motion::line_top(line, m.viewport) as f64);
    }
}

/// Moves the rendered view to the current hit: its block's top, plus the hit's estimated row inside a block taller than
/// the view. Nothing moves until a frame has laid the blocks out.
fn go_mark(m: &Model, ui: &Ui, search: &Search) {
    let Some(hit) = search.current_mark() else { return };
    let rendered = ui.markdown.index(&source(m));
    let tops = ui.markdown.block_tops();
    let Some(row) = row_of(&rendered, &tops, hit) else { return };
    ui.viewer.scroll_to(0.0, fmc::reader::motion::line_top(row as usize, m.viewport) as f64);
}

/// The document row of a hit: its block's top, plus its estimated row in the block.
fn row_of(rendered: &Rendered, tops: &[(u32, u32)], hit: &Hit) -> Option<u32> {
    let &(top, rows) = tops.get(hit.block)?;
    Some(top + row_within(hit.offset, rendered.block_len(hit.block), rows))
}

/// Replaces a rendered search's query and hits, and moves to the first hit at or below the top row (Scouter `type`).
fn retype_marks(m: &Model, ui: &Ui, search: &mut Search, query: String) {
    let rendered = ui.markdown.index(&source(m));
    let tops = ui.markdown.block_tops();
    let marks = rendered.hits(&query);
    let row = |hit: &Hit| row_of(&rendered, &tops, hit).unwrap_or(0);
    search.retype_marks(marks, query, top(m) as u32, row);
}

/// `/` in the reader: opens the prompt on an empty query (Scouter `startSearch`). The rendered view searches its own
/// text, so its search says so.
pub fn start(m: &mut Model) {
    m.search = Some(if m.shown.previewing() { Search::start_rendered() } else { Search::start() });
    show(m, "/".into());
}

/// A key while the prompt is open: the query edits, Enter and the arrows step, Esc closes (Scouter `handleKey`).
/// Only plain keys reach here; Ctrl keys keep their commands.
pub fn prompt_key(m: &mut Model, ui: &Ui, key: Key<'_>) {
    let Some(mut search) = m.search.take() else { return };
    let reveal = match key.name {
        "escape" => {
            if search.query.is_empty() {
                m.message = None;
                return;
            }
            search.editing = false;
            false
        }
        "return" => {
            search.step(!key.shift);
            true
        }
        "down" => {
            search.step(true);
            true
        }
        "up" => {
            search.step(false);
            true
        }
        "backspace" => {
            let mut query = search.query.clone();
            query.pop();
            retype(m, ui, &mut search, query)
        }
        _ => match key.text.chars().collect::<Vec<char>>().as_slice() {
            [typed] if !typed.is_control() => {
                let query = format!("{}{typed}", search.query);
                retype(m, ui, &mut search, query)
            }
            _ => false,
        },
    };
    show(m, search.message());
    m.search = Some(search);
    if reveal {
        reveal_current(m, ui);
    }
}

/// Replaces the query for whichever view the search runs in; returns whether the view should move to the hit.
fn retype(m: &Model, ui: &Ui, search: &mut Search, query: String) -> bool {
    if search.rendered {
        retype_marks(m, ui, search, query);
        return search.current_mark().is_some();
    }
    search.retype(&text(m), query, top(m)).is_some()
}

/// Moves the view to the current hit, in whichever view the search runs in.
fn reveal_current(m: &Model, ui: &Ui) {
    let Some(search) = m.search.as_ref() else { return };
    if search.rendered {
        go_mark(m, ui, search);
    } else {
        go(m, ui, search.current_line());
    }
}

/// `n` (forward) and `N` or `p` (back): the next hit, wrapping (Scouter `step`).
pub fn step(m: &mut Model, ui: &Ui, forward: bool) {
    let Some(search) = m.search.as_mut() else {
        show(m, NO_SEARCH.into());
        return;
    };
    let line = search.step(forward);
    let (rendered, message) = (search.rendered, search.message());
    show(m, message);
    if rendered {
        reveal_current(m, ui);
    } else {
        go(m, ui, line);
    }
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

/// `m` clears the search: the view it searched is no longer the one shown.
pub fn clear(m: &mut Model) {
    if m.search.take().is_some() {
        m.message = None;
    }
}

/// A text that was shown again keeps the search, with its hits recomputed (Scouter `refresh`).
pub fn follow(m: &mut Model, ui: &Ui) {
    let Some(search) = m.search.as_ref() else { return };
    if search.rendered {
        let marks = ui.markdown.index(&source(m)).hits(&search.query);
        if let Some(search) = m.search.as_mut() {
            search.refresh_marks(marks);
        }
        return;
    }
    let content = text(m);
    if let Some(search) = m.search.as_mut() {
        search.refresh(&content);
    }
}

/// Sets the rendered view's marks to the search: the query, and the hit the view is on. A code view search marks
/// nothing in the rendered view, and the rendered view's marks clear when it is not shown.
pub fn sync(m: &Model, ui: &Ui) {
    let mark = match m.search.as_ref() {
        Some(search) if search.rendered && m.shown.previewing() && !search.query.is_empty() => Some(Mark {
            query: search.query.clone(),
            current: search.current_mark().and_then(|hit| hit.slot.map(|slot| (slot, hit.at.clone()))),
        }),
        _ => None,
    };
    ui.markdown.mark(mark);
}
