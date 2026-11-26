//! Git's change list and the stacked patches for the changes screen (Scouter `src/diff/diff-screen.ts` `load` and
//! `loadAll`). The listing opens the repository; the patches read the copy of its index that listing made, so they run
//! on the same repository, after it.

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use fmc::diff::{Change, ChangeSet, FilePatch};

use super::git::Repository;

/// Context lines around each change: Scouter's normal context (`src/diff/diff-screen.ts:22`).
pub const CONTEXT: usize = 3;

/// A repository the listing and its patches share.
#[derive(Clone)]
pub struct Shared(pub Arc<Repository>);

impl fmt::Debug for Shared {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Shared").field(&self.0.root()).finish()
    }
}

/// A listing and the repository it came from.
#[derive(Clone, Debug)]
pub struct Listed {
    pub repository: Shared,
    pub set: ChangeSet,
}

/// Opens the repository that holds `dir` and lists its changes (Scouter `openRepository` and `listChanges`).
pub fn list(dir: &Path) -> Result<Listed, String> {
    let repository = Repository::open(dir)?;
    let set = repository.changes()?;
    Ok(Listed { repository: Shared(Arc::new(repository)), set })
}

/// The patches of `changes`, read from the listing's copy of the index (Scouter `readAllPatches`).
pub fn patches(repository: &Shared, changes: &[Change]) -> Result<Vec<FilePatch>, String> {
    repository.0.patches(changes, CONTEXT)
}
