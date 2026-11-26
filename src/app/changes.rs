//! The changes screen on the UI thread (Scouter `src/app/app.ts` `toggleDiff`, `loadDiff`, `enterDiff`, `leaveDiff`,
//! `handleDiffKey`, `handleChangesKey`, and `src/diff/diff-screen.ts` `load`, `select`, `loadAll`, `toggleScope`,
//! `toggleContext`, `toggleCollapse`, `openChanged`, `layoutChanged`). The list and the stacked body are drawn from the
//! core's state; the listing and the patches come back from workers as events, and a reply that no longer matches the
//! screen is dropped.

use std::path::PathBuf;

use fmc::diff::{
    ChangeKind, ChangeSet, FilePatch, Folds, Jump, Patch, active_hunk, block_at, can_split, gutter_digits, hunk_starts,
    line_at, split_path, step, totals, within_folder,
};
use fmc::keys::{Action, Focus};
use fmc::text::display_label;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::find::OpenAt;
use super::poll::{self, Chain, POLL, Tick};
use super::{Model, Ui, tree};
use crate::services::changes::{Listed, Shared};
use crate::services::{self, Event};
use crate::ui::footer::{DiffInfo, Message};
use crate::ui::header::{Badge, BadgeKind, HeaderState};

/// How many paints the body may take to lay out before a jump to a file is given up.
const JUMP_TRIES: u8 = 8;

/// Scouter's words for the footer (`src/app/app.ts`, `src/diff/diff-blocks.ts`).
const FIRST_HUNK: &str = "First hunk.";
const LAST_HUNK: &str = "Last hunk.";
const LOADING: &str = "Loading changes…";
const BUSY: &str = "Still loading. Try again when it finishes.";
const NO_FILE_TO_OPEN: &str = "This file was deleted, so there is nothing to open.";
const SPLIT_NEEDS_COLUMNS: &str = "Split needs 60+ columns.";

/// The options of the stacked body, kept for the session: Scouter's `DiffScreen` keeps its scope, view, and context from
/// one visit to the next.
#[derive(Debug)]
pub struct Options {
    pub split: bool,
    pub one_file: bool,
    pub wide: bool,
}

impl Default for Options {
    /// One file, where Scouter opens every file (`scouter:src/diff/diff-screen.ts:82`): an approved deviation
    /// (architecture.md, "the changes screen opens in one-file mode").
    fn default() -> Self {
        Self { split: false, one_file: true, wide: false }
    }
}

/// The changes screen while it shows (Scouter `DiffScreen`): the listing's repository and the body's files.
pub struct Changes {
    /// The listing this screen shows. A patch reply for another session is dropped.
    pub session: u64,
    pub repository: Shared,
    /// The browsed folder, and where it is inside the repository: `o` maps a change onto the tree with them.
    pub browsed: PathBuf,
    pub within: String,
    /// Whether the patches of the latest read have arrived; until then the body has no blocks.
    pub patched: bool,
    /// The stamp of the files the body shows, taken before they were read; a poll reads them again when it changes.
    pub stamp: String,
    /// The body's files: every listed change, or the selected one in one-file scope, with their patch text.
    pub files: Vec<FilePatch>,
    /// The line-number width the blocks share (`fmc::diff::gutter_digits`).
    pub digits: usize,
    /// Bumped with each new `files`, so the body is built once for them.
    pub generation: u64,
    /// The latest read. A patch reply for an older read is dropped (Scouter `request`).
    pub request: u64,
    /// Whether the latest read scrolls the body to the selected file once it shows (Scouter `jumpAfterLayout`).
    pub jump_when_read: bool,
    /// The files folded on this visit (Scouter `collapsed`): leaving the screen clears them.
    pub folds: Folds,
    /// Enter was pressed before the patches arrived: the body takes focus when they do (Scouter `activateWhenReady`).
    pub focus_when_ready: bool,
    /// The pane that had focus before the screen opened, which it returns to (Scouter `paneBeforeDiff`).
    pub return_focus: Focus,
}

/// Whether the body shows: the changes screen, with no key page over it.
pub fn body_shows(m: &Model) -> bool {
    m.changes.is_some() && !m.shown.shows_keys()
}

