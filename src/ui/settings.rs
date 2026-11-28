//! The Settings page's rows: each row is fmc's segments, drawn as the tree draws a row, with the row's
//! name, the value in a column, and the selected row's help line under the list. The rows sit under their group's
//! heading (`Row::section`). Not OpenTUI's `Select`: a value column is what `Select`'s name and description do not give.

use fmc::settings::{Row, Settings};
use fmc::text::display_label;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};

/// The label column's width: the row's indent, the longest name (`Light theme`, `Auto preview`), and a gap.
const LABEL_WIDTH: usize = 18;

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

fn line(app: &App) -> Text {
    Text::new(app, "").height(1).shrink(0.0).wrap(WrapMode::None).selectable(false)
}

/// The page's body: a heading per group, one view per row, then the selected row's key and help.
pub struct SettingsView {
    pub root: View,
    headings: Vec<(Text, &'static str)>,
    rows: Vec<RowView>,
    key: Text,
    help: Text,
}

impl SettingsView {
    pub fn new(app: &App) -> Self {
        let root =
            View::new(app).width(Length::percent(100.0)).grow(1.0).min_height(0).column().padding_edge(Edge::Top, 1);
        let list = View::new(app).width(Length::percent(100.0)).column();
        let mut headings = Vec::new();
        let mut rows = Vec::new();
        let mut section = None;
        for row in Row::ALL {
            if section != Some(row.section()) {
                let heading = line(app);
                if section.is_some() {
                    heading.margin_edge(Edge::Top, 1);
                }
                list.add(&heading);
                headings.push((heading, row.section()));
                section = Some(row.section());
            }
            rows.push(RowView::new(app, &list));
        }
        let key = line(app).margin_edge(Edge::Top, 1);
        let help = line(app);
        root.add(&list).add(&key).add(&help);
        Self { root, headings, rows, key, help }
    }

    /// Paints the headings, every row's name and value, and the key and help of the row at `selected`.
    pub fn paint(&self, values: &Settings, selected: usize, p: &Palette) {
        for (heading, title) in &self.headings {
            heading.set_text(content(&[Seg::new(format!("  {}", title.to_uppercase()), p.dim).bold()], p.text));
        }
        for (index, (row, view)) in Row::ALL.iter().zip(&self.rows).enumerate() {
            let on = index == selected;
            view.root.bg(if on { color(p.row_active_bg) } else { Color::TRANSPARENT });
            let label = Seg::new(
                format!("{:<LABEL_WIDTH$}", format!("    {}", row.title())),
                if on { p.text } else { p.muted },
            );
            view.label.set_text(content(&[if on { label.bold() } else { label }], p.text));
            view.value.set_text(content(&value(values, *row, on, p), p.text));
        }
        let row = Row::ALL.get(selected);
        let key = row.map_or(String::new(), |row| format!("  settings.json  {}", row.key()));
        self.key.set_text(content(&[Seg::new(key, p.dim)], p.text));
        let help = row.map_or(String::new(), |row| format!("  {}", row.help()));
        self.help.set_text(content(&[Seg::new(help, p.muted)], p.text));
    }
}

/// A row's value: on or off with a dot, a number as it is, and a choice between `‹ ›` that the keys step through.
fn value(values: &Settings, row: Row, on: bool, p: &Palette) -> Vec<Seg> {
    // A theme id is the file's own text, so it is shown as the footer shows a name: no control characters.
    let shown = display_label(&values.shown(row));
    if row.toggle() {
        let set = shown == "true";
        let (dot, word, fg) = if set { ("●", "on", p.accent) } else { ("○", "off", p.muted) };
        return vec![Seg::new(format!("{dot} {word}"), fg)];
    }
    if row.number() {
        return vec![Seg::new(shown, p.accent_soft)];
    }
    let arrows = if on { p.muted } else { p.dim };
    vec![Seg::new("‹ ", arrows), Seg::new(shown, p.accent_soft), Seg::new(" ›", arrows)]
}
