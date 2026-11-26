use super::*;
use crate::diff::hunk_starts;

fn hunk(row: usize, file: usize, line: usize) -> Hunk {
    Hunk { row, file, line }
}

#[test]
fn a_hunk_starts_at_its_new_line_and_never_before_line_one() {
    let text = "@@ -10,3 +12,4 @@ fn main\n a\n+b\n@@ -40 +0,0 @@\n-c\n";
    assert_eq!(hunk_starts(text), vec![12, 1]);
    assert_eq!(hunk_starts("no hunks here\n"), Vec::<usize>::new());
}

#[test]
fn the_context_is_three_lines_normally_and_twelve_wide() {
    assert_eq!(context(false), 3);
    assert_eq!(context(true), 12);
}

#[test]
fn a_split_needs_sixty_columns_and_no_more() {
    assert!(!can_split(59));
    assert!(can_split(60));
    assert!(can_split(160));
}

#[test]
fn a_jump_moves_to_the_next_or_previous_hunk_and_reports_the_edges() {
    let hunks = [hunk(4, 0, 1), hunk(20, 0, 9), hunk(31, 1, 3)];
    assert_eq!(step(&hunks, 0, true), Jump::To(4));
    assert_eq!(step(&hunks, 4, true), Jump::To(20), "a hunk on the top row is not the next one");
    assert_eq!(step(&hunks, 31, true), Jump::Last);
    assert_eq!(step(&hunks, 40, true), Jump::Last);
    assert_eq!(step(&hunks, 31, false), Jump::To(20));
    assert_eq!(step(&hunks, 4, false), Jump::First, "a hunk on the top row is not the previous one");
    assert_eq!(step(&hunks, 0, false), Jump::First);
    assert_eq!(step(&[], 0, true), Jump::Nothing);
}

#[test]
fn the_line_is_the_last_hunk_of_the_file_above_the_top_else_its_first() {
    let hunks = [hunk(4, 0, 1), hunk(20, 0, 9), hunk(31, 1, 3), hunk(40, 1, 50)];
    assert_eq!(line_at(&hunks, 0, 0, 7), 1, "above its first hunk, the file's first line");
    assert_eq!(line_at(&hunks, 0, 10, 7), 1);
    assert_eq!(line_at(&hunks, 0, 25, 7), 9);
    assert_eq!(line_at(&hunks, 1, 35, 7), 3);
    assert_eq!(line_at(&hunks, 2, 35, 7), 7, "a file with no hunk in the stack takes the fallback");
}

#[test]
fn a_fold_toggles_per_path() {
    let mut folds = Folds::default();
    folds.toggle("hogwarts/chamber.rs");
    assert!(folds.contains("hogwarts/chamber.rs"));
    assert!(!folds.contains("louvre/key.rs"));
    folds.toggle("hogwarts/chamber.rs");
    assert!(!folds.contains("hogwarts/chamber.rs"));
}
