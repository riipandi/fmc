//! A line of text with a caret: the edits that a prompt and a dialog's name field share. The caret is a byte offset
//! that always sits on a character boundary, so no edit cuts a UTF-8 character. Each `FieldEdit` is the OpenTUI
//! `Input` action of the same meaning (`app/overlay.rs` `editor_action`).

use crate::keys::Key;

/// An edit of a one-line field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldEdit {
    Backspace,
    DeleteWordBackward,
    DeleteWordForward,
    DeleteToLineStart,
    DeleteToLineEnd,
    WordBackward,
    WordForward,
    LineHome,
    LineEnd,
}

/// The field edit a key asks for. Plain keys are not edits here: a plain arrow moves a dialog's focus, and a prompt
/// moves its caret by character. Option is Alt, and Cmd is Super, which arrives only with the kitty protocol or from
/// a terminal that passes Cmd through. The Ctrl letters are readline's. Shift is ignored: the field has no selection,
/// so Shift+Option+Left moves by word as Option+Left does.
pub fn edit_of(key: Key<'_>) -> Option<FieldEdit> {
    use FieldEdit::*;
    Some(match (key.name, key.ctrl, key.alt, key.cmd) {
        ("backspace", false, false, false) => Backspace,
        // ESC DEL is Option+Backspace without the kitty protocol; Ctrl+Backspace is OpenTUI's own binding.
        ("backspace", false, true, false) | ("backspace", true, false, false) | ("w", true, false, false) => {
            DeleteWordBackward
        }
        ("backspace", false, false, true) | ("u", true, false, false) => DeleteToLineStart,
        ("delete", false, true, false) | ("delete", true, false, false) | ("d", false, true, false) => {
            DeleteWordForward
        }
        ("k", true, false, false) => DeleteToLineEnd,
        ("left", false, true, false) | ("left", true, false, false) | ("b", false, true, false) => WordBackward,
        ("right", false, true, false) | ("right", true, false, false) | ("f", false, true, false) => WordForward,
        ("left", false, false, true) | ("a", true, false, false) => LineHome,
        ("right", false, false, true) | ("e", true, false, false) => LineEnd,
        _ => return None,
    })
}

/// Typed text: a key that carries printable characters, not a control key or a named one.
pub fn is_text(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| !c.is_control())
}

/// One line of text and its caret.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Line {
    text: String,
    caret: usize,
}

