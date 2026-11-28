//! Lone image paragraphs, written as box fences before the view reads the source (`reader/images.rs`). A paragraph that
//! is one image and nothing else opens a fence on its own line, the image moves into the fence's info string, and the
//! blank line after it closes the fence. Each line keeps its number, so the git signs and the search keep their rows.
//! A paragraph at the end of the file needs no closing line: an open fence runs to the end.

use std::borrow::Cow;
use std::ops::Range;

use fmc::reader::frontmatter;
use fmc::reader::images::{self, Image};

use super::markdown_blocks;
use super::markdown_blocks::Split;

/// The source as the view draws it: front matter as a code fence (`reader/frontmatter.rs`), then each lone image as a box.
pub fn prepared(source: &str) -> String {
    as_boxes(&frontmatter::as_fence(source)).into_owned()
}

/// The boxes of `prepared` (`prepared` gives its `split`): each box's block index and its image, in document order.
pub fn boxes(prepared: &str, split: &Split) -> Vec<(usize, Image)> {
    let lines: Vec<&str> = prepared.split('\n').collect();
    split
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.kind == "fenced_code_block")
        .filter_map(|(index, block)| {
            let opening = lines.get(block.lines.start)?;
            let info = opening.trim_start().strip_prefix("```")?;
            images::from_info(info).map(|image| (index, image))
        })
        .collect()
}

/// The destinations of the boxes in `source`, each once: what the headers are read for, a `<picture>`'s dark source too.
pub fn targets(source: &str) -> Vec<String> {
    let prepared = prepared(source);
    let mut targets: Vec<String> = boxes(&prepared, &markdown_blocks::split(&prepared))
        .into_iter()
        .flat_map(|(_, image)| std::iter::once(image.target).chain(image.dark))
        .collect();
    targets.sort();
    targets.dedup();
    targets
}

/// `source` with each top-level image written as a box fence, line for line: a paragraph that is one image, and an HTML
/// block that is one image (`images::parse_html`). A source whose line has a backtick is left alone, since a backtick cannot
/// sit in a fence's info string. A paragraph, or a one-line HTML block, closes on the blank line after it, so one with no
/// blank line after it keeps its text; a longer HTML block closes on its own last line.
pub fn as_boxes(source: &str) -> Cow<'_, str> {
    if !source.contains("![") && !source.contains('<') {
        return Cow::Borrowed(source);
    }
    let split = markdown_blocks::split(source);
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    let mut changed = false;
    for block in &split.blocks {
        let fence = match block.kind.as_str() {
            "paragraph" if block.lines.len() == 1 => lone_paragraph(&lines, block.lines.start),
            "html_block" => html_block(&lines, block.lines.clone()),
            _ => None,
        };
        let Some(Fence { opening, closing }) = fence else { continue };
        out[block.lines.start] = opening;
        if let Some((at, line)) = closing {
            out[at] = line;
        }
        changed = true;
    }
    if changed { Cow::Owned(out.concat()) } else { Cow::Borrowed(source) }
}

/// The lines that replace a box's source: the opening line takes the source, and the closing line, when there is one,
/// replaces a line of the source or the blank line after it.
struct Fence {
    opening: String,
    closing: Option<(usize, String)>,
}

fn lone_paragraph(lines: &[&str], at: usize) -> Option<Fence> {
    let text = lines[at].trim_end_matches(['\r', '\n']);
    if text.contains('`') || images::parse(text.trim()).is_none() {
        return None;
    }
    let closing = after_one_line(lines, at)?;
    Some(Fence { opening: format!("{}{}", images::opening(text), ending(lines[at])), closing })
}

/// An HTML block that is one image. Its lines join into the opening line, so the source is whole in the info string; the
/// lines between the first and the last stay as the fence's body, and the last line closes it.
fn html_block(lines: &[&str], range: Range<usize>) -> Option<Fence> {
    // tree-sitter-md takes the blank line after an HTML block into the block; the source ends at its last text line.
    let end = (range.start..range.end).rev().find(|&at| !lines[at].trim().is_empty()).map_or(range.start, |at| at + 1);
    let range = range.start..end;
    let text = lines[range.clone()].iter().map(|line| line.trim()).collect::<Vec<_>>().join(" ");
    if text.contains('`') || images::parse_html(&text).is_none() {
        return None;
    }
    let (first, last) = (range.start, range.end - 1);
    let closing =
        if first == last { after_one_line(lines, first)? } else { Some((last, format!("```{}", ending(lines[last])))) };
    Some(Fence { opening: format!("{}{}", images::opening(&text), ending(lines[first])), closing })
}

/// The closing line of a source that is one line at `at`: the blank line after it, or none at the end of the file (an open
/// fence runs to the end). Any other line after it means the source is no box; the outer None says so.
fn after_one_line(lines: &[&str], at: usize) -> Option<Option<(usize, String)>> {
    match lines.get(at + 1) {
        None => Some(None),
        Some(line) if line.trim().is_empty() => Some(Some((at + 1, format!("```{}", ending(line))))),
        Some(_) => None,
    }
}

/// The line terminator of `line`: `\r\n`, `\n`, or nothing for a last line with none.
fn ending(line: &str) -> &str {
    &line[line.trim_end_matches(['\r', '\n']).len()..]
}

#[cfg(test)]
#[path = "markdown_images_tests.rs"]
mod tests;
