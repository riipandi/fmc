//! The `?` card (Scouter `src/ui/help.ts`): every key section, and the Mouse section too (approved deviation,
//! `architecture.md`), centred over the panes on the footer's tint.

use fmc::keys::{KEY_COLUMN, KEY_SECTIONS, KeyRow, MOUSE_ROWS};
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};

fn sections() -> impl Iterator<Item = (&'static str, &'static [KeyRow])> {
    KEY_SECTIONS.iter().copied().chain([("Mouse", MOUSE_ROWS)])
}

/// Rows inside the card: a title and its rows per section, with a blank row between sections.
pub fn list_rows() -> usize {
    let sections: Vec<_> = sections().collect();
    sections.iter().map(|(_, rows)| 1 + rows.len()).sum::<usize>() + sections.len() - 1
}

/// The card's top margin that centres it in `height` rows, as Scouter's `centre` (two rows of border).
pub fn top_margin(height: usize) -> usize {
    height.saturating_sub(list_rows() + 2) / 2
}

pub struct Help {
    pub overlay: View,
    card: View,
}

impl Help {
    pub fn new(app: &App, p: &Palette, background: &str) -> Self {
        let bg = color(background);
        let overlay = View::new(app)
            .absolute()
            .position(Edge::Top, 0)
            .position(Edge::Left, 0)
            .position(Edge::Right, 0)
            .position(Edge::Bottom, 2)
            .row()
            .align_items(Align::Start)
            .justify(Justify::Center)
            .overflow(Overflow::Hidden)
            .bg(bg);
        let card = View::new(app)
            .column()
            .shrink(0.0)
            .border(BorderStyle::Rounded)
            .border_color(color(p.accent))
            .title(" Keyboard Shortcuts ")
            .title_alignment(Alignment::Center)
            .title_color(color(p.accent_soft))
            .padding_x(2)
            .bg(bg);
        let line = |segments: Vec<Seg>| {
            Text::new(app, content(&segments, p.text)).height(1).shrink(0.0).wrap(WrapMode::None).selectable(false)
        };
        let count = sections().count();
        for (index, (title, rows)) in sections().enumerate() {
            card.add(&line(vec![Seg::new(title, p.muted).bold()]));
            for (keys, action) in rows {
                card.add(&line(vec![
                    Seg::new(format!("  {keys:<KEY_COLUMN$}"), p.accent_soft),
                    Seg::new(*action, p.text),
                ]));
            }
            if index + 1 < count {
                card.add(&line(vec![]));
            }
        }
        overlay.add(&card);
        Self { overlay, card }
    }

    /// Re-centres the card for a terminal `rows` high (the overlay leaves the two footer rows).
    pub fn centre(&self, rows: usize) {
        self.card.margin_edge(Edge::Top, top_margin(rows.saturating_sub(2)) as u32);
    }
}
