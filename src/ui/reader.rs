//! The text view (Scouter `src/reader/code-view.ts`): the document's lines in a line-number gutter, coloured by its
//! filetype. It scrolls inside the viewer (`ui/viewer.rs`); wrapping and the width it takes are set here.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use fmc::reader::document::Document;
use fmc::reader::search::match_ranges;
use fmc::theme::Palette;
use opentui::native::{LayerRun, TextView, Viewport};
use opentui::prelude::*;
use opentui::widget::LineColor;

use super::styled::color;
use super::syntax;

/// Shown in place of an empty file's lines, with no gutter (`src/reader/code-view.ts:288`).
const EMPTY_FILE: &str = "(empty file)";

/// The layer the search marks go on, drawn over the syntax layer (1), as Scouter's `search.match` group is.
const SEARCH_LAYER: u32 = 2;

pub struct CodeView {
    code: Code,
    numbers: LineNumbers,
    /// A second view of the code's text, which counts the rows the text takes when it wraps.
    counter: Option<TextView>,
    /// The columns the text takes without wrapping: the pane's width. Scouter's text is pane-wide, so the viewer pans
    /// only past the gutter (six columns), and longer lines are cut at the pane's edge.
    width: Cell<u32>,
    wrap: Cell<bool>,
    /// The search marks last applied, as the query and the current line: skipped while they stay the same.
    applied: RefCell<Option<(String, Option<usize>)>>,
}

impl CodeView {
    pub fn new(app: &App, p: &Palette) -> Self {
        let code = Code::new(app);
        // Scouter's gutter is a `LineNumberRenderable` with `minWidth: 3` and `paddingRight: 2` (`code-view.ts:99-108`).
        let numbers = LineNumbers::new(app, &code).gutter_min_width(3).gutter_padding(2).visible(false);
        let counter = TextView::new(code.buffer()).ok();
        let view =
            Self { code, numbers, counter, width: Cell::new(0), wrap: Cell::new(false), applied: RefCell::new(None) };
        view.restyle(p);
        view
    }

    pub fn widget(&self) -> LineNumbers {
        self.numbers.clone()
    }

    pub fn restyle(&self, p: &Palette) {
        self.code.fg(color(p.text)).syntax_theme(Arc::new(syntax::theme(p)));
        self.install_table(p);
        self.numbers.fg(color(p.dim));
    }

    /// Shows a document; `language` is its filetype when Scouter highlights it.
    pub fn show(&self, document: &Document, language: Option<&str>) {
        let empty = document.lines == 0;
        self.code.filetype(if empty { None } else { language });
        self.numbers.show_line_numbers(!empty);
        let content = if empty { EMPTY_FILE } else { document.content() };
        self.code.set_code(content);
        // Setting the text empties the buffer's layers, so the marks are applied again.
        *self.applied.borrow_mut() = None;
        self.apply_width();
    }

    /// The search's marks over the code, and its current line in the gutter (Scouter `markMatches`, `setActiveLine`).
    /// Nothing changes when the query and the line stay the same.
    pub fn sync_search(&self, p: &Palette, query: &str, current: Option<usize>, text: &str) {
        let key = (query.to_owned(), current);
        if self.applied.borrow().as_ref() == Some(&key) {
            return;
        }
        *self.applied.borrow_mut() = Some(key);
        let buffer = self.code.buffer();
        if let Ok(group) = buffer.context().group_set(&["opentui.search.match"]) {
            let runs: Vec<LayerRun> = match_ranges(text, query)
                .into_iter()
                .map(|range| LayerRun { start_byte: range.start as u32, end_byte: range.end as u32, group_set: group })
                .collect();
            let _ = buffer.set_layer(SEARCH_LAYER, 0..u32::MAX, &runs);
        }
        self.numbers.clear_line_colors();
        if let Some(line) = current {
            let colors = LineColor::new(color(p.target_gutter), color(p.target_bg));
            self.numbers.line_color(line as u32, colors);
        }
    }

    pub fn visible(&self, visible: bool) {
        self.numbers.visible(visible);
    }

    /// Wrapping long lines; without it a line runs past the edge and the viewer pans (Scouter `setWrap`).
    pub fn set_wrap(&self, wrap: bool) {
        if self.wrap.replace(wrap) != wrap {
            self.code.wrap(if wrap { WrapMode::Word } else { WrapMode::None });
            self.apply_width();
        }
    }

    /// The rows the text takes at the width it has: its lines, or its wrapped rows (Scouter's `scrollHeight`).
    pub fn rows(&self, lines: usize) -> usize {
        let Some(counter) = self.counter.as_ref().filter(|_| self.wrap.get()) else { return lines };
        let width = self.code.element().rect().width.max(1.0) as u32;
        let viewport = Viewport { x: 0, y: 0, width, height: u32::MAX / 2 };
        if counter.set_viewport(viewport).is_err() || counter.set_wrap(WrapMode::Word).is_err() {
            return lines;
        }
        counter.info().map_or(lines, |info| info.total_virtual_line_count as usize)
    }

    /// Sets the pane's text width in columns, the width unwrapped text takes.
    pub fn set_width(&self, columns: u32) {
        if self.width.replace(columns) != columns {
            self.apply_width();
        }
    }

    /// The buffer's style table: the widget's own names, and the search mark (see `syntax::table`).
    fn install_table(&self, p: &Palette) {
        let buffer = self.code.buffer();
        let search = Style::new().fg(color(p.match_fg)).bg(color(p.match_bg)).bold();
        if let Ok(table) = syntax::table(buffer.context(), &syntax::theme(p), search) {
            let _ = buffer.set_syntax_style(Some(&table));
        }
    }

    fn apply_width(&self) {
        let width = if self.wrap.get() { Length::percent(100.0) } else { Length::Cells(self.width.get() as f32) };
        self.code.width(width);
    }
}
