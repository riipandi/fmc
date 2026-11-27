//! The rendered view's git signs (plan-00010, D6): a column over the viewer's left edge, where the rendered text's padding
//! is, with a bar beside each changed block and a `▁` or `▔` beside a deletion. The column is absolutely positioned, so the
//! text does not move; each mark sits at its block's document row less the scroll, and it is laid out again at each paint.

use std::cell::RefCell;

use fmc::diff::Sign;
use fmc::reader::rendered_signs::BlockMark;
use fmc::theme::{Hex, Palette};
use opentui::prelude::*;
use opentui::widget::Text;

use super::styled::color;

/// The glyph of a bar, as the reader's strip draws it (`diff/signs.rs`).
const BAR: &str = "▎";
const UNDER: &str = "▁";
const OVER: &str = "▔";

pub struct SignStrip {
    overlay: View,
    /// The marks' widgets, reused from paint to paint; the ones past the current marks are hidden.
    cells: RefCell<Vec<Text>>,
}

impl SignStrip {
    pub fn new(app: &App) -> Self {
        let overlay = View::new(app)
            .absolute()
            .position(Edge::Left, 0)
            .position(Edge::Top, 0)
            .position(Edge::Bottom, 0)
            .width(1)
            .overflow(Overflow::Hidden)
            .z_index(1);
        Self { overlay, cells: RefCell::new(Vec::new()) }
    }

    /// The column, which the viewer's row holds beside the scrolling box.
    pub fn view(&self) -> View {
        self.overlay.clone()
    }

    /// Lays the marks out: `tops` are each block's document top and rows, and `scroll` is the viewer's row offset.
    pub fn show(&self, app: &App, p: &Palette, marks: &[BlockMark], tops: &[(u32, u32)], scroll: f64) {
        let mut cells = self.cells.borrow_mut();
        let mut used = 0;
        let mut put = |cells: &mut Vec<Text>, text: String, colour: Hex, row: f64, rows: u32| {
            if cells.len() == used {
                let cell = Text::new(app, "").width(1).wrap(WrapMode::None).selectable(false);
                self.overlay.add(&cell);
                cells.push(cell);
            }
            let cell = &cells[used];
            used += 1;
            cell.set_text(text)
                .fg(color(colour))
                .height(rows)
                .visible(true)
                .absolute()
                .position(Edge::Top, Length::Cells((row - scroll) as f32));
            cell.position(Edge::Left, 0);
        };
        for mark in marks {
            let Some(&(top, rows)) = tops.get(mark.block) else { continue };
            if let Some(sign) = mark.bar {
                let text = vec![BAR; rows.max(1) as usize].join("\n");
                put(&mut cells, text, colour_of(sign, p), f64::from(top), rows.max(1));
            }
            if mark.above {
                put(&mut cells, OVER.into(), p.removed, f64::from(top), 1);
            }
            if mark.below {
                put(&mut cells, UNDER.into(), p.removed, f64::from(top + rows), 1);
            }
        }
        for cell in cells.iter().skip(used) {
            cell.visible(false);
        }
    }
}

/// A bar's colour: the palette's colour of its sign.
fn colour_of(sign: Sign, p: &Palette) -> Hex {
    match sign {
        Sign::Added => p.added,
        Sign::Modified => p.modified,
        Sign::DeletedBelow | Sign::DeletedTop => p.removed,
    }
}
