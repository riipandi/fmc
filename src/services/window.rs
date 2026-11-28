//! The hex view's reads: one block of a file, and a string search over the whole file. Each runs as one worker job
//! (`services::spawn`), so a slow disk never stalls a frame. Every file is opened read-only, and a file that shrank
//! reads short instead of failing: positioned reads, where a memory map would kill the process with SIGBUS.

use std::fs::File;
use std::io;
use std::ops::Range;
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use fmc::reader::hex::{BLOCK, find};

use super::fs::describe;

/// The bytes a search reads at a time, plus the query's length less one so a hit across two chunks is found.
const CHUNK: u64 = 1024 * 1024;

/// Block `index` of the file: [`BLOCK`] bytes, fewer at the end, none past it.
pub fn read_block(path: &Path, index: u64) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|error| describe(&error, "open", Some(path)))?;
    let mut bytes = vec![0; BLOCK as usize];
    let filled = read_full(&file, &mut bytes, index * BLOCK).map_err(|error| describe(&error, "read", None))?;
    bytes.truncate(filled);
    Ok(bytes)
}

/// Fills `buffer` from `offset` until it is full or the file ends; the bytes read.
fn read_full(file: &File, buffer: &mut [u8], offset: u64) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match file.read_at(&mut buffer[filled..], offset + filled as u64) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}

/// Searches the file for `needle` (ASCII letters lowercase, not empty), ignoring ASCII case: forward, the first hit that
/// starts at or after `from`; back, the last that starts before it. Either way it wraps once round the file. `None` is
/// no hit, or a search stopped by `stop` (a newer search, or the view closed).
pub fn search(path: &Path, needle: &[u8], from: u64, forward: bool, stop: &AtomicBool) -> Result<Option<u64>, String> {
    let file = File::open(path).map_err(|error| describe(&error, "open", Some(path)))?;
    let size = file.metadata().map_err(|error| describe(&error, "stat", Some(path)))?.len();
    let from = from.min(size);
    let (first, then) = if forward { (from..size, 0..from) } else { (0..from, from..size) };
    let found = match scan(&file, needle, first, forward, stop) {
        Ok(None) => scan(&file, needle, then, forward, stop),
        other => other,
    };
    found.map_err(|error| describe(&error, "read", None))
}

/// The first (forward) or last hit whose start lies in `starts`, read in chunks.
fn scan(file: &File, needle: &[u8], starts: Range<u64>, forward: bool, stop: &AtomicBool) -> io::Result<Option<u64>> {
    let overlap = needle.len() as u64 - 1;
    let mut buffer = Vec::new();
    let mut chunk = |start: u64, end: u64| -> io::Result<Option<u64>> {
        buffer.resize((end - start + overlap) as usize, 0);
        let read = read_full(file, &mut buffer, start)?;
        let hit = find(&buffer[..read], needle, forward).filter(|&at| (at as u64) < end - start);
        Ok(hit.map(|at| start + at as u64))
    };
    let (mut low, mut high) = (starts.start, starts.end);
    while low < high {
        if stop.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let (start, end) =
            if forward { (low, high.min(low + CHUNK)) } else { (high.saturating_sub(CHUNK).max(low), high) };
        if let Some(at) = chunk(start, end)? {
            return Ok(Some(at));
        }
        if forward { low = end } else { high = start }
    }
    Ok(None)
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;
