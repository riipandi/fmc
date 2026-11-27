//! The rendered Markdown view (Scouter `src/reader/code-view.ts`, the `MarkdownRenderable` of `preview`): OpenTUI's
//! Markdown widget with the syntax styles of the palette, the text colour, and two cells of padding on each side.
//! Mermaid fences are drawn as diagrams (`ui/mermaid.rs`); Scouter draws them as code. Scrolling arrives with plan-00001
//! W3 phase 14.

use std::sync::Arc;

use fmc::reader::headings;
use fmc::theme::Palette;
use opentui::prelude::*;
use opentui::widget::{Markdown, MarkdownColors};

use super::mermaid::{Colors, Diagram};
use super::styled::color;
use super::syntax;

pub struct MarkdownView {
    doc: Markdown,
}

impl MarkdownView {
    pub fn new(app: &App, p: &Palette) -> Self {
        let doc = Markdown::new(app).padding_x(2).visible(false);
        let view = Self { doc };
        view.restyle(p);
        view
    }

    pub fn widget(&self) -> Markdown {
        self.doc.clone()
    }

    /// Scouter's prose takes the syntax styles (`scopeStyles`); the colours are the fallbacks for scopes the theme
    /// leaves out, which Core draws in the text colour. Inline and block code have no background there.
    pub fn restyle(&self, p: &Palette) {
        let text = color(p.text);
        let colors = MarkdownColors {
            text,
            heading: color(p.syntax.function),
            emphasis: text,
            code: color(p.syntax.string),
            code_bg: opentui::prelude::Color::TRANSPARENT,
            link: color(p.syntax.function),
            quote: text,
            marker: text,
            // Core draws table and rule borders in this grey when the theme has no `conceal` scope (`index.bun.js:11729`).
            rule: color("#888888"),
        };
        self.doc.colors(colors).syntax_theme(Arc::new(syntax::theme(p))).scope_styles(true);
        // An empty fence keeps the default code block; the renderer is registered again on each restyle, which replaces it.
        let diagram = Colors::new(p);
        self.doc.code_block_renderer("mermaid", move |block| {
            (!block.code.trim().is_empty())
                .then(|| Custom::measured(block.app, Diagram::new(block.code, diagram)).as_any())
        });
    }

    pub fn show(&self, markdown: &str) {
        self.doc.set_markdown(headings::with_markers(markdown));
    }

    pub fn visible(&self, visible: bool) {
        self.doc.visible(visible);
    }

    /// The rows the rendered document takes in the latest frame: the footer's scroll position and the bar count them.
    /// The block rows are screen rows, which move up as the view scrolls, so the widget's own height is the total.
    pub fn rows(&self) -> usize {
        self.doc.rect().height.max(0.0) as usize
    }
}

#[cfg(test)]
#[path = "markdown_tests.rs"]
mod tests;
