//! The change list's selection and scroll, on values: no git and no terminal.

use super::*;
use crate::diff::ChangeKind;

fn rows(count: usize) -> Vec<Change> {
    (0..count)
        .map(|index| Change { kind: ChangeKind::Modified, ..Change::untracked(&format!("row/{index:02}.rs")) })
        .collect()
}

fn list(count: usize, height: usize) -> ChangeList {
    let mut list = ChangeList::default();
    list.show(rows(count), false, None);
    list.set_height(height);
    list
}

#[test]
fn a_listing_keeps_the_path_it_is_given_or_the_nearest_row() {
    let mut list = list(10, 4);
    list.move_by(5);
    assert_eq!(list.selected_index(), 5);
    list.show(rows(10), false, Some("row/02.rs"));
    assert_eq!(list.selected_index(), 2);
    list.show(rows(3), false, Some("gone/vault.rs"));
    assert_eq!(list.selected_index(), 2, "the index stays, clamped to the shorter list");
    list.show(Vec::new(), false, None);
    assert_eq!((list.selected_index(), list.selected()), (0, None));
    assert_eq!(list.lines(), 0);
}

#[test]
fn moves_clamp_to_the_list_and_report_whether_they_moved() {
    let mut list = list(3, 2);
    assert!(list.move_by(5));
    assert_eq!(list.selected_index(), 2);
    assert!(!list.move_by(1), "already on the last row");
    assert!(list.move_to_edge(false));
    assert_eq!(list.selected_index(), 0);
    assert!(!list.select_index(9));
}

#[test]
fn a_page_is_one_row_less_than_the_viewport() {
    let mut list = list(20, 6);
    list.page(true);
    assert_eq!(list.selected_index(), 5);
    list.page(false);
    assert_eq!(list.selected_index(), 0);
}

#[test]
fn the_selection_scrolls_from_its_nearest_edge_only() {
    let mut list = list(10, 3);
    for _ in 0..3 {
        list.move_by(1);
    }
    assert_eq!((list.selected_index(), list.top()), (3, 1));
    list.move_by(-1);
    assert_eq!(list.top(), 1, "still in view, so no scroll");
    list.move_by(-2);
    assert_eq!(list.top(), 0);
}

#[test]
fn the_wheel_scrolls_without_moving_the_selection_and_stops_at_the_ends() {
    let mut list = list(10, 3);
    list.scroll(4);
    assert_eq!((list.top(), list.selected_index()), (4, 0));
    list.set_height(3);
    assert_eq!(list.top(), 4, "a repaint keeps the wheel's scroll");
    list.scroll(100);
    assert_eq!(list.top(), 7);
    assert_eq!(list.row_at(0), Some(7));
    assert_eq!(list.row_at(3), None, "past the viewport");
}

#[test]
fn a_cut_list_has_its_note_as_the_first_line() {
    let mut list = ChangeList::default();
    list.show(rows(5), true, None);
    list.set_height(3);
    assert_eq!(list.lines(), 6);
    assert_eq!(list.row_at(0), None, "the note is not a row");
    assert_eq!(list.row_at(1), Some(0));
    list.move_by(4);
    assert_eq!(list.visible_lines(), 3..6);
    assert_eq!(list.row_at(2), Some(4));
}

#[test]
fn highlighting_a_path_keeps_its_row_in_view_and_skips_an_unmoved_one() {
    let mut list = list(12, 4);
    assert!(!list.select_path("row/00.rs"));
    assert!(list.select_path("row/09.rs"));
    assert_eq!((list.selected_index(), list.top()), (9, 6));
    assert!(!list.select_path("missing.rs"));
}
