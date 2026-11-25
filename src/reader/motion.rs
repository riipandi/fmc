//! The reader's motions (Scouter `src/reader/reader-motion.ts`): paragraphs by blank lines, the line a count or `zz`
//! scrolls to, and the paging amounts. Lines are zero-based; the scroll position is in rows.

/// A page is 0.9 of the viewport, a half page 0.5, as Scouter's `scrollBy(…, "viewport")` (`src/core/keys.ts:24-35`).
pub const PAGE: f64 = 0.9;
pub const HALF_PAGE: f64 = 0.5;

/// Columns the pan keys move by (`src/core/keys.ts`, `PAN_COLUMNS`).
pub const PAN_COLUMNS: i32 = 4;

/// The lines that hold only spaces, tabs, or carriage returns: the paragraph breaks (`blankLinesOf`).
pub fn blank_lines(text: &str) -> Vec<usize> {
    text.split('\n')
        .enumerate()
        .filter(|(_, line)| line.chars().all(|c| matches!(c, ' ' | '\t' | '\r')))
        .map(|(index, _)| index)
        .collect()
}

/// The top line of the next paragraph, or of the previous one, from `top` (`moveByParagraph`). `None` means no
/// paragraph is left that way: the caller goes to the end, or to the top.
pub fn paragraph_target(blanks: &[usize], top: usize, forward: bool) -> Option<usize> {
    if forward {
        blanks.iter().copied().find(|&line| line > top)
    } else {
        blanks.iter().copied().rfind(|&line| line < top)
    }
}

/// The scroll position that puts `line` at the top of a `viewport`-row view, after Scouter's `scrollToLine`, which
/// leaves half the view above it: `line - floor(viewport / 2)`, never below 0.
pub fn line_top(line: usize, viewport: usize) -> usize {
    line.saturating_sub(viewport / 2)
}

/// The line `zz` centres: the line at the middle of the view (`centre`, with no search match).
pub fn centre_line(top: usize, viewport: usize) -> usize {
    top + viewport / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_hold_only_spaces_tabs_and_returns() {
        assert_eq!(blank_lines("Hermione\n\n  \t\r\nGranger\n"), vec![1, 2, 4]);
        // An empty text is one blank line, as Scouter counts it.
        assert_eq!(blank_lines(""), vec![0]);
    }

    #[test]
    fn paragraphs_step_over_the_blank_lines() {
        let blanks = [3, 7, 12];
        assert_eq!(paragraph_target(&blanks, 0, true), Some(3));
        assert_eq!(paragraph_target(&blanks, 3, true), Some(7));
        assert_eq!(paragraph_target(&blanks, 12, true), None);
        assert_eq!(paragraph_target(&blanks, 12, false), Some(7));
        assert_eq!(paragraph_target(&blanks, 0, false), None);
    }

    #[test]
    fn a_line_lands_with_half_a_view_above_it() {
        assert_eq!(line_top(119, 20), 109);
        assert_eq!(line_top(3, 20), 0);
        assert_eq!(centre_line(40, 21), 50);
    }
}
