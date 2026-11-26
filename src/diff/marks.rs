//! Change kinds and the git marks the tree draws (Scouter `src/diff/git.ts` `kinds` and `parseNameStatus`,
//! `src/diff/changes.ts` `kindStyles`, `src/diff/marks.ts`).

use std::collections::{HashMap, HashSet};

use crate::theme::Palette;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Typechange,
    Conflict,
    Untracked,
}

impl ChangeKind {
    /// The kind of a `--name-status` letter. Any other letter, copies included, counts as modified.
    pub fn from_status(status: &str) -> Self {
        match status.chars().next() {
            Some('A') => Self::Added,
            Some('D') => Self::Deleted,
            Some('R') => Self::Renamed,
            Some('T') => Self::Typechange,
            Some('U') => Self::Conflict,
            _ => Self::Modified,
        }
    }

    pub fn letter(self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Typechange => "T",
            Self::Conflict => "U",
            Self::Untracked => "?",
        }
    }

    pub fn color(self, p: &Palette) -> &'static str {
        match self {
            Self::Modified => p.modified,
            Self::Added => p.added,
            Self::Deleted | Self::Conflict => p.removed,
            Self::Renamed => p.accent_soft,
            Self::Typechange | Self::Untracked => p.muted,
        }
    }
}

/// Splits git's `-z` output on NUL and drops the empty token after the last NUL (Scouter `splitZ`).
pub fn split_z(text: &str) -> Vec<&str> {
    let mut tokens: Vec<&str> = text.split('\0').collect();
    if tokens.last() == Some(&"") {
        tokens.pop();
    }
    tokens
}

/// `git diff --name-status -z` output: a status, then the path. A rename or copy has an old path before the new
/// one, and the new one is the change's path.
pub fn parse_name_status(text: &str) -> Vec<(String, ChangeKind)> {
    let tokens = split_z(text);
    let mut changes = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let status = tokens[index];
        let renamed = status.starts_with('R') || status.starts_with('C');
        let first = tokens.get(index + 1).copied();
        let second = if renamed { tokens.get(index + 2).copied() } else { None };
        index += if renamed { 3 } else { 2 };
        let Some(path) = second.or(first).filter(|path| !path.is_empty()) else { continue };
        changes.push((path.to_string(), ChangeKind::from_status(status)));
    }
    changes
}

/// Scouter lists at most this many changes, sorted by path, and marks only those (`src/diff/git.ts:12`).
pub const MAX_CHANGES: usize = 1000;

/// What git reported for the repository that holds the browsed folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    /// The browsed folder relative to the repository root; empty at the root.
    pub within: String,
    /// Tracked and untracked changes, repository-relative.
    pub changes: Vec<(String, ChangeKind)>,
    /// Ignored entries, repository-relative; a folder ends with `/`.
    pub ignored: Vec<String>,
}

/// A status letter and its color, drawn at the right edge of a row.
pub struct Mark {
    pub letter: &'static str,
    pub color: &'static str,
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
        changes.sort_by(|left, right| left.0.cmp(&right.0));
        changes.truncate(MAX_CHANGES);
        let mut marks = Self::default();
        for (path, kind) in changes {
            let Some(inside) = within_folder(&within, &path) else { continue };
            marks.folders.extend(folders_above(&inside));
            marks.changes.insert(inside, kind);
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

/// `path` relative to `within`, or `None` when it is outside it.
fn within_folder(within: &str, path: &str) -> Option<String> {
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