/// `d` (Scouter `toggleDiff`): hides the screen, cancels a listing in flight, or lists the changes.
pub fn toggle(m: &mut Model) {
    if m.changes.is_some() {
        return leave(m);
    }
    if m.changes_load.is_some() {
        m.changes_load = None;
        m.message = Some(Message { text: "Cancelled.".into(), error: false });
        return;
    }
    if m.loading {
        m.message = Some(Message { text: BUSY.into(), error: false });
        return;
    }
    m.changes_session += 1;
    let session = m.changes_session;
    m.changes_load = Some(session);
    m.message = Some(Message { text: LOADING.into(), error: false });
    m.notice = None;
    let dir: PathBuf = m.browser.tree.root().to_path_buf();
    services::spawn(&m.proxy, move || Event::ChangesListed { session, result: services::changes::list(&dir) });
}

/// `r` (Scouter `loadDiff` with `onDiffChange`): lists the changes again while the screen shows. The selection keeps its
/// path, and the body reads its patches again.
pub fn reload(m: &mut Model) {
    if m.changes.is_none() {
        return;
    }
    if m.changes_reload.is_some() {
        m.message = Some(Message { text: BUSY.into(), error: false });
        return;
    }
    m.changes_session += 1;
    let session = m.changes_session;
    m.changes_reload = Some(session);
    m.notice = None;
    m.message = Some(Message { text: LOADING.into(), error: false });
    let dir: PathBuf = m.browser.tree.root().to_path_buf();
    services::spawn(&m.proxy, move || Event::ChangesListed { session, result: services::changes::list(&dir) });
}

/// A reload's listing arrives (Scouter `load` resolves, then `apply`). The body keeps its folds; a listing that
/// leaves no changes shows the empty state.
fn reloaded(m: &mut Model, result: Result<Listed, String>) {
    if m.message.as_ref().is_some_and(|message| message.text == LOADING) {
        m.message = None;
    }
    let Listed { repository, set, within } = match result {
        Ok(listed) => listed,
        Err(text) => return show_error(m, &text),
    };
    let Some(changes) = m.changes.as_mut() else { return };
    changes.repository = repository;
    changes.within = within;
    show_listing(m, set);
}

/// Shows a listing of the open screen's repository: the selection keeps its path, and the body reads its patches again,
/// or clears them when no change is left (Scouter `apply`).
fn show_listing(m: &mut Model, set: ChangeSet) {
    let keep = m.change_list.selected().map(|change| change.path.clone());
    m.change_list.show(set.changes, set.truncated, keep.as_deref());
    let empty = m.change_list.changes().is_empty();
    let Some(changes) = m.changes.as_mut() else { return };
    if empty {
        changes.files.clear();
        changes.generation += 1;
        changes.patched = true;
        changes.stamp.clear();
    } else {
        read(m, false);
    }
}

/// A changes poll's answer (Scouter `refresh`, `src/diff/diff-screen.ts:352-377`). A listing that differs is shown; else
/// the body reads its patches again when a file it shows has a new stamp. The next tick follows while the screen shows
/// the same session.
pub fn polled(m: &mut Model, session: u64, result: Result<(ChangeSet, String), String>) {
    let elapsed = m.diff_started.take().map_or(POLL, |started| started.elapsed());
    let Some(changes) = m.changes.as_ref().filter(|changes| changes.session == session) else { return };
    // A read in flight is left to finish: its stamp is checked on the next poll.
    let stale = changes.patched && result.as_ref().is_ok_and(|(_, stamp)| *stamp != changes.stamp);
    m.ticks.push(Tick { chain: Chain::Diff(session), delay: poll::backoff(elapsed) });
    let Ok((set, _)) = result else { return };
    if set.changes != m.change_list.changes() || set.truncated != m.change_list.truncated() {
        show_listing(m, set);
    } else if stale {
        read(m, false);
    }
}

/// The listing arrives (Scouter `load` resolves, then `enterDiff`). A listing the user cancelled is dropped.
pub fn listed(m: &mut Model, session: u64, result: Result<Listed, String>) {
    if m.changes_reload == Some(session) {
        m.changes_reload = None;
        return reloaded(m, result);
    }
    if m.changes_load != Some(session) {
        return;
    }
    m.changes_load = None;
    match result {
        Ok(listed) => enter(m, session, listed),
        Err(text) => show_error(m, &text),
    }
}

