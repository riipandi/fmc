//! Rendered Markdown keeps each heading's ATX marker: `# Title` at level 1 to `###### Title` at 6.
//! The engine conceals a heading's `#` run and its inline hook cannot see the level (`ui/markdown.rs`), so the
//! source is rewritten before the widget reads it: each heading keeps its real ATX run, which stays concealed, and
//! the same run is written as entities (`&#35;&#35;`) into the heading's text, where it shows in the heading's style.
//! Fenced code, indented code, and HTML blocks keep their text. Every line stays a line, so the source line numbers hold.

/// The Markdown source with each heading's marker written into its text.
pub fn with_markers(source: &str) -> String {
    let mut scan = Scan::default();
    let mut out = Vec::new();
    for line in source.split('\n') {
        scan.line(line, &mut out);
    }
    out.join("\n")
}

/// How a paragraph stands before the current line.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Paragraph {
    /// One plain line, with no quote or list marker: a setext underline turns it into a heading.
    One,
    /// Two or more lines, or a line inside a container: a setext underline is left as text (a known limit).
    Many,
}

#[derive(Clone, Copy)]
struct Fence {
    ch: u8,
    len: usize,
}

#[derive(Clone, Copy)]
enum HtmlEnd {
    /// The block ends on the first line that contains this text (lowercase).
    Contains(&'static str),
    /// The block ends before the first blank line.
    Blank,
}

#[derive(Default)]
struct Scan {
    fence: Option<Fence>,
    html: Option<HtmlEnd>,
    paragraph: Option<Paragraph>,
    /// Inside a list item: an indented line is its content, not code.
    list: bool,
}

impl Scan {
    fn line(&mut self, line: &str, out: &mut Vec<String>) {
        if let Some(fence) = self.fence {
            let (at, _, _) = content_start(line);
            if closes(&line[at..], fence) {
                self.fence = None;
            }
            out.push(line.to_owned());
            return;
        }
        if let Some(end) = self.html {
            match end {
                HtmlEnd::Blank if line.trim().is_empty() => self.html = None,
                HtmlEnd::Blank => {
                    out.push(line.to_owned());
                    return;
                }
                HtmlEnd::Contains(text) => {
                    if line.to_ascii_lowercase().contains(text) {
                        self.html = None;
                    }
                    out.push(line.to_owned());
                    return;
                }
            }
        }
        if line.trim().is_empty() {
            self.paragraph = None;
            out.push(line.to_owned());
            return;
        }
        let indented = line.starts_with("    ") || line.starts_with('\t');
        if indented && !self.list {
            // Indented code: a paragraph's continuation takes the same rule, and the line is not a heading either way.
            self.paragraph = self.paragraph.map(|_| Paragraph::Many);
            out.push(line.to_owned());
            return;
        }
        let (at, container, list) = content_start(line);
        if list {
            self.list = true;
        } else if !container && !indented {
            self.list = false;
        }
        let text = &line[at..];
        if let (Some(Paragraph::One), false, Some(level)) = (self.paragraph, container, underline(line)) {
            // The paragraph line above is the heading: its text takes the marker, and the underline becomes blank.
            if let Some(previous) = out.last_mut() {
                *previous = marked("", level, previous.trim());
            }
            out.push(String::new());
            self.paragraph = None;
            return;
        }
        if let Some(fence) = fence_open(text) {
            self.fence = Some(fence);
            self.paragraph = None;
            out.push(line.to_owned());
            return;
        }
        if !container && let Some(end) = html_start(line) {
            let ends_here = matches!(end, HtmlEnd::Contains(stop) if line.to_ascii_lowercase().contains(stop));
            self.html = (!ends_here).then_some(end);
            self.paragraph = None;
            out.push(line.to_owned());
            return;
        }
        if let Some(level) = atx(text) {
            let head = &line[..at];
            out.push(marked(head, level, text[level..].trim_start_matches([' ', '\t'])));
            self.paragraph = None;
            return;
        }
        if thematic_break(text) {
            self.paragraph = None;
            out.push(line.to_owned());
            return;
        }
        self.paragraph = Some(match self.paragraph {
            None if !container => Paragraph::One,
            _ => Paragraph::Many,
        });
        out.push(line.to_owned());
    }
}

/// `head` (the indentation and any quote or list markers), then the heading's marker at `level`, written into the
/// text, then the heading's text. The real marker is concealed; the entity copy is what shows. An entity takes the
/// heading's style, where a backslash escape takes the escape colour (`string.escape`).
fn marked(head: &str, level: usize, body: &str) -> String {
    let escaped = "&#35;".repeat(level);
    let marker = "#".repeat(level);
    if body.is_empty() { format!("{head}{marker} {escaped}") } else { format!("{head}{marker} {escaped} {body}") }
}

/// The byte offset where a line's block content starts, past its indentation and any quote or list markers; whether
/// the line has a container marker, and whether one is a list item's.
fn content_start(line: &str) -> (usize, bool, bool) {
    let (mut at, mut container, mut list) = (0, false, false);
    loop {
        let rest = &line[at..];
        let ws = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        let rest = &rest[ws..];
        let step = if let Some(after) = rest.strip_prefix('>') {
            1 + usize::from(after.starts_with([' ', '\t']))
        } else if let Some(len) = list_marker(rest) {
            list = true;
            len
        } else {
            return (at + ws, container, list);
        };
        at += ws + step;
        container = true;
    }
}

/// The length of a list marker and the spaces after it (`- `, `* `, `1. `, `2) `), or `None`.
fn list_marker(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mark = match bytes.first()? {
        b'-' | b'*' | b'+' => 1,
        b'0'..=b'9' => {
            let digits = bytes.iter().take_while(|byte| byte.is_ascii_digit()).count();
            if digits > 9 || !matches!(bytes.get(digits), Some(b'.' | b')')) {
                return None;
            }
            digits + 1
        }
        _ => return None,
    };
    let after = &rest[mark..];
    if !after.is_empty() && !after.starts_with([' ', '\t']) {
        return None;
    }
    Some(mark + after.len() - after.trim_start_matches([' ', '\t']).len())
}

/// The level of an ATX heading's text (`#` to `######`, then a space or the end), or `None`.
fn atx(text: &str) -> Option<usize> {
    let level = text.bytes().take_while(|byte| *byte == b'#').count();
    let ends_marker = text[level..].chars().next().is_none_or(|c| c == ' ' || c == '\t');
    ((1..=6).contains(&level) && ends_marker).then_some(level)
}

/// The level of a setext underline (`===` is 1, `---` is 2) at up to three spaces of indentation, or `None`.
fn underline(line: &str) -> Option<usize> {
    let body = line.trim_start_matches(' ');
    if line.len() - body.len() > 3 {
        return None;
    }
    let body = body.trim_end_matches([' ', '\t']);
    if !body.is_empty() && body.bytes().all(|byte| byte == b'=') {
        Some(1)
    } else if !body.is_empty() && body.bytes().all(|byte| byte == b'-') {
        Some(2)
    } else {
        None
    }
}

/// `***`, `---`, or `___` (three or more, with spaces), which ends a paragraph without being a heading.
fn thematic_break(text: &str) -> bool {
    let marks: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let first = marks.chars().next();
    marks.len() >= 3 && first.is_some_and(|c| matches!(c, '*' | '-' | '_')) && marks.chars().all(|c| Some(c) == first)
}

/// A fence opened by three or more backticks or tildes, at up to three spaces of indentation.
fn fence_open(text: &str) -> Option<Fence> {
    let body = text.trim_start_matches(' ');
    let ch = *body.as_bytes().first().filter(|byte| matches!(byte, b'`' | b'~'))?;
    let len = body.bytes().take_while(|byte| *byte == ch).count();
    let info = &body[len..];
    (len >= 3 && !(ch == b'`' && info.contains('`'))).then_some(Fence { ch, len })
}

/// Whether a line closes a fence: the same character, at least as long, and nothing after it.
fn closes(text: &str, fence: Fence) -> bool {
    let len = text.bytes().take_while(|byte| *byte == fence.ch).count();
    len >= fence.len && text[len..].trim().is_empty()
}

/// The HTML block a line starts (CommonMark's start conditions 1 to 6), or `None`. Condition 7 is not read: a line
/// that only opens a tag is taken as text, so a heading below it is still marked.
fn html_start(line: &str) -> Option<HtmlEnd> {
    let body = line.trim_start_matches(' ');
    if line.len() - body.len() > 3 {
        return None;
    }
    let lower = body.to_ascii_lowercase();
    if lower.starts_with("<!--") {
        return Some(HtmlEnd::Contains("-->"));
    }
    if lower.starts_with("<?") {
        return Some(HtmlEnd::Contains("?>"));
    }
    if lower.starts_with("<![cdata[") {
        return Some(HtmlEnd::Contains("]]>"));
    }
    if lower.starts_with("<!") && lower[2..].starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Some(HtmlEnd::Contains(">"));
    }
    let tag = lower.strip_prefix('<')?;
    let name_at = usize::from(tag.starts_with('/'));
    let name: String = tag[name_at..].chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    let after = &tag[name_at + name.len()..];
    if !after.is_empty() && !after.starts_with([' ', '\t', '>', '/']) {
        return None;
    }
    match (name_at, name.as_str()) {
        (0, "script") => Some(HtmlEnd::Contains("</script>")),
        (0, "pre") => Some(HtmlEnd::Contains("</pre>")),
        (0, "style") => Some(HtmlEnd::Contains("</style>")),
        (0, "textarea") => Some(HtmlEnd::Contains("</textarea>")),
        _ if BLOCK_TAGS.contains(&name.as_str()) => Some(HtmlEnd::Blank),
        _ => None,
    }
}

/// The block-level tags of CommonMark's start condition 6.
const BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "base",
    "basefont",
    "blockquote",
    "body",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "iframe",
    "legend",
    "li",
    "link",
    "main",
    "menu",
    "menuitem",
    "nav",
    "noframes",
    "ol",
    "optgroup",
    "option",
    "p",
    "param",
    "search",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
];

#[cfg(test)]
#[path = "headings_tests.rs"]
mod tests;
