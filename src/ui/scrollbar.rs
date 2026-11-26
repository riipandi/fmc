//! The viewer's scrollbar (Scouter `renderScrollbar`, `src/ui/styled.ts:28-51`): a column at the viewer's right edge,
//! drawn only while the text is taller than the viewport. The thumb is the tree's (`ui/tree.rs` `thumb`), since
//! Scouter draws both with the same routine. The columns it takes depend on what fills the pane: plain text leaves
//! one blank before the thumb, filled content (wrapped text, a picture) leaves two, and the rendered view leaves none
//! (its bar is the one-column bar of Scouter's scroll box, `app/mod.rs` paint).
//!
//! A bar's drag, track click, and position come from a transparent `Slider` laid over the column, as Scouter's bar is
//! a transparent slider under its painted thumb (`styled.ts:24-26`). The crate's slider does the pointer maths; the
//! glyphs stay ours.

use std::cell::RefCell;

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, content};
use super::tree::thumb;

/// The drag over a bar's thumb column: the bar's last column, the height of the bar it is added to. The blank columns
/// before the thumb are not the track: a press there does nothing in Scouter (`scouter:src/app/reader-pane.ts:31-48`).
pub fn track(app: &App) -> Slider {
    Slider::new(app)
        .vertical()
        .focusable(false)
        .colors(Color::TRANSPARENT, Color::TRANSPARENT)
        .absolute()
        .position(Edge::Top, 0)
        .position(Edge::Bottom, 0)
        .position(Edge::Right, 0)
        .width(1)
        // Above the thumb's cells, which are added later: a press on a cell must reach the slider.
        .z_index(i32::MAX)
        // A release leaves the thumb where the drag put it. The slider would apply the press's grab offset again, which
        // is measured on the drawn thumb, and snap the value by up to a row; Scouter's slider skips a release at the
        // press's own cell (`@opentui/core` `setupMouseHandling`, `onMouseUp`).
        .on_mouse(|cx, event| {
            if event.kind == MouseEventKind::Up(MouseButton::Left) {
                cx.prevent_default();
            }
        })
}

/// Sets the track to the frame's scroll state: `total` rows of which `viewport` show, from row `top`. The thumb is the
/// viewport's share of the range, as Scouter's scroll box sets its bar.
pub fn sync(track: &Slider, total: usize, viewport: usize, top: usize) {
    track.range(0.0, total.saturating_sub(viewport) as f64).thumb_size(viewport.max(1) as f64).set_value(top as f64);
}

pub struct ScrollBar {
    root: View,
    track: Slider,
    cells: RefCell<Vec<Text>>,
    /// Blank columns before the thumb.
    gap: usize,
}

impl ScrollBar {
    /// A bar that takes `gap + 1` columns from the row it sits in.
    pub fn new(app: &App, gap: usize) -> Self {
        let root = View::new(app).column().width(gap as u32 + 1).shrink(0.0).visible(false);
        let track = track(app);
        root.add(&track);
        Self { root, track, cells: RefCell::new(Vec::new()), gap }
    }

    pub fn root(&self) -> View {
        self.root.clone()
    }

    /// The drag over this bar, for the app to wire its moves.
    pub fn track(&self) -> Slider {
        self.track.clone()
    }

    /// Draws the thumb for `total` rows of text in a `viewport`-row view scrolled to `top`. The bar
    /// shows only when `show` and the text is taller than the view.
    pub fn paint(&self, app: &App, total: usize, viewport: usize, top: usize, show: bool, p: &Palette) {
        let height = viewport;
        self.root.visible(show && total > viewport && height > 0);
        sync(&self.track, total, viewport, top);
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

#[cfg(test)]
#[path = "scrollbar_tests.rs"]
mod tests;
