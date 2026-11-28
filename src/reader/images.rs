//! Images in the rendered Markdown view: a paragraph that is one image, or an HTML block that is one `<img>` or `<picture>`,
//! is drawn as a box, not as its text. This module reads an image's source, names the fence an image is written as, and says
//! where a destination lives. It does no I/O: the reader's header reads and the canonical paths are the services'
//! (`services/picture.rs`).

use std::path::{Component, Path, PathBuf};

use unicode_width::UnicodeWidthStr;

use super::html::{self, Tag};
use super::image::{self, is_image_path};
use super::mermaid::fit;

/// The info string of the fence a lone image is written as. The renderer is registered for it (`ui/markdown.rs`), and
/// the fence's info string carries the image, so the renderer reads it back with `from_info`.
pub const FENCE: &str = "fmc-image";

/// An image as the source wrote it: its alt text and its destination, as written (not percent-decoded). An HTML image
/// adds its width, whether it is centred, and a `<picture>`'s dark-mode source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Image {
    pub alt: String,
    pub target: String,
    /// A `<picture>`'s source for the dark theme: the first candidate of its `srcset`, when its media is the dark scheme.
    pub dark: Option<String>,
    /// The `width` attribute in pixels; a percentage or none leaves it to the image.
    pub width: Option<u32>,
    /// `align="center"` on the image or a wrapper, or a `<center>` wrapper.
    pub center: bool,
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
    Some(Image { alt: alt.to_owned(), target: target.to_owned(), ..Image::default() })
}

/// The HTML elements an image block may hold around its `<img>` or `<picture>`: headings, paragraphs, divs, links, and
/// centres. Any other element, or text, keeps the block as text.
const WRAPPERS: [&str; 9] = ["h1", "h2", "h3", "h4", "h5", "h6", "p", "div", "a"];

/// The image an HTML block is, when it holds exactly one `<img>` with a `src` (inside a `<picture>` when it has one), and
/// nothing else but wrappers and the picture's sources. Text, a comment, another element, or a second image is no image.
pub fn parse_html(text: &str) -> Option<Image> {
    let tags = html::tags(text)?;
    let mut images = tags.iter().filter(|tag| tag.name == "img" && !tag.closing);
    let img = images.next()?;
    if images.next().is_some() {
        return None;
    }
    let target = img.attr("src").map(str::trim).filter(|target| !target.is_empty())?;
    let pictures = tags.iter().filter(|tag| tag.name == "picture" && !tag.closing).count();
    let sources = tags.iter().filter(|tag| tag.name == "source" && !tag.closing).count();
    if pictures > 1 || (pictures == 0 && sources > 0) {
        return None;
    }
    let known = |tag: &Tag| match tag.name.as_str() {
        "img" | "picture" => true,
        "source" => pictures == 1,
        "center" => true,
        name => WRAPPERS.contains(&name),
    };
    if !tags.iter().all(known) {
        return None;
    }
    let dark = tags
        .iter()
        .filter(|tag| tag.name == "source")
        .find(|tag| tag.attr("media").is_some_and(is_dark_scheme))
        .and_then(|tag| tag.attr("srcset")?.split(',').next()?.split_whitespace().next())
        .map(str::to_owned);
    let center = tags.iter().any(|tag| {
        tag.name == "center" || tag.attr("align").is_some_and(|align| align.trim().eq_ignore_ascii_case("center"))
    });
    Some(Image {
        alt: img.attr("alt").unwrap_or_default().to_owned(),
        target: target.to_owned(),
        dark,
        width: img.attr("width").and_then(|width| width.trim().parse().ok()),
        center,
    })
}

/// Whether a `media` value is the dark colour scheme, whatever its spacing or case.
fn is_dark_scheme(media: &str) -> bool {
    media.chars().filter(|c| !c.is_whitespace()).collect::<String>().eq_ignore_ascii_case("(prefers-color-scheme:dark)")
}

/// The image a source names: Markdown's `![alt](target)` line, or an HTML block (`parse_html`).
pub fn parse_source(text: &str) -> Option<Image> {
    if text.trim_start().starts_with('<') { parse_html(text.trim()) } else { parse(text) }
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
    (word == FENCE).then(|| parse_source(line)).flatten()
}

/// Why a box has no picture to open, in the words its last row shows.
pub const REMOTE: &str = "remote";
pub const SVG: &str = "SVG";
pub const NOT_SHOWN: &str = "not an image fmc shows";
pub const OUTSIDE: &str = "outside the folder";
pub const NOT_FOUND: &str = "not found";
pub const UNREADABLE: &str = "could not be read";
pub const TOO_LARGE: &str = "too large (over 10 MiB)";

/// What the reader found for one image, read from the disk on a worker (`services/picture.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Header {
    /// A picture the reader opens: its canonical path, and its size and format.
    Picture { path: PathBuf, info: image::Info },
    /// No picture: the reason, from the constants above.
    Refused(&'static str),
}

/// What a box says about its image: the name, the path with the size and format, and the hint or the reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    pub title: String,
    pub detail: String,
    pub hint: Hint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hint {
    /// The picture opens with Enter.
    Open,
    Reason(&'static str),
    /// No header has come back yet.
    Reading,
}

/// Which colour a box's text takes: the name and an open hint in the text colour, the rest muted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Text,
    Muted,
}

pub fn card(image: &Image, header: Option<&Header>) -> Card {
    let title = format!("▣ {}", image.label());
    match header {
        Some(Header::Picture { info, .. }) => Card {
            title,
            detail: format!("{} · {}×{} · {}", image.target, info.width, info.height, info.format),
            hint: Hint::Open,
        },
        Some(Header::Refused(reason)) => Card { title, detail: image.target.clone(), hint: Hint::Reason(reason) },
        None => Card { title, detail: image.target.clone(), hint: Hint::Reading },
    }
}

/// The three text rows of a box, each cut or padded to `inner` columns, with the tone of its text.
pub fn text_rows(card: &Card, inner: usize) -> [(Tone, String); 3] {
    let hint = match card.hint {
        Hint::Open => (Tone::Text, "Enter opens it".to_owned()),
        Hint::Reason(reason) => (Tone::Muted, reason.to_owned()),
        Hint::Reading => (Tone::Muted, "reading the image…".to_owned()),
    };
    [(Tone::Text, card.title.clone()), (Tone::Muted, card.detail.clone()), hint].map(|(tone, text)| {
        let cut = fit(&text, inner);
        let pad = inner.saturating_sub(cut.width());
        (tone, format!("{cut}{}", " ".repeat(pad)))
    })
}

/// The opening fence line for an image's source: the source moves into the info string, so the fence takes no line of
/// its own. The source's indentation goes, since a fence opens at the margin.
pub fn opening(source: &str) -> String {
    format!("```{FENCE} {}", source.trim())
}

impl Image {
    /// The box's name: the alt text, or the file's name when the alt text is empty.
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
/// show. A remote source and an SVG draw their box with the reason; `Other` is any other non-image.
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
