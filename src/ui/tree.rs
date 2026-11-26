//! The file tree's rows and scrollbar (Scouter `src/explorer/explorer.ts` `paintRow`, `src/ui/styled.ts` scrollbar).
//! The row text comes from `row_segments` and the thumb from `thumb`; both are pure, so tests run without a terminal.

use std::path::Path;

use fmc::diff::Marks;
use fmc::explorer::{Browser, Entry, Kind, Row, relative};
use fmc::find::text::Snippet;
use fmc::text::display_label;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};

/// Screen rows above the list: the panel's top border and the two header rows.
pub const LIST_TOP: u16 = 3;

/// One row's text: the selection bar, depth guides, folder marker, icon, and name (Scouter `paintRow`).
pub fn row_segments(row: &Row<'_>, selected: bool, ignored: bool, p: &Palette) -> Vec<Seg> {
    if let Some((line, snippet)) = row.hit {
        return hit_segments(row.entry, line, snippet, selected, p);
    }
    let folder = row.folder;
    let marker = match (folder, row.expanded) {
        (true, true) => "▾ ",
        (true, false) => "▸ ",
        _ => "  ",
    };
    let name_fg = if selected {
        p.accent
    } else if ignored {
        p.ignored
    } else {
        p.text
    };
    let glyph_fg = if selected {
        p.accent
    } else if ignored {
        p.ignored
    } else {
        icon_color(row.entry, folder, p)
    };
    let mut segments = vec![Seg::new(if selected { "▎" } else { " " }, p.accent)];
    if row.depth > 0 {
        segments.push(Seg::new("│ ".repeat(row.depth), p.ignored));
    }
    segments.push(Seg::new(marker, if selected { p.accent } else { p.dim }));
    segments.push(Seg::new(format!("{} ", icon(row.entry, folder, row.expanded)), glyph_fg));
    let name = Seg::new(format!("{}{}", display_label(&row.entry.name), if folder { "/" } else { "" }), name_fg);
    segments.push(if selected { name.bold() } else { name });
    segments
}

/// A text hit's row: the file's name and line, the snippet with the match in accent, and the folder dimmed at the end
/// (Scouter `matchContent`).
fn hit_segments(entry: &Entry, line: usize, snippet: &Snippet, selected: bool, p: &Palette) -> Vec<Seg> {
    let (folder, name) = match entry.name.rsplit_once('/') {
        Some((folder, name)) => (Some(folder), name),
        None => (None, entry.name.as_str()),
    };
    let name_fg = if selected { p.accent } else { p.text };
    let name = Seg::new(format!("{} {}", icon(entry, false, false), display_label(name)), name_fg);
    let mut segments =
        vec![Seg::new(if selected { "▎" } else { " " }, p.accent), if selected { name.bold() } else { name }];
    segments.push(Seg::new(format!(":{line}  "), p.dim));
    segments.push(Seg::new(display_label(&snippet.before), p.muted));
    segments.push(Seg::new(display_label(&snippet.matched), p.accent_soft).bold());
    segments.push(Seg::new(display_label(&snippet.after), p.muted));
    if let Some(folder) = folder {
        segments.push(Seg::new(format!("  {}/", display_label(folder)), p.dim));
    }
    segments
}

/// The Nerd Font glyph for a row (Scouter `src/explorer/explorer.ts:32-54`, `:264-270`).
fn icon(entry: &Entry, folder: bool, expanded: bool) -> &'static str {
    if folder {
        return if expanded { "\u{f07c}" } else { "\u{f07b}" };
    }
    if entry.kind == Kind::Link {
        return "\u{f0c1}";
    }
    match extension(entry).as_str() {
        "js" | "mjs" | "cjs" => "\u{e74e}",
        "ts" | "mts" | "cts" => "\u{e628}",
        "jsx" | "tsx" => "\u{e7ba}",
        "json" => "\u{e60b}",
        "md" | "mdx" => "\u{f48a}",
        "py" => "\u{e73c}",
        "go" => "\u{e627}",
        "rs" => "\u{e7a8}",
        "zig" => "\u{e6a9}",
        "html" => "\u{e736}",
        "css" => "\u{e749}",
        "sh" => "\u{f489}",
        "yaml" | "yml" | "toml" => "\u{f013}",
        _ => "\u{f016}",
    }
}

/// The icon's color (Scouter `iconColor`, `src/explorer/explorer.ts:56-91`).
fn icon_color(entry: &Entry, folder: bool, p: &Palette) -> &'static str {
    if folder {
        return p.folder;
    }
    if entry.kind == Kind::Link {
        return p.accent_soft;
    }
    match extension(entry).as_str() {
        "ts" | "mts" | "cts" | "tsx" | "jsx" | "go" | "css" => p.syntax.function,
        "js" | "mjs" | "cjs" | "json" | "yaml" | "yml" | "toml" | "zig" => p.syntax.constant,
        "md" | "mdx" => p.accent_soft,
        "py" | "sh" => p.syntax.string,
        "rs" | "html" => p.syntax.keyword,
        _ => p.muted,
    }
}

/// The lowercase extension without its dot; a dot-file such as `.gitignore` has none.
fn extension(entry: &Entry) -> String {
    entry.path.extension().map_or_else(String::new, |ext| ext.to_string_lossy().to_lowercase())
}

