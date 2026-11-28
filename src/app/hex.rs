//! The hex view on the UI thread: where it is in the file, the blocks it holds, and its keys. Each paint asks workers
//! for the blocks around the view (`services/window.rs`) and draws only the visible rows, so a file of several GiB
//! opens, scrolls, and jumps without reading more than the screen needs.

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use fmc::keys::{Action, Focus, Key};
use fmc::line_edit::{edit_of, is_text};
use fmc::reader::hex::{self, BLOCK, Blocks};
use fmc::reader::motion::{HALF_PAGE, PAGE, line_top};
use fmc::theme::Palette;

use super::reader::Shown;
use super::{Model, Ui};
use crate::services::{self, Event, window};
use crate::ui::footer::Message;

/// A running search's flag: dropping it stops the worker, which a newer search and a closed view both do.
#[derive(Debug)]
struct Stop(Arc<AtomicBool>);

impl Drop for Stop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

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
    /// The search's last hit and the query that found it; `n` and `N` step from it while the query is the same.
    hit: Option<(Range<u64>, String)>,
    /// The latest search's number, whose answer is kept, and its stop flag.
    searches: u64,
    stop: Option<Stop>,
    /// The footer's word on the search: `searching…`, the hit's offset, or `no matches`.
    status: String,
}

impl Hex {
    fn new(path: PathBuf, size: u64) -> Self {
        let blocks = Blocks::default();
        Self {
            path,
            size,
            top: 0,
            per_row: 16,
            viewport: 0,
            blocks,
            hit: None,
            searches: 0,
            stop: None,
            status: String::new(),
        }
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

/// `X`: the hex view of the file the notice names, or of the refused file `t` showed as text. Focus moves to the
/// reader, whose keys scroll it.
pub fn open_shown(m: &mut Model) {
    let (path, size) = match &m.shown {
        Shown::Binary { path, size, .. } | Shown::Text { path, lossy: Some(size), .. } => (path.clone(), *size),
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
    // The hit is marked while the search is kept; Esc drops the search and the mark.
    let searching = m.search.is_some();
    let Shown::Hex(hex) = &mut m.shown else { return None };
    let hit = hex.hit.as_ref().map(|(hit, _)| hit.clone()).filter(|_| searching);
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
        .map(|offset| {
            let bytes = hex.blocks.bytes(offset, per_row).unwrap_or_default();
            hex::row(offset, bytes, per_row, digits, hit.as_ref())
        })
        .collect();
    ui.hex.show(&rows, p);
    Some((hex.rows() as usize, hex.top_row() as usize))
}

/// What the search prompt shows after the query in the hex view: the search's state.
pub fn status(m: &Model) -> Option<String> {
    match &m.shown {
        Shown::Hex(hex) if !hex.status.is_empty() => Some(format!("  {}", hex.status)),
        Shown::Hex(_) => Some(String::new()),
        _ => None,
    }
}

fn show(m: &mut Model) {
    let query = m.search.as_ref().map_or(String::new(), |search| search.query.text().to_owned());
    let status = status(m).unwrap_or_default();
    m.message = Some(Message { text: format!("/{query}{status}"), error: false });
}

/// A key while the search prompt is open in the hex view. The query is not searched as it is typed, since a search may
/// read the whole file: Enter (or `↓`) finds the next hit, Shift+Enter (or `↑`) the one before, and Esc closes the
/// prompt and keeps the search for `n` and `N`.
pub fn prompt_key(m: &mut Model, key: Key<'_>) {
    let Some(mut search) = m.search.take() else { return };
    let mut forward = None;
    if let Some(edit) = edit_of(key) {
        search.query.edit(edit);
    } else {
        match key.name {
            "escape" if search.query.text().is_empty() => {
                m.message = None;
                return;
            }
            "escape" => search.editing = false,
            "return" => forward = Some(!key.shift),
            "down" => forward = Some(true),
            "up" => forward = Some(false),
            "left" => search.query.left(),
            "right" => search.query.right(),
            _ if is_text(key.text) => search.query.insert(key.text),
            _ => {}
        }
    }
    m.search = Some(search);
    if let Some(forward) = forward {
        find(m, forward);
    }
    show(m);
}

/// `n` and `N` in the hex view: the next or previous hit of the kept query.
pub fn step(m: &mut Model, forward: bool) {
    find(m, forward);
    show(m);
}

/// Starts a search for the query on a worker: forward from just past the last hit of the same query, or from the
/// view's top; back from the last hit, or from the top. A running search stops.
fn find(m: &mut Model, forward: bool) {
    let proxy = m.proxy.clone();
    let query = m.search.as_ref().map_or(String::new(), |search| search.query.text().to_owned());
    let Some(hex) = shown_mut(m) else { return };
    if query.is_empty() {
        return;
    }
    let from = match &hex.hit {
        Some((hit, found)) if *found == query => {
            if forward {
                hit.start + 1
            } else {
                hit.start
            }
        }
        _ => hex.top,
    };
    hex.searches += 1;
    let flag = Arc::new(AtomicBool::new(false));
    hex.stop = Some(Stop(flag.clone()));
    hex.status = "searching…".into();
    let (path, search, needle) = (hex.path.clone(), hex.searches, query.as_bytes().to_ascii_lowercase());
    services::spawn(&proxy, move || Event::HexFound {
        result: window::search(&path, &needle, from, forward, &flag),
        path,
        search,
    });
}

/// A search's answer: a hit is marked and brought into view, half a screen down when it was off screen. An answer for
/// another file, or for an older search, is dropped.
pub fn found(m: &mut Model, path: &Path, search: u64, result: &Result<Option<u64>, String>) {
    let query = m.search.as_ref().map_or(String::new(), |search| search.query.text().to_owned());
    let Some(hex) = shown_mut(m).filter(|hex| hex.path == path && hex.searches == search) else { return };
    hex.stop = None;
    match result {
        Ok(Some(at)) => {
            let row = at / hex.per_row as u64;
            if row < hex.top_row() || row >= hex.top_row() + hex.viewport as u64 {
                hex.set_row(line_top(row as usize, hex.viewport) as u64);
            }
            hex.status = format!("at {at:#x}");
            hex.hit = Some((*at..at + query.len() as u64, query));
        }
        Ok(None) => {
            hex.status = "no matches".into();
            hex.hit = None;
        }
        Err(text) => {
            hex.status = String::new();
            m.message = Some(Message { text: text.clone(), error: true });
            return;
        }
    }
    show(m);
}

#[cfg(test)]
#[path = "hex_tests.rs"]
mod tests;
