//! The binary notice (Scouter `src/reader/binary-view.ts`): a centred block with the file's name and size, and what
//! the reader does show.

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, content};

/// The line under the title: what the reader shows instead (the product name is fmc, `docs/parity.md`).
pub const HINT: &str = "fmc only displays UTF-8 text and images.";

pub struct BinaryView {
    root: View,
    title: Text,
    meta: Text,
    hint: Text,
}

impl BinaryView {
    pub fn new(app: &App) -> Self {
        let root = View::new(app)
            .grow(1.0)
            .min_height(0)
            .min_width(0)
            .column()
            .align_items(Align::Center)
            .justify(Justify::Center)
            .overflow(Overflow::Hidden)
            .visible(false);
        let line = |margin: u16| {
            Text::new(app, "")
                .height(1)
                .shrink(0.0)
                .margin_edge(Edge::Top, margin)
                .wrap(WrapMode::None)
                .selectable(false)
        };
        let (title, meta, hint) = (line(0), line(1), line(1));
        root.add(&title).add(&meta).add(&hint);
        Self { root, title, meta, hint }
    }

    pub fn root(&self) -> View {
        self.root.clone()
    }

    pub fn visible(&self, visible: bool) {
        self.root.visible(visible);
    }

    /// Paints the notice for `name` (already display-safe) and `size` (already formatted).
    pub fn paint(&self, name: &str, size: &str, p: &Palette) {
        self.title.set_text(content(&[Seg::new("Binary files are not supported.", p.text).bold()], p.text));
        self.meta.set_text(content(
            &[Seg::new(name, p.accent_soft), Seg::new("  ·  ", p.ignored), Seg::new(size, p.muted)],
            p.text,
        ));
        self.hint.set_text(content(&[Seg::new(HINT, p.dim)], p.text));
    }
}

#[cfg(test)]
#[path = "binary_tests.rs"]
mod tests;
