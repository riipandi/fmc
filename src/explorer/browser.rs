//! The browsed tree with its selection and scroll position (Scouter `src/explorer/explorer.ts`, `handleListKey`).
//! The list is drawn from `top` as rows; scrolling keeps the selected row in view from the nearest edge, as
//! OpenTUI's `scrollChildIntoView` does for Scouter. While the finder shows, the rows are its flat list, and the
//! selection is the row's index: a text search lists one file once per line, so a path does not name a row.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{Entry, Kind, Row, Tree};
use crate::find::text::Snippet;

/// One row of the finder's list: a path, and for a text hit the line it matched and the snippet to show. `matched` holds
/// the character positions a fuzzy name match landed on, which the row draws in the match colour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flat {
    pub entry: Entry,
    pub hit: Option<(usize, Snippet)>,
    pub matched: Vec<usize>,
}

impl Flat {
    pub fn name(entry: Entry) -> Self {
        Self { entry, hit: None, matched: Vec::new() }
    }
}

pub struct Browser {
    pub tree: Tree,
    selected: PathBuf,
    top: usize,
    height: usize,
    /// The finder's rows, in place of the tree; `None` shows the tree.
    flat: Option<Vec<Flat>>,
    /// The selected index among the flat rows.
    flat_at: usize,
    /// The entries the user marked with `s`. A listing that drops an entry drops its mark, and a new root
    /// starts with none, since `-` builds a new browser.
    marked: BTreeSet<PathBuf>,
}

impl Browser {
    /// A browser with the root selected, which has no row: the first row is selected once the root is listed.
    pub fn new(tree: Tree) -> Self {
        let selected = tree.root().to_path_buf();
        Self { tree, selected, top: 0, height: 0, flat: None, flat_at: 0, marked: BTreeSet::new() }
    }

    /// The marked paths, in path order: a folder comes before the paths under it.
    pub fn marked(&self) -> &BTreeSet<PathBuf> {
        &self.marked
    }

    pub fn is_marked(&self, path: &Path) -> bool {
        self.marked.contains(path)
    }

    /// Marks `path` if it is not marked, and unmarks it if it is.
    pub fn toggle_mark(&mut self, path: &Path) {
        if !self.marked.remove(path) {
            self.marked.insert(path.to_path_buf());
        }
    }

    pub fn clear_marks(&mut self) {
        self.marked.clear();
    }

    /// The paths a copy, move, or Trash acts on: the marked set in path order, without a marked path inside a marked
    /// folder, since the folder's action already takes it.
    pub fn targets(&self) -> Vec<PathBuf> {
        let mut kept: Vec<PathBuf> = Vec::new();
        for path in &self.marked {
            if !kept.iter().any(|folder| path.starts_with(folder)) {
                kept.push(path.clone());
            }
        }
        kept
    }

    /// Drops the marks of entries the tree no longer has: a file that was renamed, moved, or trashed, or that a listing
    /// dropped. Marks on a collapsed folder's children stay, since the tree keeps them.
    fn prune_marks(&mut self) {
        let tree = &self.tree;
        self.marked.retain(|path| tree.entry(path).is_some());
    }

    pub fn selected(&self) -> &Path {
        &self.selected
    }

    /// The selected row's entry, if there is a row.
    pub fn selected_entry(&self) -> Option<Entry> {
        self.rows().get(self.selected_index()).map(|row| row.entry.clone())
    }

    /// The selected flat row, while the finder shows.
    pub fn selected_flat(&self) -> Option<Flat> {
        let rows = self.flat.as_ref()?;
        rows.get(self.selected_index()).cloned()
    }

    /// How many rows the list shows: the flat rows while the finder is shown, else the tree's visible rows.
    pub fn row_count(&self) -> usize {
        self.rows().len()
    }

    pub fn is_flat(&self) -> bool {
        self.flat.is_some()
    }

    /// Shows `entries` as flat rows in place of the tree, selecting the first row whose path is `selected`, else the
    /// first row (Scouter `show`).
    pub fn show(&mut self, entries: Vec<Flat>, selected: &Path) {
        self.flat = Some(entries);
        self.flat_at = 0;
        self.select(selected);
    }

    /// Shows the tree again, selecting `selected` if it has a row, else the first row (Scouter `renderTree`).
    pub fn show_tree(&mut self, selected: &Path) {
        self.flat = None;
        self.select(selected);
    }

