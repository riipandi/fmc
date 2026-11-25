//! The viewer's scrollbar (Scouter `renderScrollbar`, `src/ui/styled.ts:28-51`): a column at the viewer's right edge,
//! drawn only while the text is taller than the viewport. The thumb is the tree's (`ui/tree.rs` `thumb`), since
//! Scouter draws both with the same routine. The columns it takes depend on what fills the pane: plain text leaves
//! one blank before the thumb, filled content (wrapped text, the rendered view, a picture) leaves two.

use std::cell::RefCell;

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, content};
use super::tree::thumb;

pub struct ScrollBar {
    root: View,
    cells: RefCell<Vec<Text>>,
    /// Blank columns before the thumb.
    gap: usize,
}

impl ScrollBar {
    /// A bar that takes `gap + 1` columns from the row it sits in.
    pub fn new(app: &App, gap: usize) -> Self {
        let root = View::new(app).column().width(gap as u32 + 1).shrink(0.0).visible(false);
        Self { root, cells: RefCell::new(Vec::new()), gap }
    }

    pub fn root(&self) -> View {
        self.root.clone()
    }

    /// Draws the thumb for `total` rows of text in a `viewport`-row view scrolled to `top`. The bar
    /// shows only when `show` and the text is taller than the view.
    pub fn paint(&self, app: &App, total: usize, viewport: usize, top: usize, show: bool, p: &Palette) {
        let height = viewport;
        self.root.visible(show && total > viewport && height > 0);
        let glyphs = thumb(total, viewport, top, height);
        let mut cells = self.cells.borrow_mut();
        let width = self.gap as u32 + 1;
        while cells.len() < height {
            let cell = Text::new(app, "").height(1).width(width).wrap(WrapMode::None).selectable(false);
            self.root.add(&cell);
            cells.push(cell);
        }
        // The gap is blank, drawn in Scouter's default foreground so the captures agree cell for cell.
        let gap = Seg::new(" ".repeat(self.gap), "#ffffff");
        for (index, cell) in cells.iter().enumerate() {
            let thumb =
                glyphs.get(index).copied().flatten().map_or_else(|| Seg::new(" ", "#ffffff"), |g| Seg::new(g, p.muted));
            cell.set_text(content(&[gap.clone(), thumb], p.text));
        }
    }
}
