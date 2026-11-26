//! Patch text for the changes screen (Scouter `src/diff/patch.ts`): the notices, the 1 MiB and 4 MiB limits, the split of
//! one whole-tree diff into files, and the patch of an untracked file. Pure: `services/git.rs` runs git and reads files.

use std::collections::HashMap;

use super::{Change, ChangeKind};
use crate::reader::document::Document;
use crate::text::display_text;

/// One file's patch may be this long, counted in UTF-16 units as Scouter counts (`src/diff/patch.ts:5`).
pub const PATCH_LIMIT: usize = 1024 * 1024;
/// The whole-tree diff is read up to this many bytes, then split by file (`src/diff/patch.ts:6`).
pub const ALL_LIMIT: usize = 4 * 1024 * 1024;

/// The line ends a regex `m` flag or JavaScript's `.` respects; Scouter's chunk and binary checks split on them.
const LINE_ENDS: [char; 4] = ['\n', '\r', '\u{2028}', '\u{2029}'];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Patch {
    Text { text: String, hunks: usize, added: usize, removed: usize },
    Notice(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePatch {
    pub change: Change,
    pub patch: Patch,
}

pub fn binary() -> Patch {
    Patch::Notice("Binary file: no preview.".into())
}

pub fn too_large() -> Patch {
    Patch::Notice("This diff is too large to display (limit: 1 MiB).".into())
}

pub fn not_shown() -> Patch {
    Patch::Notice("Too much to show at once. Press a to view this file alone.".into())
}

/// The length Scouter's `string.length` gives: UTF-16 code units, which the limits are counted in.
pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Scouter's `toPatch` (`src/diff/patch.ts:75-87`): a diff over the limit is too large, a binary diff is a notice, and
/// a diff with no hunk says why it is empty.
pub fn from_diff(change: &Change, raw: &str) -> Patch {
    if utf16_len(raw) > PATCH_LIMIT {
        return too_large();
    }
    let body = display_text(raw).replace('\t', "    ");
    if body.split(LINE_ENDS).any(is_binary_line) {
        return binary();
    }
    let (hunks, added, removed) = count_lines(&body);
    if hunks == 0 {
        return Patch::Notice(empty_message(change));
    }
    Patch::Text { text: body, hunks, added, removed }
}

/// `Binary files a and b differ`: the regex `/^Binary files .* differ$/` needs both ends, so a line shorter than the
/// two ends together does not match.
fn is_binary_line(line: &str) -> bool {
    line.starts_with("Binary files ") && line.ends_with(" differ") && line.len() >= "Binary files  differ".len()
}

fn count_lines(patch: &str) -> (usize, usize, usize) {
    let (mut hunks, mut added, mut removed) = (0, 0, 0);
    for line in patch.split('\n') {
        if line.starts_with("@@") {
            hunks += 1;
        } else if hunks > 0 && line.starts_with('+') {
            added += 1;
        } else if hunks > 0 && line.starts_with('-') {
            removed += 1;
        }
    }
    (hunks, added, removed)
}

fn empty_message(change: &Change) -> String {
    match change.kind {
        ChangeKind::Renamed => {
            format!("Renamed from {} with no content changes.", change.old_path.as_deref().unwrap_or("another path"))
        }
        ChangeKind::Conflict => "Unmerged: resolve the conflict to see a diff.".into(),
        _ => "No line changes (permissions or file type only).".into(),
    }
}

/// Scouter's `splitPatch` (`src/diff/patch.ts:108-117`): the chunk of each file, by path. A cut diff drops its last
/// chunk, which may be partly read; the last chunk wins when two share a path.
pub fn split(text: &str, cut: bool) -> HashMap<String, &str> {
    let complete = if cut { text.rfind("\ndiff --git ").map_or("", |at| &text[..=at]) } else { text };
    let mut chunks = HashMap::new();
    for chunk in chunks_of(complete) {
        if let Some(path) = chunk_path(chunk) {
            chunks.insert(path, chunk);
        }
    }
    chunks
}

/// The text cut before each line that starts with `diff --git `, the first chunk starting at zero.
fn chunks_of(text: &str) -> Vec<&str> {
    let mut starts = vec![0];
    starts.extend(
        text.match_indices("diff --git ")
            .map(|(at, _)| at)
            .filter(|&at| text[..at].chars().next_back().is_some_and(|end| LINE_ENDS.contains(&end))),
    );
    starts.push(text.len());
    starts.windows(2).map(|pair| &text[pair[0]..pair[1]]).collect()
}

/// The path a chunk is for (Scouter `chunkPath`, `src/diff/patch.ts:119-142`): the rename target, then the new side and
/// the old side of the file lines, then the header. Each pattern takes the first line that has it.
fn chunk_path(chunk: &str) -> Option<String> {
    let lines = || chunk.split(LINE_ENDS);
    if let Some(path) = lines().find_map(|line| line.strip_prefix("rename to ").filter(|path| !path.is_empty())) {
        return Some(path.to_string());
    }
    for prefix in ["+++ b/", "--- a/"] {
        if let Some(path) = lines().find_map(|line| line.strip_prefix(prefix)) {
            // The pattern `(.*?)\t?$` leaves out one tab that ends the line.
            return Some(path.strip_suffix('\t').unwrap_or(path).to_string());
        }
    }
    let header = lines().find_map(|line| line.strip_prefix("diff --git a/").filter(|header| !header.is_empty()))?;
    header_path(header)
}

/// A header names its path twice: `<path> b/<path>` after the `a/`. The check works in UTF-16 units, as Scouter's
/// string lengths do, so a non-ASCII path splits at the same place.
fn header_path(header: &str) -> Option<String> {
    let units: Vec<u16> = header.encode_utf16().collect();
    let length = units.len().checked_sub(3)?;
    if length % 2 != 0 {
        return None;
    }
    let half = length / 2;
    let separator: Vec<u16> = " b/".encode_utf16().collect();
    let names_twice = units[..half] == units[half + 3..];
    (units[half..half + 3] == separator[..] && names_twice).then(|| String::from_utf16_lossy(&units[..half]))
}

/// Scouter's `untrackedPatch` (`src/diff/patch.ts:158-186`): a new file as one hunk of additions. `read` is the file's
/// document, or the reason it could not be read.
pub fn from_file(path: &str, read: Result<Document, String>) -> Patch {
    let document = match read {
        Ok(document) => document,
        Err(message) => return Patch::Notice(format!("New file: {}", display_text(&message))),
    };
    if document.truncated {
        return Patch::Notice("New file: File is too large (limit: 1 MiB).".into());
    }
    if document.lines == 0 {
        return Patch::Notice("New empty file.".into());
    }
    let safe = display_text(path).replace('\t', "    ");
    let lines: Vec<&str> = document.content().split('\n').collect();
    let mut text = format!(
        "diff --git a/{safe} b/{safe}\nnew file mode 100644\n--- /dev/null\n+++ b/{safe}\n@@ -0,0 +1,{} @@\n",
        lines.len()
    );
    for line in &lines {
        text.push('+');
        text.push_str(line);
        text.push('\n');
    }
    if utf16_len(&text) > PATCH_LIMIT {
        return too_large();
    }
    Patch::Text { text, hunks: 1, added: lines.len(), removed: 0 }
}
