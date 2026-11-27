//! The git marks the tree draws (Scouter `src/diff/marks.ts`): a status letter on a changed file, a dot on a folder
//! that holds one, and the ignored entries.

use std::collections::{HashMap, HashSet};

use super::change::{Change, ChangeKind, MAX_CHANGES, order};
use crate::theme::{Hex, Palette};

/// What git reported for the repository that holds the browsed folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    /// The browsed folder relative to the repository root; empty at the root.
    pub within: String,
    /// Tracked and untracked changes, repository-relative.
    pub changes: Vec<Change>,
    /// Ignored entries, repository-relative; a folder ends with `/`.
    pub ignored: Vec<String>,
}

/// A status letter and its color, drawn at the right edge of a row.
pub struct Mark {
    pub letter: &'static str,
    pub color: Hex,
}

/// The git marks of the browsed folder, keyed by path relative to it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    changes: HashMap<String, ChangeKind>,
    folders: HashSet<String>,
    ignored: HashSet<String>,
}

impl Marks {
    pub fn new(listing: Listing) -> Self {
        let Listing { within, mut changes, ignored } = listing;
        // Scouter caps before it filters to the browsed folder, so a change outside it still uses up a slot.
        order(&mut changes);
        changes.truncate(MAX_CHANGES);
        let mut marks = Self::default();
        for change in changes {
            let Some(inside) = within_folder(&within, &change.path) else { continue };
            marks.folders.extend(folders_above(&inside));
            marks.changes.insert(inside, change.kind);
        }
        for path in &ignored {
            if let Some(inside) = within_folder(&within, path.trim_end_matches('/')) {
                marks.ignored.insert(inside);
            }
        }
        marks
    }

    /// The change at `path`, or a dot on a folder that holds one (Scouter `styleOf`).
    pub fn mark(&self, path: &str, p: &Palette) -> Option<Mark> {
        if let Some(kind) = self.changes.get(path) {
            return Some(Mark { letter: kind.letter(), color: kind.color(p) });
        }
        self.folders.contains(path).then_some(Mark { letter: "•", color: p.modified })
    }

    /// Whether `path`, or a folder above it, is git-ignored (Scouter `isIgnored`).
    pub fn is_ignored(&self, path: &str) -> bool {
        self.ignored.contains(path) || folders_above(path).iter().any(|folder| self.ignored.contains(folder))
    }
}

/// `path` relative to the browsed folder `within` (relative to the repository root), or `None` outside it.
pub fn within_folder(within: &str, path: &str) -> Option<String> {
    if within.is_empty() {
        return Some(path.to_string());
    }
    path.strip_prefix(within)?.strip_prefix('/').map(str::to_string)
}

/// Every folder above `path`, nearest first: `a/b/c.md` gives `a/b` then `a`.
fn folders_above(path: &str) -> Vec<String> {
    let mut folders = Vec::new();
    let mut rest = path;
    while let Some(at) = rest.rfind('/') {
        rest = &rest[..at];
        folders.push(rest.to_string());
    }
    folders
}
