//! The file finder on the UI thread (Scouter `src/find/file-finder.ts`): `/` in the tree opens a prompt over the
//! sidebar's header. A worker walks the folder, then the names or the text hits take the tree's rows until the prompt
//! ends. Tab switches between the two searches; the highlighted hit is previewed in the reader; Enter opens it.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use fmc::explorer::{Entry, Flat, Kind};
use fmc::find::text::{Generation, Progress, SCAN_DELAY_MS, text_status, wants_scan};
use fmc::find::{self, Index};
use fmc::keys::Key;
use fmc::reader::document::Document;
use fmc::reader::search::Search;
use fmc::theme::Palette;

use super::{Model, Ui, reader, tree};
use crate::services::{self, Event};
use crate::ui::footer::Message;
use crate::ui::hints::Prompt;
use crate::ui::syntax::language_for;

const HINT_FILES: &str = "type to find a file, Tab to search inside files";
const HINT_TEXT: &str = "type to search inside files";
const INDEXING: &str = "indexing…";
const SEARCHING: &str = "searching…";
const SHORT: &str = "type 2+ characters";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Names of files and folders (`/`).
    Files,
    /// Lines of text in files (`Tab` inside the finder).
    Text,
}

/// The finder while it is open (Scouter `Finding`).
pub struct Finder {
    /// Names the workers that answer this finder; answers from an earlier finder are dropped.
    session: u64,
    pub mode: Mode,
    pub query: String,
    /// The footer's text after the query: a hint, `indexing…`, the search's progress, or its count.
    pub status: String,
    /// `None` until the worker has walked the folder.
    index: Option<Arc<Index>>,
    /// The row selected when the finder opened; Esc shows the tree with it selected (Scouter `returnTo`).
    returned_to: PathBuf,
    /// The latest text search; a debounce or a result for an older search is dropped.
    scan: Generation,
    /// Set to stop the running text search (Scouter drops a scan when the query changes).
    cancelled: Option<Arc<AtomicBool>>,
    text: Option<Progress>,
    /// Counts the previews asked for; a file read that answers an older request is dropped.
    request: u64,
    /// The file the reader shows in place of the document, while the finder is open (Scouter `previewHit`).
    pub preview: Option<Preview>,
}

/// A preview: the file on screen, and the read the latest request waits for.
pub struct Preview {
    pub shown: Option<Shown>,
    pending: Option<Target>,
}

/// A file the preview shows: the target line (one-based), the query it marks, and the text.
pub struct Shown {
    pub path: PathBuf,
    pub line: usize,
    pub query: String,
    pub document: Document,
}

struct Target {
    request: u64,
    path: PathBuf,
    line: usize,
    query: String,
}

/// A text hit to open once its file is shown (Scouter `openAt`).
pub struct OpenAt {
    pub path: PathBuf,
    /// One-based.
    pub line: usize,
    pub query: String,
    /// The path the footer shows, when it is not the tree's (a change opened from the changes screen, which Scouter names
    /// by git's top-level path).
    pub shown: Option<PathBuf>,
}

/// What the preview view shows, as the paint last drew it: its file, target line, and query, and whether its scroll
/// has taken.
pub struct PreviewAt {
    pub path: PathBuf,
    pub line: usize,
    pub query: String,
    pub settled: bool,
}

/// `/` in the tree (Scouter `startFind`). Refused while an open or a reveal is in flight, as Scouter refuses it while
/// it loads.
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
    spawn_index(m, session);
    m.finder = Some(Finder {
        session,
        mode: Mode::Files,
        query: String::new(),
        status: String::new(),
        index: None,
        returned_to: m.browser.selected().to_path_buf(),
        scan: Generation::default(),
        cancelled: None,
        text: None,
        request: 0,
        preview: None,
    });
    update(m);
}

/// Walks the folder for the finder's index. Dotfiles are left out while the tree hides them (plan-00002).
fn spawn_index(m: &Model, session: u64) {
    let root = m.browser.tree.root().to_path_buf();
    let marks = m.marks.clone();
    let dotfiles = m.browser.dotfiles();
    services::spawn(&m.proxy, move || {
        let skip = |path: &Path| {
            let dot = path.file_name().is_some_and(|name| fmc::explorer::is_dotfile(&name.to_string_lossy()));
            (!dotfiles && dot) || marks.is_ignored(&fmc::explorer::relative(&root, path))
        };
        Event::Indexed { session, index: services::find::index(&root, skip, find::MAX_INDEXED) }
    });
}

