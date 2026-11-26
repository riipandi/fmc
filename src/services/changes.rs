//! Git's change list and the stacked patches for the changes screen (Scouter `src/diff/diff-screen.ts` `load` and
//! `loadAll`). The listing opens the repository; the patches read the copy of its index that listing made, so they run
//! on the same repository, after it.

use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use fmc::diff::{Change, ChangeSet, FilePatch, Patch};

use super::git::Repository;

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

/// One file's patch with `context` lines (Scouter `readPatch`). A failed read is a notice, which the body shows in
/// place of the diff, as Scouter's `read` does.
pub fn patch(repository: &Shared, change: &Change, context: usize) -> FilePatch {
    let patch = repository.0.patch(change, context).unwrap_or_else(Patch::Notice);
    FilePatch { change: change.clone(), patch }
}
