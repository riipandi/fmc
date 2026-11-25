use std::path::Path;

use fmc::explorer::{Entry, Kind, Row};
use fmc::theme::DARK;

use super::*;

fn entry(name: &str, kind: Kind) -> Entry {
    Entry::new(Path::new("/vault"), name, kind)
}

fn row<'a>(entry: &'a Entry, depth: usize, folder: bool, expanded: bool) -> Row<'a> {
    Row { entry, depth, folder, expanded }
}

fn text(segments: &[Seg]) -> String {
    segments.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn a_selected_folder_row_reads_like_scouter() {
    let hogwarts = entry("hogwarts", Kind::Directory);
    let segments = row_segments(&row(&hogwarts, 1, true, true), true, false, &DARK);
    assert_eq!(text(&segments), "▎│ ▾ \u{f07c} hogwarts/");
    let name = segments.last().unwrap();
    assert!(name.bold);
    assert_eq!(name.fg, Some(DARK.accent));
}

#[test]
fn a_file_row_has_two_spaces_for_the_marker_and_the_default_icon() {
    let notes = entry("Notes.md", Kind::File);
    let segments = row_segments(&row(&notes, 0, false, false), false, false, &DARK);
    assert_eq!(text(&segments), "   \u{f48a} Notes.md");
    assert_eq!(segments.last().unwrap().fg, Some(DARK.text));
}

#[test]
fn an_ignored_row_is_dimmed_in_its_icon_and_name() {
    let target = entry("target", Kind::Directory);
    let segments = row_segments(&row(&target, 0, true, false), false, true, &DARK);
    assert_eq!(text(&segments), " ▸ \u{f07b} target/");
    assert_eq!(segments[2].fg, Some(DARK.ignored));
    assert_eq!(segments.last().unwrap().fg, Some(DARK.ignored));
}

#[test]
fn icons_and_colors_follow_the_extension() {
    let cases = [
        ("chamber13.ts", Kind::File, "\u{e628}", DARK.syntax.function),
        ("gringotts.rs", Kind::File, "\u{e7a8}", DARK.syntax.keyword),
        ("Lumos.JSON", Kind::File, "\u{e60b}", DARK.syntax.constant),
        ("diagon-alley.py", Kind::File, "\u{e73c}", DARK.syntax.string),
        ("Makefile", Kind::File, "\u{f016}", DARK.muted),
        (".gitignore", Kind::File, "\u{f016}", DARK.muted),
        ("azkaban", Kind::Link, "\u{f0c1}", DARK.accent_soft),
    ];
    for (name, kind, glyph, colour) in cases {
        let e = entry(name, kind);
        let segments = row_segments(&row(&e, 0, false, false), false, false, &DARK);
        assert_eq!(segments[2].text, format!("{glyph} "), "{name}");
        assert_eq!(segments[2].fg, Some(colour), "{name}");
    }
}

#[test]
fn a_thumb_is_drawn_only_when_the_list_scrolls() {
    assert_eq!(thumb(8, 10, 0, 4), vec![None; 4]);
    // 30 rows in 10 at the top: a thumb of 7 half-rows on a 20-half-row track.
    assert_eq!(thumb(30, 10, 0, 10), [Some("▐"), Some("▐"), Some("▐"), Some("▝"), None, None, None, None, None, None]);
    // At the bottom the thumb sits at the end of the track.
    let bottom = thumb(30, 10, 20, 10);
    assert_eq!(bottom[5], None);
    assert_eq!(bottom[6], Some("▗"));
    assert_eq!(bottom[9], Some("▐"));
}

#[test]
fn a_click_maps_a_screen_row_to_a_list_line() {
    assert_eq!(line_at(LIST_TOP, LIST_TOP, 5), Some(0));
    assert_eq!(line_at(LIST_TOP + 4, LIST_TOP, 5), Some(4));
    assert_eq!(line_at(LIST_TOP + 5, LIST_TOP, 5), None, "below the list");
    assert_eq!(line_at(LIST_TOP - 1, LIST_TOP, 5), None, "the header");
}
