//! The file operations of the tree (plan-00012, D2 to D9): what `a`, `A`, `r`, `c`, `x`, `v`, `D`, Delete, and the context
//! menu ask for, the name and confirm dialogs they open, the clipboard, and the reveal of what a write made. The I/O is
//! `services/write.rs`; this file picks the target and says what a dialog's answer does.

use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use fmc::explorer::{Entry, Kind};
use fmc::keys::Action;
use fmc::overlay::{Dialog, Item, Menu, Overlay};
use fmc::text::{display_label, display_text};

use super::*;
use crate::services::write as store;

/// What a second paste or duplicate is told while a run is on the worker.
const BUSY: &str = "Still working. Try again when it finishes.";

/// What a run says for an entry that was replaced since it was taken: the run stops there, and the item is left alone.
const CHANGED: &str = "An item changed after it was copied or cut, so it was skipped.";

/// The entries a delete confirm names before `and K more`.
const LISTED: usize = 5;

/// What the tree's keys and the context menu ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    NewFile,
    NewFolder,
    Rename,
    Copy,
    Cut,
    Paste,
    Duplicate,
    Delete,
}

impl Act {
    /// The tree key that does the act (`keys/parser.rs`), shown at the menu row's right edge.
    fn hint(self) -> &'static str {
        match self {
            Act::NewFile => "a",
            Act::NewFolder => "A",
            Act::Rename => "r",
            Act::Copy => "c",
            Act::Cut => "x",
            Act::Paste => "v",
            Act::Duplicate => "R",
            Act::Delete => "D",
        }
    }

    /// The menu's groups: creating, changing an entry, and deleting. A rule separates them.
    fn group(self) -> u8 {
        match self {
            Act::NewFile | Act::NewFolder => 0,
            Act::Delete => 2,
            _ => 1,
        }
    }
}

/// The entries copy or cut put on fmc's clipboard (plan-00012, D2; plan-00014, D11): their paths, and whether a paste
/// moves them. The clipboard is fmc's own, not the system's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    pub paths: Vec<PathBuf>,
    /// Each path's device and inode when `c` or `x` took it, in the same order. A run refuses an entry that has been
    /// replaced since, as the Trash does (security review of phase 3, finding 1).
    pub ids: Vec<(u64, u64)>,
    pub cut: bool,
}

/// One item of a run: the entry it acts on, the folder it goes into, the name a copy takes (`None` takes the free name
/// a paste gives, plan-00012 D5), and the entry's device and inode when the run was started.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Op {
    id: (u64, u64),
    source: PathBuf,
    into: PathBuf,
    name: Option<String>,
}

/// A run of items on the worker, one at a time (plan-00014, D7 and D12). The run stops at its first error, and the
/// marks of a run that lands every item go away (D13).
pub struct Bulk {
    kind: store::Kind,
    queue: VecDeque<Op>,
    total: usize,
    done: usize,
    /// Every entry the run was started for, so the clipboard can end when its cut has run.
    sources: Vec<PathBuf>,
    /// The new entry the last item made, which the tree reveals when the run ends.
    reveal: Option<PathBuf>,
    /// The folder the tree lists again when the run ends.
    folder: PathBuf,
}

impl Bulk {
    fn new(kind: store::Kind, queue: Vec<Op>, sources: Vec<PathBuf>, folder: PathBuf) -> Self {
        let total = queue.len();
        Self { kind, queue: VecDeque::from(queue), total, done: 0, sources, reveal: None, folder }
    }
}

