//! The change list's rows and scrollbar (Scouter `src/diff/changes.ts` `ChangeList`). The rows are `fmc::diff::list_row`
//! and the note is `truncated_note`; the thumb is the tree's, since Scouter draws both with one routine.

use fmc::diff::{ChangeList, list_row, truncated_note};
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};
use super::tree::thumb;

/// The status column's width: the selection bar, the letter, a space, and the index mark (Scouter `width: 5`).
const STATUS_WIDTH: u32 = 5;

/// One change row's widgets: the status, the name, and the counts at the right.
struct RowView {
    root: View,
    status: Text,
    name: Text,
    counts: Text,
}

impl RowView {
    fn new(app: &App, rows: &View) -> Self {
        let root = View::new(app).height(1).width(Length::percent(100.0)).row().shrink(0.0);
        let status =
            Text::new(app, "").height(1).width(STATUS_WIDTH).shrink(0.0).wrap(WrapMode::None).selectable(false);
        let name =
            Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
        let counts =
            Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Left, 1).wrap(WrapMode::None).selectable(false);
        root.add(&status).add(&name).add(&counts);
        rows.add(&root);
        Self { root, status, name, counts }
    }
}

/// The changes list: the note of a cut list, then the rows from the list's scroll position, and the scrollbar column.
pub struct ChangesView {
    /// The list area; the app sizes it, and its top row is the tree's `LIST_TOP`.
    pub root: View,
    rows: View,
    bar: View,
    note: Text,
    views: Vec<RowView>,
    thumbs: Vec<Text>,
}

impl ChangesView {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).width(Length::percent(100.0)).grow(1.0).min_height(0).column();
        let body = View::new(app).width(Length::percent(100.0)).grow(1.0).min_height(0).row();
        let rows = View::new(app).grow(1.0).min_width(0).column();
        let bar = View::new(app).width(1).shrink(0.0).column();
        let note = Text::new(app, "").height(1).shrink(0.0).wrap(WrapMode::None).selectable(false).visible(false);
        rows.add(&note);
        body.add(&rows).add(&bar);
        root.add(&body);
        Self { root, rows, bar, note, views: Vec::new(), thumbs: Vec::new() }
    }

    /// Paints the lines the list's scroll position shows. The note is the first line of a cut list.
    pub fn paint(&mut self, app: &App, list: &ChangeList, p: &Palette) {
        let lines = list.visible_lines();
        let shows_note = list.truncated() && lines.start == 0 && !lines.is_empty();
        let rows = lines.len() - usize::from(shows_note);
        let first = if list.truncated() { lines.start.max(1) - 1 } else { lines.start };
        self.bar.visible(list.lines() > list.height());
        self.note.visible(shows_note);
        if shows_note {
            self.note.set_text(content(&[Seg::new(truncated_note(list.changes().len()), p.modified)], p.text));
        }
        while self.views.len() < rows {
            self.views.push(RowView::new(app, &self.rows));
        }
        for (index, view) in self.views.iter().enumerate() {
            view.root.visible(index < rows);
            if index >= rows {
                continue;
            }
            let position = first + index;
            let selected = position == list.selected_index();
            let row = list_row(&list.changes()[position], selected, p);
            view.root.bg(if selected { color(p.row_active_bg) } else { Color::TRANSPARENT });
            view.status.set_text(content(&row.status, p.text));
            view.name.set_text(content(&row.name, p.text));
            view.counts.set_text(content(&row.counts, p.text));
        }
        // One thumb cell per visible line, the note's line included, so the column lines up with the rows.
        while self.thumbs.len() < lines.len() {
            let thumb = Text::new(app, "").height(1).width(1).wrap(WrapMode::None).selectable(false);
            self.bar.add(&thumb);
            self.thumbs.push(thumb);
        }
        let glyphs = thumb(list.lines(), list.height(), list.top(), lines.len());
        for (index, cell) in self.thumbs.iter().enumerate() {
            cell.visible(index < lines.len());
            let glyph = glyphs.get(index).copied().flatten();
            cell.set_text(match glyph {
                Some(glyph) => content(&[Seg::new(glyph, p.muted)], p.text),
                None => content(&[], p.text),
            });
        }
    }
}
