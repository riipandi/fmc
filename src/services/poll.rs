//! The browse poll (Scouter `refreshBrowse`, `src/app/app.ts:579-611`): one job on a worker re-lists the expanded
//! folders, reads the git marks, and checks the open text document. The app applies its `Event::BrowsePolled`.

use std::path::PathBuf;

use opentui::prelude::AppProxy;

use super::{Event, fs, git, spawn};

/// A browse poll of `root`. `folders` are the expanded folders with their loop parents; `document` is the open text
/// document with its stamp. `chain` marks a timed poll, whose answer schedules the next one.
pub fn browse(
    proxy: &AppProxy,
    chain: bool,
    root: PathBuf,
    folders: Vec<(PathBuf, Vec<PathBuf>)>,
    document: Option<(PathBuf, String)>,
) {
    spawn(proxy, move || {
        // A folder that cannot be listed keeps its rows, as Scouter's `refresh` returns false for it.
        let folders = folders
            .into_iter()
            .filter_map(|(path, parents)| fs::list(&path, &parents).ok().map(|entries| (path, entries)))
            .collect();
        let marks = git::marks(&root);
        let document = document.map(|(path, stamp)| {
            let reread = fs::reread(&path, &stamp);
            (path, reread)
        });
        Event::BrowsePolled { root, chain, folders, marks, document }
    });
}
