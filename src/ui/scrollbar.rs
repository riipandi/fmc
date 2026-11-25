//! The viewer's scrollbar (Scouter `renderScrollbar`, `src/ui/styled.ts:28-51`): one column beside the text, drawn
//! only while the text is taller than the viewport, so the text gives up one column to it. The thumb is the tree's
//! (`ui/tree.rs` `thumb`), since Scouter draws both with the same routine.

use std::cell::RefCell;

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, content};
use super::tree::thumb;

pub struct ScrollBar {
    root: View,
    cells: RefCell<Vec<Text>>,
}

impl ScrollBar {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).column().width(1).shrink(0.0).visible(false);
        Self { root, cells: RefCell::new(Vec::new()) }
    }

    pub fn root(&self) -> View {
        self.root.clone()
    }

    /// Draws the thumb for `total` rows of text in a `viewport`-row view scrolled to `top`, `height` rows tall.
    pub fn paint(&self, app: &App, total: usize, viewport: usize, top: usize, height: usize, p: &Palette) {
        self.root.visible(total > viewport && height > 0);
        let glyphs = thumb(total, viewport, top, height);
        let mut cells = self.cells.borrow_mut();
        while cells.len() < height {
            let cell = Text::new(app, "").height(1).width(1).wrap(WrapMode::None).selectable(false);
            self.root.add(&cell);
            cells.push(cell);
        }
        for (index, cell) in cells.iter().enumerate() {
            let segments: Vec<Seg> =
                glyphs.get(index).copied().flatten().map(|g| Seg::new(g, p.muted)).into_iter().collect();
            cell.set_text(content(&segments, p.text));
        }
    }
}
