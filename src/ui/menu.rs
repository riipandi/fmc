//! The context menu: a bordered list over the page, at a cell and kept on the screen (plan-00012, D1). In the compact tier
//! it is a bottom sheet across the width. Its rows are fmc's own rather than OpenTUI's `Select`: a row can be disabled
//! (muted, skipped by the selection), and a press picks it at once.

use fmc::layout::Tier;
use fmc::overlay::{Item, Menu};
use fmc::theme::{Hex, Palette};
use opentui::prelude::*;

use super::styled::{Seg, color, content};

/// The rows the menu can hold. shortcut: a longer menu is cut at this many rows; grow the pool when a menu needs more.
pub const MAX_ROWS: usize = 16;

/// A rectangle in terminal cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// The gap between a row's label and its key hint.
const HINT_GAP: usize = 3;

/// The box a menu of `items` takes: two cells of padding on each side, the longest label, the gap and the longest key
/// hint, and the border; a row per item and per group rule.
pub fn size(items: &[Item]) -> (u16, u16) {
    let longest = items.iter().map(|item| item.label.chars().count()).max().unwrap_or(0);
    let hint = items.iter().map(|item| item.hint.chars().count()).max().unwrap_or(0);
    let hint = if hint > 0 { HINT_GAP + hint } else { 0 };
    let rules = items.iter().filter(|item| item.rule).count();
    ((longest + hint) as u16 + 6, (items.len() + rules) as u16 + 2)
}

/// Where a box of `size` goes at `anchor`: flipped above the anchor when it would run off the bottom, and moved left
/// when it would run off the right, so it stays on the screen. `screen` is the columns and the rows.
pub fn place(anchor: (u16, u16), size: (u16, u16), screen: (u16, u16)) -> (u16, u16) {
    let (x, y) = anchor;
    let (width, height) = size;
    let (columns, rows) = screen;
    let x = if x.saturating_add(width) <= columns { x } else { columns.saturating_sub(width) };
    let y = if y.saturating_add(height) <= rows {
        y
    } else if y >= height {
        y - height
    } else {
        rows.saturating_sub(height)
    };
    (x, y)
}

/// The compact tier's bottom sheet: the width of the screen, above the bottom bar's row.
pub fn sheet(screen: (u16, u16), height: u16) -> Rect {
    let (columns, rows) = screen;
    Rect { x: 0, y: rows.saturating_sub(height + 1), width: columns, height }
}

/// The menu's box on a screen of `tier`, with its anchor cell in the medium and wide tiers.
pub fn frame(items: &[Item], anchor: (u16, u16), screen: (u16, u16), tier: Tier) -> Rect {
    let (width, height) = size(items);
    if tier == Tier::Compact {
        return sheet(screen, height);
    }
    let (x, y) = place(anchor, (width, height), screen);
    Rect { x, y, width, height }
}

pub struct MenuView {
    pub root: View,
    /// One slot per row: the group rule above it, when it starts a group, and the row.
    slots: Vec<(View, Text)>,
    /// The rows in item order; a row past the items is hidden.
    pub rows: Vec<Text>,
}

impl MenuView {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).absolute().z_index(i32::MAX).column().border(BorderStyle::Rounded).visible(false);
        let line = || Text::new(app, "").height(1).wrap(WrapMode::None).selectable(false);
        let slots: Vec<(View, Text)> = (0..MAX_ROWS)
            .map(|_| {
                let (slot, rule) = (View::new(app).column(), line());
                root.add(&slot.add(&rule));
                (slot, rule)
            })
            .collect();
        let rows = slots
            .iter()
            .map(|(slot, _)| {
                let row = line();
                slot.add(&row);
                row
            })
            .collect();
        Self { root, slots, rows }
    }

    /// Draws `menu` in `rect` on the `background` tint. A disabled row is muted, a destructive one takes the error
    /// colour, and the selected row has the row surface and a bold label (no marker: the owner, 2026-11-27). A key hint sits at the right edge, and a
    /// rule in the border colour opens each group after the first.
    pub fn show(&self, menu: &Menu, rect: Rect, p: &Palette, background: Hex) {
        self.root
            .position(Edge::Left, Length::Cells(f32::from(rect.x)))
            .position(Edge::Top, Length::Cells(f32::from(rect.y)))
            .width(Length::Cells(f32::from(rect.width)))
            .height(Length::Cells(f32::from(rect.height)))
            .bg(color(background))
            .border_color(color(p.accent));
        let inner = usize::from(rect.width).saturating_sub(2);
        for (index, (row, (slot, rule))) in self.rows.iter().zip(&self.slots).enumerate() {
            let item = menu.items().get(index);
            slot.visible(item.is_some());
            let Some(item) = item else { continue };
            rule.visible(item.rule);
            let line = Seg {
                text: format!(" {} ", "─".repeat(inner.saturating_sub(2))),
                fg: Some(p.border),
                bg: None,
                bold: false,
            };
            rule.width(Length::Cells(inner as f32)).set_text(content(&[line], p.text));
            let chosen = index == menu.selected();
            let fill = chosen.then_some(p.row_active_bg);
            let hint = item.hint.chars().count();
            let pad = inner.saturating_sub(2 + item.label.chars().count() + hint + 2);
            let lead = Seg { text: "  ".into(), fg: None, bg: fill, bold: false };
            let label = Seg {
                text: format!("{}{}", item.label, " ".repeat(pad)),
                fg: Some(match (item.enabled, item.danger) {
                    (false, _) => p.muted,
                    (true, true) => p.error,
                    (true, false) => p.text,
                }),
                bg: fill,
                bold: chosen,
            };
            let hint = Seg {
                text: format!("{}  ", item.hint),
                fg: Some(if item.enabled { p.muted } else { p.dim }),
                bg: fill,
                bold: false,
            };
            row.width(Length::Cells(inner as f32)).set_text(content(&[lead, label, hint], p.text));
        }
    }
}

#[cfg(test)]
#[path = "menu_tests.rs"]
mod tests;
