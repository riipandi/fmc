//! Text search for the finder (Scouter `src/find/search.ts`, `hitsIn`, `searchText`, `snippetOf`): a case-insensitive
//! substring per line, with the limits that stop a scan, and the snippet each hit shows. Pure: the caller reads the
//! files and feeds their text in.

use std::path::{Path, PathBuf};

use super::grouped;

/// Hits a scan keeps (Scouter `MAX_TEXT_HITS`).
pub const MAX_TEXT_HITS: usize = 200;
/// Hits one file adds (Scouter `MAX_HITS_PER_FILE`).
pub const MAX_HITS_PER_FILE: usize = 10;
/// UTF-16 units a scan reads before it stops (Scouter `MAX_SCANNED_CHARS`, 64 MiB).
pub const MAX_SCANNED_UNITS: usize = 64 * 1024 * 1024;
/// Characters a text search needs before it starts (Scouter `MIN_TEXT_QUERY`), counted in UTF-16 units as JavaScript
/// counts `query.length`.
pub const MIN_QUERY: usize = 2;
/// The wait after the last key before a text search starts (Scouter `SCAN_DELAY_MS`).
pub const SCAN_DELAY_MS: u64 = 150;
/// Units before the match a snippet keeps, and the snippet's whole length (Scouter `SNIPPET_LEAD`, `SNIPPET_LENGTH`).
const SNIPPET_LEAD: usize = 40;
const SNIPPET_LENGTH: usize = 300;

/// Whether a query starts a text search: two or more UTF-16 units.
pub fn wants_scan(query: &str) -> bool {
    query.encode_utf16().count() >= MIN_QUERY
}

/// The number of the latest search. A search or a debounce whose number is no longer the latest is dropped, as Scouter
/// drops a scan when the query changes (`restartScan`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Generation(u64);

impl Generation {
    /// Starts a new search: the number it gets is the only one that is current.
    pub fn bump(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }

    pub fn is_current(&self, number: u64) -> bool {
        self.0 == number
    }
}

/// A line's text around the match: the text before it, the match, and the text after it, as the row shows them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snippet {
    pub before: String,
    pub matched: String,
    pub after: String,
}

/// One line that contains the query: the file, its line (one-based), and the snippet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub path: PathBuf,
    pub relative: String,
    pub line: usize,
    pub snippet: Snippet,
}

/// What a scan has found so far: its hits, the files that had one, how far it got, and whether it was cut short.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub hits: Vec<Hit>,
    pub files: usize,
    pub scanned: usize,
    pub total: usize,
    pub truncated: bool,
    pub done: bool,
}

/// A scan in progress over `total` files. The caller reads each file and passes its text to [`Scan::file`], or says
/// it could not read it with [`Scan::skipped`], until [`Scan::stopped`] says the limits are reached.
pub struct Scan {
    needle: String,
    hits: Vec<Hit>,
    found: usize,
    scanned: usize,
    total: usize,
    units: usize,
    hidden: bool,
    stopped: bool,
}

impl Scan {
    pub fn new(query: &str, total: usize) -> Self {
        Self {
            needle: query.to_lowercase(),
            hits: Vec::new(),
            found: 0,
            scanned: 0,
            total,
            units: 0,
            hidden: false,
            stopped: false,
        }
    }

    /// The limits are reached: the caller stops reading.
    pub fn stopped(&self) -> bool {
        self.stopped
    }

    /// A file that could not be read or was skipped (an image, a binary): it counts as scanned.
    pub fn skipped(&mut self) {
        self.scanned += 1;
    }

    /// Searches one file's text. Once the scan has stopped, the file counts as scanned and is not read.
    pub fn file(&mut self, path: &Path, relative: &str, text: &str) {
        self.scanned += 1;
        if self.stopped {
            return;
        }
        self.units += text.encode_utf16().count();
        let room = MAX_HITS_PER_FILE.min(MAX_TEXT_HITS - self.hits.len());
        let (hits, more) = hits_in(path, relative, text, &self.needle, room);
        if !hits.is_empty() {
            self.found += 1;
        }
        self.hits.extend(hits);
        self.hidden |= more;
        self.stopped = self.hits.len() >= MAX_TEXT_HITS || self.units > MAX_SCANNED_UNITS;
    }

