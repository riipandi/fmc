//! The hex view's pure parts: how many bytes a row holds at a width, what each byte looks like, the blocks of the file
//! the view keeps, the offset a jump names, and the string match a search runs over each chunk it reads. The bytes come
//! from workers (`services/window.rs`); nothing here reads a file.

use std::ops::Range;

/// The bytes one read asks for. A row never crosses a block: every row width divides it.
pub const BLOCK: u64 = 64 * 1024;
/// The blocks the view keeps, 1 MiB: the view and a block or two each side.
pub const MAX_BLOCKS: usize = 16;
/// Bytes between the extra spaces of the hex column.
const GROUP: usize = 8;
/// The row widths tried, widest first.
const WIDTHS: [usize; 4] = [32, 16, 8, 4];

/// A byte's kind, which picks its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Null,
    Printable,
    Whitespace,
    Control,
    High,
}

pub fn class(byte: u8) -> Class {
    match byte {
        0 => Class::Null,
        b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c => Class::Whitespace,
        0x21..=0x7e => Class::Printable,
        0x01..=0x1f | 0x7f => Class::Control,
        _ => Class::High,
    }
}

/// The ASCII column's character: the byte itself when it prints, else a dot.
pub fn ascii(byte: u8) -> char {
    if byte == b' ' || byte.is_ascii_graphic() { char::from(byte) } else { '.' }
}

/// What one cell of a row is drawn as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Blank,
    Offset,
    /// A byte's hex digit or its ASCII character; `hit` while a search hit covers the byte.
    Byte {
        class: Class,
        hit: bool,
    },
    /// The space between two hex bytes, filled while both are in the hit so the hit reads as one run.
    Gap {
        hit: bool,
    },
}

/// The offset column's digits: eight, or more once the last offset needs them.
pub fn offset_digits(size: u64) -> usize {
    let last = size.saturating_sub(1);
    (64 - last.leading_zeros() as usize).div_ceil(4).max(8)
}

/// The columns a row of `per_row` bytes takes: the offset, two spaces, the hex bytes with a space between each and
/// one more between groups of eight, two spaces, and the ASCII column.
pub fn row_width(per_row: usize, digits: usize) -> usize {
    digits + 2 + per_row * 3 - 1 + (per_row - 1) / GROUP + 2 + per_row
}

/// The widest row that fits `columns`; four bytes when none does, and the row is cut at the pane's edge.
pub fn per_row(columns: usize, digits: usize) -> usize {
    WIDTHS.into_iter().find(|&bytes| row_width(bytes, digits) <= columns).unwrap_or(4)
}

/// The rows a file of `size` bytes takes; an empty file still has its first row.
pub fn rows(size: u64, per_row: usize) -> u64 {
    size.div_ceil(per_row as u64).max(1)
}

/// One row's cells for the row at `offset`. `bytes` holds the row's bytes that are known: fewer than `per_row` at the
/// file's end, none while the row's block is still being read. `hit` is the search hit's byte range.
pub fn row(offset: u64, bytes: &[u8], per_row: usize, digits: usize, hit: Option<&Range<u64>>) -> Vec<(char, Ink)> {
    let mut cells = Vec::with_capacity(row_width(per_row, digits));
    cells.extend(format!("{offset:0digits$x}").chars().map(|c| (c, Ink::Offset)));
    cells.extend([(' ', Ink::Blank); 2]);
    let in_hit = |index: usize| hit.is_some_and(|hit| hit.contains(&(offset + index as u64)));
    for index in 0..per_row {
        if index > 0 {
            let joined = index < bytes.len() && in_hit(index - 1) && in_hit(index);
            cells.push((' ', Ink::Gap { hit: joined }));
            if index % GROUP == 0 {
                cells.push((' ', Ink::Gap { hit: joined }));
            }
        }
        match bytes.get(index) {
            Some(&byte) => {
                let ink = Ink::Byte { class: class(byte), hit: in_hit(index) };
                cells.push((hex_digit(byte >> 4), ink));
                cells.push((hex_digit(byte & 0xf), ink));
            }
            None => cells.extend([(' ', Ink::Blank); 2]),
        }
    }
    cells.extend([(' ', Ink::Blank); 2]);
    for index in 0..per_row {
        cells.push(match bytes.get(index) {
            Some(&byte) => (ascii(byte), Ink::Byte { class: class(byte), hit: in_hit(index) }),
            None => (' ', Ink::Blank),
        });
    }
    cells
}