    fn rows(&self) -> Vec<Row<'_>> {
        match &self.flat {
            Some(entries) => entries
                .iter()
                .map(|flat| Row {
                    entry: &flat.entry,
                    depth: 0,
                    folder: flat.entry.kind == Kind::Directory,
                    expanded: false,
                    hit: flat.hit.as_ref().map(|(line, snippet)| (*line, snippet)),
                    matched: &flat.matched,
                })
                .collect(),
            None => self.tree.rows(),
        }
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Index of the selected row; the first row when there is none (Scouter `show`).
    pub fn selected_index(&self) -> usize {
        if self.flat.is_some() {
            return self.flat_at.min(self.row_count().saturating_sub(1));
        }
        self.rows().iter().position(|row| row.entry.path == self.selected).unwrap_or(0)
    }

    /// Sets the number of rows the list shows, then keeps the selection in view.
    /// A repaint at the same height keeps the wheel's scroll: only a new height brings the selection into view again.
    pub fn set_height(&mut self, height: usize) {
        if height == self.height {
            self.top = self.clamped_top(self.top as isize);
            return;
        }
        self.height = height;
        self.keep_visible();
    }

    /// Lists a folder into the tree; a selection without a row (the root, before its listing) moves to the first row.
    /// While the finder's rows show, a listing that lands leaves the selection alone.
    pub fn listed(&mut self, path: &Path, entries: Vec<Entry>) -> bool {
        let listed = self.tree.listed(path, entries);
        self.prune_marks();
        if !self.is_flat() {
            let selected = self.selected.clone();
            self.select(&selected);
        }
        listed
    }

    /// A poll's fresh listing of an expanded folder (`Tree::refreshed`). Returns whether the rows changed.
    pub fn refreshed(&mut self, path: &Path, entries: Vec<Entry>) -> bool {
        let changed = self.tree.refreshed(path, entries);
        self.prune_marks();
        if changed && !self.is_flat() {
            let selected = self.selected.clone();
            self.select(&selected);
        }
        changed
    }

    pub fn dotfiles(&self) -> bool {
        self.tree.dotfiles()
    }

    /// Shows or hides dotfiles. A selection that has no row after the change falls back to the first row, as it does
    /// for any removed row. The finder's rows are not the tree's, so the caller walks the folder again for them.
    pub fn set_dotfiles(&mut self, shown: bool) {
        self.tree.set_dotfiles(shown);
        if !self.is_flat() {
            let selected = self.selected.clone();
            self.select(&selected);
        }
    }

    /// Selects `path` if it has a row, else the first row (Scouter `show`, which renders after each change).
    pub fn select(&mut self, path: &Path) {
        let index = self.rows().iter().position(|row| row.entry.path == path).unwrap_or(0);
        self.set_index(index);
    }

    /// Selects `path` if it has a row, and reports whether it did (Scouter `selectPath`).
    pub fn select_row(&mut self, path: &Path) -> bool {
        let index = self.rows().iter().position(|row| row.entry.path == path);
        match index {
            Some(index) => {
                self.set_index(index);
                true
            }
            None => false,
        }
    }

    /// Makes the row at `index` the selection, clamped to the list. With no rows the selection is the tree's root.
    fn set_index(&mut self, index: usize) {
        let rows = self.rows();
        let selected = match rows.get(index.min(rows.len().saturating_sub(1))) {
            Some(row) => row.entry.path.clone(),
            None => self.tree.root().to_path_buf(),
        };
        drop(rows);
        self.selected = selected;
        self.flat_at = index;
        self.keep_visible();
    }

    /// Moves the selection by `delta` rows, clamped to the list.
    pub fn move_by(&mut self, delta: isize) {
        let rows = self.row_count();
        if rows == 0 {
            return;
        }
        let index = (self.selected_index() as isize + delta).clamp(0, rows as isize - 1);
        self.set_index(index as usize);
    }

    /// Moves the selection to the first row, or the last row.
    pub fn move_to_edge(&mut self, last: bool) {
        let rows = self.row_count();
        if rows > 0 {
            self.set_index(if last { rows - 1 } else { 0 });
        }
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
        (line < self.height && index < self.row_count()).then_some(index)
    }

    /// Selects the row at `index` (a click on it).
    pub fn select_index(&mut self, index: usize) {
        if index < self.row_count() {
            self.set_index(index);
        }
    }

    /// Re-clamps the scroll position and keeps the selection visible after the tree changed.
    pub fn reflow(&mut self) {
        self.top = self.clamped_top(self.top as isize);
        self.keep_visible();
    }

    /// The rows that fit the viewport, from `top`.
    pub fn visible_rows(&self) -> Vec<Row<'_>> {
        let rows = self.rows();
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
        let max = self.row_count().saturating_sub(self.height);
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