/// What the open menu or dialog is for: the act of each menu row, or the entry a dialog acts on.
#[derive(Clone, Debug)]
pub enum Task {
    Menu(Vec<Act>),
    /// A name for a new entry in `dir`. `folder` makes a folder, and so does a name that ends in `/`.
    Name {
        folder: bool,
        dir: PathBuf,
    },
    Rename {
        path: PathBuf,
    },
    /// A copy of `source` in its folder `dir`, under the name the dialog holds.
    Duplicate {
        source: PathBuf,
        dir: PathBuf,
    },
    /// A number row of the Settings page: the dialog's text is the new value (`app/settings.rs`).
    Setting(fmc::settings::Row),
    /// The entries to trash, each with its device and inode when the dialog opened (an entry swapped since then is not
    /// trashed, plan-00014 D15), and the row to select after them: the next row, or the one before it when it was last.
    Delete {
        items: Vec<(PathBuf, (u64, u64))>,
        next: Option<PathBuf>,
    },
}

/// The menu's rows: New file and New folder always, the entry's acts when a row is selected, and Paste when the
/// clipboard holds something (its target is the selected folder, or the root). Copy, Cut, and Paste name how many
/// entries they take (plan-00014, D6).
fn menu_rows(row: bool, clipped: usize, acted: usize) -> Vec<(Act, String, bool)> {
    let mut rows = vec![(Act::NewFile, "New file…".into(), true), (Act::NewFolder, "New folder…".into(), true)];
    if row {
        rows.extend([
            (Act::Rename, "Rename…".into(), true),
            (Act::Copy, counted("Copy", acted), true),
            (Act::Cut, counted("Cut", acted), true),
        ]);
    }
    rows.push((Act::Paste, counted("Paste", clipped), clipped > 0));
    if row {
        rows.push((Act::Duplicate, "Duplicate…".into(), true));
        rows.push((Act::Delete, format!("{}…", counted("Delete", acted)), true));
    }
    rows
}

/// `Copy`, or `Copy 3 items` for more than one entry.
fn counted(label: &str, count: usize) -> String {
    if count > 1 { format!("{label} {count} items") } else { label.to_owned() }
}

/// `Space`: the context menu, anchored at the selected row's cell (plan-00012, D3).
pub fn open_menu(m: &mut Model) {
    let row = m.browser.selected_index().saturating_sub(m.browser.top());
    let cell = (2, LIST_TOP + row as u16);
    open_menu_at(m, cell);
}

/// A right-click on a row, or `Space`: the context menu at `cell`.
pub fn open_menu_at(m: &mut Model, cell: (u16, u16)) {
    let clipped = m.clipboard.as_ref().map_or(0, |clip| clip.paths.len());
    let rows = menu_rows(m.browser.selected_entry().is_some(), clipped, acted_on(m).len());
    let items = rows
        .iter()
        .enumerate()
        .map(|(index, &(act, ref label, enabled))| Item {
            label: label.clone(),
            enabled,
            hint: act.hint(),
            danger: act == Act::Delete,
            rule: index > 0 && rows[index - 1].0.group() != act.group(),
        })
        .collect();
    m.overlay_at = cell;
    m.overlay = Some(Overlay::Menu(Menu::new(items)));
    m.overlay_task = Some(Task::Menu(rows.into_iter().map(|(act, ..)| act).collect()));
}

/// `a`, `A`, `r`, `D`, and Delete on the tree.
pub fn key(m: &mut Model, ui: &Ui, action: Action) {
    let act = match action {
        Action::NewFile => Act::NewFile,
        Action::NewFolder => Act::NewFolder,
        Action::Rename => Act::Rename,
        Action::Duplicate => Act::Duplicate,
        Action::Delete => Act::Delete,
        _ => return,
    };
    start(m, ui, act);
}

