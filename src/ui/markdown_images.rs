//! Lone image paragraphs, written as box fences before the view reads the source (`reader/images.rs`). A paragraph that
//! is one image and nothing else opens a fence on its own line, the image moves into the fence's info string, and the
//! blank line after it closes the fence. Each line keeps its number, so the git signs and the search keep their rows.
//! A paragraph at the end of the file needs no closing line: an open fence runs to the end.

use std::borrow::Cow;

use fmc::reader::frontmatter;
use fmc::reader::images;

use super::markdown_blocks;

/// The source as the view draws it: front matter as a code fence (`reader/frontmatter.rs`), then each lone image as a box.
pub fn prepared(source: &str) -> String {
    as_boxes(&frontmatter::as_fence(source)).into_owned()
}

/// `source` with each top-level paragraph that is one image rewritten as a box fence, line for line. A paragraph whose
/// line has a backtick is left alone, since a backtick cannot sit in a fence's info string; so is one with no blank line
/// after it, since the fence would have no line to close on.
pub fn as_boxes(source: &str) -> Cow<'_, str> {
    if !source.contains("![") {
        return Cow::Borrowed(source);
    }
    let split = markdown_blocks::split(source);
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    let mut changed = false;
    for block in split.blocks.iter().filter(|block| block.kind == "paragraph" && block.lines.len() == 1) {
        let at = block.lines.start;
        let text = lines[at].trim_end_matches(['\r', '\n']);
        if text.contains('`') || images::parse(text.trim()).is_none() {
            continue;
        }
        let after = at + 1;
        match lines.get(after) {
            None => {}
            Some(line) if line.trim().is_empty() => out[after] = format!("```{}", ending(line)),
            Some(_) => continue,
        }
        out[at] = format!("{}{}", images::opening(text), ending(lines[at]));
        changed = true;
    }
    if changed { Cow::Owned(out.concat()) } else { Cow::Borrowed(source) }
}

/// The line terminator of `line`: `\r\n`, `\n`, or nothing for a last line with none.
fn ending(line: &str) -> &str {
    &line[line.trim_end_matches(['\r', '\n']).len()..]
}

#[cfg(test)]
#[path = "markdown_images_tests.rs"]
mod tests;