fn enter(m: &mut Model, session: u64, listed: Listed) {
    let Listed { repository, set, within } = listed;
    m.change_list.show(set.changes, set.truncated, None);
    m.ticks.push(Tick { chain: Chain::Diff(session), delay: POLL });
    let return_focus = m.focus;
    m.focus = Focus::List;
    m.message = None;
    m.notice = None;
    let empty = m.change_list.changes().is_empty();
    m.changes = Some(Changes {
        session,
        repository,
        browsed: m.browser.tree.root().to_path_buf(),
        within,
        patched: empty,
        stamp: String::new(),
        files: Vec::new(),
        digits: 2,
        generation: 0,
        request: 0,
        jump_when_read: false,
        folds: Folds::default(),
        focus_when_ready: false,
        return_focus,
    });
    if !empty {
        read(m, false);
    }
}

/// Reads the patches the body shows: the selected file in one-file scope, else every listed file (Scouter `select` and
/// `loadAll`). `jump` scrolls the body to the selected file once it shows. A reply for an older read is dropped.
fn read(m: &mut Model, jump: bool) {
    let context = fmc::diff::context(m.diff_options.wide);
    let one_file = m.diff_options.one_file;
    let selected = m.change_list.selected().cloned();
    let all = m.change_list.changes().to_vec();
    let Some(changes) = m.changes.as_mut() else { return };
    changes.request += 1;
    changes.patched = false;
    changes.jump_when_read = jump;
    let (session, request, repository) = (changes.session, changes.request, changes.repository.clone());
    if one_file {
        let Some(change) = selected else {
            changes.patched = true;
            return;
        };
        services::spawn(&m.proxy, move || Event::ChangesPatched {
            session,
            request,
            stamp: repository.0.stamp(std::slice::from_ref(&change)),
            result: Ok(vec![services::changes::patch(&repository, &change, context)]),
        });
    } else {
        services::spawn(&m.proxy, move || Event::ChangesPatched {
            session,
            request,
            stamp: repository.0.stamp(&all),
            result: services::changes::patches(&repository, &all, context),
        });
    }
}

/// The patches arrive. A reply for a read that is no longer the latest, or for a screen that has left, is dropped
/// (Scouter `loadAll` and `select`). A file that the one-file body moves to scrolls to its top, as Scouter's `presentOne`
/// does; the same file keeps its place.
pub fn patched(m: &mut Model, session: u64, request: u64, stamp: String, result: Result<Vec<FilePatch>, String>) {
    let one_file = m.diff_options.one_file;
    let selected = m.change_list.selected().map(|change| change.path.clone());
    let Some(changes) = m.changes.as_mut() else { return };
    if changes.session != session || changes.request != request {
        return;
    }
    changes.patched = true;
    let focus_when_ready = std::mem::take(&mut changes.focus_when_ready);
    let jump = std::mem::take(&mut changes.jump_when_read);
    // Scouter ignores a failed read of the patches: the list stays, and the body keeps what it showed.
    let mut target = jump.then_some(selected).flatten();
    if let Ok(files) = result {
        let moved = files.first().map(|file| &file.change.path) != changes.files.first().map(|file| &file.change.path);
        if one_file && moved {
            target = files.first().map(|file| file.change.path.clone());
        }
        changes.digits = gutter_digits(&files);
        changes.files = files;
        changes.stamp = stamp;
        changes.generation += 1;
    }
    if let Some(path) = target {
        m.diff_jump = Some((path, JUMP_TRIES));
    }
    if focus_when_ready {
        focus_body(m);
    }
}

/// `d` again, `Esc` from the body, or `q` on the screen: back to browsing, with the focus where it was before
/// (Scouter `leaveDiff`). The file's document, if any, is still open.
pub fn leave(m: &mut Model) {
    let Some(changes) = m.changes.take() else { return };
    // A reload still in flight is dropped when it arrives: the screen it was for has gone.
    m.changes_reload = None;
    m.diff_jump = None;
    m.focus = if changes.return_focus == Focus::Reader && m.shown.has_document() { Focus::Reader } else { Focus::List };
    // Scouter refreshes the tree and the marks as it leaves (`src/app/app.ts` `leaveDiff`); the timed chain goes on.
    poll::browse_now(m, false);
}

