//! The file finder's index, matching, and result order (Scouter `src/find/search.ts`, `matchEntries`, `compareRelative`).
//! Pure: the index is built by `services/find.rs`, and the app shows what this module returns.

pub mod text;

#[cfg(test)]
mod tests;

use std::cmp::Ordering;

use neo_frizbee::{CaseMatching, Config, Matcher};

use crate::explorer::{Entry, Kind};

/// Entries the index holds before it stops (Scouter `MAX_INDEXED_ENTRIES`).
pub const MAX_INDEXED: usize = 20_000;
/// Rows the name search shows (Scouter `MAX_MATCHES`).
pub const MAX_MATCHES: usize = 200;

/// How a name query matches a path: Scouter's words (`Substring`, the default) or one fuzzy needle (`Fuzzy`, plan-00006
/// D5). `settings.json` `find_mode` picks the start, and `Ctrl+F` in the finder flips it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Matching {
    #[default]
    Substring,
    Fuzzy,
}

impl Matching {
    /// Every mode, in the order the Settings page and the schema list them.
    pub const ALL: [Self; 2] = [Self::Substring, Self::Fuzzy];

    /// The `find_mode` value that names this mode.
    pub fn name(self) -> &'static str {
        match self {
            Self::Substring => "substring",
            Self::Fuzzy => "fuzzy",
        }
    }

    /// The mode a `find_mode` value names; `None` for any other text.
    pub fn named(value: &str) -> Option<Self> {
        match value {
            "substring" => Some(Self::Substring),
            "fuzzy" => Some(Self::Fuzzy),
            _ => None,
        }
    }

    pub fn flipped(self) -> Self {
        match self {
            Self::Substring => Self::Fuzzy,
            Self::Fuzzy => Self::Substring,
        }
    }
}

/// One indexed path: `entry.name` is the path relative to the searched folder, `/` between parts, as Scouter's
/// `IndexedEntry.relative`. The lowercase forms and the UTF-16 length are kept for matching and order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indexed {
    pub entry: Entry,
    lower: String,
    name_lower: String,
    units: usize,
}

impl Indexed {
    pub fn new(entry: Entry) -> Self {
        let lower = entry.name.to_lowercase();
        let name_lower = lower[lower.rfind('/').map_or(0, |at| at + 1)..].to_string();
        let units = entry.name.encode_utf16().count();
        Self { entry, lower, name_lower, units }
    }
}

/// The index of one folder: its paths in walk order, and whether the walk stopped at the limit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Index {
    pub entries: Vec<Indexed>,
    pub truncated: bool,
}

/// One row of a name query: the path, and the character positions in it that a fuzzy match landed on (ascending; empty
/// for a substring match, which marks nothing).
#[derive(Debug, PartialEq, Eq)]
pub struct Found<'a> {
    pub entry: &'a Entry,
    pub matched: Vec<usize>,
}

/// The rows a query shows and how many paths matched in all (the count the footer reports).
#[derive(Debug, PartialEq, Eq)]
pub struct Matches<'a> {
    pub entries: Vec<Found<'a>>,
    pub total: usize,
}

/// Folders first, then names by UTF-16 code unit (Scouter `compareEntryName`). The walk lists in this order, so the
/// index is cut in the same place.
pub fn sort_listing(entries: &mut [Entry]) {
    entries.sort_by(|left, right| {
        let folders = (right.kind == Kind::Directory).cmp(&(left.kind == Kind::Directory));
        folders.then_with(|| left.name.encode_utf16().cmp(right.name.encode_utf16()))
    });
}

/// Each word must appear in the lowercase path. A path whose name holds the first word comes first; within a group,
/// the shorter path (in UTF-16 units) comes first, then code-unit order (Scouter `compareRelative`).
fn compare(left: &Indexed, right: &Indexed) -> Ordering {
    left.units.cmp(&right.units).then_with(|| left.entry.name.encode_utf16().cmp(right.entry.name.encode_utf16()))
}

