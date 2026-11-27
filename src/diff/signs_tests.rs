use super::*;
use crate::diff::{Change, from_diff, from_file};
use crate::reader::document::decode;

/// The signs of a whole `git diff -U0` text for `spells.txt`, through the patch the services read.
fn signs_of(raw: &str) -> Vec<(u32, Sign)> {
    signs(&from_diff(&Change::untracked("spells.txt"), raw))
}

fn diff(hunks: &str) -> String {
    format!(
        "diff --git a/spells.txt b/spells.txt\nindex 1a2b3c4..5d6e7f8 100644\n--- a/spells.txt\n+++ b/spells.txt\n{hunks}"
    )
}

#[test]
fn a_pure_addition_marks_each_new_line() {
    let raw = diff("@@ -2,0 +3,2 @@\n+Hermione Granger\n+Sophie Neveu\n");
    assert_eq!(signs_of(&raw), vec![(2, Sign::Added), (3, Sign::Added)]);
}

#[test]
fn a_deletion_in_the_middle_marks_the_line_above() {
    let raw = diff("@@ -4,2 +3,0 @@\n-Leigh Teabing\n-Katherine Solomon\n");
    assert_eq!(signs_of(&raw), vec![(2, Sign::DeletedBelow)]);
}

#[test]
fn a_deletion_at_the_top_marks_line_one_from_above() {
    let raw = diff("@@ -1 +0,0 @@\n-Robert Langdon\n");
    assert_eq!(signs_of(&raw), vec![(0, Sign::DeletedTop)]);
}

#[test]
fn a_one_for_one_change_is_modified() {
    let raw = diff("@@ -5 +5 @@\n-Expelliarmus\n+Expecto Patronum\n");
    assert_eq!(signs_of(&raw), vec![(4, Sign::Modified)]);
}

#[test]
fn more_new_lines_than_old_mark_the_rest_added() {
    let raw = diff("@@ -5 +5,3 @@\n-Vittoria Vetra\n+Vittoria Vetra\n+the Louvre\n+CERN\n");
    assert_eq!(signs_of(&raw), vec![(4, Sign::Modified), (5, Sign::Added), (6, Sign::Added)]);
}

#[test]
fn fewer_new_lines_than_old_mark_only_those_modified() {
    let raw = diff("@@ -5,3 +5 @@\n-Gringotts\n-Diagon Alley\n-Hogwarts\n+Hogsmeade\n");
    assert_eq!(signs_of(&raw), vec![(4, Sign::Modified)]);
}

#[test]
fn every_hunk_contributes_its_own_signs() {
    let raw = diff(
        "@@ -2 +2 @@\n-Minerva McGonagall\n+Minerva McGonagall, Head of House\n@@ -9,0 +10 @@\n+Albus Dumbledore\n@@ -20,2 +19,0 @@\n-Hermione Granger\n-Harry Potter\n",
    );
    assert_eq!(signs_of(&raw), vec![(1, Sign::Modified), (9, Sign::Added), (18, Sign::DeletedBelow)]);
}

#[test]
fn an_untracked_file_is_all_added() {
    let document = decode(b"Hermione Granger\nRobert Langdon\nSophie Neveu\n").expect("a text document");
    let patch = from_file("notes.txt", Ok(document));
    assert_eq!(signs(&patch), vec![(0, Sign::Added), (1, Sign::Added), (2, Sign::Added)]);
}

#[test]
fn a_notice_has_no_signs() {
    assert!(signs(&Patch::Notice("No line changes (permissions or file type only).".into())).is_empty());
    assert!(signs(&from_diff(&Change::untracked("spells.txt"), "")).is_empty());
}