fn show_error(m: &mut Model, text: &str) {
    let text = display_label(text);
    m.message = Some(Message { text: text.clone(), error: true });
    m.notice = Some(text);
}

/// Enter, `l`, `→`, or a click on a change row: focuses the body when it has blocks; before the patches arrive, the
/// body takes focus when they do (Scouter `openSelected`).
pub fn activate(m: &mut Model) {
    let Some(changes) = m.changes.as_mut() else { return };
    if !changes.patched {
        changes.focus_when_ready = true;
    } else if !changes.files.is_empty() {
        m.focus = Focus::Reader;
    }
}

/// `Tab` on the screen: the list goes to the body when it has blocks, and the body back to the list (Scouter
/// `switchPane`).
pub fn switch_pane(m: &mut Model) {
    let has_blocks = m.changes.as_ref().is_some_and(|changes| !changes.files.is_empty());
    m.focus = if m.focus == Focus::List && has_blocks { Focus::Reader } else { Focus::List };
}

/// A list key: `j` and `k`, `J` and `K`, the page keys, and `g` and `G` (Scouter `handleListKey`). A moved selection
/// shows its file (Scouter `select`).
pub fn move_list(m: &mut Model, action: Action) {
    let moved = match action {
        Action::Move(delta) => m.change_list.move_by(delta as isize),
        Action::MovePage(direction) => m.change_list.page(direction > 0),
        Action::Top => m.change_list.move_to_edge(false),
        Action::Bottom => m.change_list.move_to_edge(true),
        _ => false,
    };
    if moved {
        selection_moved(m);
    }
}

/// A click on a change row selects it and opens its file (Scouter `ChangeList` `onMouseDown`).
pub fn click_row(m: &mut Model, index: usize) {
    if m.change_list.select_index(index) {
        selection_moved(m);
    }
    activate(m);
}

/// The list's selection moved: the stacked body scrolls to the file, and the one-file body reads it (Scouter `select`).
fn selection_moved(m: &mut Model) {
    if m.diff_options.one_file {
        read(m, false);
    } else if let Some(change) = m.change_list.selected() {
        m.diff_jump = Some((change.path.clone(), JUMP_TRIES));
    }
}

fn focus_body(m: &mut Model) {
    if m.changes.as_ref().is_some_and(|changes| !changes.files.is_empty()) {
        m.focus = Focus::Reader;
    }
}

/// Scrolls the body to the file the list selected (Scouter `jumpTo`). The body is laid out by a frame after its blocks
/// are built, so the jump waits for it; a file with no block is not jumped to.
pub fn jump(m: &mut Model, ui: &Ui) {
    let Some((path, tries)) = m.diff_jump.take() else { return };
    let Some(changes) = m.changes.as_ref() else { return };
    let Some(index) = changes.files.iter().position(|file| file.change.path == path) else { return };
    match ui.diff.tops() {
        Some(tops) if tops.len() == changes.files.len() => ui.diff.scroller.scroll_to(0.0, tops[index] as f64),
        _ if tries > 0 => m.diff_jump = Some((path, tries - 1)),
        _ => {}
    }
}

/// While the body has focus, the file at its top is the selected one (Scouter `followScroll`). Only the list moves;
/// the body is not scrolled again. A one-file body has no other file to follow to.
pub fn follow(m: &mut Model, ui: &Ui) {
    if m.focus != Focus::Reader || m.diff_jump.is_some() || m.diff_options.one_file {
        return;
    }
    let Some(changes) = m.changes.as_ref() else { return };
    let Some(tops) = ui.diff.tops() else { return };
    if tops.len() != changes.files.len() {
        return;
    }
    let top = ui.diff.scroller.position().1.floor() as usize;
    let Some(index) = block_at(&tops, top) else { return };
    let path = changes.files[index].change.path.clone();
    m.change_list.select_path(&path);
}

/// `a`: all files (stacked) or the selected file alone (Scouter `toggleScope`). Going to all files scrolls to the
/// selected file.
pub fn toggle_scope(m: &mut Model) {
    if m.changes.is_none() {
        return;
    }
    m.diff_options.one_file = !m.diff_options.one_file;
    let stacked = !m.diff_options.one_file;
    read(m, stacked);
}

