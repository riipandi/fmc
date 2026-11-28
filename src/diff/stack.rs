//! The stacked diff's facts (Scouter `src/diff/diff-blocks.ts`): the line-number width every block shares, the totals
//! the footer shows, the hunk at a scroll row, and the block at one. Pure: the view measures the rows and passes them in.

use unicode_width::UnicodeWidthStr;

use super::{FilePatch, Patch};

/// The width of the line numbers every block shares (Scouter `maxGutterDigits`, `src/diff/diff-blocks.ts:55-70`): the
/// digits of the highest line a hunk header reaches, and at least two. One width for all blocks keeps their code aligned.
pub fn gutter_digits(files: &[FilePatch]) -> usize {
    let mut highest = 1;
    for file in files {
        let Patch::Text { text, .. } = &file.patch else { continue };
        for line in text.lines() {
            if let Some((old_end, new_end)) = hunk_ends(line) {
                highest = highest.max(old_end).max(new_end);
            }
        }
    }
    highest.to_string().len().max(2)
}

/// The lines added and removed and the hunks, over the text patches (Scouter `totals`, `src/diff/diff-blocks.ts:182-205`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub added: usize,
    pub removed: usize,
    pub hunks: usize,
}

/// `None` when no file has text to count, as Scouter's footer shows no counts then.
pub fn totals(files: &[FilePatch]) -> Option<Totals> {
    let mut totals = Totals::default();
    let mut any_text = false;
    for file in files {
        if let Patch::Text { hunks, added, removed, .. } = &file.patch {
            any_text = true;
            totals.added += added;
            totals.removed += removed;
            totals.hunks += hunks;
        }
    }
    any_text.then_some(totals)
}

/// The hunk at scroll row `top` (Scouter `currentHunkIndex`, `src/diff/diff-blocks.ts:572-591`): one more than the index
/// of the last hunk that starts at or above `top`, or the first while the view is above them all. `rows` are the
/// hunks' rows in the stack, in order; `None` when there are no hunks.
pub fn active_hunk(rows: &[usize], top: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    Some(rows.iter().rposition(|&row| row <= top).unwrap_or(0) + 1)
}

/// The block at scroll row `top` (Scouter `followScroll`, `src/diff/diff-blocks.ts:382-386`): the last block whose top is
/// at or above the row, or the first block. `tops` are the blocks' rows in the stack, in order.
pub fn block_at(tops: &[usize], top: usize) -> Option<usize> {
    if tops.is_empty() {
        return None;
    }
    Some(tops.iter().rposition(|&block_top| block_top <= top).unwrap_or(0))
}

/// The new-file line each hunk of `text` starts at, in order, and at least line 1 (Scouter `hunkStartLines`,
/// `src/diff/diff-blocks.ts`, the `@@ -a[,b] +c` regex).
pub fn hunk_starts(text: &str) -> Vec<usize> {
    text.lines().filter_map(|line| hunk_header(line).map(|(_, _, new_start, _)| new_start.max(1) as usize)).collect()
}

/// The line numbers a hunk header's ranges reach: the first line past the old range and past the new one (Scouter's
/// `maxGutterDigits` regex, `@@ -a[,b] +c[,d] @@`, with a missing count as 1).
fn hunk_ends(line: &str) -> Option<(u64, u64)> {
    let (old_start, old_count, new_start, new_count) = hunk_header(line)?;
    Some((old_start + old_count, new_start + new_count))
}

/// `(old start, old count, new start, new count)` of a hunk header line.
fn hunk_header(line: &str) -> Option<(u64, u64, u64, u64)> {
    let rest = line.strip_prefix("@@ -")?;
    let (old_start, old_count, rest) = range(rest)?;
    let (new_start, new_count, _) = range(rest.strip_prefix(" +")?)?;
    Some((old_start, old_count, new_start, new_count))
}

/// `start[,count]` at the front of `text`, with the rest of `text` after it.
fn range(text: &str) -> Option<(u64, u64, &str)> {
    let end = text.find(|c: char| !c.is_ascii_digit()).unwrap_or(text.len());
    let start = text[..end].parse().ok()?;
    let rest = &text[end..];
    let Some(counted) = rest.strip_prefix(',') else { return Some((start, 1, rest)) };
    let end = counted.find(|c: char| !c.is_ascii_digit()).unwrap_or(counted.len());
    Some((start, counted[..end].parse().ok()?, &counted[end..]))
}

/// The display columns of a diff's widest line, after its sign column. Only the hunk lines are drawn, so the file header
/// before the first `@@` is not counted. shortcut: a tab counts as four columns, not the native buffer's tab stop, so a
/// tab that lands on another stop is off by a few columns; upgrade when the tab width is read from the buffer.
pub fn widest_line(text: &str) -> usize {
    let mut in_hunk = false;
    let mut widest = 0;
    for line in text.lines() {
        if line.starts_with("@@") {
            in_hunk = true;
        } else if in_hunk && !line.starts_with('\\') {
            let code = line.get(1..).unwrap_or("");
            widest = widest.max(UnicodeWidthStr::width(code.replace('\t', "    ").as_str()));
        }
    }
    widest
}

/// The columns an unwrapped stack takes: the pane, or the widest need of the shown diffs. A folded diff or a notice
/// passes `None`, so it does not widen the stack.
pub fn stack_columns(pane: usize, shown: impl IntoIterator<Item = Option<usize>>) -> usize {
    shown.into_iter().flatten().fold(pane, usize::max)
}

#[cfg(test)]
#[path = "stack_tests.rs"]
mod tests;
