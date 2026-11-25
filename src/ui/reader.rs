//! The text view (Scouter `src/reader/code-view.ts`): the document's lines in a line-number gutter, coloured by its
//! filetype. Scrolling and the motions arrive with plan-00001 W3 phase 14.

use std::sync::Arc;

use fmc::reader::document::Document;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::color;
use super::syntax;

/// Shown in place of an empty file's lines, with no gutter (`src/reader/code-view.ts:288`).
const EMPTY_FILE: &str = "(empty file)";

pub struct CodeView {
    code: Code,
    numbers: LineNumbers,
}

impl CodeView {
    pub fn new(app: &App, p: &Palette) -> Self {
        let code = Code::new(app).grow(1.0).min_width(0).min_height(0);
        // Scouter's gutter is a `LineNumberRenderable` with `minWidth: 3` and `paddingRight: 2` (`code-view.ts:99-108`).
        let numbers = LineNumbers::new(app, &code)
            .gutter_min_width(3)
            .gutter_padding(2)
            .grow(1.0)
            .min_width(0)
            .min_height(0)
            .visible(false);
        let view = Self { code, numbers };
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
        self.code.set_code(if empty { EMPTY_FILE } else { document.content() });
    }

    pub fn visible(&self, visible: bool) {
        self.numbers.visible(visible);
    }
}
