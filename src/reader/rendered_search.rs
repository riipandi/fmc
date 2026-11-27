//! Search in the rendered Markdown view (plan-00010, D1 to D4). The text searched is the text OpenTUI draws: the spans of
//! each paragraph, heading, and table cell, and each code block's text, so `Harry Potter` finds `**Harry** Potter`.
//! A match is an occurrence, not a line as in the code view, and occurrences count in document order.

use std::ops::Range;

use super::search::match_ranges;

/// Which piece a mark belongs to, counted in document order as OpenTUI builds the view: a prose piece (a paragraph,
/// heading, or table cell) or a code block the renderer draws. A fence the renderer does not draw has no slot (D5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Prose(usize),
    Code(usize),
}

/// A piece of rendered text: the top-level block it is in, its slot, and its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub block: usize,
    pub slot: Option<Slot>,
    pub text: String,
}

/// One occurrence: its block, its slot, its byte range in the piece, and its byte offset in the block's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub block: usize,
    pub slot: Option<Slot>,
    pub at: Range<usize>,
    pub offset: usize,
}

/// The rendered text of a document, one piece per leaf, in document order. A block's text is its pieces' texts in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rendered {
    pub pieces: Vec<Piece>,
}

impl Rendered {
    /// The byte length of a block's text (the sum of its pieces).
    pub fn block_len(&self, block: usize) -> usize {
        self.pieces.iter().filter(|piece| piece.block == block).map(|piece| piece.text.len()).sum()
    }

    /// Every occurrence of `query` in document order, ignoring case (the code view's rule), with its offset in the block.
    pub fn hits(&self, query: &str) -> Vec<Hit> {
        let mut out = Vec::new();
        let (mut block, mut offset) = (usize::MAX, 0);
        for piece in &self.pieces {
            if piece.block != block {
                (block, offset) = (piece.block, 0);
            }
            for at in match_ranges(&piece.text, query) {
                out.push(Hit { block, slot: piece.slot, offset: offset + at.start, at });
            }
            offset += piece.text.len();
        }
        out
    }
}

/// The row inside a block of `rows` rows that an offset falls on. OpenTUI does not expose a byte's row, so the row is
/// the offset's share of the block's text, the estimate plan-00010 D4 names.
pub fn row_within(offset: usize, len: usize, rows: u32) -> u32 {
    if len == 0 || rows == 0 {
        return 0;
    }
    let row = offset as u64 * u64::from(rows) / len as u64;
    (row as u32).min(rows - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(block: usize, slot: Option<Slot>, text: &str) -> Piece {
        Piece { block, slot, text: text.to_owned() }
    }

    #[test]
    fn occurrences_count_across_concealed_markers() {
        // `**Harry** Potter` renders as `Harry Potter`: the markers are not in the piece's text.
        let rendered = Rendered { pieces: vec![piece(0, Some(Slot::Prose(0)), "Harry Potter and Harry Potter")] };
        assert_eq!(rendered.hits("harry potter").len(), 2);
        assert_eq!(rendered.hits("HARRY").len(), 2, "case is ignored, as in the code view");
        assert!(rendered.hits("").is_empty());
        assert!(rendered.hits("Dumbledore").is_empty());
    }

    #[test]
    fn offsets_run_through_the_pieces_of_a_block() {
        let rendered = Rendered {
            pieces: vec![
                piece(0, Some(Slot::Prose(0)), "Gryffindor"),
                piece(0, Some(Slot::Prose(1)), "Hermione"),
                piece(1, Some(Slot::Prose(2)), "Hermione"),
            ],
        };
        let hits = rendered.hits("hermione");
        assert_eq!(hits.iter().map(|hit| (hit.block, hit.offset)).collect::<Vec<_>>(), vec![(0, 10), (1, 0)]);
        assert_eq!(rendered.block_len(0), 18);
        assert_eq!(hits[0].slot, Some(Slot::Prose(1)));
    }

    #[test]
    fn a_plain_fence_is_counted_without_a_slot() {
        let rendered = Rendered { pieces: vec![piece(3, None, "let vault = 713;")] };
        let hits = rendered.hits("vault");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].slot, None);
    }

    #[test]
    fn a_row_is_the_offset_share_of_the_block_rows() {
        assert_eq!(row_within(0, 100, 10), 0);
        assert_eq!(row_within(50, 100, 10), 5);
        assert_eq!(row_within(99, 100, 10), 9);
        assert_eq!(row_within(500, 100, 10), 9, "an offset past the text stays in the block");
        assert_eq!(row_within(3, 0, 10), 0);
    }
}