/// Walks the folder again after a dotfile toggle, with the open finder's query kept. The old index answers nothing
/// once the session moves on, and its text search stops.
pub fn reindex(m: &mut Model) {
    if m.finder.is_none() {
        return;
    }
    m.find_session += 1;
    let session = m.find_session;
    if let Some(finder) = m.finder.as_mut() {
        finder.session = session;
        finder.index = None;
    }
    restart_scan(m);
    spawn_index(m, session);
    update(m);
}

/// The index arrives from its worker (Scouter `finding.entries = entries`).
pub fn indexed(m: &mut Model, session: u64, index: Index) {
    let Some(finder) = m.finder.as_mut() else { return };
    if finder.session != session {
        return;
    }
    finder.index = Some(Arc::new(index));
    restart_scan(m);
    update(m);
}

/// A plain key while the finder is open (Scouter `FileFinder.handleKey`): typing edits the query, the arrows move the
/// highlight, Tab switches the search, and Enter and Esc end the finder. Other plain keys do nothing.
pub fn prompt_key(m: &mut Model, key: Key<'_>) {
    match key.name {
        "escape" => end(m, None),
        "return" => pick(m),
        "tab" => {
            if let Some(finder) = m.finder.as_mut() {
                finder.mode = if finder.mode == Mode::Files { Mode::Text } else { Mode::Files };
            }
            restart_scan(m);
            update(m);
        }
        "up" | "down" => {
            m.browser.move_by(if key.name == "up" { -1 } else { 1 });
            preview_selected(m);
        }
        "backspace" => {
            if let Some(finder) = m.finder.as_mut() {
                finder.query.pop();
            }
            restart_scan(m);
            update(m);
        }
        _ => {
            let mut chars = key.text.chars();
            if let (Some(typed), None) = (chars.next(), chars.next())
                && !typed.is_control()
                && let Some(finder) = m.finder.as_mut()
            {
                finder.query.push(typed);
                restart_scan(m);
                update(m);
            }
        }
    }
}

/// A text search starts after the debounce, if no key came in the meantime (Scouter `restartScan`). Any running
/// search stops now.
fn restart_scan(m: &mut Model) {
    let Some(finder) = m.finder.as_mut() else { return };
    let scan = finder.scan.bump();
    if let Some(flag) = finder.cancelled.take() {
        flag.store(true, Ordering::Relaxed);
    }
    finder.text = None;
    if finder.mode != Mode::Text || finder.index.is_none() || !wants_scan(&finder.query) {
        return;
    }
    let session = finder.session;
    services::spawn(&m.proxy, move || {
        std::thread::sleep(Duration::from_millis(SCAN_DELAY_MS));
        Event::ScanDue { session, scan }
    });
}

/// The debounce has passed: the text search starts, if it is still the latest one.
pub fn scan_due(m: &mut Model, session: u64, scan: u64) {
    let Some(finder) = m.finder.as_mut() else { return };
    if finder.session != session || !finder.scan.is_current(scan) || finder.mode != Mode::Text {
        return;
    }
    let (Some(index), query) = (finder.index.clone(), finder.query.clone()) else { return };
    let cancelled = Arc::new(AtomicBool::new(false));
    finder.cancelled = Some(cancelled.clone());
    services::stream(&m.proxy, move |send| {
        services::find::search(&index, &query, &cancelled, &|progress| {
            send(Event::Searched { session, scan, progress })
        });
    });
}

/// A text search's progress arrives (Scouter `showText`).
pub fn searched(m: &mut Model, session: u64, scan: u64, progress: Progress) {
    let Some(finder) = m.finder.as_mut() else { return };
    if finder.session != session || !finder.scan.is_current(scan) {
        return;
    }
    finder.text = Some(progress);
    update(m);
}

/// A preview's file has been read. A read the latest request does not wait for is dropped, and so is one that failed:
/// the reader keeps what it showed (Scouter `show` returns false).
pub fn previewed(m: &mut Model, session: u64, request: u64, result: Result<Document, String>) {
    let Some(finder) = m.finder.as_mut() else { return };
    if finder.session != session {
        return;
    }
    let Some(preview) = finder.preview.as_mut() else { return };
    if preview.pending.as_ref().map(|target| target.request) != Some(request) {
        return;
    }
    let Some(target) = preview.pending.take() else { return };
    if let Ok(document) = result {
        preview.shown = Some(Shown { path: target.path, line: target.line, query: target.query, document });
    }
}

