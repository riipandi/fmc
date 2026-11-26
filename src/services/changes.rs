//! Git's change list and the stacked patches for the changes screen (Scouter `src/diff/diff-screen.ts` `load` and
//! `loadAll`). The listing opens the repository; the patches read the copy of its index that listing made, so they run
//! on the same repository, after it.

use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use fmc::diff::{Change, ChangeSet, FilePatch, Patch};
use opentui::prelude::AppProxy;

use super::git::Repository;
use super::{Event, spawn};

/// A repository the listing and its patches share.
#[derive(Clone)]
pub struct Shared(pub Arc<Repository>);

impl fmt::Debug for Shared {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Shared").field(&self.0.root()).finish()
    }
}

/// A listing, the repository it came from, and the browsed folder inside it (`within`, empty at the root).
#[derive(Clone, Debug)]
pub struct Listed {
    pub repository: Shared,
    pub set: ChangeSet,
    pub within: String,
}

/// Opens the repository that holds `dir` and lists its changes (Scouter `openRepository` and `listChanges`). The folder
/// is found the way the marks find it, so a change maps onto the tree whatever form `dir` takes (`/tmp` or `/private/tmp`).
pub fn list(dir: &Path) -> Result<Listed, String> {
    let repository = Repository::open(dir)?;
    let set = repository.changes()?;
    let root = fs::canonicalize(repository.root()).map_err(|error| error.to_string())?;
    let within = fs::canonicalize(dir)
        .ok()
        .and_then(|browsed| browsed.strip_prefix(&root).ok().map(|rest| rest.to_string_lossy().replace('\\', "/")))
        .unwrap_or_default();
    Ok(Listed { repository: Shared(Arc::new(repository)), set, within })
}

/// The patches of `changes` with `context` lines, read from the listing's copy of the index (Scouter `readAllPatches`).
pub fn patches(repository: &Shared, changes: &[Change], context: usize) -> Result<Vec<FilePatch>, String> {
    repository.0.patches(changes, context)
}

/// The changes poll (Scouter `refresh`, `src/diff/diff-screen.ts:352-377`): lists the changes again on the listing's
/// repository, and stamps the files the body shows: all of them, or the one at `selected`.
pub fn poll(proxy: &AppProxy, session: u64, repository: Shared, all: bool, selected: Option<String>) {
    spawn(proxy, move || {
        let result = repository.0.changes().map(|set| {
            let targets: Vec<Change> = if all {
                set.changes.clone()
            } else {
                set.changes.iter().filter(|change| Some(change.path.as_str()) == selected.as_deref()).cloned().collect()
            };
            let stamp = repository.0.stamp(&targets);
            (set, stamp)
        });
        Event::DiffPolled { session, result }
    });
}

/// One file's patch with `context` lines (Scouter `readPatch`). A failed read is a notice, which the body shows in
/// place of the diff, as Scouter's `read` does.
pub fn patch(repository: &Shared, change: &Change, context: usize) -> FilePatch {
    let patch = repository.0.patch(change, context).unwrap_or_else(Patch::Notice);
    FilePatch { change: change.clone(), patch }
}