/// Carries out an act on the selected row, or on the root when no row is selected. A dialog opens where it needs a name
/// or a confirmation.
pub fn start(m: &mut Model, ui: &Ui, act: Act) {
    let entry = m.browser.selected_entry();
    match act {
        Act::NewFile | Act::NewFolder => {
            let folder = act == Act::NewFolder;
            let dir = target_folder(m, entry.as_ref());
            let dialog = if folder {
                Dialog::new("New folder", "Name the folder.", true, &["Cancel", "OK"], 1)
            } else {
                Dialog::new("New file", "Name it. A name ending in / makes a folder.", true, &["Cancel", "OK"], 1)
            };
            ask(m, ui, Task::Name { folder, dir }, dialog, "");
        }
        Act::Rename => {
            let Some(entry) = entry else { return };
            // The field types the name as text: a name that is not UTF-8 has no text, and a control character would
            // reach the terminal from the field, which does not filter it when it is set.
            let Some(name) = entry.path.file_name().and_then(|name| name.to_str()) else {
                return refuse(m, "This name is not UTF-8, so fmc cannot rename it.");
            };
            if display_text(name) != name {
                return refuse(m, "This name has a control character, so fmc cannot rename it.");
            }
            let dialog = Dialog::new("Rename", "Give it a new name.", true, &["Cancel", "OK"], 1);
            let name = name.to_owned();
            ask(m, ui, Task::Rename { path: entry.path }, dialog, &name);
        }
        Act::Delete => {
            let paths = acted_paths(m);
            // The Trash names each entry by its path as text (plan-00012, D6), as the write layer refuses.
            if paths.iter().any(|path| path.file_name().and_then(|name| name.to_str()).is_none()) {
                return refuse(m, "This name is not UTF-8, so fmc cannot move it to the Trash.");
            }
            let items = acted_on(m);
            if items.is_empty() {
                // Entries that are all gone are a refusal, not a silent no-op; no row and no mark is nothing to delete.
                if !paths.is_empty() {
                    refuse(m, &store::Error::Missing.to_string());
                }
                return;
            }
            // Finder refuses a link to the Trash (error -5000, checked on macOS 27 with a link and a dangling link), so a
            // link stops the delete before the dialog, and nothing moves.
            if items.iter().any(|(path, _)| fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()))
            {
                return refuse(m, "A link cannot go to the Trash through Finder, so nothing was moved.");
            }
            // The row to select afterwards is kept out of the run: a trashed row cannot be revealed.
            let next = successor(m).filter(|next| !items.iter().any(|(path, _)| path == next));
            let names: Vec<String> = items.iter().map(|(path, _)| label(path)).collect();
            let (title, message) = if names.len() == 1 {
                ("Move to Trash?".to_owned(), format!("{} goes to the Trash. Finder's Put Back restores it.", names[0]))
            } else {
                (
                    format!("Move {} items to the Trash?", names.len()),
                    format!("These go to the Trash. Finder's Put Back restores them:\n{}", listed(&names)),
                )
            };
            let dialog = Dialog::new(&title, &message, false, &["Cancel", "Delete"], 0);
            ask(m, ui, Task::Delete { items, next }, dialog, "");
        }
        Act::Duplicate => {
            let Some(entry) = entry else { return };
            // The field types the name as text, as a rename does. The copy is named as a paste names one (D5).
            let Some(name) = entry.path.file_name().and_then(|name| name.to_str()) else {
                return refuse(m, "This name is not UTF-8, so fmc cannot duplicate it.");
            };
            if display_text(name) != name {
                return refuse(m, "This name has a control character, so fmc cannot duplicate it.");
            }
            let root = m.browser.tree.root().to_path_buf();
            let dir = parent_of(&entry.path, &root);
            let is_dir = fs::symlink_metadata(&entry.path).is_ok_and(|metadata| metadata.is_dir());
            let copy = match store::free_name(&dir, name, is_dir) {
                Ok(copy) => copy,
                Err(error) => return refuse(m, &error.to_string()),
            };
            let dialog =
                Dialog::new("Duplicate", "Name the copy. It goes in the same folder.", true, &["Cancel", "OK"], 1);
            ask(m, ui, Task::Duplicate { source: entry.path, dir }, dialog, &copy);
        }
        Act::Copy => take(m, false),
        Act::Cut => take(m, true),
        Act::Paste => paste(m),
    }
}

