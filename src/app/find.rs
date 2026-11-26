//! The file finder on the UI thread (Scouter `src/find/file-finder.ts`): `/` in the tree opens a prompt over the
//! sidebar's header, a worker walks the folder, and the matches take the tree's rows until the prompt ends. Enter
//! opens or reveals the selected row, and Esc puts the tree back where it was.

use std::path::{Path, PathBuf};

use fmc::explorer::{Entry, Kind, relative};
use fmc::find::{self, Index};
use fmc::keys::Key;

use super::{Model, tree};
use crate::services::{self, Event};
use crate::ui::footer::Message;

const HINT: &str = "type to find a file, Tab to search inside files";
const INDEXING: &str = "indexing…";

/// The finder while it is open (Scouter `Finding`, the files mode).
pub struct Finder {
    /// Names the worker that answers this finder; an index from an earlier finder is dropped.
    session: u64,
    pub query: String,
    /// `None` until the worker has walked the folder.
    index: Option<Index>,
    /// The row selected when the finder opened; Esc shows the tree with it selected (Scouter `returnTo`).
    returned_to: PathBuf,
    /// The text after the query: the hint, `indexing…`, or the count (Scouter `say`).
    pub status: String,
}

/// `/` in the tree (Scouter `startFind`). Refused while an open or a reveal is in flight, as Scouter refuses it
/// while it loads.
pub fn start(m: &mut Model) {
    if m.finder.is_some() {
        return;
    }
    if m.loading || m.pending.is_some() {
        m.message = Some(Message { text: "Still loading. Try again when it finishes.".into(), error: false });
        return;
    }
    m.find_session += 1;
    let session = m.find_session;
    m.finder = Some(Finder {
        session,
        query: String::new(),
        index: None,
        returned_to: m.browser.selected().to_path_buf(),
        status: String::new(),
    });
    let root = m.browser.tree.root().to_path_buf();
    let marks = m.marks.clone();
    services::spawn(&m.proxy, move || {
        let skip = |path: &Path| marks.is_ignored(&relative(&root, path));
        Event::Indexed { session, index: services::find::index(&root, skip, find::MAX_INDEXED) }
    });
    update(m);
}

/// The index arrives from its worker (Scouter `scan` start). An index for a finder that has ended is dropped.
pub fn indexed(m: &mut Model, session: u64, index: Index) {
    let Some(finder) = m.finder.as_mut() else { return };
    if finder.session != session {
        return;
    }
    finder.index = Some(index);
    update(m);
}

/// A plain key while the finder is open (Scouter `FileFinder.handleKey`): it edits the query or moves the
/// selection, and Esc and Enter end the finder. Other plain keys do nothing. Ctrl and Alt keys keep their commands.
pub fn prompt_key(m: &mut Model, key: Key<'_>) {
    match key.name {
        "escape" => end(m, None),
        "return" => pick(m),
        "up" => m.browser.move_by(-1),
        "down" => m.browser.move_by(1),
        "backspace" => {
            if let Some(finder) = m.finder.as_mut() {
                finder.query.pop();
            }
            update(m);
        }
        _ => {
            let mut chars = key.text.chars();
            if let (Some(typed), None) = (chars.next(), chars.next())
                && !typed.is_control()
                && let Some(finder) = m.finder.as_mut()
            {
                finder.query.push(typed);
                update(m);
            }
        }
    }
}

/// Shows what the query asks for: the tree when it is empty, `indexing…` until the index arrives, else the matches
/// (Scouter `update`, `showEmptyPrompt`, `showIndexing`, `showNames`).
fn update(m: &mut Model) {
    let Some(finder) = m.finder.as_mut() else { return };
    if finder.query.is_empty() {
        finder.status = HINT.into();
        let selected = m.browser.selected().to_path_buf();
        m.browser.show_tree(&selected);
        return;
    }
    let Some(index) = finder.index.as_ref() else {
        finder.status = INDEXING.into();
        m.browser.show(Vec::new(), Path::new(""));
        return;
    };
    let found = find::matches(&index.entries, &finder.query);
    finder.status = find::names_status(found.total, index.truncated);
    let rows: Vec<Entry> = found.entries.into_iter().cloned().collect();
    let first = rows.first().map(|row| row.path.clone()).unwrap_or_default();
    m.browser.show(rows, &first);
}

/// Enter (Scouter `pick`): the finder ends, then the picked row opens as the tree opens it when the query is empty,
/// or is revealed (a folder expanded, a file opened) when it is not.
pub fn pick(m: &mut Model) {
    let Some(finder) = m.finder.as_ref() else { return };
    let typed = finder.query.clone();
    let picked = m.browser.selected_entry();
    end(m, picked.as_ref().map(|entry| entry.path.clone()));
    let Some(entry) = picked else { return };
    if typed.trim().is_empty() {
        // Scouter opens the file with the reader focused here, as it does for any file the finder opens.
        if entry.kind != Kind::Directory {
            m.focus_reader_on_open = true;
        }
        tree::activate(m);
    } else if entry.kind == Kind::Directory {
        tree::reveal(m, entry.path, true, false);
    } else {
        tree::reveal(m, entry.path, false, true);
    }
}

/// Ends the finder and shows the tree again: the row `select` names, else the row the finder opened on (Scouter
/// `end`). The prompt and the sidebar header come back with the next paint.
fn end(m: &mut Model, select: Option<PathBuf>) {
    let Some(finder) = m.finder.take() else { return };
    m.browser.show_tree(&select.unwrap_or(finder.returned_to));
}