pub fn matches<'a>(index: &'a [Indexed], query: &str, matching: Matching) -> Matches<'a> {
    match matching {
        Matching::Substring => substring(index, query),
        Matching::Fuzzy => fuzzy(index, query),
    }
}

fn substring<'a>(index: &'a [Indexed], query: &str) -> Matches<'a> {
    let lowered = query.to_lowercase();
    let mut words = lowered.split_whitespace();
    let Some(first) = words.next() else { return Matches { entries: Vec::new(), total: 0 } };
    let words: Vec<&str> = std::iter::once(first).chain(words).collect();

    let mut by_name: Vec<&Indexed> = Vec::new();
    let mut by_path: Vec<&Indexed> = Vec::new();
    for entry in index {
        if !words.iter().all(|word| entry.lower.contains(word)) {
            continue;
        }
        if entry.name_lower.contains(first) {
            by_name.push(entry);
        } else {
            by_path.push(entry);
        }
    }
    let total = by_name.len() + by_path.len();
    by_name.sort_by(|left, right| compare(left, right));
    if by_name.len() >= MAX_MATCHES {
        by_name.truncate(MAX_MATCHES);
        return Matches { entries: by_name.into_iter().map(|found| plain(&found.entry)).collect(), total };
    }
    by_path.sort_by(|left, right| compare(left, right));
    by_path.truncate(MAX_MATCHES - by_name.len());
    by_name.extend(by_path);
    Matches { entries: by_name.into_iter().map(|found| plain(&found.entry)).collect(), total }
}

fn plain(entry: &Entry) -> Found<'_> {
    Found { entry, matched: Vec::new() }
}

/// The query is one needle, spaces included, matched as a subsequence of the path with no typos. The ranking is the
/// match's score, then Scouter's order (`compare`) for equal scores. The case is ignored, as the substring match is.
fn fuzzy<'a>(index: &'a [Indexed], query: &str) -> Matches<'a> {
    if query.is_empty() {
        return Matches { entries: Vec::new(), total: 0 };
    }
    let config = Config::default().casing(CaseMatching::Ignore);
    let names: Vec<&str> = index.iter().map(|found| found.entry.name.as_str()).collect();
    let mut hits = Matcher::new(query, &config).match_list_indices(&names);
    hits.sort_by(|left, right| {
        right.score.cmp(&left.score).then_with(|| compare(&index[left.index as usize], &index[right.index as usize]))
    });
    let total = hits.len();
    let entries = hits
        .into_iter()
        .take(MAX_MATCHES)
        .map(|hit| {
            let entry = &index[hit.index as usize].entry;
            Found { entry, matched: char_positions(&entry.name, &hit.indices) }
        })
        .collect();
    Matches { entries, total }
}

/// The character positions of a match. The matcher reports byte offsets, and a character of several bytes is reported
/// by each of them, so a character counts as matched when one of its bytes is.
fn char_positions(name: &str, bytes: &[u32]) -> Vec<usize> {
    name.char_indices()
        .enumerate()
        .filter(|(_, (start, character))| {
            bytes.iter().any(|&byte| (*start..*start + character.len_utf8()).contains(&(byte as usize)))
        })
        .map(|(at, _)| at)
        .collect()
}

/// The footer's count for a name query (Scouter `file-finder.ts` `showNames`): `no matches`, or the count with
/// `, first 20,000 files` when the index stopped early. A fuzzy query names its mode first, where a narrow sidebar still shows it.
pub fn names_status(total: usize, truncated: bool, matching: Matching) -> String {
    let count = if total == 0 {
        "no matches".to_string()
    } else {
        format!("{} {}", grouped(total), if total == 1 { "match" } else { "matches" })
    };
    let count = if matching == Matching::Fuzzy { format!("fuzzy, {count}") } else { count };
    if truncated { format!("{count}, first {} files", grouped(MAX_INDEXED)) } else { count }
}

/// A count with thousands separators, as `toLocaleString("en-US")` writes it.
pub fn grouped(count: usize) -> String {
    let digits = count.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}