/// The entries `c`, `x`, and `D` act on: the marked entries in path order, or the selected row when none is marked
/// (plan-00014, D4). A mark on an entry that is gone from disk is left out, so it cannot stop a run.
fn acted_on(m: &Model) -> Vec<(PathBuf, (u64, u64))> {
    acted_paths(m).into_iter().filter_map(with_id).collect()
}

/// The paths `c`, `x`, and `D` mean before the check that they are still there: the marked paths, or the selected row.
fn acted_paths(m: &Model) -> Vec<PathBuf> {
    let marked = m.browser.targets();
    if marked.is_empty() {
        return m.browser.selected_entry().map(|entry| entry.path).into_iter().collect();
    }
    marked
}

/// A path with its device and inode, or `None` when nothing is there.
fn with_id(path: PathBuf) -> Option<(PathBuf, (u64, u64))> {
    let id = entry_identity(&path)?;
    Some((path, id))
}

/// `c` and `x`, and the menu's Copy and Cut: the entries go on the clipboard, and a notice says so (plan-00014, D4).
pub fn take(m: &mut Model, cut: bool) {
    let (paths, ids): (Vec<PathBuf>, Vec<(u64, u64)>) = acted_on(m).into_iter().unzip();
    let verb = if cut { "Cut" } else { "Copied" };
    let text = match paths.as_slice() {
        [] => return,
        [only] => format!("{verb} {}. Paste it with v.", label(only)),
        many => format!("{verb} {} items. Paste them with v.", many.len()),
    };
    m.clipboard = Some(Clip { paths, ids, cut });
    m.message = Some(Message { text, error: false });
}

/// `v`, and the menu's Paste: the clipboard goes into the target folder, one item at a time on the worker (plan-00012,
/// D8; plan-00014, D7). A cut entry already in that folder is skipped, and a cut with nothing left to move ends.
pub fn paste(m: &mut Model) {
    let Some(clip) = m.clipboard.clone() else { return };
    if m.bulk.is_some() {
        m.message = Some(Message { text: BUSY.into(), error: false });
        return;
    }
    let root = m.browser.tree.root().to_path_buf();
    let into = target_folder(m, m.browser.selected_entry().as_ref());
    let items: Vec<(PathBuf, (u64, u64))> = clip
        .paths
        .iter()
        .zip(&clip.ids)
        .filter(|(path, _)| !clip.cut || parent_of(path, &root) != into)
        .map(|(path, id)| (path.clone(), *id))
        .collect();
    if items.is_empty() {
        if clip.cut {
            m.clipboard = None;
        }
        return;
    }
    let kind = if clip.cut { store::Kind::Move } else { store::Kind::Copy };
    let queue = items.into_iter().map(|(source, id)| Op { source, into: into.clone(), name: None, id }).collect();
    begin(m, Bulk::new(kind, queue, clip.paths, into));
}

/// Starts a run: its first item goes to the worker, and the rest follow as each one answers.
fn begin(m: &mut Model, bulk: Bulk) {
    m.bulk = Some(bulk);
    advance(m);
}

/// Sends the run's next item to the worker, and names it in the footer: `copying Robert Langdon…` for one entry, else
/// `copying 2 of 5…`.
fn advance(m: &mut Model) {
    let root = m.browser.tree.root().to_path_buf();
    let Some(bulk) = m.bulk.as_mut() else { return };
    let Some(op) = bulk.queue.pop_front() else { return };
    let kind = bulk.kind;
    let verb = match kind {
        store::Kind::Copy => "copying",
        store::Kind::Move => "moving",
        store::Kind::Trash => "trashing",
    };
    let text = if bulk.total == 1 {
        format!("{verb} {}…", label(&op.source))
    } else {
        format!("{verb} {} of {}…", bulk.done + 1, bulk.total)
    };
    m.message = Some(Message { text, error: false });
    let source = op.source.clone();
    crate::services::spawn(&m.proxy, move || {
        let result = run(&root, kind, &op);
        Event::Written { kind, source, result }
    });
}

