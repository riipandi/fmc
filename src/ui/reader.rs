//! The text view (Scouter `src/reader/code-view.ts`): the document's lines in a line-number gutter, coloured by its
//! filetype. It scrolls inside the viewer (`ui/viewer.rs`); wrapping and the width it takes are set here.

use std::cell::Cell;
use std::sync::Arc;

use fmc::reader::document::Document;
use fmc::theme::Palette;
use opentui::native::{TextView, Viewport};
use opentui::prelude::*;

use super::styled::color;
use super::syntax;

/// Shown in place of an empty file's lines, with no gutter (`src/reader/code-view.ts:288`).
const EMPTY_FILE: &str = "(empty file)";

pub struct CodeView {
    code: Code,
    numbers: LineNumbers,
    /// A second view of the code's text, which counts the rows the text takes when it wraps.
    counter: Option<TextView>,
    /// The columns the text takes without wrapping: the pane's width. Scouter's text is pane-wide, so the viewer pans
    /// only past the gutter (six columns), and longer lines are cut at the pane's edge.
    width: Cell<u32>,
    wrap: Cell<bool>,
}

impl CodeView {
    pub fn new(app: &App, p: &Palette) -> Self {
        let code = Code::new(app);
        // Scouter's gutter is a `LineNumberRenderable` with `minWidth: 3` and `paddingRight: 2` (`code-view.ts:99-108`).
        let numbers = LineNumbers::new(app, &code).gutter_min_width(3).gutter_padding(2).visible(false);
        let counter = TextView::new(code.buffer()).ok();
        let view = Self { code, numbers, counter, width: Cell::new(0), wrap: Cell::new(false) };
        view.restyle(p);
        view
    }

    pub fn widget(&self) -> LineNumbers {
        self.numbers.clone()
    }

    pub fn restyle(&self, p: &Palette) {
        self.code.fg(color(p.text)).syntax_theme(Arc::new(syntax::theme(p)));
        self.numbers.fg(color(p.dim));
    }

    /// Shows a document; `language` is its filetype when Scouter highlights it.
    pub fn show(&self, document: &Document, language: Option<&str>) {
        let empty = document.lines == 0;
        self.code.filetype(if empty { None } else { language });
        self.numbers.show_line_numbers(!empty);
        let content = if empty { EMPTY_FILE } else { document.content() };
        self.code.set_code(content);
        self.apply_width();
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

    fn apply_width(&self) {
        let width = if self.wrap.get() { Length::percent(100.0) } else { Length::Cells(self.width.get() as f32) };
        self.code.width(width);
    }
}
