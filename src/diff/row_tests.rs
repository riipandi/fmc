//! Row, header, and stat-bar text on values: no git and no terminal.

use super::*;
use crate::diff::ChangeKind;
use crate::theme::DARK;

fn change(path: &str, kind: ChangeKind) -> Change {
    Change { kind, ..Change::untracked(path) }
}

fn counted(path: &str, added: u64, removed: u64) -> Change {
    Change { added: Some(added), removed: Some(removed), ..change(path, ChangeKind::Modified) }
}

fn text(segments: &[Seg]) -> String {
    segments.iter().map(|segment| segment.text.as_str()).collect()
}

fn bars(change: &Change) -> Vec<(String, Option<&'static str>)> {
    stat_bar(change, &DARK).into_iter().map(|segment| (segment.text, segment.fg)).collect()
}

#[test]
fn counts_name_a_binary_an_untracked_file_and_skip_zero_sides() {
    let binary = Change { binary: true, ..change("hogwarts/owl.bin", ChangeKind::Modified) };
    assert_eq!(text(&count_segments(&binary, &DARK)), "bin");
    assert_eq!(text(&count_segments(&change("gringotts/vault.ts", ChangeKind::Untracked), &DARK)), "new");
    assert_eq!(text(&count_segments(&counted("hogwarts/chamber.rs", 3, 1), &DARK)), "+3 −1");
    assert_eq!(text(&count_segments(&counted("hogwarts/chamber.rs", 0, 2), &DARK)), "−2");
    assert!(count_segments(&counted("hogwarts/chamber.rs", 0, 0), &DARK).is_empty());
}

#[test]
fn the_stat_bar_keeps_a_cell_for_each_side_it_has() {
    let green = (String::from("▪▪▪▪"), Some(DARK.added));
    let red = (String::from("▪"), Some(DARK.removed));
    assert_eq!(bars(&counted("a.rs", 4, 1)), vec![("  ".into(), None), green, red]);
    // One added line of 100 still shows a green cell; the bar never gives all five cells to removals.
    assert_eq!(bars(&counted("a.rs", 1, 99)).len(), 3);
    assert_eq!(text(&stat_bar(&counted("a.rs", 1, 99), &DARK)), "  ▪▪▪▪▪");
    assert_eq!(text(&stat_bar(&counted("a.rs", 9, 1), &DARK)), "  ▪▪▪▪▪");
    assert_eq!(text(&stat_bar(&counted("a.rs", 0, 3), &DARK)), "  ▪▪▪");
    assert!(stat_bar(&counted("a.rs", 0, 0), &DARK).is_empty());
    let binary = Change { binary: true, ..counted("a.bin", 4, 1) };
    assert!(stat_bar(&binary, &DARK).is_empty());
}

#[test]
fn a_header_names_the_file_with_its_index_mark_and_folder() {
    let staged_and_not = Change {
        staged: Some(true),
        unstaged: Some(true),
        added: Some(2),
        removed: Some(0),
        ..change("hogwarts/chamber.rs", ChangeKind::Modified)
    };
    let title = header_title(&staged_and_not, true, false, &DARK);
    assert_eq!(text(&title), "▎ ▾ M ◐ chamber.rs  hogwarts/");
    assert_eq!(title[0].fg, Some(DARK.accent));
    assert_eq!(
        text(&header_title(&change("README.md", ChangeKind::Untracked), false, false, &DARK)),
        "  ▾ ? README.md"
    );
    // A folded file shows the closed chevron (Scouter `headerTitleContent`, `folded ? "▸ " : "▾ "`).
    assert_eq!(text(&header_title(&change("README.md", ChangeKind::Untracked), false, true, &DARK)), "  ▸ ? README.md");
}

#[test]
fn a_rename_header_keeps_the_old_path_in_full_and_the_list_row_its_name() {
    let rename = Change { old_path: Some("gringotts/key.rs".into()), ..change("louvre/key.rs", ChangeKind::Renamed) };
    assert!(text(&header_title(&rename, false, false, &DARK)).contains("← gringotts/key.rs"));
    let row = list_row(&rename, false, &DARK);
    assert_eq!(text(&row.name), "key.rs  ← key.rs  louvre/");
}

#[test]
fn a_list_row_has_the_selection_bar_the_letter_and_the_mark() {
    let staged = Change { staged: Some(true), unstaged: Some(false), ..counted("langdon/symbology.py", 5, 0) };
    let row = list_row(&staged, true, &DARK);
    assert_eq!(text(&row.status), "▎M ●");
    assert_eq!(text(&row.name), "symbology.py  langdon/");
    assert!(row.name[0].bold, "the selected name is bold");
    assert_eq!(text(&row.counts), "+5");
    let plain = list_row(&counted("langdon/symbology.py", 5, 0), false, &DARK);
    assert_eq!(text(&plain.status), " M ");
    assert!(!plain.name[0].bold);
}

#[test]
fn the_empty_state_and_the_truncation_note_read_as_scouter_writes_them() {
    assert_eq!(empty_message(true), ("No changes", "The working tree matches HEAD.".into()));
    assert_eq!(empty_message(false), ("No changes", "The working tree matches an empty repository.".into()));
    assert_eq!(truncated_note(1000), "Only the first 1,000 changes are listed.");
}