/// Runs one item on the worker: the write layer's copy, paste, or move, (plan-00014, D12).
fn run(root: &Path, kind: store::Kind, op: &Op) -> Result<PathBuf, String> {
    // An entry replaced since `c`, `x`, or the duplicate's dialog is left alone. An entry that is gone gets the write
    // layer's own error instead. The check and the write are two calls, the same shortcut as the Trash's (`submit`).
    if entry_identity(&op.source).is_some_and(|id| id != op.id) {
        return Err(CHANGED.to_owned());
    }
    let result = match (kind, &op.name) {
        (store::Kind::Copy, Some(name)) => store::copy(root, &op.source, &op.into, name),
        (store::Kind::Copy, None) => store::paste(root, &op.source, &op.into, false),
        (store::Kind::Move, _) => store::paste(root, &op.source, &op.into, true),
        (store::Kind::Trash, _) => trash(root, &op.source).map(|()| op.source.clone()),
    };
    result.map_err(|error| error.to_string())
}

/// An item's answer (`Event::Written`). The run goes on with its next item, or ends. At the first error it stops, and
/// the items it did not reach stay where they are. An entry that left its folder is listed again, so the tree drops it.
pub fn written(m: &mut Model, kind: store::Kind, source: &Path, result: Result<PathBuf, String>) {
    if kind != store::Kind::Copy {
        let root = m.browser.tree.root().to_path_buf();
        relist_shown(m, parent_of(source, &root));
    }
    let Some(mut bulk) = m.bulk.take() else { return };
    match result {
        Ok(target) => {
            bulk.done += 1;
            // A Trash reveals the row chosen when its confirm opened: the entry it trashed is gone, so it names nothing.
            if kind != store::Kind::Trash {
                bulk.reveal = Some(target);
            }
            if bulk.queue.is_empty() {
                end(m, bulk, None);
            } else {
                m.bulk = Some(bulk);
                advance(m);
            }
        }
        Err(error) => end(m, bulk, Some(error)),
    }
}

/// Ends a run. The cut it moved ends with it. The marks go when every item landed (plan-00014, D13). The tree reveals
/// the last new entry, or lists the folder again when no entry was made.
fn end(m: &mut Model, bulk: Bulk, error: Option<String>) {
    if bulk.kind == store::Kind::Move && m.clipboard.as_ref().is_some_and(|clip| clip.cut && clip.paths == bulk.sources)
    {
        m.clipboard = None;
    }
    match &error {
        None => {
            m.message = None;
            m.browser.clear_marks();
        }
        Some(text) => refuse(m, &stopped(&bulk, text)),
    }
    tree::refresh_marks(m);
    match bulk.reveal {
        Some(target) => tree::relist_to(m, bulk.folder, target),
        None => relist_shown(m, bulk.folder),
    }
}

/// The footer's words for a run that stopped: a single entry's error alone, or `Copied 2 of 5. <error>`.
fn stopped(bulk: &Bulk, error: &str) -> String {
    if bulk.total == 1 {
        return error.to_owned();
    }
    let verb = match bulk.kind {
        store::Kind::Copy => "Copied",
        store::Kind::Move => "Moved",
        store::Kind::Trash => "Trashed",
    };
    format!("{verb} {} of {}. {error}", bulk.done, bulk.total)
}

/// Lists a folder again if the tree shows it open: a listing of a closed folder would open it.
fn relist_shown(m: &mut Model, folder: PathBuf) {
    if m.browser.tree.is_expanded(&folder) {
        tree::list(m, folder);
    }
}

/// An entry's name as the footer shows it.
fn label(path: &Path) -> String {
    display_label(&path.file_name().unwrap_or(OsStr::new("")).to_string_lossy())
}

/// A menu row or a dialog button was picked, by a key or a press. The menu closes and its act starts. A dialog's Cancel
/// closes it; any other button runs its task, and a refusal keeps the dialog open with its message.
pub fn pick(app: &App, m: &mut Model, ui: &Ui, index: usize) {
    match m.overlay_task.clone() {
        Some(Task::Menu(acts)) => {
            overlay::close(app, m);
            if let Some(act) = acts.get(index) {
                start(m, ui, *act);
            }
        }
        Some(task) => submit(app, m, ui, task, index),
        None => overlay::close(app, m),
    }
}