/// Shows what the query asks for (Scouter `update`): the tree when it is empty, `indexing…` until the index arrives,
/// then the names or the text hits.
fn update(m: &mut Model) {
    let Some(finder) = m.finder.as_ref() else { return };
    let (index, query, mode) = (finder.index.clone(), finder.query.clone(), finder.mode);
    if query.is_empty() {
        set_status(m, if mode == Mode::Files { HINT_FILES } else { HINT_TEXT });
        end_preview(m);
        let selected = m.browser.selected().to_path_buf();
        m.browser.show_tree(&selected);
        return;
    }
    let Some(index) = index else {
        set_status(m, INDEXING);
        m.browser.show(Vec::new(), Path::new(""));
        return;
    };
    match mode {
        Mode::Files => show_names(m, &index, &query),
        Mode::Text => show_text(m, &query),
    }
}

fn show_names(m: &mut Model, index: &Index, query: &str) {
    let found = find::matches(&index.entries, query);
    set_status(m, &find::names_status(found.total, index.truncated));
    end_preview(m);
    let rows: Vec<Flat> = found.entries.into_iter().map(|entry| Flat::name(entry.clone())).collect();
    let first = rows.first().map(|row| row.entry.path.clone()).unwrap_or_default();
    m.browser.show(rows, &first);
}

fn show_text(m: &mut Model, query: &str) {
    if !wants_scan(query) {
        set_status(m, SHORT);
        end_preview(m);
        m.browser.show(Vec::new(), Path::new(""));
        return;
    }
    let Some(progress) = m.finder.as_ref().and_then(|finder| finder.text.clone()) else {
        set_status(m, SEARCHING);
        m.browser.show(Vec::new(), Path::new(""));
        return;
    };
    set_status(m, &text_status(&progress));
    let rows: Vec<Flat> = progress
        .hits
        .iter()
        .map(|hit| Flat {
            entry: Entry { name: hit.relative.clone(), path: hit.path.clone(), kind: Kind::File },
            hit: Some((hit.line, hit.snippet.clone())),
        })
        .collect();
    // The selection stays on its file while the file still has a hit (Scouter `showText`).
    let kept = m.browser.selected().to_path_buf();
    let selected = match rows.iter().any(|row| row.entry.path == kept) {
        true => kept,
        false => rows.first().map(|row| row.entry.path.clone()).unwrap_or_default(),
    };
    m.browser.show(rows, &selected);
    preview_selected(m);
}

fn set_status(m: &mut Model, text: &str) {
    if let Some(finder) = m.finder.as_mut() {
        finder.status = text.to_string();
    }
}

/// Previews the highlighted row when it is a text hit; any other row ends the preview (Scouter `preview`).
fn preview_selected(m: &mut Model) {
    let target = m.browser.selected_flat().and_then(|flat| flat.hit.map(|(line, _)| (flat.entry.path, line)));
    match target {
        Some((path, line)) => request_preview(m, path, line),
        None => end_preview(m),
    }
}

/// Asks for a preview of `path` at `line`. The file on screen is reused without a read; another file is read on a
/// worker, and the reader keeps what it showed until the read answers.
fn request_preview(m: &mut Model, path: PathBuf, line: usize) {
    let Some(finder) = m.finder.as_mut() else { return };
    finder.request += 1;
    let (session, request, query) = (finder.session, finder.request, finder.query.clone());
    let preview = finder.preview.get_or_insert(Preview { shown: None, pending: None });
    if let Some(shown) = preview.shown.as_mut().filter(|shown| shown.path == path) {
        shown.line = line;
        shown.query = query;
        preview.pending = None;
        return;
    }
    let read = path.clone();
    preview.pending = Some(Target { request, path, line, query });
    services::spawn(&m.proxy, move || Event::Previewed { session, request, result: services::find::preview(&read) });
}

/// Ends the preview (Scouter `endPreview`): the paint hides its view and the reader shows again with its own row.
fn end_preview(m: &mut Model) {
    if let Some(finder) = m.finder.as_mut() {
        finder.preview = None;
    }
}