/// `x`: normal or wide context (Scouter `toggleContext`). The patches are read again; the stacked body keeps the selected
/// file at the top.
pub fn toggle_context(m: &mut Model) {
    if m.changes.is_none() {
        return;
    }
    m.diff_options.wide = !m.diff_options.wide;
    let stacked = !m.diff_options.one_file;
    read(m, stacked);
}

/// `s`: side by side or one column (Scouter `toggleSplit`). Side by side needs a body of 60 columns; going back to one
/// column is always allowed.
pub fn toggle_split(m: &mut Model, body_columns: u32) {
    if m.diff_options.split {
        m.diff_options.split = false;
    } else if can_split(body_columns) {
        m.diff_options.split = true;
    } else {
        m.message = Some(Message { text: SPLIT_NEEDS_COLUMNS.into(), error: false });
    }
}

/// `c`: folds or unfolds the selected file (Scouter `toggleCollapse`). The header keeps its chevron.
pub fn toggle_fold(m: &mut Model) {
    let Some(path) = m.change_list.selected().map(|change| change.path.clone()) else { return };
    if let Some(changes) = m.changes.as_mut() {
        changes.folds.toggle(&path);
    }
}

/// A click on a file's header folds it (Scouter `createHeader`'s `onMouseDown`). The selection stays where it is.
pub fn click_header(m: &mut Model, ui: &Ui, y: f64) {
    let Some(index) = ui.diff.header_at(y) else { return };
    let Some(changes) = m.changes.as_mut() else { return };
    if let Some(file) = changes.files.get(index) {
        changes.folds.toggle(&file.change.path);
    }
}

/// `n` and `]` (forward), `N`, `p`, and `[` (back): the next or previous hunk across the files (Scouter `jumpHunk`). The
/// body scrolls to the hunk's row, as Scouter's `scrollTo` does; the edges say so in the footer.
pub fn step_hunk(m: &mut Model, ui: &Ui, forward: bool) {
    let Some(changes) = m.changes.as_ref() else { return };
    let top = ui.diff.scroller.position().1.floor() as usize;
    let hunks = ui.diff.hunks(&changes.files, &changes.folds);
    let message = match step(&hunks, top, forward) {
        Jump::To(row) => {
            ui.diff.scroller.scroll_to(0.0, row as f64);
            return;
        }
        Jump::First => FIRST_HUNK,
        Jump::Last => LAST_HUNK,
        Jump::Nothing => return,
    };
    m.message = Some(Message { text: message.into(), error: false });
}

/// `o`: opens the file at the hunk the body shows (Scouter `openChanged`). The screen is left, and the file opens in the
/// reader with its focus, at the hunk's new-file line. A deleted file has nothing to open.
pub fn open_changed(m: &mut Model, ui: &Ui) {
    let opened = {
        let Some(changes) = m.changes.as_ref() else { return };
        match m.change_list.selected() {
            Some(change) if change.kind != ChangeKind::Deleted => {
                let top = ui.diff.scroller.position().1.floor() as usize;
                let hunks = ui.diff.hunks(&changes.files, &changes.folds);
                let line = match changes.files.iter().position(|file| file.change.path == change.path) {
                    Some(file) => line_at(&hunks, file, top, first_line(&changes.files[file])),
                    None => 1,
                };
                Some((tree_path(changes, &change.path), changes.repository.0.root().join(&change.path), line))
            }
            _ => None,
        }
    };
    let Some((path, shown, line)) = opened else {
        m.message = Some(Message { text: NO_FILE_TO_OPEN.into(), error: false });
        return;
    };
    leave(m);
    m.open_at = Some(OpenAt { path: path.clone(), line, query: String::new(), shown: Some(shown) });
    tree::reveal(m, path, false, true);
}

/// A change's path in the tree: the browsed folder joined with the change's path inside it. A change outside the folder
/// keeps its path from the repository root.
fn tree_path(changes: &Changes, path: &str) -> PathBuf {
    match within_folder(&changes.within, path) {
        Some(inside) => changes.browsed.join(inside),
        None => changes.repository.0.root().join(path),
    }
}

/// The line a file's first hunk starts at, which `o` opens a file without a hunk in the stack at (Scouter's fallback in
/// `activeLineFor`).
fn first_line(file: &FilePatch) -> usize {
    match &file.patch {
        Patch::Text { text, .. } => hunk_starts(text).first().copied().unwrap_or(1),
        Patch::Notice(_) => 1,
    }
}

