//! The sidebar width (Scouter `src/explorer/sidebar-resizer.ts`): a quarter of the terminal within 28..=40
//! columns, never below 24 or so wide that the reader gets fewer than 28. The mark that shows and hides it (plan-00002).

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, content};

const MIN_SIDEBAR_COLUMNS: i64 = 24;
const MIN_READER_COLUMNS: i64 = 28;

/// The one-cell mark on the top border row: `«` on the sidebar's border while it shows, `»` on the reader's corner
/// while it is hidden (plan-00002, D8). It is a child of the row, above the borders, so its press is its own.
pub struct Mark {
    pub root: View,
    glyph: Text,
}

impl Mark {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).absolute().width(1).height(1).position(Edge::Top, 0);
        let glyph = Text::new(app, "").height(1).wrap(WrapMode::None).selectable(false);
        root.add(&glyph);
        Self { root, glyph }
    }

    pub fn paint(&self, shown: bool, column: u16, p: &Palette) {
        self.root.position(Edge::Left, column);
        self.glyph.set_text(content(&[Seg::new(if shown { "«" } else { "»" }, p.dim)], p.text));
    }
}

pub fn constrain(terminal: u16, width: i64) -> u16 {
    let maximum = (i64::from(terminal) - MIN_READER_COLUMNS).max(1);
    width.clamp(MIN_SIDEBAR_COLUMNS.min(maximum), maximum) as u16
}

/// The width for a terminal `terminal` columns wide, or the user's own width when set.
pub fn fit(terminal: u16, preferred: Option<i64>) -> u16 {
    let default = (i64::from(terminal) / 4).clamp(28, 40);
    constrain(terminal, preferred.unwrap_or(default))
}

/// The preferred width after a drag that started at column `start_x` with the sidebar `start_width` wide, now at
/// column `x` (Scouter `dragTo`). The caller constrains it.
pub fn dragged(start_width: u16, start_x: u16, x: u16) -> i64 {
    i64::from(start_width) + i64::from(x) - i64::from(start_x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_of_the_terminal_within_bounds() {
        assert_eq!(fit(80, None), 28);
        assert_eq!(fit(140, None), 35);
        assert_eq!(fit(187, None), 40);
        assert_eq!(fit(50, None), 22);
        assert_eq!(fit(20, None), 1);
        assert_eq!(fit(80, Some(10)), 24);
    }

    #[test]
    fn a_drag_moves_the_edge_by_its_distance() {
        assert_eq!(dragged(28, 27, 33), 34);
        assert_eq!(dragged(28, 27, 10), 11);
        assert_eq!(constrain(80, dragged(28, 27, 0)), 24, "the constraint applies after the drag");
    }
}