/// Runs a dialog's task for its button `index`: Cancel is button 0 in every dialog here.
fn submit(app: &App, m: &mut Model, ui: &Ui, task: Task, index: usize) {
    if index == 0 {
        return overlay::close(app, m);
    }
    let root = m.browser.tree.root().to_path_buf();
    let typed = ui.overlay.dialog.field.value();
    let done = match &task {
        Task::Name { folder, dir } => {
            // One trailing `/` makes a folder; the name itself is what the write layer checks.
            let (name, slash) = typed.strip_suffix('/').map_or((typed.as_str(), false), |name| (name, true));
            let made = if *folder || slash {
                store::create_dir(&root, dir, name)
            } else {
                store::create_file(&root, dir, name)
            };
            made.map(|path| (dir.clone(), path))
        }
        Task::Rename { path } => {
            let parent = parent_of(path, &root);
            if path.file_name() == Some(OsStr::new(&typed)) {
                Ok((parent, path.clone()))
            } else {
                store::rename(&root, path, &typed).map(|dest| (parent, dest))
            }
        }
        Task::Delete { items, next } => return delete(app, m, &root, items, next.clone()),
        Task::Duplicate { source, dir } => return duplicate(app, m, &root, &typed, source, dir),
        Task::Setting(row) => return settings::submit(app, m, *row, &typed),
        Task::Menu(_) => return overlay::close(app, m),
    };
    match done {
        Ok((folder, target)) => {
            overlay::close(app, m);
            tree::relist_to(m, folder, target);
            tree::refresh_marks(m);
        }
        Err(error) => {
            if let Some(Overlay::Dialog(dialog)) = m.overlay.as_mut() {
                dialog.error = Some(display_label(&error.to_string()));
            }
        }
    }
}

/// Starts the Trash of a confirm's entries, as a run on the worker (plan-00014, D12). Every entry is checked before the
/// first call: one that was replaced since the dialog opened refuses the whole run, as a single delete always did (D15).
fn delete(app: &App, m: &mut Model, root: &Path, items: &[(PathBuf, (u64, u64))], next: Option<PathBuf>) {
    let Some((first, _)) = items.first() else { return overlay::close(app, m) };
    if m.bulk.is_some() {
        if let Some(Overlay::Dialog(dialog)) = m.overlay.as_mut() {
            dialog.error = Some(BUSY.to_owned());
        }
        return;
    }
    let folder = next.as_ref().map_or_else(|| parent_of(first, root), |next| parent_of(next, root));
    overlay::close(app, m);
    if items.iter().any(|(path, id)| entry_identity(path) != Some(*id)) {
        // Another entry now holds a name: nothing is trashed, and the folder is listed again with that entry shown.
        refuse(m, "That entry changed since it was listed, so nothing went to the Trash.");
        tree::relist_to(m, parent_of(first, root), first.clone());
        return;
    }
    let queue = items
        .iter()
        .map(|(source, id)| Op { source: source.clone(), into: parent_of(source, root), name: None, id: *id })
        .collect();
    let sources = items.iter().map(|(path, _)| path.clone()).collect();
    let mut bulk = Bulk::new(store::Kind::Trash, queue, sources, folder);
    bulk.reveal = next;
    begin(m, bulk);
}

/// The names a confirm lists: one a line, up to `LISTED` of them, then `and K more` (plan-00014, D6).
fn listed(names: &[String]) -> String {
    let mut text = names.iter().take(LISTED).cloned().collect::<Vec<_>>().join("\n");
    let more = names.len().saturating_sub(LISTED);
    if more > 0 {
        text.push_str(&format!("\nand {more} more"));
    }
    text
}

