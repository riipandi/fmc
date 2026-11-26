//! The change list's selection and scroll (Scouter `src/diff/changes.ts` `ChangeList`, `src/core/keys.ts`
//! `handleListKey`). The scroll position counts lines, and the truncation note is the first line of a cut list. Pure:
//! the view paints the lines the position shows.

use std::ops::Range;

use super::Change;

#[derive(Debug, Default)]
pub struct ChangeList {
    changes: Vec<Change>,
    truncated: bool,
    /// The selected row, an index into `changes`. Zero while the list is empty: Scouter keeps -1 there, and the next
    /// list starts at its first row either way.
    selected: usize,
    /// The first visible line; the note is line zero of a cut list.
    top: usize,
    height: usize,
}

impl ChangeList {
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected(&self) -> Option<&Change> {
        self.changes.get(self.selected)
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// The lines the list has: its rows, and the note when it is cut.
    pub fn lines(&self) -> usize {
        self.changes.len() + self.note()
    }

    /// The lines the viewport shows.
    pub fn visible_lines(&self) -> Range<usize> {
        self.top..(self.top + self.height).min(self.lines())
    }

    /// Sets the viewport's height on each frame. The scroll position only clamps: a wheel scroll away from the selection
    /// must survive the repaint (Scouter's scroll box does not follow its selection on a frame).
    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        self.top = self.clamped_top(self.top as isize);
    }

    /// Shows a new listing (Scouter `show`). `keep` is the path to select when it is listed; otherwise the row at the
    /// last selection's index, or the nearest row to it, stays selected.
    pub fn show(&mut self, changes: Vec<Change>, truncated: bool, keep: Option<&str>) {
        let kept = keep.and_then(|path| changes.iter().position(|change| change.path == path));
        self.selected = kept.unwrap_or_else(|| self.selected.min(changes.len().saturating_sub(1)));
        self.changes = changes;
        self.truncated = truncated;
        self.keep_visible();
    }

    /// Selects the row of `path` and keeps it in view, as Scouter's `highlight` does; a row already selected is left
    /// alone. Returns whether the selection moved.
    pub fn select_path(&mut self, path: &str) -> bool {
        match self.changes.iter().position(|change| change.path == path) {
            Some(index) if index != self.selected => self.select(index),
            _ => false,
        }
    }

    /// Moves the selection by `delta` rows, clamped to the list (Scouter `selectIndex`).
    pub fn move_by(&mut self, delta: isize) -> bool {
        if self.changes.is_empty() {
            return false;
        }
        let index = (self.selected as isize + delta).clamp(0, self.changes.len() as isize - 1);
        self.select(index as usize)
    }

    /// Selects the first row, or the last.
    pub fn move_to_edge(&mut self, last: bool) -> bool {
        if self.changes.is_empty() {
            return false;
        }
        self.select(if last { self.changes.len() - 1 } else { 0 })
    }

    /// Moves a screen down or up: one row less than the viewport, and at least one (Scouter `handleListKey`).
    pub fn page(&mut self, down: bool) -> bool {
        let step = self.height.saturating_sub(1).max(1) as isize;
        self.move_by(if down { step } else { -step })
    }

    /// Selects the row at `index`, as a click does. Returns whether the selection moved.
    pub fn select_index(&mut self, index: usize) -> bool {
        index < self.changes.len() && self.select(index)
    }

    /// Scrolls the viewport by `delta` lines without moving the selection: the mouse wheel.
    pub fn scroll(&mut self, delta: isize) {
        self.top = self.clamped_top(self.top as isize + delta);
    }

    /// The row under the viewport's `line`, if a row is there and not the note.
    pub fn row_at(&self, line: usize) -> Option<usize> {
        if line >= self.height {
            return None;
        }
        let index = (self.top + line).checked_sub(self.note())?;
        (index < self.changes.len()).then_some(index)
    }

    fn note(&self) -> usize {
        usize::from(self.truncated)
    }

    fn select(&mut self, index: usize) -> bool {
        let moved = index != self.selected;
        self.selected = index;
        self.keep_visible();
        moved
    }

    /// Scrolls the least distance that keeps the selected row in view (OpenTUI's `scrollChildIntoView`).
    fn keep_visible(&mut self) {
        let line = self.selected + self.note();
        if line < self.top {
            self.top = line;
        } else if self.height > 0 && line >= self.top + self.height {
            self.top = line + 1 - self.height;
        }
        self.top = self.clamped_top(self.top as isize);
    }

    fn clamped_top(&self, top: isize) -> usize {
        let max = self.lines().saturating_sub(self.height);
        top.clamp(0, max as isize) as usize
    }
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
