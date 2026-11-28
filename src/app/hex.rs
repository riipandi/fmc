//! The hex view on the UI thread: where it is in the file, the blocks it holds, and its keys. Each paint asks workers
//! for the blocks around the view (`services/window.rs`) and draws only the visible rows, so a file of several GiB
//! opens, scrolls, and jumps without reading more than the screen needs.

use std::path::{Path, PathBuf};

use fmc::keys::{Action, Focus};
use fmc::reader::hex::{self, BLOCK, Blocks};
use fmc::reader::motion::{HALF_PAGE, PAGE};
use fmc::theme::Palette;

use super::reader::Shown;
use super::{Model, Ui};
use crate::services::{self, Event, window};
use crate::ui::footer::Message;

/// The hex view of one file.
#[derive(Debug)]
pub struct Hex {
    pub path: PathBuf,
    /// The file's size at the open; a short block moves it down when the file shrank.
    pub size: u64,
    /// The first byte shown, a multiple of `per_row`. A byte, not a row, so a new width keeps the place.
    top: u64,
    per_row: usize,
    /// The screen rows of the last paint, which the scroll keys page by.
    viewport: usize,
    blocks: Blocks,
}

impl Hex {
    fn new(path: PathBuf, size: u64) -> Self {
        Self { path, size, top: 0, per_row: 16, viewport: 0, blocks: Blocks::default() }
    }

    fn rows(&self) -> u64 {
        hex::rows(self.size, self.per_row)
    }

    fn top_row(&self) -> u64 {
        self.top / self.per_row as u64
    }

    /// Puts row `row` at the top, as far as the last screen allows.
    fn set_row(&mut self, row: u64) {
        let last = self.rows().saturating_sub(self.viewport as u64);
        self.top = row.min(last) * self.per_row as u64;
    }

    fn scroll(&mut self, rows: i64) {
        self.set_row(self.top_row().saturating_add_signed(rows));
    }
}

/// `X`: the hex view of the file the notice names. Focus moves to the reader, whose keys scroll it.
pub fn open_shown(m: &mut Model) {
    let (path, size) = match &m.shown {
        Shown::Binary { path, size } => (path.clone(), *size),
        _ => return,
    };
    m.shown = Shown::Hex(Box::new(Hex::new(path, size)));
    m.focus = Focus::Reader;
    m.search = None;
    m.message = None;
    m.restore = None;
}

fn shown_mut(m: &mut Model) -> Option<&mut Hex> {
    match &mut m.shown {
        Shown::Hex(hex) => Some(hex),
        _ => None,
    }
}

/// The reader's scroll keys in the hex view, by rows of bytes.
pub fn motion(m: &mut Model, action: Action) {
    let Some(hex) = shown_mut(m) else { return };
    let viewport = hex.viewport.max(1) as f64;
    match action {
        Action::Scroll(rows) => hex.scroll(i64::from(rows)),
        Action::Page(direction) => hex.scroll((f64::from(direction) * PAGE * viewport).round() as i64),
        Action::HalfPage(direction) => hex.scroll((f64::from(direction) * HALF_PAGE * viewport).round() as i64),
        Action::Top => hex.set_row(0),
        Action::Bottom => hex.set_row(u64::MAX),
        _ => {}
    }
}

/// A wheel step or a bar drag: `rows` down (up when negative).
pub fn scroll(m: &mut Model, rows: isize) {
    if let Some(hex) = shown_mut(m) {
        hex.scroll(rows as i64);
    }
}

/// A bar drag to row `row`.
pub fn to_row(m: &mut Model, row: usize) {
    if let Some(hex) = shown_mut(m) {
        hex.set_row(row as u64);
    }
}

/// `:` with an offset while the hex view shows: the offset's row goes to the top. Returns false when no hex view
/// shows, so the command line reads the text as a command.
pub fn go_to(m: &mut Model, text: &str) -> bool {
    let Some(hex) = shown_mut(m) else { return false };
    match hex::parse_offset(text, hex.size) {
        Some(offset) => {
            hex.set_row(offset / hex.per_row as u64);
            m.message = Some(Message { text: format!("Offset {offset:#x}."), error: false });
        }
        None => m.message = Some(Message { text: format!("Not an offset: {}", text.trim()), error: true }),
    }
    true
}

/// A block a worker read. A block of a file that is no longer shown is dropped; a short one is the file's new end.
pub fn block(m: &mut Model, path: &Path, index: u64, result: &Result<Vec<u8>, String>) {
    let Some(hex) = shown_mut(m).filter(|hex| hex.path == path) else { return };
    match result {
        Ok(bytes) => {
            if (bytes.len() as u64) < BLOCK {
                hex.size = hex.size.min(index * BLOCK + bytes.len() as u64);
            }
            let near = hex.top / BLOCK;
            hex.blocks.insert(index, bytes.clone(), near);
        }
        Err(text) => m.message = Some(Message { text: text.clone(), error: true }),
    }
}

/// Lays the view out for a pane of `columns` by `viewport` rows: the row width, the top kept in range, the blocks
/// asked for (the view's and one either side, so a scroll seldom waits), and the visible rows drawn. Returns the rows
/// and the top row, for the bar and the footer, or `None` when no hex view shows.
pub fn sync(m: &mut Model, ui: &Ui, columns: usize, viewport: usize, p: &Palette) -> Option<(usize, usize)> {
    let proxy = m.proxy.clone();
    let Shown::Hex(hex) = &mut m.shown else { return None };
    let digits = hex::offset_digits(hex.size);
    hex.per_row = hex::per_row(columns, digits);
    hex.viewport = viewport;
    hex.set_row(hex.top_row());
    let per_row = hex.per_row;
    let end = (hex.top + (viewport * per_row) as u64).min(hex.size);
    let around = hex.top.saturating_sub(BLOCK)..end.saturating_add(BLOCK).min(hex.size);
    for index in hex.blocks.wanted(around) {
        hex.blocks.ask(index);
        let path = hex.path.clone();
        services::spawn(&proxy, move || Event::Block { result: window::read_block(&path, index), path, index });
    }
    let rows: Vec<_> = (0..viewport as u64)
        .map(|row| hex.top + row * per_row as u64)
        .take_while(|&offset| offset < hex.size.max(1))
        .map(|offset| hex::row(offset, hex.blocks.bytes(offset, per_row).unwrap_or_default(), per_row, digits, None))
        .collect();
    ui.hex.show(&rows, p);
    Some((hex.rows() as usize, hex.top_row() as usize))
}

#[cfg(test)]
#[path = "hex_tests.rs"]
mod tests;