/// The selected file, for the `y` key while the screen shows (Scouter `currentPath`).
pub fn selected_path(m: &Model) -> Option<PathBuf> {
    m.change_list.selected().map(|change| PathBuf::from(&change.path))
}

/// The sidebar's header: the title and the count of listed changes, `+` when the list is cut (Scouter
/// `sidebarHeaderFor`).
pub fn sidebar_header(m: &Model) -> HeaderState {
    let count = format!("{}{}", m.change_list.changes().len(), if m.change_list.truncated() { "+" } else { "" });
    HeaderState {
        title: "Changes".into(),
        badges: vec![Badge::quiet(&count)],
        focused: m.focus == Focus::List,
        ..Default::default()
    }
}

/// The viewer's title and parent folder, and its badges: the selected file's kind, the scope, the split and wide-context
/// pills, and the wrap pill (Scouter `viewerHeaderFor`, `viewerBadgesFor`).
pub fn viewer_header(m: &Model) -> (String, Option<String>, Vec<Badge>) {
    let mut badges = Vec::new();
    let (title, parent) = match m.change_list.selected() {
        Some(change) => {
            badges.push(Badge::quiet(change.kind.name()));
            let (folder, name) = split_path(&change.path);
            (display_label(name), (folder != ".").then(|| format!("{}/", display_label(folder))))
        }
        None => ("Changes".into(), None),
    };
    let options = &m.diff_options;
    badges.push(Badge::quiet(if options.one_file { "one file" } else { "all files" }));
    if options.split {
        badges.push(Badge { text: "split".into(), kind: BadgeKind::Pill });
    }
    if options.wide {
        badges.push(Badge { text: "wide context".into(), kind: BadgeKind::Pill });
    }
    if m.wrap {
        badges.push(Badge { text: "wrap".into(), kind: BadgeKind::Pill });
    }
    (title, parent, badges)
}

/// The footer's facts for the screen (Scouter `diffFooterFor`): the selected file or the repository, the listed count,
/// the totals, and the hunk at the top of the body.
pub fn footer(m: &Model, ui: &Ui) -> Option<DiffInfo> {
    let changes = m.changes.as_ref()?;
    let location = match m.change_list.selected() {
        Some(change) => change.path.clone(),
        None => changes.repository.0.root().file_name().map_or(String::new(), |name| name.to_string_lossy().into()),
    };
    let totals = totals(&changes.files);
    let top = ui.diff.scroller.position().1.floor() as usize;
    let rows: Vec<usize> = ui.diff.hunks(&changes.files, &changes.folds).iter().map(|hunk| hunk.row).collect();
    Some(DiffInfo {
        location,
        files: m.change_list.changes().len(),
        all_files: !m.diff_options.one_file,
        truncated: m.change_list.truncated(),
        counts: totals.as_ref().map(|totals| (totals.added, totals.removed)),
        hunks: totals.as_ref().map(|totals| totals.hunks),
        hunk_index: active_hunk(&rows, top),
    })
}

/// The empty state's text: the listing has no change (Scouter `showEmpty`).
pub fn empty_state(m: &Model) -> Option<(&'static str, String)> {
    let changes = m.changes.as_ref()?;
    m.change_list.changes().is_empty().then(|| fmc::diff::empty_message(changes.repository.0.on_head()))
}

/// Paints the body's facts after the frame: the blocks follow the files, the active header follows the selection in
/// the stacked body, and folds show. Called from `paint` once the body is built.
pub fn sync(app: &App, m: &Model, ui: &Ui, p: &Palette, tint: &str) {
    let Some(changes) = m.changes.as_ref() else { return };
    if m.change_list.changes().is_empty() {
        let Some((title, detail)) = empty_state(m) else { return };
        ui.diff.empty(title, &detail, p);
        return;
    }
    let key = (changes.session, changes.generation, changes.digits, m.mode);
    ui.diff.sync(app, key, &changes.files, p);
    // Scouter marks the active header only in the stacked body (`activeHeaderPath`).
    let active =
        if m.diff_options.one_file { None } else { m.change_list.selected().map(|change| change.path.as_str()) };
    ui.diff.paint_headers(&changes.files, active, &changes.folds, tint, p);
}

#[cfg(test)]
#[path = "changes_tests.rs"]
mod tests;