impl Line {
    /// A line with its caret at the end.
    pub fn with_text(text: &str) -> Self {
        Self { text: text.to_owned(), caret: text.len() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text before the caret.
    pub fn before(&self) -> &str {
        &self.text[..self.caret]
    }

    /// The text from the caret on.
    pub fn after(&self) -> &str {
        &self.text[self.caret..]
    }

    /// Types `typed` at the caret, and the caret moves past it.
    pub fn insert(&mut self, typed: &str) {
        self.text.insert_str(self.caret, typed);
        self.caret += typed.len();
    }

    /// Moves the caret back one character; at the start it stays.
    pub fn left(&mut self) {
        if let Some((at, _)) = self.text[..self.caret].char_indices().next_back() {
            self.caret = at;
        }
    }

    /// Moves the caret forward one character; at the end it stays.
    pub fn right(&mut self) {
        if let Some(c) = self.text[self.caret..].chars().next() {
            self.caret += c.len_utf8();
        }
    }

    /// Runs `edit` at the caret. Returns whether the text changed; a caret move returns false.
    pub fn edit(&mut self, edit: FieldEdit) -> bool {
        let caret = self.caret;
        let (from, to) = match edit {
            FieldEdit::Backspace => (self.text[..caret].char_indices().next_back().map_or(0, |(at, _)| at), caret),
            FieldEdit::DeleteWordBackward => (self.word_back(), caret),
            FieldEdit::DeleteWordForward => (caret, self.word_forward()),
            FieldEdit::DeleteToLineStart => (0, caret),
            FieldEdit::DeleteToLineEnd => (caret, self.text.len()),
            FieldEdit::WordBackward => {
                self.caret = self.word_back();
                return false;
            }
            FieldEdit::WordForward => {
                self.caret = self.word_forward();
                return false;
            }
            FieldEdit::LineHome => {
                self.caret = 0;
                return false;
            }
            FieldEdit::LineEnd => {
                self.caret = self.text.len();
                return false;
            }
        };
        if from == to {
            return false;
        }
        self.text.replace_range(from..to, "");
        self.caret = from;
        true
    }

    /// The byte offset of the word boundary before the caret.
    fn word_back(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        let caret = self.text[..self.caret].chars().count();
        byte_of(&self.text, prev_word(&chars, caret))
    }

    /// The byte offset of the word boundary after the caret.
    fn word_forward(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        let caret = self.text[..self.caret].chars().count();
        byte_of(&self.text, next_word(&chars, caret))
    }
}

/// The byte offset of character `index`, or the end of `text` past its last character.
fn byte_of(text: &str, index: usize) -> usize {
    text.char_indices().nth(index).map_or(text.len(), |(at, _)| at)
}

/// The script a word character belongs to. Latin letters, digits, and `_` are ASCII words; CJK runs are words of
/// their own (`utf8.zig` `classifyWordClass`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    Ascii,
    Cjk,
    Other,
}

fn script(c: char) -> Script {
    if c.is_ascii() {
        if c.is_ascii_alphanumeric() || c == '_' { Script::Ascii } else { Script::Other }
    } else if is_cjk(c) {
        Script::Cjk
    } else {
        Script::Other
    }
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xF900..=0xFAFF
        | 0x20000..=0x2A6DF
        | 0x2A700..=0x2B73F
        | 0x2B740..=0x2B81F
        | 0x2B820..=0x2CEAF
        | 0x2CEB0..=0x2EBEF
        | 0x2EBF0..=0x2EE5D
        | 0x2F800..=0x2FA1F
        // Hiragana, katakana, Hangul jamo, Hangul syllables
        | 0x3040..=0x309F
        | 0x30A0..=0x30FF
        | 0x31F0..=0x31FF
        | 0xFF66..=0xFF9D
        | 0x1100..=0x11FF
        | 0x3130..=0x318F
        | 0xA960..=0xA97F
        | 0xAC00..=0xD7AF
        | 0xD7B0..=0xD7FF)
}

/// Whitespace and punctuation: each ends a word on its own (`utf8.zig` `asciiLayoutWrapBreakKind` and
/// `unicodeLayoutWrapBreakKind`). The em dash is not one, so it is a word of its own.
const SEPARATORS: &str = " \t-/\\.,;:!?()[]{}\u{a0}\u{ad}\u{1680}\u{2010}\u{202f}\u{205f}\u{3000}\u{3001}\u{3002}\u{ff01}\u{ff0c}\u{ff1a}\u{ff1f}";

fn is_separator(c: char) -> bool {
    SEPARATORS.contains(c) || matches!(c, '\u{2000}'..='\u{200b}')
}

/// The character indices after which a word ends: each separator, and the last character before a change between
/// ASCII letters and CJK (`utf8.zig` `isCjkAsciiTransition`). A CJK run is one word, and so is an ASCII run.
fn breaks(chars: &[char]) -> impl Iterator<Item = usize> + '_ {
    chars.iter().enumerate().filter_map(move |(at, &c)| {
        let changes = chars.get(at + 1).is_some_and(|&next| {
            matches!((script(c), script(next)), (Script::Ascii, Script::Cjk) | (Script::Cjk, Script::Ascii))
        });
        (is_separator(c) || changes).then_some(at)
    })
}

/// The character index of the word boundary before `caret`, or 0 (`getPrevWordBoundary`). A boundary at the caret
/// does not count.
fn prev_word(chars: &[char], caret: usize) -> usize {
    breaks(chars).filter(|&at| at + 1 < caret).last().map_or(0, |at| at + 1)
}

/// The character index of the word boundary after `caret`, or the end (`getNextWordBoundary`). A boundary at the
/// caret counts only when the caret is on a word character, so a caret on a space steps past the next word.
fn next_word(chars: &[char], caret: usize) -> usize {
    breaks(chars)
        .find(|&at| at > caret || (at == caret && script(chars[at]) != Script::Other))
        .map_or(chars.len(), |at| at + 1)
}

#[cfg(test)]
#[path = "line_edit_tests.rs"]
mod tests;
