//! The tags of a small HTML block, read without a parser. An HTML image is a block of tags and whitespace (`reader/images.rs`
//! `parse_html`); text between the tags, a comment, or a tag this reads as malformed makes the block something else.

/// One tag: its lowercase name, whether it closes, and its attributes (names lowercase, values as written).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub closing: bool,
    pub attrs: Vec<(String, String)>,
}

impl Tag {
    /// The value of the attribute `name`, when the tag has it.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }
}

/// The tags of `text` in order, when `text` is nothing but tags and whitespace.
pub fn tags(text: &str) -> Option<Vec<Tag>> {
    let mut rest = text.trim_start();
    let mut tags = Vec::new();
    while !rest.is_empty() {
        let (tag, after) = tag(rest)?;
        tags.push(tag);
        rest = after.trim_start();
    }
    Some(tags)
}

/// The tag at the start of `text`, and the text after it.
fn tag(text: &str) -> Option<(Tag, &str)> {
    let body = text.strip_prefix('<')?;
    let (closing, body) = match body.strip_prefix('/') {
        Some(rest) => (true, rest),
        None => (false, body),
    };
    let name_end = body.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-')).unwrap_or(body.len());
    let name = &body[..name_end];
    if !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut rest = &body[name_end..];
    let mut attrs = Vec::new();
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix("/>").or_else(|| rest.strip_prefix('>')) {
            let tag = Tag { name: name.to_ascii_lowercase(), closing, attrs };
            return Some((tag, after));
        }
        let key_end =
            rest.find(|c: char| c.is_ascii_whitespace() || matches!(c, '=' | '>' | '/')).unwrap_or(rest.len());
        if key_end == 0 {
            return None;
        }
        let key = rest[..key_end].to_ascii_lowercase();
        rest = rest[key_end..].trim_start();
        let value = match rest.strip_prefix('=') {
            Some(after) => {
                let (value, after) = value(after.trim_start())?;
                rest = after;
                value
            }
            None => String::new(),
        };
        attrs.push((key, value));
    }
}

/// An attribute value: quoted with `"` or `'`, or a run of characters up to a space or the tag's end.
fn value(text: &str) -> Option<(String, &str)> {
    match text.chars().next() {
        Some(quote @ ('"' | '\'')) => {
            let close = text[1..].find(quote)?;
            Some((text[1..1 + close].to_owned(), &text[close + 2..]))
        }
        _ => {
            let end = text.find(|c: char| c.is_ascii_whitespace() || c == '>').unwrap_or(text.len());
            Some((text[..end].to_owned(), &text[end..]))
        }
    }
}

#[cfg(test)]
#[path = "html_tests.rs"]
mod tests;