/// Starts a duplicate. The name is checked here, so a taken name or a paste that is running keeps the dialog open with its
/// line; the copy runs on the paste worker, and `pasted` reveals it (D9).
fn duplicate(app: &App, m: &mut Model, root: &Path, typed: &str, source: &Path, dir: &Path) {
    let refused = if m.bulk.is_some() {
        Some(BUSY.to_owned())
    } else {
        store::check_copy(root, source, dir, typed).err().map(|error| error.to_string())
    };
    if let Some(text) = refused {
        if let Some(Overlay::Dialog(dialog)) = m.overlay.as_mut() {
            dialog.error = Some(display_label(&text));
        }
        return;
    }
    overlay::close(app, m);
    // `check_copy` has just looked at the source, so it has an identity; if it is gone, the run reports that instead.
    let id = entry_identity(source).unwrap_or_default();
    let op = Op { source: source.to_path_buf(), into: dir.to_path_buf(), name: Some(typed.to_owned()), id };
    begin(m, Bulk::new(store::Kind::Copy, vec![op], vec![source.to_path_buf()], dir.to_path_buf()));
}

/// Moves the entry to the Trash, where Finder's Put Back restores it (plan-00012, D4).
#[cfg(not(test))]
fn trash(root: &Path, path: &Path) -> Result<(), store::Error> {
    store::trash(root, path)
}

/// The tests stand in for the Trash, so a test never writes to the owner's Trash. The root and the name checks still run.
#[cfg(test)]
fn trash(root: &Path, path: &Path) -> Result<(), store::Error> {
    store::trash_with(root, path, |path| {
        if std::fs::symlink_metadata(path)?.is_dir() {
            std::fs::remove_dir_all(path)?
        } else {
            std::fs::remove_file(path)?
        }
        Ok(())
    })
}

/// The device and inode of the entry itself, not of a link's target: an entry swapped for another has another pair.
fn entry_identity(path: &Path) -> Option<(u64, u64)> {
    fs::symlink_metadata(path).ok().map(|metadata| (metadata.dev(), metadata.ino()))
}

/// Opens a dialog over the page with `task`, and its field set to `text`.
fn ask(m: &mut Model, ui: &Ui, task: Task, dialog: Dialog, text: &str) {
    ui.overlay.dialog.field.set_value(text);
    m.overlay = Some(Overlay::Dialog(dialog));
    m.overlay_task = Some(task);
}

/// The folder a new entry goes into: the selected folder, the folder of the selected file (as paste does, plan-00012
/// D2), or the root when no row is selected.
fn target_folder(m: &Model, entry: Option<&Entry>) -> PathBuf {
    let root = m.browser.tree.root();
    match entry {
        Some(entry) if entry.kind == Kind::Directory => entry.path.clone(),
        Some(entry) => entry.path.parent().unwrap_or(root).to_path_buf(),
        None => root.to_path_buf(),
    }
}

/// The folder an entry lies in, the root for a top-level entry.
fn parent_of(path: &Path, root: &Path) -> PathBuf {
    path.parent().unwrap_or(root).to_path_buf()
}

/// The row a delete selects (plan-00012, D9): the row after the entry and the rows under it, or the row before the entry
/// when it was the last.
fn successor(m: &Model) -> Option<PathBuf> {
    let rows = m.browser.tree.rows();
    let at = m.browser.selected_index();
    let depth = rows.get(at)?.depth;
    let after = rows[at + 1..].iter().position(|row| row.depth <= depth).map_or(rows.len(), |offset| at + 1 + offset);
    let next = rows.get(after).or_else(|| at.checked_sub(1).and_then(|before| rows.get(before)));
    next.map(|row| row.entry.path.clone())
}

/// A refusal in the footer, as an error (the footer's message).
fn refuse(m: &mut Model, text: &str) {
    m.message = Some(Message { text: text.into(), error: true });
}

#[cfg(test)]
#[path = "write_tests.rs"]
mod tests;
