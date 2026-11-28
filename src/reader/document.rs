//! A text document from its bytes (Scouter `readDocument`, `src/core/files.ts:90-189`): the 1 MiB cap, the NUL and
//! UTF-8 checks, and the display text. The bytes come from a worker (`services/fs.rs`).

use crate::text::display_text;

/// Scouter reads at most this many bytes (`src/core/files.ts:6`).
pub const MAX_BYTES: usize = 1024 * 1024;

/// What a reader keeps of a file: its display text, its logical lines, and whether the cap cut it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub text: String,
    /// `countLogicalLines`: a final newline does not start a line, and an empty file has none.
    pub lines: usize,
    /// The file is longer than [`MAX_BYTES`]; the footer says `first 1 MiB`.
    pub truncated: bool,
}

impl Document {
    /// The lines the code view draws: the text without its final newline.
    pub fn content(&self) -> &str {
        self.text.strip_suffix('\n').unwrap_or(&self.text)
    }
}

/// Why the bytes are not a text document. The message is Scouter's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Binary,
    NotUtf8,
}

impl Refusal {
    pub fn message(self) -> &'static str {
        match self {
            Self::Binary => "Binary files are not supported.",
            Self::NotUtf8 => "Only UTF-8 text files are supported.",
        }
    }
}

/// Decodes a file's bytes. A NUL in the first MiB makes it binary; a cut at the cap steps back to a character
/// boundary; a leading byte order mark goes, as `TextDecoder` drops it; tabs become four spaces.
pub fn decode(bytes: &[u8]) -> Result<Document, Refusal> {
    let probe = &bytes[..bytes.len().min(MAX_BYTES)];
    if probe.contains(&0) {
        return Err(Refusal::Binary);
    }
    let truncated = bytes.len() > MAX_BYTES;
    let end = if truncated { character_boundary(bytes, MAX_BYTES) } else { bytes.len() };
    let text = std::str::from_utf8(&bytes[..end]).map_err(|_| Refusal::NotUtf8)?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let text = display_text(text).replace('\t', "    ");
    Ok(Document { lines: count_lines(&text), text, truncated })
}

/// Decodes bytes the reader refused, for the user who asked to read them as text: invalid UTF-8 becomes U+FFFD, and
/// NUL and the C0 controls become their control pictures (`␀`, `␛`, `␡` for DEL), so the bytes that differ still look
/// different. Line ends, tabs, the cap, and the other controls are as [`decode`] has them.
pub fn decode_lossy(bytes: &[u8]) -> Document {
    let truncated = bytes.len() > MAX_BYTES;
    let end = if truncated { character_boundary(bytes, MAX_BYTES) } else { bytes.len() };
    let text = String::from_utf8_lossy(&bytes[..end]);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let pictured: String = text
        .chars()
        .map(|c| match c {
            '\0'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' => {
                char::from_u32(0x2400 + u32::from(c)).unwrap_or('\u{fffd}')
            }
            '\u{7f}' => '\u{2421}',
            c => c,
        })
        .collect();
    let text = display_text(&pictured).replace('\t', "    ");
    Document { lines: count_lines(&text), text, truncated }
}

/// The cut moves back over continuation bytes (`10xxxxxx`), so the last character is whole.
fn character_boundary(bytes: &[u8], end: usize) -> usize {
    let mut cut = end;
    while cut > 0 && bytes.get(cut).is_some_and(|byte| byte & 0xc0 == 0x80) {
        cut -= 1;
    }
    cut
}

fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    1 + text.strip_suffix('\n').unwrap_or(text).matches('\n').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_lines_as_scouter_does() {
        assert_eq!(decode(b"").unwrap().lines, 0);
        assert_eq!(decode(b"\n").unwrap().lines, 1);
        assert_eq!(decode(b"Hogwarts\n").unwrap().lines, 1);
        assert_eq!(decode(b"Hogwarts\n\n").unwrap().lines, 2);
        assert_eq!(decode(b"Hogwarts\nGringotts").unwrap().lines, 2);
    }

    #[test]
    fn content_drops_one_final_newline() {
        assert_eq!(decode(b"Langdon\n\n").unwrap().content(), "Langdon\n");
        assert_eq!(decode(b"Langdon").unwrap().content(), "Langdon");
    }

    #[test]
    fn normalizes_line_ends_tabs_and_controls() {
        let document = decode(b"a\r\nb\rc\t\x1b\x07\n").unwrap();
        assert_eq!(document.text, "a\nb\nc    \u{fffd}\u{fffd}\n");
        assert_eq!(decode("\u{202e}Sophie".as_bytes()).unwrap().text, "\u{fffd}Sophie");
    }

    #[test]
    fn a_byte_order_mark_is_dropped() {
        assert_eq!(decode("\u{feff}Vittoria\n".as_bytes()).unwrap().text, "Vittoria\n");
    }

    #[test]
    fn a_nul_makes_the_file_binary_and_invalid_utf8_is_refused() {
        assert_eq!(decode(b"Teabing\0").unwrap_err(), Refusal::Binary);
        assert_eq!(decode(b"Katherine \xff").unwrap_err(), Refusal::NotUtf8);
        assert_eq!(Refusal::NotUtf8.message(), "Only UTF-8 text files are supported.");
    }

    #[test]
    fn a_nul_past_the_first_mebibyte_does_not_make_it_binary() {
        let mut bytes = vec![b'a'; MAX_BYTES + 10];
        bytes[MAX_BYTES + 5] = 0;
        let document = decode(&bytes).unwrap();
        assert!(document.truncated);
        assert_eq!(document.text.len(), MAX_BYTES);
    }

    #[test]
    fn lossy_text_shows_controls_as_pictures_and_bad_bytes_as_replacements() {
        let document = decode_lossy(b"Vittoria\0Vetra\x1b[0m\x7f\tCERN\r\n\xe9t\xc3\xa9\n");
        assert_eq!(document.text, "Vittoria\u{2400}Vetra\u{241b}[0m\u{2421}    CERN\n\u{fffd}t\u{e9}\n");
        assert_eq!(document.lines, 2);
        assert!(!document.truncated);
        // Bidi overrides and C1 controls stay replaced, as in any text.
        assert_eq!(decode_lossy("\u{202e}Sophie\u{85}".as_bytes()).text, "\u{fffd}Sophie\u{fffd}");
    }

    #[test]
    fn lossy_text_is_cut_at_the_cap() {
        let mut bytes = vec![0u8; MAX_BYTES - 1];
        bytes.extend("é".as_bytes());
        let document = decode_lossy(&bytes);
        assert!(document.truncated);
        assert_eq!(document.text.chars().count(), MAX_BYTES - 1);
        assert!(document.text.chars().all(|c| c == '\u{2400}'));
    }

    #[test]
    fn the_cut_steps_back_to_a_character_boundary() {
        // `é` is two bytes; a cut in its middle moves back so the text stays valid UTF-8.
        let mut bytes = vec![b'a'; MAX_BYTES - 1];
        bytes.extend("é".as_bytes());
        bytes.extend(b"tail");
        let document = decode(&bytes).unwrap();
        assert!(document.truncated);
        assert_eq!(document.text.len(), MAX_BYTES - 1);
    }
}
