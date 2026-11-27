//! The Settings page's rows (plan-00015, A6): each row is fmc's segments, drawn as the tree draws a row, with the key's
//! word, the value in a column, and the selected row's help line under the list. Not OpenTUI's `Select`: a value column
//! is what `Select`'s name and description do not give.

use fmc::settings::{Row, Settings};
use fmc::text::display_label;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};

/// The label column's width: the longest key word (`sidebar_jump`, `auto_preview`) and a space.
const LABEL_WIDTH: usize = 13;

/// One row's widgets: the label and the value.
struct RowView {
    root: View,
    label: Text,
    value: Text,
}

impl RowView {
    fn new(app: &App, list: &View) -> Self {
        let root = View::new(app).height(1).width(Length::percent(100.0)).row().shrink(0.0);
        let label =
            Text::new(app, "").height(1).width(LABEL_WIDTH as u32).shrink(0.0).wrap(WrapMode::None).selectable(false);
        let value =
            Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
        root.add(&label).add(&value);
        list.add(&root);
        Self { root, label, value }
    }
}

/// The page's body: one view per row, then the help line of the selected row.
pub struct SettingsView {
    pub root: View,
    rows: Vec<RowView>,
    help: Text,
}

impl SettingsView {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).width(Length::percent(100.0)).grow(1.0).min_height(0).column();
        let list = View::new(app).width(Length::percent(100.0)).column();
        let rows = Row::ALL.iter().map(|_| RowView::new(app, &list)).collect();
        let help =
            Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Top, 1).wrap(WrapMode::None).selectable(false);
        root.add(&list).add(&help);
        Self { root, rows, help }
    }

    /// Paints every row's word and value, and the help of the row at `selected`.
    pub fn paint(&self, values: &Settings, selected: usize, p: &Palette) {
        for (index, (row, view)) in Row::ALL.iter().zip(&self.rows).enumerate() {
            let on = index == selected;
            view.root.bg(if on { color(p.row_active_bg) } else { Color::TRANSPARENT });
            let label = format!("{:<LABEL_WIDTH$}", row.key());
            view.label.set_text(content(&[Seg::new(label, p.muted)], p.text));
            // A theme id is the file's own text, so it is shown as the footer shows a name: no control characters.
            let value = display_label(&values.shown(*row));
            view.value.set_text(content(&[Seg::new(value, p.accent_soft)], p.text));
        }
        let help = Row::ALL.get(selected).map_or("", |row| row.help());
        self.help.set_text(content(&[Seg::new(help, p.muted)], p.text));
    }
}
