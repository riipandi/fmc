//! Opens the selected row, the shown file, or the browsed folder in another app (plan-00017): `o` hands the target to
//! macOS's default app, `e` opens the folder in the editor the settings name (Finder when none is set), and `E` reveals
//! the target in Finder. The launch runs on a worker (`services/launch.rs`); this file picks the target and words it.

use std::ffi::OsString;

use super::*;
use crate::services::launch::{self, How};
use fmc::text::display_label;

/// `o`, and the menu's Open row: the target goes to its default app. A folder opens in Finder.
pub fn default_app(m: &mut Model) {
    let Some(target) = target(m) else { return };
    let done = format!("Opened {}.", write::label(&target));
    spawn_open(m, &target, How::Default, done);
}

/// `E`, and the menu's Show in Finder row: the target is selected in its folder.
pub fn reveal(m: &mut Model) {
    let Some(target) = target(m) else { return };
    let done = format!("Showed {} in Finder.", write::label(&target));
    spawn_open(m, &target, How::Reveal, done);
}

/// `e`, and the menu's Open Folder in Editor row: the browsed folder goes to the editor the settings name, or to Finder
/// when none is set.
pub fn editor(m: &mut Model) {
    let root = m.browser.tree.root().to_path_buf();
    let label = write::label(&root);
    let editor = m.settings.values.editor.clone();
    if editor.is_empty() {
        spawn_open(m, &root, How::Default, format!("Opened {label}."));
    } else {
        let app = display_label(app_name(&editor));
        spawn_open(m, &root, How::App(&editor), format!("Opened {label} in {app}."));
    }
}

/// The answer of a launch (`Event::Launched`): a success shows `done`, and a failure is `open`'s first error line.
pub fn launched(m: &mut Model, done: &str, result: &Result<(), String>) {
    m.message = Some(match result {
        Ok(()) => Message { text: done.to_owned(), error: false },
        Err(text) => Message { text: text.clone(), error: true },
    });
}

/// The target of `o` and `E`: the selected row in the tree, or the shown file in the reader (D5). The key page is a
/// reader with no file, so it has no target.
fn target(m: &Model) -> Option<PathBuf> {
    match m.focus {
        Focus::List => m.browser.selected_entry().map(|entry| entry.path),
        Focus::Reader => m.shown.path().map(Path::to_path_buf),
    }
}

/// The app's name as the footer shows it: `Zed` for `Zed` and for `/Applications/Zed.app`.
fn app_name(app: &str) -> &str {
    let file = app.rsplit('/').next().unwrap_or(app);
    file.strip_suffix(".app").unwrap_or(file)
}

/// Hands `path` to `open` as `how` says, on a worker. `done` is what the footer says if the launch works.
fn spawn_open(m: &Model, path: &Path, how: How, done: String) {
    let args = launch::open_args(path, how);
    crate::services::spawn(&m.proxy, move || Event::Launched { done, result: launch(&args) });
}

/// Hands the arguments to `open`. The tests stand in for it, so a test never launches an app (`write.rs`, `trash`).
#[cfg(not(test))]
fn launch(args: &[OsString]) -> Result<(), String> {
    launch::run_open(args)
}

/// The test stand-in: the target must exist, as `open` requires, and no app opens.
#[cfg(test)]
fn launch(args: &[OsString]) -> Result<(), String> {
    let target = args.last().map(std::path::PathBuf::from).unwrap_or_default();
    if target.exists() {
        Ok(())
    } else {
        Err(format!("The file {} couldn't be opened because there is no such file.", target.display()))
    }
}

#[cfg(test)]
#[path = "open_tests.rs"]
mod tests;
