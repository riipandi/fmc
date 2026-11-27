//! Git signs beside the rendered view's blocks (plan-00010, D6). A block takes the signs of its source lines: a block with
//! an added or changed line gets a bar in its strongest colour (modified over added), a deletion is drawn under the block
//! above it (`▁`), and a deletion above line 1 over the first block (`▔`). Signs on a blank line between blocks belong to
//! no block, so an added blank line draws nothing; a deletion there goes under the block above it.

use std::ops::Range;

use crate::diff::Sign;

/// What one block shows in the sign column: its bar, and whether a deletion sits under it or above it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockMark {
    pub block: usize,
    /// The bar's sign, `Added` or `Modified`, the strongest of the block's changed lines.
    pub bar: Option<Sign>,
    pub below: bool,
    pub above: bool,
}

/// The marks of the blocks that show anything, in document order. `blocks` are the zero-based source lines of each
/// top-level block, and `signs` are the file's signs by zero-based line (`diff/signs.rs`).
pub fn block_marks(blocks: &[Range<usize>], signs: &[(u32, Sign)]) -> Vec<BlockMark> {
    let mut marks: Vec<BlockMark> =
        (0..blocks.len()).map(|block| BlockMark { block, bar: None, below: false, above: false }).collect();
    for &(line, sign) in signs {
        let line = line as usize;
        match sign {
            Sign::Added | Sign::Modified => {
                if let Some(block) = blocks.iter().position(|lines| lines.contains(&line)) {
                    marks[block].bar = Some(stronger(marks[block].bar, sign));
                }
            }
            Sign::DeletedBelow => {
                // The block that holds the line, or else the last block that ends above it.
                let holder = blocks.iter().position(|lines| lines.contains(&line));
                let above = blocks.iter().rposition(|lines| lines.end <= line);
                if let Some(block) = holder.or(above) {
                    marks[block].below = true;
                } else if let Some(first) = marks.first_mut() {
                    first.above = true;
                }
            }
            Sign::DeletedTop => {
                if let Some(first) = marks.first_mut() {
                    first.above = true;
                }
            }
        }
    }
    marks.retain(|mark| mark.bar.is_some() || mark.below || mark.above);
    marks
}

/// The sign a block's bar shows when two of its lines change: modified over added (D6).
fn stronger(current: Option<Sign>, next: Sign) -> Sign {
    match (current, next) {
        (Some(Sign::Modified), _) | (_, Sign::Modified) => Sign::Modified,
        _ => Sign::Added,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Title, two paragraphs, a list: lines 0, 2, 4, and 6 to 7.
    const BLOCKS: [Range<usize>; 4] = [0..1, 2..3, 4..5, 6..8];

    #[test]
    fn an_edited_paragraph_gets_a_modified_bar() {
        let marks = block_marks(&BLOCKS, &[(4, Sign::Modified)]);
        assert_eq!(marks, vec![BlockMark { block: 2, bar: Some(Sign::Modified), below: false, above: false }]);
    }

    #[test]
    fn an_added_section_gets_an_added_bar_on_each_block_it_touches() {
        let marks = block_marks(&BLOCKS, &[(2, Sign::Added), (6, Sign::Added), (7, Sign::Added)]);
        assert_eq!(
            marks.iter().map(|mark| (mark.block, mark.bar)).collect::<Vec<_>>(),
            vec![(1, Some(Sign::Added)), (3, Some(Sign::Added)),]
        );
    }

    #[test]
    fn a_block_takes_its_strongest_change() {
        let marks = block_marks(&BLOCKS, &[(6, Sign::Added), (7, Sign::Modified)]);
        assert_eq!(marks[0].bar, Some(Sign::Modified));
        let marks = block_marks(&BLOCKS, &[(6, Sign::Modified), (7, Sign::Added)]);
        assert_eq!(marks[0].bar, Some(Sign::Modified));
    }

    #[test]
    fn a_deletion_after_a_block_is_drawn_under_it() {
        // Deleted after line 4 (the paragraph's own line): the paragraph's block takes the mark under it.
        let marks = block_marks(&BLOCKS, &[(4, Sign::DeletedBelow)]);
        assert_eq!(marks, vec![BlockMark { block: 2, bar: None, below: true, above: false }]);
    }

    #[test]
    fn a_deletion_on_a_blank_line_goes_under_the_block_above() {
        // Line 1 is blank, between the title (line 0) and the first paragraph.
        let marks = block_marks(&BLOCKS, &[(1, Sign::DeletedBelow)]);
        assert_eq!(marks, vec![BlockMark { block: 0, bar: None, below: true, above: false }]);
    }

    #[test]
    fn a_deletion_above_line_one_goes_over_the_first_block() {
        let marks = block_marks(&BLOCKS, &[(0, Sign::DeletedTop)]);
        assert_eq!(marks, vec![BlockMark { block: 0, bar: None, below: false, above: true }]);
    }

    #[test]
    fn a_blank_line_added_between_blocks_draws_nothing() {
        assert!(block_marks(&BLOCKS, &[(3, Sign::Added)]).is_empty());
    }

    #[test]
    fn no_signs_no_marks() {
        assert!(block_marks(&BLOCKS, &[]).is_empty());
        assert!(block_marks(&[], &[(0, Sign::Added)]).is_empty());
    }
}
