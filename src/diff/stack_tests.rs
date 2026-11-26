//! The stacked diff's gutter width, totals, active hunk, and followed block, on values.

use super::*;
use crate::diff::{Change, Patch};

fn text(path: &str, text: &str, hunks: usize, added: usize, removed: usize) -> FilePatch {
    let patch = Patch::Text { text: text.into(), hunks, added, removed };
    FilePatch { change: Change::untracked(path), patch }
}

fn notice(path: &str) -> FilePatch {
    FilePatch { change: Change::untracked(path), patch: Patch::Notice("Binary file: no preview.".into()) }
}

#[test]
fn the_gutter_is_as_wide_as_the_highest_line_of_any_hunk_and_never_under_two() {
    let small = text("a.rs", "@@ -1,2 +1,2 @@\n-a\n+b\n", 1, 1, 1);
    assert_eq!(gutter_digits(&[small, notice("b.bin")]), 2);
    let wide = text("c.rs", "@@ -998,12 +998,14 @@\n a\n", 1, 2, 0);
    assert_eq!(gutter_digits(&[text("d.rs", "@@ -1 +1 @@\n", 1, 0, 0), wide]), 4, "998 + 14 is 1012");
    assert_eq!(gutter_digits(&[notice("e.bin")]), 2);
}

#[test]
fn a_hunk_header_without_counts_reaches_one_line() {
    assert_eq!(hunk_ends("@@ -12 +12 @@ fn main"), Some((13, 13)));
    assert_eq!(hunk_ends("@@ -3,4 +5,0 @@"), Some((7, 5)));
    assert_eq!(hunk_ends("@@ -x +1 @@"), None);
    assert_eq!(hunk_ends("diff --git a/a b/a"), None);
}

#[test]
fn totals_sum_the_text_patches_and_are_absent_without_any() {
    let files = [text("a.rs", "", 2, 5, 1), notice("b.bin"), text("c.rs", "", 1, 0, 4)];
    assert_eq!(totals(&files), Some(Totals { added: 5, removed: 5, hunks: 3 }));
    assert_eq!(totals(&[notice("b.bin")]), None);
    assert_eq!(totals(&[]), None);
}

#[test]
fn the_active_hunk_is_the_last_one_above_the_scroll_row() {
    let rows = [3, 10, 20];
    assert_eq!(active_hunk(&rows, 0), Some(1), "above the first hunk, the first is active");
    assert_eq!(active_hunk(&rows, 10), Some(2));
    assert_eq!(active_hunk(&rows, 25), Some(3));
    assert_eq!(active_hunk(&[], 25), None);
}

#[test]
fn the_followed_block_is_the_last_whose_top_is_above_the_scroll_row() {
    let tops = [0, 5, 9];
    assert_eq!(block_at(&tops, 4), Some(0));
    assert_eq!(block_at(&tops, 5), Some(1));
    assert_eq!(block_at(&tops, 500), Some(2));
    assert_eq!(block_at(&[3, 8], 1), Some(0), "above every block, the first is followed");
    assert_eq!(block_at(&[], 1), None);
}

#[test]
fn an_unfolded_diff_widens_the_stack_and_a_folded_diff_or_a_notice_does_not() {
    assert!(widens_stack([(false, true)]));
    assert!(widens_stack([(true, true), (false, true)]));
    assert!(!widens_stack([(true, true), (false, false)]), "a notice is narrower than the pane");
    assert!(!widens_stack([(true, true)]));
    assert!(!widens_stack([]));
}
