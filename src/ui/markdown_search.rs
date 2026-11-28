//! Search marks in the rendered view and the text index they count. OpenTUI's inline
//! hook sees spans, not blocks, so the hook and the code renderers count their calls in document order: a call's
//! number is its slot, and the index names each slot's block. The index walks the blocks in the same order, one at a
//! time, through an offscreen Markdown widget that is never shown.

use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::sync::Arc;

use fmc::reader::headings;
use fmc::reader::rendered_search::{Piece, Rendered, Slot};
use fmc::reader::search::match_ranges;
use opentui::native::LayerRun;
use opentui::prelude::*;
use opentui::style::{Span, Style};
use opentui::syntax::SyntaxTheme;
use opentui::widget::{Code, CodeBlock, InlineSpan, Markdown, View, WrapMode};

use super::markdown_blocks;
use super::markdown_images;
use super::syntax;

/// The layer the marks go on, over the syntax layer (1), as the code view's search layer is (`ui/reader.rs`).
pub const MARK_LAYER: u32 = 2;

/// The styles of a mark: every match, and the match the view is on (`target_bg`). A palette change rebuilds them.
pub struct Styles {
    pub text: Color,
    pub theme: Arc<SyntaxTheme>,
    pub matched: Style,
    pub current: Style,
}

/// The search the marks follow: its query, and the occurrence the view is on, as its slot and its byte range in the
/// slot's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    pub query: String,
    pub current: Option<(Slot, Range<usize>)>,
}

/// What the hook and the renderers draw, shared by the view's widgets. The counters number the slots of one build.
#[derive(Default)]
pub struct Paint {
    pub mark: RefCell<Option<Mark>>,
    pub styles: RefCell<Option<Styles>>,
    prose: Cell<usize>,
    code: Cell<usize>,
}

impl Paint {
    /// Starts a build's numbering; every build that marks counts from zero.
    pub fn restart(&self) {
        self.prose.set(0);
        self.code.set(0);
    }
}

/// What the offscreen index saw OpenTUI build, in order.
pub enum Captured {
    Prose(String),
    Code(String),
}

/// The text of a span run: what the reader draws, without its markers.
fn text_of(spans: &[InlineSpan]) -> String {
    spans.iter().map(|span| span.span.content.as_ref()).collect()
}

/// The inline hook of the shown view: marks the occurrences of the query in a paragraph, heading, or table cell.
pub fn paint_prose(spans: Vec<InlineSpan>, paint: &Paint) -> Vec<InlineSpan> {
    let slot = paint.prose.get();
    paint.prose.set(slot + 1);
    let mark = paint.mark.borrow();
    let styles = paint.styles.borrow();
    let (Some(mark), Some(styles)) = (mark.as_ref(), styles.as_ref()) else { return spans };
    let ranges = match_ranges(&text_of(&spans), &mark.query);
    if ranges.is_empty() {
        return spans;
    }
    let current = mark.current.as_ref().filter(|(of, _)| *of == Slot::Prose(slot)).map(|(_, at)| at);
    split(spans, &ranges, current, styles)
}

/// Splits the spans at the ranges' edges, and gives each piece inside a range the match style, the current one's style
/// for the range the view is on. A match across spans, such as `**Harry** Potter`, marks each of its pieces.
fn split(
    spans: Vec<InlineSpan>,
    ranges: &[Range<usize>],
    current: Option<&Range<usize>>,
    styles: &Styles,
) -> Vec<InlineSpan> {
    let mut out = Vec::new();
    let mut offset = 0;
    for inline in spans {
        let content = inline.span.content.to_string();
        let mut points = vec![0, content.len()];
        for edge in ranges.iter().flat_map(|range| [range.start, range.end]) {
            if edge > offset && edge < offset + content.len() {
                points.push(edge - offset);
            }
        }
        points.sort_unstable();
        points.dedup();
        for pair in points.windows(2) {
            let (from, to) = (pair[0], pair[1]);
            let (start, end) = (offset + from, offset + to);
            let style = match ranges.iter().find(|range| range.start <= start && end <= range.end) {
                Some(range) if current == Some(range) => styles.current,
                Some(_) => styles.matched,
                None => inline.span.style,
            };
            let mut piece = InlineSpan::from(Span::styled(content[from..to].to_owned(), style));
            piece.span.link = inline.span.link.clone();
            piece.code = inline.code;
            piece.target = inline.target.clone();
            out.push(piece);
        }
        offset += content.len();
    }
    out
}

