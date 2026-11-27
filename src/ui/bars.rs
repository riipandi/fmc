//! The compact tier's bottom bar (plan-00005, D3): equal segments, each naming the key its tap presses, so a tap runs the
//! same action as the key (plan-00004, D2; `app/keys.rs` `hint_presses`).

use std::ops::Range;

use fmc::layout::Page;
use fmc::theme::Palette;

use super::styled::Seg;

/// The `·` between two segments, drawn in the muted colour.
const SEPARATOR: &str = "·";

/// One segment of the bottom bar: the word it shows, and the key its tap presses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub label: String,
    pub key: &'static str,
}

fn segment(label: &str, key: &'static str) -> Segment {
    Segment { label: label.into(), key }
}

/// The segments of a page's bottom bar (D3). `body` is the changes body shown on the reader page, which keeps the
/// changes labels. `All` names the scope on screen, and `Copy` takes the search's place while text is selected
/// (architecture.md, record 7 and the phone rule P8).
pub fn page_segments(page: Page, body: bool, all_files: bool, selection: bool) -> Vec<Segment> {
    let changes =
        || vec![segment("Back", "q"), segment(if all_files { "All" } else { "One" }, "a"), segment("Split", "s")];
    match page {
        Page::Files => vec![segment("Find", "/"), segment("Changes", "d"), segment("Keys", "?")],
        Page::Changes => changes(),
        Page::Reader if body => changes(),
        Page::Reader => {
            let middle = if selection { segment("Copy", "y") } else { segment("Search", "/") };
            vec![segment("Back", "q"), middle, segment("Keys", "?")]
        }
        Page::Keys | Page::Find => vec![segment("Back", "q")],
    }
}

/// The columns of each cell in a bar of `count` segments: equal widths, with one column for each `·` between them, so
/// the cells and the separators fill `columns`. A tap finds its segment by the cell its column falls in.
pub fn cells(columns: usize, count: usize) -> Vec<Range<usize>> {
    if count == 0 {
        return Vec::new();
    }
    let room = columns.saturating_sub(count - 1);
    let (each, extra) = (room / count, room % count);
    let mut at = 0;
    (0..count)
        .map(|i| {
            let width = each + usize::from(i < extra);
            let cell = at..at + width;
            at += width + 1;
            cell
        })
        .collect()
}

/// A label centred in `width` columns, cut when it is wider.
fn centred(label: &str, width: usize) -> String {
    let label: String = label.chars().take(width).collect();
    format!("{label:^width$}")
}

/// The bottom bar's segments as styled text, each label centred in its cell, with the cell and key each tap finds.
pub fn render(segments: &[Segment], columns: usize, p: &Palette) -> (Vec<Seg>, Vec<(Range<usize>, &'static str)>) {
    let cells = cells(columns, segments.len());
    let mut drawn = Vec::new();
    for (i, (segment, cell)) in segments.iter().zip(&cells).enumerate() {
        if i > 0 {
            drawn.push(Seg::new(SEPARATOR, p.muted));
        }
        drawn.push(Seg::new(centred(&segment.label, cell.len()), p.text));
    }
    (drawn, cells.into_iter().zip(segments.iter().map(|segment| segment.key)).collect())
}

/// The bottom bar while a prompt is open (D3): `Cancel` in the first cell, then the prompt's own text, which the bar
/// clips at its width. The cell's tap presses `Esc`.
pub fn prompt(text: &[Seg], columns: usize, p: &Palette) -> (Vec<Seg>, Vec<(Range<usize>, &'static str)>) {
    let cancel = cells(columns, 3)[0].clone();
    let mut drawn =
        vec![Seg::new(centred("Cancel", cancel.len()), p.text), Seg::new(SEPARATOR, p.muted), Seg::plain(" ")];
    drawn.extend(text.iter().cloned());
    (drawn, vec![(cancel, "Esc")])
}

#[cfg(test)]
#[path = "bars_tests.rs"]
mod tests;