/// The scrollbar thumb for each of `height` rows, in half-block glyphs (Scouter `renderScrollbar`,
/// `src/ui/styled.ts:28-51`). Nothing is drawn when the list fits.
pub fn thumb(total: usize, viewport: usize, top: usize, height: usize) -> Vec<Option<&'static str>> {
    if height == 0 || total <= viewport {
        return vec![None; height];
    }
    let track = height * 2;
    let range = (total - viewport) as f64;
    let ratio = (top as f64 / range).clamp(0.0, 1.0);
    let size = (((viewport as f64 / total as f64) * track as f64).round() as usize).min(track).max(2.min(track));
    let start = (ratio * (track - size) as f64).round() as usize;
    let end = start + size;
    (0..height)
        .map(|row| {
            let (upper, lower) = (start <= row * 2 && end > row * 2, start <= row * 2 + 1 && end > row * 2 + 1);
            match (upper, lower) {
                (true, true) => Some("▐"),
                (true, false) => Some("▝"),
                (false, true) => Some("▗"),
                (false, false) => None,
            }
        })
        .collect()
}

/// The list line under screen row `y`, if the list starts at `top` and is `height` lines tall.
pub fn line_at(y: u16, top: u16, height: usize) -> Option<usize> {
    let line = usize::from(y.checked_sub(top)?);
    (line < height).then_some(line)
}

/// One list line as the view paints it: its text, whether it is selected, its mark, and its thumb glyph.
struct Line {
    segments: Vec<Seg>,
    selected: bool,
    mark: Option<(&'static str, &'static str)>,
    thumb: Option<&'static str>,
}

/// A row's widgets: the name and the mark at the right. The row's background stops before the thumb column.
struct RowView {
    root: View,
    name: Text,
    mark: Text,
}

impl RowView {
    fn new(app: &App, rows: &View) -> Self {
        let root = View::new(app).height(1).width(Length::percent(100.0)).row().shrink(0.0);
        let name =
            Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
        let mark = Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Left, 1).wrap(WrapMode::None);
        root.add(&name).add(&mark);
        rows.add(&root);
        Self { root, name, mark }
    }
}

/// The file tree's list: rows from the browser's scroll position, and the scrollbar column beside them.
pub struct TreeView {
    /// The list area; the app sizes it, and its top row sets `LIST_TOP`.
    pub root: View,
    rows: View,
    bar: View,
    views: Vec<RowView>,
    thumbs: Vec<Text>,
}

impl TreeView {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).width(Length::percent(100.0)).grow(1.0).min_height(0).column();
        let body = View::new(app).width(Length::percent(100.0)).grow(1.0).min_height(0).row();
        let rows = View::new(app).grow(1.0).min_width(0).column();
        let bar = View::new(app).width(1).shrink(0.0).column();
        body.add(&rows).add(&bar);
        root.add(&body);
        Self { root, rows, bar, views: Vec::new(), thumbs: Vec::new() }
    }

    /// Paints the visible rows. Before the root is listed the list is empty (no flash of a collapsed root).
    pub fn paint(&mut self, app: &App, browser: &Browser, marks: &Marks, p: &Palette, root: &Path) {
        // The finder's rows need no listed root; the tree does (Scouter draws nothing before the root lists).
        let lines = if browser.is_flat() || browser.tree.is_listed(root) {
            self.lines(browser, marks, p, root)
        } else {
            Vec::new()
        };
        // The thumb column exists only while the list scrolls; a hidden widget takes no space (Scouter's scrollbar).
        self.bar.visible(browser.row_count() > browser.height());
        while self.views.len() < lines.len() {
            self.views.push(RowView::new(app, &self.rows));
            let thumb = Text::new(app, "").height(1).width(1).wrap(WrapMode::None).selectable(false);
            self.bar.add(&thumb);
            self.thumbs.push(thumb);
        }
        for ((view, thumb), line) in self.views.iter().zip(&self.thumbs).zip(&lines) {
            view.root.visible(true);
            view.name.set_text(content(&line.segments, p.text));
            let (letter, color_hex) = line.mark.unwrap_or(("", p.text));
            view.mark.set_text(content(&[Seg::new(letter, color_hex).bold()], p.text));
            view.root.bg(if line.selected { color(p.row_active_bg) } else { Color::TRANSPARENT });
            thumb.set_text(match line.thumb {
                Some(glyph) => content(&[Seg::new(glyph, p.muted)], p.text),
                None => content(&[], p.text),
            });
        }
        for (view, thumb) in self.views.iter().zip(&self.thumbs).skip(lines.len()) {
            view.root.visible(false);
            thumb.set_text(content(&[], p.text));
        }
    }

    fn lines(&self, browser: &Browser, marks: &Marks, p: &Palette, root: &Path) -> Vec<Line> {
        let visible = browser.visible_rows();
        let thumbs = thumb(browser.row_count(), browser.height(), browser.top(), visible.len());
        // The selection is a row, not a path: a file with several text hits has a row for each (Browser `flat_at`).
        let selected_row = browser.selected_index();
        visible
            .iter()
            .zip(thumbs)
            .enumerate()
            .map(|(line, (row, thumb))| {
                let selected = browser.top() + line == selected_row;
                let key = relative(root, &row.entry.path);
                Line {
                    segments: row_segments(row, selected, marks.is_ignored(&key), p),
                    selected,
                    mark: marks.mark(&key, p).map(|mark| (mark.letter, mark.color)),
                    thumb,
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tests;
