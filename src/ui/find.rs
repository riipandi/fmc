//! The finder's prompt bar (Scouter `src/find/file-finder.ts`, `promptLine` and `bar`): `/ <query>▎  <status>` over a
//! hairline, in the place of the sidebar's header while the finder is open.

use fmc::line_edit::Line;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Measure, Seg, color, content, fit_end, fit_start};

/// An open prompt's line: its glyph, the query with its caret, and the status after it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptLine {
    pub glyph: &'static str,
    pub line: Line,
    pub status: String,
}

impl PromptLine {
    pub fn new(glyph: &'static str, line: &Line, status: &str) -> Self {
        Self { glyph, line: line.clone(), status: status.to_owned() }
    }

    /// The prompt in `room` columns: see [`prompt_segments`].
    pub fn segments(&self, room: usize, p: &Palette, measure: Measure<'_>) -> Vec<Seg> {
        prompt_segments(self.glyph, &self.line, &self.status, room, p, measure)
    }
}

/// The prompt line: the glyph in accent, the query with the caret in accent at its place, and the status in dim. The
/// query takes `room` columns for the glyph, the query, and the caret; a query longer than that shows the part around
/// the caret, so the caret stays on screen. The status follows, and the bar clips it when it is too long.
pub fn prompt_segments(
    glyph: &str,
    line: &Line,
    status: &str,
    room: usize,
    p: &Palette,
    measure: Measure<'_>,
) -> Vec<Seg> {
    let head = format!("{glyph} ");
    let (before, after) = around_caret(line, room.saturating_sub(measure(&head) + 1), measure);
    vec![
        Seg::new(head, p.accent).bold(),
        Seg::new(before, p.text),
        Seg::new("▎", p.accent),
        Seg::new(after, p.text),
        Seg::new(format!("  {status}"), p.dim),
    ]
}

/// The text before and after the caret, cut to `room` columns. The end of the text before the caret and the start of
/// the text after it are kept, and each cut shows `…`.
fn around_caret(line: &Line, room: usize, measure: Measure<'_>) -> (String, String) {
    if measure(line.before()) + measure(line.after()) <= room {
        return (line.before().to_owned(), line.after().to_owned());
    }
    let before = fit_end(line.before(), room, measure);
    let after = fit_start(line.after(), room.saturating_sub(measure(&before)), measure);
    (before, after)
}

pub struct FindBar {
    pub root: View,
    text: Text,
}

impl FindBar {
    /// The columns the prompt may take in a sidebar of `width`: the bar's padding is one column on each side.
    pub fn room(width: usize) -> usize {
        width.saturating_sub(2)
    }

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

    fn chars(text: &str) -> usize {
        text.chars().count()
    }

    fn drawn(segments: &[Seg]) -> String {
        segments.iter().map(|seg| seg.text.as_str()).collect()
    }

    #[test]
    fn the_prompt_reads_glyph_query_caret_then_status() {
        let p = &fmc::theme::DARK;
        let line = Line::with_text("ledger");
        assert_eq!(drawn(&prompt_segments("/", &line, "1 match", 80, p, &chars)), "/ ledger▎  1 match");
    }

    #[test]
    fn the_caret_sits_where_the_caret_is() {
        let p = &fmc::theme::DARK;
        let mut line = Line::with_text("ledger");
        line.left();
        line.left();
        assert_eq!(drawn(&prompt_segments("/", &line, "", 80, p, &chars)), "/ ledg▎er  ");
    }

    #[test]
    fn a_query_wider_than_the_room_keeps_the_caret_on_screen() {
        let p = &fmc::theme::DARK;
        // Twelve columns: the glyph and its space take two, the caret one, and the query nine.
        let end = drawn(&prompt_segments("/", &Line::with_text("Hermione Granger — Gringotts"), "", 12, p, &chars));
        assert_eq!(end, "/ …ringotts▎  ", "the end of the text shows when the caret is at the end");
        let mut line = Line::with_text("Hermione Granger — Gringotts");
        for _ in 0..12 {
            line.left();
        }
        let middle = drawn(&prompt_segments("/", &line, "", 12, p, &chars));
        assert_eq!(middle, "/ … Granger▎  ", "the caret stays in view when it is in the middle");
        assert_eq!(chars(&middle), 14, "the status spaces follow the caret");
    }
}
