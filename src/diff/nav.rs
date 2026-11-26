//! The stacked diff's hunks, folds, and the split and context limits (Scouter `src/diff/diff-blocks.ts` `jumpHunk`,
//! `activeLineFor`, `toggleCollapse`, `MIN_SPLIT_COLUMNS`, and `src/diff/diff-screen.ts` `NORMAL_CONTEXT_LINES`). Pure:
//! the view measures the hunks' rows and passes them in.

use std::collections::HashSet;

/// Context lines around each change, normal and wide (Scouter `src/diff/diff-screen.ts:22-23`).
pub const NORMAL_CONTEXT: usize = 3;
pub const WIDE_CONTEXT: usize = 12;

/// The narrowest body that shows a split view (Scouter `MIN_SPLIT_COLUMNS`, `src/diff/diff-blocks.ts:20`).
pub const SPLIT_MIN_COLUMNS: u32 = 60;

/// The context lines a patch is read with.
pub fn context(wide: bool) -> usize {
    if wide { WIDE_CONTEXT } else { NORMAL_CONTEXT }
}

/// Whether a body of `columns` columns can show a split view. Going back to unified is always allowed.
pub fn can_split(columns: u32) -> bool {
    columns >= SPLIT_MIN_COLUMNS
}

/// One hunk in the stack: the row it starts on, the file it belongs to (its index in the stack), and the new-file line
/// it starts at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hunk {
    pub row: usize,
    pub file: usize,
    pub line: usize,
}

/// Where a hunk jump lands. `First` and `Last` are the edges the footer reports; `Nothing` is a stack without hunks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jump {
    To(usize),
    First,
    Last,
    Nothing,
}

/// The next hunk below `top` when `forward`, else the previous one above it (Scouter `jumpHunk`). `hunks` are in stack
/// order.
pub fn step(hunks: &[Hunk], top: usize, forward: bool) -> Jump {
    if hunks.is_empty() {
        return Jump::Nothing;
    }
    let target =
        if forward { hunks.iter().find(|hunk| hunk.row > top) } else { hunks.iter().rfind(|hunk| hunk.row < top) };
    match (target, forward) {
        (Some(hunk), _) => Jump::To(hunk.row),
        (None, true) => Jump::Last,
        (None, false) => Jump::First,
    }
}

/// The new-file line `file` is on at `top` (Scouter `activeLineFor`): its last hunk above the top, else its first, else
/// `fallback` when it has no hunk in the stack (a folded file).
pub fn line_at(hunks: &[Hunk], file: usize, top: usize, fallback: usize) -> usize {
    let own = || hunks.iter().filter(|hunk| hunk.file == file);
    own().rfind(|hunk| hunk.row <= top).or_else(|| own().next()).map_or(fallback, |hunk| hunk.line)
}

/// The files whose bodies are folded, by path. Scouter keeps the set per visit: leaving the screen clears it.
#[derive(Debug, Default)]
pub struct Folds(HashSet<String>);

impl Folds {
    pub fn toggle(&mut self, path: &str) {
        if !self.0.remove(path) {
            self.0.insert(path.to_string());
        }
    }

    pub fn contains(&self, path: &str) -> bool {
        self.0.contains(path)
    }
}

#[cfg(test)]
#[path = "nav_tests.rs"]
mod tests;
