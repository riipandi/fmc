//! The file tree's keys and worker events, on the UI thread (Scouter `src/app/app.ts`: `openSelected`,
//! `collapseSelected`, `goUp`, `reveal`, `openDirectory`, `onExplorerActivate`).

use std::path::{Path, PathBuf};

use fmc::diff::Marks;
use fmc::explorer::{Browser, Expand, Reveal, Tree};
use fmc::keys::{Action, Focus};
use fmc::text::display_label;

use super::poll::{self, Chain, POLL, Tick};
use super::{Model, Pending, Ui, changes, find, reader};
use crate::services::{self, Event, fs};
use crate::ui::footer::Message;

/// Runs a list key on the selected row.
pub fn act(m: &mut Model, action: Action) {
    match action {
        Action::Move(delta) => m.browser.move_by(delta as isize),
        Action::MovePage(direction) => m.browser.page(direction > 0),
        Action::Top => m.browser.move_to_edge(false),
        Action::Bottom => m.browser.move_to_edge(true),
        Action::Toggle => activate(m),
        Action::Expand => expand(m),
        Action::Collapse => collapse(m),
        Action::GoUp => go_up(m),
        _ => {}
    }
}

/// Enter, a click: collapse an expanded folder, or open the entry (Scouter `openSelected("toggle")`).
pub fn activate(m: &mut Model) {
    let path = m.browser.selected().to_path_buf();
    if m.browser.tree.is_expanded(&path) {
        m.browser.tree.collapse(&path);
        m.browser.select(&path);
        return;
    }
    open(m, path);
}

/// `l` and Right: expand a folder; on an expanded folder with children, step into the first one.
fn expand(m: &mut Model) {
    let path = m.browser.selected().to_path_buf();
    if m.browser.tree.is_expanded(&path) {
        let rows = m.browser.tree.rows();
        let at = m.browser.selected_index();
        let has_children = rows.get(at + 1).is_some_and(|next| next.depth > rows[at].depth);
        drop(rows);
        if has_children {
            m.browser.move_by(1);
        }
        return;
    }
    open(m, path);
}

/// `h`, Left, Backspace: collapse an expanded folder, or go to the parent row (a top-level row has none).
fn collapse(m: &mut Model) {
    let path = m.browser.selected().to_path_buf();
    if m.browser.tree.is_expanded(&path) {
        m.browser.tree.collapse(&path);
        m.browser.select(&path);
        return;
    }
    if let Some(parent) = m.browser.tree.parent_row(&path) {
        m.browser.select(&parent);
    }
}

/// `-`: the folder above becomes the root, and the old root stays expanded and selected.
fn go_up(m: &mut Model) {
    let root = m.browser.tree.root().to_path_buf();
    let Some(parent) = root.parent().map(Path::to_path_buf) else {
        m.message = Some(Message { text: "Already at the top of the file system.".into(), error: false });
        return;
    };
    let height = m.browser.height();
    m.browser = Browser::new(Tree::new(parent.clone()));
    m.browser.set_height(height);
    m.pending = Some(Pending { target: root, expand: true, open: false });
    list(m, parent);
    refresh_marks(m);
}

/// Opens an entry: a folder expands, and a file is read; both off the UI thread when they are not in memory. One
/// open runs at a time: another is refused until it finishes (Scouter `runLoading`).
/// Opens an entry from the tree, as Scouter's `open`: the place left is recorded for the jump history first.
pub fn open(m: &mut Model, path: PathBuf) {
    m.jumps.leave(reader::here(m));
    open_entry(m, path);
}

/// Opens an entry without recording the place left: the open itself, or a jump that is already in the history.
pub fn open_entry(m: &mut Model, path: PathBuf) {
    match m.browser.tree.expand(&path) {
        Expand::Ready => {
            m.browser.select(&path);
            folder_opened(m, &path);
        }
        Expand::NeedsListing if m.loading => {
            m.message = Some(Message { text: "Still loading. Try again when it finishes.".into(), error: false });
        }
        Expand::NeedsListing => {
            m.loading = true;
            m.message = Some(Message { text: "Loading…".into(), error: false });
            let parents = m.browser.tree.ancestors(&path);
            let work = path.clone();
            services::spawn(&m.proxy, move || Event::Opened { path: work.clone(), result: fs::open(&work, &parents) });
        }
        Expand::Missing => {}
    }
}

/// The footer shows the folder just opened (Scouter `openDirectory`, with no document open).
fn folder_opened(m: &mut Model, path: &Path) {
    m.focus = Focus::List;
    m.location = path.to_string_lossy().into_owned();
    m.message = None;
}