/// The code renderer of a highlighted language, for the shown view: a fence with a match draws its code with the marks,
/// and any other fence keeps the default block (`None`), so its look does not change.
pub fn paint_code(block: &CodeBlock<'_>, paint: &Paint) -> Option<AnyWidget> {
    let slot = count_code(paint);
    let mark = paint.mark.borrow();
    let mark = mark.as_ref()?;
    let styles = paint.styles.borrow();
    let styles = styles.as_ref()?;
    let ranges = match_ranges(block.code, &mark.query);
    if ranges.is_empty() {
        return None;
    }
    let current = mark.current.as_ref().filter(|(slot_of, _)| *slot_of == Slot::Code(slot)).map(|(_, at)| at);
    let code = Code::new(block.app)
        .syntax_theme(Arc::clone(&styles.theme))
        .fg(styles.text)
        .filetype(block.language)
        .set_code(block.code)
        .wrap(WrapMode::Word)
        .focusable(false);
    let buffer = code.buffer();
    let context = buffer.context();
    let (Ok(matched), Ok(current_group)) =
        (context.group_set(&["opentui.search.match"]), context.group_set(&["opentui.search.current"]))
    else {
        return None;
    };
    if let Ok(table) = syntax::table(context, &styles.theme, styles.matched, styles.current) {
        let _ = buffer.set_syntax_style(Some(&table));
    }
    let runs: Vec<LayerRun> = ranges
        .iter()
        .map(|range| {
            let group_set = if current == Some(range) { current_group } else { matched };
            LayerRun { start_byte: range.start as u32, end_byte: range.end as u32, group_set }
        })
        .collect();
    let _ = buffer.set_layer(MARK_LAYER, 0..u32::MAX, &runs);
    Some(View::new(block.app).bg(Color::TRANSPARENT).child(&code).as_any())
}

/// Numbers one code block, drawn by a renderer or not. Every renderer calls it, the diagram renderer included, so the
/// numbers match the index's (`Slot::Code`).
pub fn count_code(paint: &Paint) -> usize {
    let slot = paint.code.get();
    paint.code.set(slot + 1);
    slot
}

/// The offscreen index's hook: records a paragraph, heading, or table cell, and draws it unchanged.
pub fn capture_prose(spans: Vec<InlineSpan>, captured: &RefCell<Vec<Captured>>) -> Vec<InlineSpan> {
    captured.borrow_mut().push(Captured::Prose(text_of(&spans)));
    spans
}

/// The offscreen index's code renderer: records a code block, and keeps the default block. A diagram has no text the
/// search can count, so it records nothing.
pub fn capture_code(block: &CodeBlock<'_>, captured: &RefCell<Vec<Captured>>) -> Option<AnyWidget> {
    let text = if block.language == "mermaid" { String::new() } else { block.code.to_owned() };
    captured.borrow_mut().push(Captured::Code(text));
    None
}

/// The index of `source`: each top-level block's rendered text, in document order. The blocks are built one at a time,
/// with the link definitions above each, so a reference resolves as it does in the whole document. A block with no
/// captured text that is a code or HTML block is counted as plain text, and it is not marked.
pub fn index(offscreen: &Markdown, captured: &RefCell<Vec<Captured>>, source: &str) -> Rendered {
    // The view draws the source as `markdown_images::prepared` gives it, so the blocks and the lines are read from the
    // same text.
    let source = markdown_images::prepared(source);
    let split = markdown_blocks::split(&source);
    let shown = headings::with_markers(&source);
    let lines: Vec<&str> = shown.split('\n').collect();
    let definitions: String =
        split.definitions.iter().flat_map(|range| lines_of(&lines, range)).collect::<Vec<_>>().join("\n");
    let (mut prose, mut code) = (0, 0);
    let mut pieces = Vec::new();
    for (block, lines_of_block) in split.blocks.iter().enumerate().map(|(index, block)| (index, &block.lines)) {
        let body = lines_of(&lines, lines_of_block).join("\n");
        let text = if definitions.is_empty() { body } else { format!("{definitions}\n\n{body}") };
        offscreen.set_markdown("");
        captured.borrow_mut().clear();
        offscreen.set_markdown(text);
        let seen = std::mem::take(&mut *captured.borrow_mut());
        if seen.is_empty() {
            let kind = split.blocks[block].kind.as_str();
            if matches!(kind, "fenced_code_block" | "indented_code_block" | "html_block") {
                let text = plain(kind, &lines_of(&lines, lines_of_block));
                pieces.push(Piece { block, slot: None, text });
            }
            continue;
        }
        for seen in seen {
            match seen {
                Captured::Prose(text) => {
                    pieces.push(Piece { block, slot: Some(Slot::Prose(prose)), text });
                    prose += 1;
                }
                Captured::Code(text) => {
                    pieces.push(Piece { block, slot: Some(Slot::Code(code)), text });
                    code += 1;
                }
            }
        }
    }
    Rendered { pieces }
}

fn lines_of<'a>(lines: &[&'a str], range: &Range<usize>) -> Vec<&'a str> {
    lines.get(range.clone()).map_or_else(Vec::new, <[&str]>::to_vec)
}

/// The text a block shows when OpenTUI draws it with no renderer: a fence's body, an indented block without its
/// indent, or an HTML block as written.
fn plain(kind: &str, lines: &[&str]) -> String {
    match kind {
        "fenced_code_block" => {
            let body = lines.get(1..).unwrap_or_default();
            let closed = body
                .last()
                .is_some_and(|line| line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~"));
            let body = if closed { &body[..body.len() - 1] } else { body };
            body.join("\n")
        }
        "indented_code_block" => lines
            .iter()
            .map(|line| line.strip_prefix("    ").or_else(|| line.strip_prefix('\t')).unwrap_or(line))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => lines.join("\n"),
    }
}

#[cfg(test)]
#[path = "markdown_search_tests.rs"]
mod tests;
