use fmc::layout::Page;
use fmc::theme::{Mode, palette};

use super::*;

fn labels(segments: &[Segment]) -> Vec<&str> {
    segments.iter().map(|segment| segment.label.as_str()).collect()
}

fn keys(segments: &[Segment]) -> Vec<&'static str> {
    segments.iter().map(|segment| segment.key).collect()
}

/// The text a bar draws, with each segment's cell filled in.
fn text(drawn: &[Seg]) -> String {
    drawn.iter().map(|seg| seg.text.as_str()).collect()
}

#[test]
fn the_cells_and_separators_fill_the_bar_at_40_49_and_59_columns() {
    for columns in [40, 49, 59] {
        let cells = cells(columns, 3);
        assert_eq!(cells.len(), 3);
        let used: usize = cells.iter().map(|cell| cell.len()).sum::<usize>() + 2;
        assert_eq!(used, columns, "{columns}: three cells and two separators");
        assert_eq!(cells[0].start, 0);
        assert_eq!(cells[2].end, columns, "{columns}: the last cell ends at the edge");
        for pair in cells.windows(2) {
            assert_eq!(pair[0].end + 1, pair[1].start, "{columns}: one separator column between cells");
        }
    }
}

#[test]
fn the_cells_are_equal_to_within_one_column() {
    for columns in [40, 49, 59] {
        let widths: Vec<usize> = cells(columns, 3).iter().map(|cell| cell.len()).collect();
        let (most, least) = (widths.iter().max().unwrap(), widths.iter().min().unwrap());
        assert!(most - least <= 1, "{columns}: {widths:?}");
    }
}

#[test]
fn a_tap_on_a_cell_presses_its_segments_key_and_a_separator_presses_nothing() {
    let p = palette(Mode::Dark);
    let segments = page_segments(Page::Files, false, true, false);
    for columns in [40, 49, 59] {
        let (_, targets) = render(&segments, columns, p);
        let hit = |column| targets.iter().find(|(cell, _)| cell.contains(&column)).map(|(_, key)| *key);
        let cells = cells(columns, 3);
        assert_eq!(hit(cells[0].start), Some("/"), "{columns}: the first cell's first column");
        assert_eq!(hit(cells[1].end - 1), Some("d"), "{columns}: the middle cell's last column");
        assert_eq!(hit(cells[2].end - 1), Some("?"), "{columns}: the last cell's last column");
        assert_eq!(hit(cells[0].end), None, "{columns}: the separator presses nothing");
    }
}

#[test]
fn the_files_page_names_find_changes_and_keys() {
    let segments = page_segments(Page::Files, false, true, false);
    assert_eq!(labels(&segments), ["Find", "Changes", "Keys"]);
    assert_eq!(keys(&segments), ["/", "d", "?"]);
}

#[test]
fn the_reader_page_names_back_search_and_keys_and_copy_takes_search_while_text_is_selected() {
    let segments = page_segments(Page::Reader, false, true, false);
    assert_eq!(labels(&segments), ["Back", "Search", "Keys"]);
    assert_eq!(keys(&segments), ["q", "/", "?"]);
    let selected = page_segments(Page::Reader, false, true, true);
    assert_eq!(labels(&selected), ["Back", "Copy", "Keys"]);
    assert_eq!(keys(&selected)[1], "y");
}

#[test]
fn the_changes_labels_show_the_scope_on_screen() {
    assert_eq!(labels(&page_segments(Page::Changes, true, true, false)), ["Back", "All", "Split"]);
    assert_eq!(labels(&page_segments(Page::Changes, true, false, false)), ["Back", "One", "Split"]);
    assert_eq!(keys(&page_segments(Page::Changes, true, true, false)), ["q", "a", "s"]);
}

#[test]
fn the_changes_body_on_the_reader_page_keeps_the_changes_labels() {
    assert_eq!(labels(&page_segments(Page::Reader, true, true, false)), ["Back", "All", "Split"]);
}

#[test]
fn the_key_page_has_only_back() {
    let segments = page_segments(Page::Keys, false, true, false);
    assert_eq!(labels(&segments), ["Back"]);
    assert_eq!(keys(&segments), ["q"]);
}

#[test]
fn a_prompt_bar_cancels_with_esc_in_its_first_cell_and_shows_the_prompt_after_it() {
    let p = palette(Mode::Dark);
    let text_seg = Seg::new("/ hedwig", p.text);
    let (drawn, targets) = prompt(&[text_seg], 40, p);
    assert!(text(&drawn).trim_start().starts_with("Cancel"), "{:?}", text(&drawn));
    assert!(text(&drawn).contains("/ hedwig"));
    assert_eq!(targets, vec![(cells(40, 3)[0].clone(), "Esc")]);
}

#[test]
fn back_text_names_the_page_it_returns_to() {
    assert_eq!(back_text(Page::Files), "‹ Files");
    assert_eq!(back_text(Page::Changes), "‹ Changes");
}

#[test]
fn the_segment_labels_are_centred_in_their_cells() {
    let p = palette(Mode::Dark);
    let (drawn, _) = render(&page_segments(Page::Files, false, true, false), 40, p);
    let line = text(&drawn);
    assert_eq!(line.chars().count(), 40, "{line:?}");
    assert!(line.contains('·'), "{line:?}");
}
