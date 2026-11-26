//! The text of the changes screen's rows, headers, and empty state (Scouter `src/diff/changes.ts` `paintRow`,
//! `indexMark`, `countSegments`; `src/diff/diff-blocks.ts` `headerTitleContent`, `statBarSegments`). Pure: the views
//! turn the runs into styled text.

use super::Change;
use crate::text::{Seg, display_label, grouped};
use crate::theme::Palette;

/// Cells in a file's stat bar (Scouter `STAT_BAR_CELLS`, `src/diff/diff-blocks.ts:21`).
const STAT_BAR_CELLS: u64 = 5;

/// A path's folder and name, split as `posix.dirname` and `posix.basename` split it: the folder is `.` at the top.
pub fn split_path(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or((".", path))
}

/// The mark for a file's index state: `●` staged, `○` unstaged, `◐` both (Scouter `indexMark`).
pub fn index_mark(change: &Change, p: &Palette) -> Option<Seg> {
    match (change.staged == Some(true), change.unstaged == Some(true)) {
        (true, true) => Some(Seg::new("◐", p.modified)),
        (true, false) => Some(Seg::new("●", p.added)),
        (false, true) => Some(Seg::new("○", p.dim)),
        (false, false) => None,
    }
}

/// `bin` for a binary file, `new` when git gave no counts (an untracked file), else `+N −N` with the zero side left out
/// (Scouter `countSegments`).
pub fn count_segments(change: &Change, p: &Palette) -> Vec<Seg> {
    if change.binary {
        return vec![Seg::new("bin", p.dim)];
    }
    let (Some(added), Some(removed)) = (change.added, change.removed) else {
        return vec![Seg::new("new", p.dim)];
    };
    let mut segments = Vec::new();
    if added > 0 {
        segments.push(Seg::new(format!("+{added}"), p.added));
    }
    if removed > 0 {
        let lead = if segments.is_empty() { "" } else { " " };
        segments.push(Seg::new(format!("{lead}−{removed}"), p.removed));
    }
    segments
}

/// The five-cell bar of a file's size: green for added lines, red for removed ones, and at least one cell of each when
/// both exist. Empty for a binary file or one with no counted lines (Scouter `statBarSegments`).
pub fn stat_bar(change: &Change, p: &Palette) -> Vec<Seg> {
    if change.binary {
        return Vec::new();
    }
    let (added, removed) = (change.added.unwrap_or(0), change.removed.unwrap_or(0));
    let total = added + removed;
    if total == 0 {
        return Vec::new();
    }
    let filled = total.min(STAT_BAR_CELLS);
    let mut green = (added as f64 / total as f64 * filled as f64).round() as u64;
    if added > 0 && green == 0 {
        green = 1;
    }
    if removed > 0 && green == filled && filled > 1 {
        green = filled - 1;
    }
    let red = filled - green;
    let mut segments = vec![Seg::plain("  ")];
    if green > 0 {
        segments.push(Seg::new("▪".repeat(green as usize), p.added));
    }
    if red > 0 {
        segments.push(Seg::new("▪".repeat(red as usize), p.removed));
    }
    segments
}

/// The right side of a file's header: its counts, then its stat bar (Scouter `headerCounts`).
pub fn header_counts(change: &Change, p: &Palette) -> Vec<Seg> {
    let mut segments = count_segments(change, p);
    segments.extend(stat_bar(change, p));
    segments
}

/// The title of a file's block: the active mark, the fold chevron, the status letter, the index mark, the name, the old
/// name of a rename, and the folder (Scouter `headerTitleContent`, `src/diff/diff-blocks.ts:425-449`).
pub fn header_title(change: &Change, active: bool, folded: bool, p: &Palette) -> Vec<Seg> {
    let (folder, name) = split_path(&change.path);
    let mut segments = vec![
        Seg::new(if active { "▎ " } else { "  " }, p.accent),
        Seg::new(if folded { "▸ " } else { "▾ " }, p.dim),
        Seg::new(format!("{} ", change.kind.letter()), change.kind.color(p)).bold(),
    ];
    if let Some(mark) = index_mark(change, p) {
        segments.push(mark);
        segments.push(Seg::plain(" "));
    }
    let name_color = if active { p.accent_soft } else { p.text };
    segments.push(Seg::new(display_label(name), name_color).bold());
    if let Some(old) = &change.old_path {
        segments.push(Seg::new(format!("  ← {}", display_label(old)), p.dim));
    }
    if folder != "." {
        segments.push(Seg::new(format!("  {}/", display_label(folder)), p.dim));
    }
    segments
}

/// One row of the change list, in three parts: the status column (selection bar, letter, index mark), the name with
/// its old name and folder, and the counts at the right (Scouter `paintRow`, `src/diff/changes.ts:226-253`).
#[derive(Debug, PartialEq, Eq)]
pub struct ListRow {
    pub status: Vec<Seg>,
    pub name: Vec<Seg>,
    pub counts: Vec<Seg>,
}

pub fn list_row(change: &Change, selected: bool, p: &Palette) -> ListRow {
    let (folder, name) = split_path(&change.path);
    let mut status = vec![
        Seg::new(if selected { "▎" } else { " " }, p.accent),
        Seg::new(change.kind.letter(), change.kind.color(p)).bold(),
        Seg::plain(" "),
    ];
    status.extend(index_mark(change, p));
    let name_color = if selected { p.accent } else { p.text };
    let name_seg = Seg::new(display_label(name), name_color);
    let mut name_segments = vec![if selected { name_seg.bold() } else { name_seg }];
    if let Some(old) = &change.old_path {
        let old_name = display_label(split_path(old).1);
        name_segments.push(Seg::new(format!("  ← {old_name}"), p.dim));
    }
    if folder != "." {
        name_segments.push(Seg::new(format!("  {}/", display_label(folder)), p.dim));
    }
    ListRow { status, name: name_segments, counts: count_segments(change, p) }
}

/// The empty state's title and detail, for a repository whose base is `HEAD` or the empty tree (Scouter `showEmpty`,
/// `src/diff/diff-screen.ts:395-402`).
pub fn empty_message(on_head: bool) -> (&'static str, String) {
    let base = if on_head { "HEAD" } else { "an empty repository" };
    ("No changes", format!("The working tree matches {base}."))
}

/// The note above a cut list (Scouter `src/diff/changes.ts:104`).
pub fn truncated_note(listed: usize) -> String {
    format!("Only the first {} changes are listed.", grouped(listed))
}

#[cfg(test)]
#[path = "row_tests.rs"]
mod tests;