/// An error goes to the footer and to the reader's header notice (Scouter `showError`).
pub fn show_error(m: &mut Model, text: &str) {
    let text = display_label(text);
    m.message = Some(Message { text: text.clone(), error: true });
    m.notice = Some(text);
}

/// Lists a folder for the tree; the result comes back as `Event::Listed`.
pub fn list(m: &mut Model, folder: PathBuf) {
    let parents = m.browser.tree.ancestors(&folder);
    let work = folder.clone();
    services::spawn(&m.proxy, move || Event::Listed { path: work.clone(), result: fs::list(&work, &parents) });
}

/// Asks git for the marks of the browsed folder; the result comes back as `Event::Marks`.
pub fn refresh_marks(m: &mut Model) {
    let root = m.browser.tree.root().to_path_buf();
    let work = root.clone();
    services::spawn(&m.proxy, move || Event::Marks { root: work.clone(), result: services::git::marks(&work) });
}

/// Applies a worker's event on the UI thread.
pub fn apply(m: &mut Model, ui: &Ui, event: &Event) {
    match event {
        Event::Indexed { session, index } => find::indexed(m, *session, index.clone()),
        Event::ScanDue { session, scan } => find::scan_due(m, *session, *scan),
        Event::Searched { session, scan, progress } => find::searched(m, *session, *scan, progress.clone()),
        Event::Previewed { session, request, result } => find::previewed(m, *session, *request, result.clone()),
        Event::ChangesListed { session, result } => changes::listed(m, *session, result.clone()),
        Event::ChangesPatched { session, request, stamp, result } => {
            changes::patched(m, *session, *request, stamp.clone(), result.clone())
        }
        Event::DiffPolled { session, result } => changes::polled(m, *session, result.clone()),
        Event::BrowsePolled { root, chain, folders, marks, document } => {
            if *chain {
                let elapsed = m.browse_started.take().map_or(POLL, |started| started.elapsed());
                m.ticks.push(Tick { chain: Chain::Browse, delay: poll::backoff(elapsed) });
            }
            if root == m.browser.tree.root() {
                for (path, entries) in folders {
                    m.browser.refreshed(path, entries.clone());
                }
                if let Ok(listing) = marks {
                    let next = listing.clone().map_or_else(Default::default, Marks::new);
                    if next != m.marks {
                        m.marks = next;
                    }
                }
            }
            if let Some((path, reread)) = document {
                reader::reread(m, ui, path, reread.clone());
            }
        }
        Event::Listed { path, result } => match result {
            Ok(entries) => {
                m.browser.listed(path, entries.clone());
                advance(m);
            }
            Err(text) => {
                m.pending = None;
                show_error(m, text);
            }
        },
        Event::Opened { path, result } => {
            m.loading = false;
            match result {
                Ok(fs::Opened::Folder(entries)) => {
                    m.browser.listed(path, entries.clone());
                    m.browser.select(path);
                    folder_opened(m, path);
                }
                Ok(fs::Opened::Text { document, stamp }) => {
                    reader::show_text(m, ui, path, document.clone(), stamp.clone())
                }
                Ok(fs::Opened::Image { info, bytes }) => reader::show_image(m, ui, path, info.clone(), bytes),
                Ok(fs::Opened::Binary { size }) => reader::show_binary(m, path, *size),
                Err(text) => {
                    m.focus_reader_on_open = false;
                    m.restore = None;
                    m.focus = Focus::List;
                    show_error(m, text);
                }
            }
        }
        Event::Marks { root, result } => {
            if let (true, Ok(listing)) = (root == m.browser.tree.root(), result) {
                m.marks = listing.clone().map_or_else(Default::default, Marks::new);
            }
        }
    }
}

/// Reveals `target` as Scouter's `reveal` and `revealFolder` do: the folders above it are listed and expanded, the
/// folder itself too when `expand`, then `target` is selected and opened when `open`.
pub fn reveal(m: &mut Model, target: PathBuf, expand: bool, open: bool) {
    m.pending = Some(Pending { target, expand, open });
    advance(m);
}

/// Moves a pending reveal (startup on a file, going up) one step: list the next folder, or finish by selecting
/// the target, expanding it first when going up.
fn advance(m: &mut Model) {
    let Some(pending) = m.pending.take() else { return };
    let target = pending.target.clone();
    match m.browser.tree.reveal(&target) {
        Reveal::Need(folder) => {
            m.pending = Some(pending);
            list(m, folder);
        }
        Reveal::Done | Reveal::Outside => {
            if pending.expand && m.browser.tree.expand(&target) == Expand::NeedsListing {
                m.pending = Some(pending);
                list(m, target);
                return;
            }
            m.browser.select(&target);
            if pending.open {
                m.focus_reader_on_open = true;
                open(m, target);
            }
        }
    }
}