/// Enter (Scouter `pick`): the finder ends, then the picked row opens as the tree opens it when the query is empty, or
/// is revealed (a folder expanded, a file opened) when it is not. A text hit opens at its line.
pub fn pick(m: &mut Model) {
    let Some(finder) = m.finder.as_ref() else { return };
    let typed = finder.query.clone();
    let picked = m.browser.selected_flat().or_else(|| m.browser.selected_entry().map(Flat::name));
    end(m, picked.as_ref().map(|flat| flat.entry.path.clone()));
    let Some(flat) = picked else { return };
    if let Some((line, _)) = flat.hit {
        m.open_at = Some(OpenAt { path: flat.entry.path.clone(), line, query: typed, shown: None });
        m.focus_reader_on_open = true;
        tree::reveal(m, flat.entry.path, false, true);
    } else if typed.trim().is_empty() {
        // Scouter opens the file with the reader focused here, as it does for any file the finder opens.
        if flat.entry.kind != Kind::Directory {
            m.focus_reader_on_open = true;
        }
        tree::activate(m);
    } else if flat.entry.kind == Kind::Directory {
        tree::reveal(m, flat.entry.path, true, false);
    } else {
        tree::reveal(m, flat.entry.path, false, true);
    }
}

/// Ends the finder and shows the tree again: the row `select` names, else the row the finder opened on (Scouter
/// `end`). A running text search is stopped; the preview goes with the finder.
pub(super) fn end(m: &mut Model, select: Option<PathBuf>) {
    let Some(finder) = m.finder.take() else { return };
    if let Some(flag) = finder.cancelled {
        flag.store(true, Ordering::Relaxed);
    }
    m.browser.show_tree(&select.unwrap_or(finder.returned_to));
}

/// The paints an open-at scroll is set in before it is dropped: a view that lays out after the scroll may clamp it.
pub const OPEN_SCROLL_TRIES: u8 = 20;

/// Enter on a text hit, once its file is shown (Scouter `openAt`): a Markdown file opens in its source, the line goes to
/// the top of the view, and the query stays as a search for `n` and `N`. The top is where Scouter's live run puts it;
/// its `scrollToLine` would centre the line, so the difference is observed, not read from the source.
pub fn arrive(m: &mut Model, path: &Path) {
    if !m.open_at.as_ref().is_some_and(|open| open.path == path) {
        return;
    }
    let Some(open) = m.open_at.take() else { return };
    if m.shown.previewing() {
        reader::toggle_markdown(m);
    }
    let Some((document, ..)) = m.shown.text() else { return };
    // A file the changes screen opens has no query, so it starts no search (Scouter `openChanged`).
    if !open.query.is_empty() {
        let search = Search::seed(document.content(), &open.query, open.line - 1);
        m.message = Some(Message { text: search.message(), error: false });
        m.search = Some(search);
    }
    // The reader scrolls from the paint, once it shows: a view that is still hidden takes no scroll.
    m.pending_top = Some((open.line - 1, OPEN_SCROLL_TRIES));
}

/// Shows the preview in its own view beside the reader (Scouter's `HitPreview` root): the text is set when the file or
/// the query changes, and the view scrolls until it takes the target line a third of the way down.
pub fn paint_preview(m: &Model, ui: &mut Ui, p: &Palette) {
    let shown = m.finder.as_ref().and_then(|finder| finder.preview.as_ref()).and_then(|preview| preview.shown.as_ref());
    let Some(shown) = shown else {
        ui.preview_viewer.visible(false);
        ui.preview_at = None;
        return;
    };
    let before = ui.preview_at.take();
    let text_changed = before.as_ref().is_none_or(|at| at.path != shown.path || at.query != shown.query);
    let moved = before.as_ref().is_none_or(|at| at.line != shown.line);
    if text_changed {
        ui.preview.show(&shown.document, language_for(&m.associations, &m.opened, &shown.path));
    }
    ui.preview_viewer.visible(true);
    ui.preview_viewer.set_wrap(false);
    ui.preview.visible(true);
    ui.preview.sync_search(p, &shown.query, Some(shown.line - 1), shown.document.content());
    let target = (shown.line - 1).saturating_sub(m.viewport / 3) as f64;
    // The scroll is set again until the view takes it, since a view laid out after the text may clamp it; a wheel
    // scroll after that is left alone.
    let settled = !(text_changed || moved) && before.as_ref().is_some_and(|at| at.settled);
    let settled = settled || (ui.preview_viewer.position().1 - target).abs() < 0.5;
    if !settled {
        ui.preview_viewer.scroll_to(0.0, target);
    }
    ui.preview_at = Some(PreviewAt { path: shown.path.clone(), line: shown.line, query: shown.query.clone(), settled });
}

/// The prompt label and the query, as the bar and the footer show them.
pub fn prompt(finder: &Finder) -> (&'static str, Prompt) {
    match finder.mode {
        Mode::Files => ("/", Prompt::Files),
        Mode::Text => ("≡", Prompt::Text),
    }
}
