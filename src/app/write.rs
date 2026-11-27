//! The file operations of the tree (plan-00012, D2 to D9): what `a`, `A`, `r`, `c`, `x`, `v`, `D`, Delete, and the context
//! menu ask for, the name and confirm dialogs they open, the clipboard, and the reveal of what a write made. The I/O is
//! `services/write.rs`; this file picks the target and says what a dialog's answer does.

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

/// What the tree's keys and the context menu ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    NewFile,
    NewFolder,
    Rename,
    Copy,
    Cut,
    Paste,
    Delete,
}

/// The entry copy or cut put on fmc's clipboard (plan-00012, D2): its path, and whether a paste moves it. The clipboard
/// is fmc's own, not the system's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    pub path: PathBuf,
    pub cut: bool,
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
    /// The entry to trash, the row to select after it (the next row, or the previous one when it was last), and the
    /// entry's device and inode when the dialog opened: an entry swapped since then is not trashed.
    Delete {
        path: PathBuf,
        next: Option<PathBuf>,
        identity: (u64, u64),
    },
}

/// The menu's rows: New file and New folder always, the entry's acts when a row is selected, and Paste when the
/// clipboard holds something (its target is the selected folder, or the root).
fn menu_rows(row: bool, pastable: bool) -> Vec<(Act, &'static str, bool)> {
    let mut rows = vec![(Act::NewFile, "New file…", true), (Act::NewFolder, "New folder…", true)];
    if row {
        rows.extend([(Act::Rename, "Rename…", true), (Act::Copy, "Copy", true), (Act::Cut, "Cut", true)]);
    }
    rows.push((Act::Paste, "Paste", pastable));
    if row {
        rows.push((Act::Delete, "Delete…", true));
    }
    rows
}

/// `Space`: the context menu, anchored at the selected row's cell (plan-00012, D3).
pub fn open_menu(m: &mut Model) {
    let row = m.browser.selected_index().saturating_sub(m.browser.top());
    let cell = (2, LIST_TOP + row as u16);
    open_menu_at(m, cell);
}

/// A right-click on a row, or `Space`: the context menu at `cell`.
pub fn open_menu_at(m: &mut Model, cell: (u16, u16)) {
    let rows = menu_rows(m.browser.selected_entry().is_some(), m.clipboard.is_some());
    let items = rows.iter().map(|(_, label, enabled)| Item { label: (*label).into(), enabled: *enabled }).collect();
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
            let Some(entry) = entry else { return };
            // The Trash names the entry by its path as text (plan-00012, D6), as the write layer refuses.
            if entry.path.file_name().and_then(|name| name.to_str()).is_none() {
                return refuse(m, "This name is not UTF-8, so fmc cannot move it to the Trash.");
            }
            let Some(identity) = entry_identity(&entry.path) else {
                return refuse(m, &store::Error::Missing.to_string());
            };
            let message = format!("{} goes to the Trash. Finder's Put Back restores it.", display_label(&entry.name));
            let dialog = Dialog::new("Move to Trash?", &message, false, &["Cancel", "Delete"], 0);
            let task = Task::Delete { path: entry.path.clone(), next: successor(m), identity };
            ask(m, ui, task, dialog, "");
        }
        Act::Copy => take(m, false),
        Act::Cut => take(m, true),
        Act::Paste => paste(m),
    }
}

/// `c` and `x` on the selected row, and the menu's Copy and Cut: the row goes on the clipboard, and a notice says so.
pub fn take(m: &mut Model, cut: bool) {
    let Some(entry) = m.browser.selected_entry() else { return };
    let text = format!("{} {}. Paste it with v.", if cut { "Cut" } else { "Copied" }, label(&entry.path));
    m.clipboard = Some(Clip { path: entry.path, cut });
    m.message = Some(Message { text, error: false });
}

/// `v`, and the menu's Paste: the clipboard goes into the target folder, on a worker, one paste at a time (plan-00012,
/// D8). A cut pasted where it lies changes nothing, and ends the cut.
pub fn paste(m: &mut Model) {
    let Some(clip) = m.clipboard.clone() else { return };
    if m.pasting {
        m.message = Some(Message { text: "Still pasting. Try again when it finishes.".into(), error: false });
        return;
    }
    let root = m.browser.tree.root().to_path_buf();
    let into = target_folder(m, m.browser.selected_entry().as_ref());
    if clip.cut && parent_of(&clip.path, &root) == into {
        m.clipboard = None;
        return;
    }
    m.pasting = true;
    let verb = if clip.cut { "moving" } else { "copying" };
    m.message = Some(Message { text: format!("{verb} {}…", label(&clip.path)), error: false });
    let (source, cut) = (clip.path, clip.cut);
    crate::services::spawn(&m.proxy, move || {
        let result = store::paste(&root, &source, &into, cut).map_err(|error| error.to_string());
        Event::Pasted { source, into, moved: cut, result }
    });
}

/// A paste's answer (`Event::Pasted`). The new entry is revealed (D9), or its error goes to the footer. The folders the
/// paste touched are listed again; a folder that is closed stays closed.
pub fn pasted(m: &mut Model, source: &Path, into: &Path, moved: bool, result: Result<PathBuf, String>) {
    m.pasting = false;
    // A cut ends with its paste, whatever the result, unless the user has cut something else meanwhile.
    if moved && m.clipboard.as_ref().is_some_and(|clip| clip.cut && clip.path == source) {
        m.clipboard = None;
    }
    if moved {
        let root = m.browser.tree.root().to_path_buf();
        relist_shown(m, parent_of(source, &root));
    }
    match result {
        Ok(target) => {
            m.message = None;
            tree::relist_to(m, into.to_path_buf(), target);
        }
        Err(error) => {
            refuse(m, &error);
            relist_shown(m, into.to_path_buf());
        }
    }
    tree::refresh_marks(m);
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
        Task::Delete { path, next, identity } => {
            let parent = parent_of(path, &root);
            if entry_identity(path) != Some(*identity) {
                // Another entry now holds the name: nothing is trashed, and the folder is listed again with that entry shown.
                overlay::close(app, m);
                refuse(m, "That entry changed since it was listed, so nothing went to the Trash.");
                tree::relist_to(m, parent, path.clone());
                return;
            }
            trash(&root, path).map(|()| (parent.clone(), next.clone().unwrap_or(parent)))
        }
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
