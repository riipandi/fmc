//! The changes screen on the UI thread (Scouter `src/app/app.ts` `toggleDiff`, `loadDiff`, `enterDiff`, `leaveDiff`,
//! `handleChangesKey`, and `src/diff/diff-screen.ts` `load`, `select`, `layoutChanged`). The list and the stacked body
//! are drawn from the core's state; the listing and the patches come back from workers as events, and a reply that
//! no longer matches the screen is dropped.

use std::path::PathBuf;

use fmc::diff::{FilePatch, active_hunk, block_at, gutter_digits, split_path, totals};
use fmc::keys::{Action, Focus};
use fmc::text::display_label;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::{Model, Ui};
use crate::services::changes::{Listed, Shared};
use crate::services::{self, Event};
use crate::ui::footer::{DiffInfo, Message};
use crate::ui::header::{Badge, BadgeKind, HeaderState};

/// How many paints the body may take to lay out before a jump to a file is given up.
const JUMP_TRIES: u8 = 8;

/// The changes screen while it shows (Scouter `DiffScreen`): the listing's repository and the stacked files.
pub struct Changes {
    /// The listing this screen shows. A patch reply for another session is dropped.
    pub session: u64,
    pub repository: Shared,
    /// Whether the patches of the listing have arrived; until then the body has no blocks.
    pub patched: bool,
    /// The stacked files, one per listed change, with their patch text.
    pub files: Vec<FilePatch>,
    /// The line-number width the blocks share (`fmc::diff::gutter_digits`).
    pub digits: usize,
    /// Bumped with each new `files`, so the body is built once for them.
    pub generation: u64,
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
        m.message = Some(Message { text: "Still loading. Try again when it finishes.".into(), error: false });
        return;
    }
    m.changes_session += 1;
    let session = m.changes_session;
    m.changes_load = Some(session);
    m.message = Some(Message { text: "Loading changes…".into(), error: false });
    m.notice = None;
    let dir: PathBuf = m.browser.tree.root().to_path_buf();
    services::spawn(&m.proxy, move || Event::ChangesListed { session, result: services::changes::list(&dir) });
}

/// The listing arrives (Scouter `load` resolves, then `enterDiff`). A listing the user cancelled is dropped.
pub fn listed(m: &mut Model, session: u64, result: Result<Listed, String>) {
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
    let Listed { repository, set } = listed;
    m.change_list.show(set.changes, set.truncated, None);
    let changes = m.change_list.changes().to_vec();
    let return_focus = m.focus;
    m.focus = Focus::List;
    m.message = None;
    m.notice = None;
    let empty = changes.is_empty();
    m.changes = Some(Changes {
        session,
        repository: repository.clone(),
        patched: empty,
        files: Vec::new(),
        digits: 2,
        generation: 0,
        focus_when_ready: false,
        return_focus,
    });
    if !empty {
        services::spawn(&m.proxy, move || Event::ChangesPatched {
            session,
            result: services::changes::patches(&repository, &changes),
        });
    }
}

/// The stacked patches arrive. A reply for a screen that has left is dropped (Scouter `loadAll`).
pub fn patched(m: &mut Model, session: u64, result: Result<Vec<FilePatch>, String>) {
    let Some(changes) = m.changes.as_mut() else { return };
    if changes.session != session {
        return;
    }
    changes.patched = true;
    let focus_when_ready = std::mem::take(&mut changes.focus_when_ready);
    // Scouter ignores a failed read of the patches: the list stays, and the body has no blocks.
    if let Ok(files) = result {
        changes.digits = gutter_digits(&files);
        changes.files = files;
        changes.generation += 1;
    }
    if focus_when_ready {
        focus_body(m);
    }
}

/// `d` again, `Esc` from the body, or `q` on the screen: back to browsing, with the focus where it was before
/// (Scouter `leaveDiff`). The file's document, if any, is still open.
pub fn leave(m: &mut Model) {
    let Some(changes) = m.changes.take() else { return };
    m.diff_jump = None;
    m.focus = if changes.return_focus == Focus::Reader && m.shown.has_document() { Focus::Reader } else { Focus::List };
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
/// scrolls the body to its file (Scouter `select`, `jumpTo`).
pub fn move_list(m: &mut Model, action: Action) {
    let moved = match action {
        Action::Move(delta) => m.change_list.move_by(delta as isize),
        Action::MovePage(direction) => m.change_list.page(direction > 0),
        Action::Top => m.change_list.move_to_edge(false),
        Action::Bottom => m.change_list.move_to_edge(true),
        _ => false,
    };
    if moved {
        jump_to_selected(m);
    }
}

/// A click on a change row selects it and opens its file (Scouter `ChangeList` `onMouseDown`).
pub fn click_row(m: &mut Model, index: usize) {
    if m.change_list.select_index(index) {
        jump_to_selected(m);
    }
    activate(m);
}

fn jump_to_selected(m: &mut Model) {
    if let Some(change) = m.change_list.selected() {
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
/// the body is not scrolled again.
pub fn follow(m: &mut Model, ui: &Ui) {
    if m.focus != Focus::Reader || m.diff_jump.is_some() {
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

/// The file the `y` key copies while the screen shows: the selected change (Scouter `currentPath`).
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

/// The viewer's title and parent folder, and its badges: the selected file's kind, the scope, and the wrap pill
/// (Scouter `viewerHeaderFor`, `viewerBadgesFor`).
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
    badges.push(Badge::quiet("all files"));
    if m.wrap {
        badges.push(Badge { text: "wrap".into(), kind: BadgeKind::Pill });
    }
    (title, parent, badges)
}

/// The footer's facts for the screen (Scouter `diffFooterFor`): the selected file or the repository, the listed
/// count, the totals, and the hunk at the top of the body.
pub fn footer(m: &Model, ui: &Ui) -> Option<DiffInfo> {
    let changes = m.changes.as_ref()?;
    let location = match m.change_list.selected() {
        Some(change) => change.path.clone(),
        None => changes.repository.0.root().file_name().map_or(String::new(), |name| name.to_string_lossy().into()),
    };
    let totals = totals(&changes.files);
    let top = ui.diff.scroller.position().1.floor() as usize;
    Some(DiffInfo {
        location,
        files: m.change_list.changes().len(),
        all_files: true,
        truncated: m.change_list.truncated(),
        counts: totals.as_ref().map(|totals| (totals.added, totals.removed)),
        hunks: totals.as_ref().map(|totals| totals.hunks),
        hunk_index: active_hunk(&ui.diff.hunk_rows(), top),
    })
}

/// The empty state's text: the listing has no change (Scouter `showEmpty`).
pub fn empty_state(m: &Model) -> Option<(&'static str, String)> {
    let changes = m.changes.as_ref()?;
    m.change_list.changes().is_empty().then(|| fmc::diff::empty_message(changes.repository.0.on_head()))
}

/// Paints the body's facts after the frame: the blocks follow the files, a pending jump runs, and the list follows the
/// body. Called from `paint` once the body is built.
pub fn sync(app: &App, m: &Model, ui: &Ui, p: &Palette, tint: &str) {
    let Some(changes) = m.changes.as_ref() else { return };
    if m.change_list.changes().is_empty() {
        let Some((title, detail)) = empty_state(m) else { return };
        ui.diff.empty(title, &detail, p);
        return;
    }
    let key = (changes.session, changes.generation, changes.digits, m.mode);
    ui.diff.sync(app, key, &changes.files, p);
    let active = m.change_list.selected().map(|change| change.path.as_str());
    ui.diff.paint_headers(&changes.files, active, tint, p);
}