fn hex_digit(nibble: u8) -> char {
    char::from(b"0123456789abcdef"[usize::from(nibble)])
}

/// The byte a jump names, clamped to the last byte: `0x1f40` in hex, `8000` in decimal, or `50%` of the file.
pub fn parse_offset(text: &str, size: u64) -> Option<u64> {
    let text = text.trim();
    let offset = if let Some(percent) = text.strip_suffix('%') {
        let percent: f64 = percent.trim().parse().ok()?;
        if !(0.0..=100.0).contains(&percent) {
            return None;
        }
        (size as f64 * percent / 100.0) as u64
    } else if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()?
    } else {
        text.parse().ok()?
    };
    Some(offset.min(size.saturating_sub(1)))
}

/// The blocks of one file the view holds, and the ones asked for and not yet back.
#[derive(Debug, Default)]
pub struct Blocks {
    held: Vec<(u64, Vec<u8>)>,
    asked: Vec<u64>,
}

impl Blocks {
    /// The blocks that `range` of the file needs and that are neither held nor asked for.
    pub fn wanted(&self, range: Range<u64>) -> Vec<u64> {
        if range.is_empty() {
            return Vec::new();
        }
        (range.start / BLOCK..=(range.end - 1) / BLOCK)
            .filter(|index| !self.asked.contains(index) && self.held.iter().all(|(held, _)| held != index))
            .collect()
    }

    pub fn ask(&mut self, index: u64) {
        self.asked.push(index);
    }

    /// Keeps a block that came back. Past [`MAX_BLOCKS`] the block farthest from `near` goes, so the view's blocks stay.
    pub fn insert(&mut self, index: u64, bytes: Vec<u8>, near: u64) {
        self.asked.retain(|asked| *asked != index);
        self.held.retain(|(held, _)| *held != index);
        self.held.push((index, bytes));
        while self.held.len() > MAX_BLOCKS {
            let far = (0..self.held.len()).max_by_key(|&at| self.held[at].0.abs_diff(near)).unwrap_or(0);
            self.held.swap_remove(far);
        }
    }

    /// The known bytes of the `len` bytes at `offset`, which lie in one block: `None` while that block is not back, and
    /// fewer than `len` at the file's end.
    pub fn bytes(&self, offset: u64, len: usize) -> Option<&[u8]> {
        let (_, block) = self.held.iter().find(|(index, _)| *index == offset / BLOCK)?;
        let start = ((offset % BLOCK) as usize).min(block.len());
        Some(&block[start..(start + len).min(block.len())])
    }
}

/// The first place `needle` starts in `haystack` (forward), or the last (back), ignoring ASCII case. `needle` is already
/// lowercase in its ASCII letters, and not empty.
pub fn find(haystack: &[u8], needle: &[u8], forward: bool) -> Option<usize> {
    let last = haystack.len().checked_sub(needle.len())?;
    let (lower, upper) = (needle[0], needle[0].to_ascii_uppercase());
    let matches = |at: usize| {
        let byte = haystack[at];
        (byte == lower || byte == upper) && haystack[at..at + needle.len()].eq_ignore_ascii_case(needle)
    };
    if forward { (0..=last).find(|&at| matches(at)) } else { (0..=last).rev().find(|&at| matches(at)) }
}

#[cfg(test)]
#[path = "hex_tests.rs"]
mod tests;
