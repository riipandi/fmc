//! Changed files and git's listings of them (Scouter `src/diff/git.ts` `splitZ`, `parseNameStatus`, `parseNumstat`,
//! `readChanges`, and `src/diff/types.ts` `Change`). Pure: `services/git.rs` runs git and hands the output here.

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

    /// The kind's name as the header's badge shows it (Scouter `ChangeKind`, `src/diff/types.ts`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Modified => "modified",
            Self::Added => "added",
            Self::Deleted => "deleted",
            Self::Renamed => "renamed",
            Self::Typechange => "typechange",
            Self::Conflict => "conflict",
            Self::Untracked => "untracked",
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

/// Scouter lists at most this many changes, sorted by path, and marks only those (`src/diff/git.ts:12`).
pub const MAX_CHANGES: usize = 1000;

/// One changed path. The counts and `binary` come from `--numstat`; an untracked file has neither counts nor marks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub path: String,
    /// The path before a rename.
    pub old_path: Option<String>,
    pub kind: ChangeKind,
    pub added: Option<u64>,
    pub removed: Option<u64>,
    pub binary: bool,
    /// Whether the change is in the index (`●`); `None` for an untracked file.
    pub staged: Option<bool>,
    /// Whether the change is in the work tree (`○`); `None` for an untracked file.
    pub unstaged: Option<bool>,
}

impl Change {
    pub fn untracked(path: &str) -> Self {
        Self {
            path: path.to_string(),
            old_path: None,
            kind: ChangeKind::Untracked,
            added: None,
            removed: None,
            binary: false,
            staged: None,
            unstaged: None,
        }
    }
}

/// One path's `--numstat` line. A binary file has no counts (`src/diff/git.ts:350-370`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineCounts {
    pub added: Option<u64>,
    pub removed: Option<u64>,
    pub binary: bool,
}

/// The change list. `truncated` says that git's output was cut, or that more than [`MAX_CHANGES`] changes exist.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChangeSet {
    pub changes: Vec<Change>,
    pub truncated: bool,
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
pub fn parse_name_status(text: &str, counts: &HashMap<String, LineCounts>) -> Vec<Change> {
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
        let line = counts.get(path).copied().unwrap_or_default();
        changes.push(Change {
            path: path.to_string(),
            old_path: if renamed { first.map(str::to_string) } else { None },
            kind: ChangeKind::from_status(status),
            added: line.added,
            removed: line.removed,
            binary: line.binary,
            staged: None,
            unstaged: None,
        });
    }
    changes
}

/// `git diff --numstat -z` output by path. A rename's token ends with an empty path, and the path follows the old one.
pub fn parse_numstat(text: &str) -> HashMap<String, LineCounts> {
    let tokens = split_z(text);
    let mut result = HashMap::new();
    let mut index = 0;
    while index < tokens.len() {
        let Some((added, removed, rest)) = numstat_fields(tokens[index]) else {
            index += 1;
            continue;
        };
        let path = if rest.is_empty() { tokens.get(index + 2).copied() } else { Some(rest) };
        index += if rest.is_empty() { 3 } else { 1 };
        let Some(path) = path.filter(|path| !path.is_empty()) else { continue };
        let binary = added == "-";
        let counts = LineCounts {
            added: if binary { None } else { added.parse().ok() },
            removed: if binary { None } else { removed.parse().ok() },
            binary,
        };
        result.insert(path.to_string(), counts);
    }
    result
}

/// `added<TAB>removed<TAB>path`, where each count is digits or `-` (Scouter's `parseNumstat` pattern).
fn numstat_fields(token: &str) -> Option<(&str, &str, &str)> {
    let mut parts = token.splitn(3, '\t');
    let (added, removed, rest) = (parts.next()?, parts.next()?, parts.next()?);
    let count = |text: &str| text == "-" || (!text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()));
    (count(added) && count(removed)).then_some((added, removed, rest))
}

/// Sorts by path in UTF-16 code units, as Scouter's `<` on strings does (`src/diff/git.ts:233`).
pub fn order(changes: &mut [Change]) {
    changes.sort_by(|left, right| left.path.encode_utf16().cmp(right.path.encode_utf16()));
}

/// Scouter's `readChanges` (`src/diff/git.ts:204-236`) from git's output: the tracked changes with their counts and
/// marks, then the untracked files, sorted and capped. `cut` says that a command's output hit its limit; the service
/// then passes that output as empty text.
pub fn change_set(
    statuses: &str,
    numstat: &str,
    untracked: &str,
    staged: &str,
    unstaged: &str,
    cut: bool,
) -> ChangeSet {
    let counts = parse_numstat(numstat);
    let staged: HashSet<&str> = split_z(staged).into_iter().collect();
    let unstaged: HashSet<&str> = split_z(unstaged).into_iter().collect();
    let mut changes: Vec<Change> = parse_name_status(statuses, &counts)
        .into_iter()
        .map(|mut change| {
            change.staged = Some(staged.contains(change.path.as_str()));
            change.unstaged = Some(unstaged.contains(change.path.as_str()));
            change
        })
        .collect();
    changes.extend(split_z(untracked).into_iter().map(Change::untracked));
    order(&mut changes);
    let truncated = cut || changes.len() > MAX_CHANGES;
    changes.truncate(MAX_CHANGES);
    ChangeSet { changes, truncated }
}
