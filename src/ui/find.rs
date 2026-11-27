//! The finder's prompt bar (Scouter `src/find/file-finder.ts`, `promptLine` and `bar`): `/ <query>▎  <status>` over a
//! hairline, in the place of the sidebar's header while the finder is open.

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};

/// The prompt line: the glyph in accent, the query, the caret, and the status in dim.
pub fn prompt_segments(glyph: &str, query: &str, status: &str, p: &Palette) -> Vec<Seg> {
    vec![
        Seg::new(format!("{glyph} "), p.accent).bold(),
        Seg::new(query, p.text),
        Seg::new("▎", p.accent),
        Seg::new(format!("  {status}"), p.dim),
    ]
}

pub struct FindBar {
    pub root: View,
    text: Text,
}

impl FindBar {
    pub fn new(app: &App) -> Self {
        let root = View::new(app)
            .height(2)
            .width(Length::percent(100.0))
            .row()
            .shrink(0.0)
            .padding_x(1)
            .border_sides(Sides::BOTTOM);
        let text =
            Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
        root.add(&text);
        Self { root, text }
    }

    /// Shows the bar with `segments`, or hides it when the finder is closed.
    pub fn paint(&self, segments: Option<Vec<Seg>>, p: &Palette) {
        match segments {
            Some(segments) => {
                self.root.visible(true);
                self.root.border_color(color(p.border));
                self.text.set_text(content(&segments, p.text));
            }
            None => {
                self.root.visible(false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_reads_glyph_query_caret_then_status() {
        let p = &fmc::theme::DARK;
        let text: String = prompt_segments("/", "ledger", "1 match", p).iter().map(|seg| seg.text.as_str()).collect();
        assert_eq!(text, "/ ledger▎  1 match");
    }
}