    pub fn progress(&self, done: bool) -> Progress {
        Progress {
            hits: self.hits.clone(),
            files: self.found,
            scanned: self.scanned,
            total: self.total,
            truncated: self.hidden || self.stopped,
            done,
        }
    }
}

/// The footer's text for a scan (Scouter `textStatus`): the progress while it runs, then the count, `N+` when hits
/// were left out, or `first 200 matches` at the limit.
pub fn text_status(progress: &Progress) -> String {
    if !progress.done {
        return format!("searching {}/{}", grouped(progress.scanned), grouped(progress.total));
    }
    if progress.hits.is_empty() {
        return "no matches".into();
    }
    let count =
        format!("{} {}", grouped(progress.hits.len()), if progress.hits.len() == 1 { "match" } else { "matches" });
    if !progress.truncated {
        count
    } else if progress.hits.len() >= MAX_TEXT_HITS {
        format!("first {MAX_TEXT_HITS} matches")
    } else {
        format!("{count}+")
    }
}

/// The hits in one text, at most `limit`, and whether more lines matched (Scouter `hitsIn`): one hit per line.
pub fn hits_in(path: &Path, relative: &str, text: &str, needle: &str, limit: usize) -> (Vec<Hit>, bool) {
    let mut hits = Vec::new();
    if needle.is_empty() {
        return (hits, false);
    }
    for (index, line) in text.split('\n').enumerate() {
        let Some(snippet) = find_in_line(line, needle) else { continue };
        if hits.len() >= limit {
            return (hits, true);
        }
        hits.push(Hit { path: path.to_path_buf(), relative: relative.to_string(), line: index + 1, snippet });
    }
    (hits, false)
}

/// The first match in one line, found in lowercase and cut from the line itself. Offsets are UTF-16 units, as
/// Scouter's are, and a lowercase letter that grows (`İ`) maps back to the character it came from.
fn find_in_line(line: &str, needle: &str) -> Option<Snippet> {
    let raw: Vec<u16> = line.encode_utf16().collect();
    let mut lower: Vec<u16> = Vec::with_capacity(raw.len());
    // For each lowercase unit, the raw unit its character starts at; the extra last entry is the line's end.
    let mut origin: Vec<usize> = Vec::with_capacity(raw.len() + 1);
    let mut at = 0;
    for character in line.chars() {
        let start = at;
        at += character.len_utf16();
        for lowered in character.to_lowercase() {
            let mut buffer = [0u16; 2];
            for unit in lowered.encode_utf16(&mut buffer).iter() {
                lower.push(*unit);
                origin.push(start);
            }
        }
    }
    origin.push(raw.len());
    let needle: Vec<u16> = needle.encode_utf16().collect();
    let found = lower.windows(needle.len()).position(|window| window == needle.as_slice())?;
    let start = origin[found];
    let end = origin[found + needle.len()];
    Some(snippet(&raw, start, end - start))
}

/// The snippet around the match at `start` (Scouter `snippetOf`): up to 40 units before it, from the first non-blank
/// unit when the match is nearer the start, and 300 units in all.
fn snippet(raw: &[u16], start: usize, length: usize) -> Snippet {
    let indent: usize = char::decode_utf16(raw.iter().copied())
        .map(|unit| unit.unwrap_or(char::REPLACEMENT_CHARACTER))
        .take_while(|c| c.is_whitespace())
        .map(char::len_utf16)
        .sum();
    let from = if start > SNIPPET_LEAD { start - SNIPPET_LEAD } else { indent.min(start) };
    let stop = (from + SNIPPET_LENGTH).min(raw.len());
    let matched_end = (start + length).min(stop);
    let text = |range: std::ops::Range<usize>| String::from_utf16_lossy(&raw[range]);
    Snippet { before: text(from..start), matched: text(start..matched_end), after: text(matched_end..stop) }
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
