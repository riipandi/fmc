//! The browsed tree with its selection and scroll position (Scouter `src/explorer/explorer.ts`, `handleListKey`).
//! The list is drawn from `top` as rows; scrolling keeps the selected row in view from the nearest edge, as
//! OpenTUI's `scrollChildIntoView` does for Scouter.

use std::path::{Path, PathBuf};

use super::{Entry, Row, Tree};

pub struct Browser {
    pub tree: Tree,
    selected: PathBuf,
    top: usize,
    height: usize,
}

impl Browser {
    /// A browser with the root selected, which has no row: the first row is selected once the root is listed.
    pub fn new(tree: Tree) -> Self {
        let selected = tree.root().to_path_buf();
        Self { tree, selected, top: 0, height: 0 }
    }

    pub fn selected(&self) -> &Path {
        &self.selected
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Index of the selected row; the first row when the selected path has none (Scouter `show`).
    pub fn selected_index(&self) -> usize {
        self.tree.rows().iter().position(|row| row.entry.path == self.selected).unwrap_or(0)
    }

    /// Sets the number of rows the list shows, then keeps the selection in view.
    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        self.keep_visible();
    }

    /// Lists a folder into the tree; a selection without a row (the root, before its listing) moves to the first row.
    pub fn listed(&mut self, path: &Path, entries: Vec<Entry>) -> bool {
        let listed = self.tree.listed(path, entries);
        let selected = self.selected.clone();
        self.select(&selected);
        listed
    }

    /// Selects `path` if it has a row, else the first row (Scouter `show`, which renders after each change).
    pub fn select(&mut self, path: &Path) {
        let rows = self.tree.rows();
        self.selected = rows.iter().find(|row| row.entry.path == path).map_or_else(
            || rows.first().map_or_else(|| self.tree.root().to_path_buf(), |row| row.entry.path.clone()),
            |row| row.entry.path.clone(),
        );
        drop(rows);
        self.keep_visible();
    }

    /// Selects `path` if it has a row, and reports whether it did (Scouter `selectPath`).
    pub fn select_row(&mut self, path: &Path) -> bool {
        if !self.tree.rows().iter().any(|row| row.entry.path == path) {
            return false;
        }
        self.select(path);
        true
    }

    /// Moves the selection by `delta` rows, clamped to the list.
    pub fn move_by(&mut self, delta: isize) {
        let index = self.selected_index() as isize;
        let rows = self.tree.rows();
        let to = (index + delta).clamp(0, rows.len() as isize - 1) as usize;
        self.selected = rows[to].entry.path.clone();
        drop(rows);
        self.keep_visible();
    }

    /// Moves the selection to the first row, or the last row.
    pub fn move_to_edge(&mut self, last: bool) {
        let rows = self.tree.rows();
        let row = if last { rows.last() } else { rows.first() };
        if let Some(row) = row {
            self.selected = row.entry.path.clone();
        }
        drop(rows);
        self.keep_visible();
    }

    /// Moves the selection a screen up or down: one row less than the viewport (Scouter `handleListKey`).
    pub fn page(&mut self, down: bool) {
        let step = self.height.saturating_sub(1).max(1) as isize;
        self.move_by(if down { step } else { -step });
    }

    /// Scrolls the viewport by `delta` rows without moving the selection (the mouse wheel).
    pub fn scroll(&mut self, delta: isize) {
        self.top = self.clamped_top(self.top as isize + delta);
    }

    /// The row index under the list's `line`-th visible line, if a row is there.
    pub fn row_at(&self, line: usize) -> Option<usize> {
        let index = self.top + line;
        (line < self.height && index < self.tree.rows().len()).then_some(index)
    }

    /// Selects the row at `index` (a click on it).
    pub fn select_index(&mut self, index: usize) {
        if let Some(row) = self.tree.rows().get(index) {
            self.selected = row.entry.path.clone();
        }
        self.keep_visible();
    }

    /// Re-clamps the scroll position and keeps the selection visible after the tree changed.
    pub fn reflow(&mut self) {
        self.top = self.clamped_top(self.top as isize);
        self.keep_visible();
    }

    /// The rows that fit the viewport, from `top`.
    pub fn visible_rows(&self) -> Vec<Row<'_>> {
        let rows = self.tree.rows();
        let end = (self.top + self.height).min(rows.len());
        rows.into_iter().skip(self.top).take(end.saturating_sub(self.top)).collect()
    }

    fn keep_visible(&mut self) {
        let index = self.selected_index();
        if index < self.top {
            self.top = index;
        } else if self.height > 0 && index >= self.top + self.height {
            self.top = index + 1 - self.height;
        }
        self.top = self.clamped_top(self.top as isize);
    }

    fn clamped_top(&self, top: isize) -> usize {
        let max = self.tree.rows().len().saturating_sub(self.height);
        top.clamp(0, max as isize) as usize
    }
}

/// `path` relative to `root`, with `/` between parts, as git marks and the ignore list are keyed (Scouter
/// `treeRelative`).
pub fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).map_or_else(
        |_| path.to_string_lossy().into_owned(),
        |rest| rest.iter().map(|part| part.to_string_lossy()).collect::<Vec<_>>().join("/"),
    )
}
