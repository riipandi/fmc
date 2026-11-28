//! Images in the rendered Markdown view: a paragraph that is one image is drawn as a box, not as its alt text. This module
//! reads an image's source, names the fence a lone image is written as, and says where a destination lives. It does no
//! I/O: the reader's header reads and the canonical paths are the services' (`services/picture.rs`).

use std::path::{Component, Path, PathBuf};

use super::image::is_image_path;

/// The info string of the fence a lone image is written as. The renderer is registered for it (`ui/markdown.rs`), and
/// the fence's info string carries the image, so the renderer reads it back with `from_info`.
pub const FENCE: &str = "fmc-image";

/// An image as the Markdown wrote it: its alt text and its destination, as written (not percent-decoded).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub alt: String,
    pub target: String,
}

/// The image `text` is, when it is `![alt](target)` or `![alt](target "title")` and nothing else. A sentence, a link, or
/// an HTML `<img>` is not an image here, so it keeps its text.
pub fn parse(text: &str) -> Option<Image> {
    let rest = text.strip_prefix("![")?;
    let close = closing_bracket(rest)?;
    let alt = &rest[..close];
    let after = rest[close + 1..].strip_prefix('(')?;
    let end = closing_paren(after)?;
    if end + 1 != after.len() {
        return None;
    }
    let inner = after[..end].trim();
    let (target, title) = match inner.strip_prefix('<') {
        Some(angled) => angled.split_once('>')?,
        None => inner.split_once(char::is_whitespace).unwrap_or((inner, "")),
    };
    if target.is_empty() || !is_title(title.trim()) {
        return None;
    }
    Some(Image { alt: alt.to_owned(), target: target.to_owned() })
}

/// The index of the `]` that closes the alt text: brackets nest, and a backslash escapes the next character.
fn closing_bracket(text: &str) -> Option<usize> {
    let (mut depth, mut escaped) = (0usize, false);
    for (at, c) in text.char_indices() {
        if std::mem::take(&mut escaped) {
            continue;
        }
        match c {
            '\\' => escaped = true,
            '[' => depth += 1,
            ']' if depth == 0 => return Some(at),
            ']' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// The index of the `)` that closes the destination: parentheses in a destination must balance, as CommonMark has them.
fn closing_paren(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (at, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(at),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// A title is empty, or quoted with `"`, `'`, or parentheses; anything else after the destination is not an image.
fn is_title(title: &str) -> bool {
    match (title.chars().next(), title.chars().last()) {
        (None, _) => true,
        (Some('"'), Some('"')) | (Some('\''), Some('\'')) => title.len() >= 2,
        (Some('('), Some(')')) => title.len() >= 2,
        _ => false,
    }
}

/// The image a fence's info string carries: the `FENCE` word, then the image's own text. None for any other fence.
pub fn from_info(info: &str) -> Option<Image> {
    let (word, line) = info.trim().split_once(char::is_whitespace)?;
    (word == FENCE).then(|| parse(line.trim())).flatten()
}

/// The opening fence line for a lone image's line: the image moves into the info string, so the fence takes no line
/// of its own. The line's indentation goes, since a fence opens at the margin.
pub fn opening(line: &str) -> String {
    format!("```{FENCE} {}", line.trim())
}

impl Image {
    /// The box's name: the alt text, or the file's name when the alt text is empty (D5).
    pub fn label(&self) -> String {
        if self.alt.trim().is_empty() {
            let decoded = decoded(&self.target);
            decoded.rsplit(['/', '\\']).next().unwrap_or_default().to_owned()
        } else {
            self.alt.trim().to_owned()
        }
    }
}

/// Where an image's file is, from its destination as written: a local file the reader may open, or a source it cannot
/// show (D3, D2). A remote source and an SVG draw their box with the reason; `Other` is any other non-image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// A local path, percent-decoded, relative to the Markdown file's folder (or absolute).
    Local(String),
    Remote,
    Svg,
    Other,
}

pub fn classify(target: &str) -> Source {
    let lower = target.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("//") {
        return Source::Remote;
    }
    if has_scheme(target) {
        return Source::Other;
    }
    let path = decoded(target);
    if Path::new(&path).extension().is_some_and(|extension| extension.eq_ignore_ascii_case("svg")) {
        return Source::Svg;
    }
    if !is_image_path(Path::new(&path)) {
        return Source::Other;
    }
    Source::Local(path)
}

/// `scheme:` at the start of a destination, such as `data:` or `file:`. A colon after a slash is a path.
fn has_scheme(target: &str) -> bool {
    target.split_once(':').is_some_and(|(scheme, _)| {
        scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

/// The lexical path of a local destination under `folder`, when it stays under `root`; None when it climbs out of the
/// root or the folder names a place outside it. A symbolic link is checked again on the disk by the reader.
pub fn inside(root: &Path, folder: &Path, target: &str) -> Option<PathBuf> {
    let resolved = lexical(&folder.join(decoded(target)));
    resolved.starts_with(root).then_some(resolved)
}

/// The path with `.` removed and each `..` taking back one name.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// The destination with its percent-escapes read as bytes (`Marauder%20map.png` is `Marauder map.png`). An escape that is
/// not two hex digits stays as written.
pub fn decoded(target: &str) -> String {
    let bytes = target.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let escaped = (bytes[at] == b'%')
            .then(|| bytes.get(at + 1..at + 3))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(byte) => {
                out.push(byte);
                at += 3;
            }
            None => {
                out.push(bytes[at]);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
#[path = "images_tests.rs"]
mod tests;
