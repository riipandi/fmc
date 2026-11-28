//! Mermaid fences in rendered Markdown: the engine's box diagram at the reader's width, or the fence's source in a frame
//! with the reason it was not drawn. Scouter has no Mermaid and shows the fence as plain code; fmc draws it as boxes.
//! The rows are plain text; `ui/mermaid.rs`
//! colours and places them.

use mermaid_text::{Error, RenderOptions};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Why a fence is shown as its source, as one line of text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fallback {
    pub reason: String,
}

/// The diagram's rows. `Some(width)` is a hard budget: a diagram wider than it is a fallback, not a clipped one. `None`
/// is the diagram's natural width.
pub fn render(source: &str, width: Option<usize>) -> Result<Vec<String>, Fallback> {
    let options = RenderOptions { max_width: width, max_width_strict: true, ..RenderOptions::default() };
    match mermaid_text::render_with_options(source, &options) {
        Ok(text) => Ok(text.lines().map(str::to_owned).collect()),
        Err(error) => Err(Fallback { reason: reason(error) }),
    }
}

fn reason(error: Error) -> String {
    match error {
        Error::EmptyInput => "empty diagram".to_owned(),
        Error::UnsupportedDiagram(kind) => format!("unsupported diagram: {kind}"),
        Error::ParseError(message) => format!("parse error: {message}"),
        Error::TooWide { requested, actual } => format!("too wide: {actual} columns, the reader has {requested}"),
    }
}

/// The source in a frame of `│` and `─`, then the reason on its own row. Source lines wrap to the frame's inner width;
/// `width` bounds the frame, and `None` keeps every line whole. Fewer than five columns leave no room for the frame.
pub fn framed(source: &str, reason: &str, width: Option<usize>) -> Vec<String> {
    let source = source_text(source);
    let inner = match width {
        Some(width) => width.saturating_sub(4).min(source.lines().map(UnicodeWidthStr::width).max().unwrap_or(0)),
        None => source.lines().map(UnicodeWidthStr::width).max().unwrap_or(0),
    };
    if inner == 0 {
        return Vec::new();
    }
    let body: Vec<String> = source.lines().flat_map(|line| wrap(line, inner)).collect();
    let mut rows = vec![format!("┌{}┐", "─".repeat(inner + 2))];
    for line in body {
        let pad = inner.saturating_sub(line.width());
        rows.push(format!("│ {line}{} │", " ".repeat(pad)));
    }
    rows.push(format!("└{}┘", "─".repeat(inner + 2)));
    rows.push(fit(reason, width.unwrap_or(inner + 4)));
    rows
}

/// The widest row in columns, for a measured widget.
pub fn columns(rows: &[String]) -> usize {
    rows.iter().map(|row| row.width()).max().unwrap_or(0)
}

/// Display text with tabs as four spaces: a tab's width is the terminal's, not one the frame can count.
fn source_text(source: &str) -> String {
    crate::text::display_text(source).replace('\t', "    ")
}

/// Splits `line` into chunks of at most `columns` display columns, breaking after a space where one fits so words stay
/// whole; a word wider than the frame is cut where it overflows. An empty line is one empty chunk.
fn wrap(line: &str, columns: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let (mut chunk, mut used) = (String::new(), 0);
    // Byte offset in `chunk` just past its last space, and the columns up to it: where the chunk can break.
    let mut space: Option<(usize, usize)> = None;
    for c in line.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > columns && used > 0 {
            match space.take() {
                // Break after the last space, unless that space is all the chunk holds (leading indentation).
                Some((at, before)) if chunk[..at].trim() != "" => {
                    let rest = chunk.split_off(at);
                    chunks.push(std::mem::replace(&mut chunk, rest));
                    used -= before;
                }
                _ => {
                    chunks.push(std::mem::take(&mut chunk));
                    used = 0;
                }
            }
        }
        chunk.push(c);
        used += w;
        if c == ' ' {
            space = Some((chunk.len(), used));
        }
    }
    chunks.push(chunk);
    chunks
}

/// `text` cut to `columns`, with `…` when cut.
pub fn fit(text: &str, columns: usize) -> String {
    if columns == 0 {
        return String::new();
    }
    if text.width() <= columns {
        return text.to_owned();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > columns - 1 {
            break;
        }
        out.push(c);
        used += w;
    }
    out.push('…');
    out
}

#[cfg(test)]
#[path = "mermaid_tests.rs"]
mod tests;
