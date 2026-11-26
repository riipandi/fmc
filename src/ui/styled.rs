//! Styled text segments and fitting to a width (Scouter `src/ui/styled.ts`). Widths come from a measure function,
//! so the logic runs in tests without a terminal.

use opentui::prelude::*;

pub use fmc::text::Seg;

/// Display width of a string in cells.
pub type Measure<'a> = &'a dyn Fn(&str) -> usize;

pub fn color(hex: &str) -> Color {
    Color::parse(hex).unwrap_or(Color::WHITE)
}

/// Spans for a `Text`; segments without text are skipped, a missing color is `text`.
pub fn content(segments: &[Seg], text: &'static str) -> Content {
    let spans: Vec<Span<'static>> = segments
        .iter()
        .filter(|s| !s.text.is_empty())
        .map(|s| {
            let mut span = Span::raw(s.text.clone()).fg(color(s.fg.unwrap_or(text)));
            if let Some(bg) = s.bg {
                span = span.bg(color(bg));
            }
            if s.bold { span.bold() } else { span }
        })
        .collect();
    spans.into()
}

pub fn width(segments: &[Seg], measure: Measure<'_>) -> usize {
    segments.iter().map(|s| measure(&s.text)).sum()
}

/// Keeps the end of `text`, with `…` in front when cut.
pub fn fit_end(text: &str, columns: usize, measure: Measure<'_>) -> String {
    if columns == 0 {
        return String::new();
    }
    if measure(text) <= columns {
        return text.to_string();
    }
    let mut tail = text;
    while !tail.is_empty() && measure(tail) > columns - 1 {
        tail = &tail[tail.chars().next().map_or(0, char::len_utf8)..];
    }
    format!("…{tail}")
}

/// Keeps the start of `text`, with `…` after it when cut.
pub fn fit_start(text: &str, columns: usize, measure: Measure<'_>) -> String {
    if columns == 0 {
        return String::new();
    }
    if measure(text) <= columns {
        return text.to_string();
    }
    let mut head = text;
    while !head.is_empty() && measure(head) > columns - 1 {
        head = &head[..head.len() - head.chars().last().map_or(0, char::len_utf8)];
    }
    format!("{head}…")
}
