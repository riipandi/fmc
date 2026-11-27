use std::path::Path;

use fmc::explorer::{Entry, Kind, Row};
use fmc::find::text::Snippet;
use fmc::theme::DARK;

use super::*;

fn entry(name: &str, kind: Kind) -> Entry {
    Entry::new(Path::new("/vault"), name, kind)
}

fn row<'a>(entry: &'a Entry, depth: usize, folder: bool, expanded: bool) -> Row<'a> {
    Row { entry, depth, folder, expanded, hit: None, matched: &[] }
}

/// A text hit's row on Scouter's line: the name and line, the snippet, and the folder dimmed at the end.
fn hit_row<'a>(entry: &'a Entry, line: usize, snippet: &'a Snippet) -> Row<'a> {
    Row { entry, depth: 0, folder: false, expanded: false, hit: Some((line, snippet)), matched: &[] }
}

fn text(segments: &[Seg]) -> String {
    segments.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn a_selected_folder_row_reads_like_scouter() {
    let hogwarts = entry("hogwarts", Kind::Directory);
    let segments = row_segments(&row(&hogwarts, 1, true, true), true, false, false, false, false, &DARK);
    assert_eq!(text(&segments), "▎│ ▾ \u{f07c} hogwarts/");
    let name = segments.last().unwrap();
    assert!(name.bold);
    assert_eq!(name.fg, Some(DARK.accent));
}

#[test]
fn a_file_row_has_two_spaces_for_the_marker_and_the_default_icon() {
    let notes = entry("Notes.md", Kind::File);
    let segments = row_segments(&row(&notes, 0, false, false), false, false, false, false, false, &DARK);
    assert_eq!(text(&segments), "   \u{f48a} Notes.md");
    assert_eq!(segments.last().unwrap().fg, Some(DARK.text));
}

#[test]
fn a_marked_row_takes_a_check_in_the_bar_cell_and_a_selected_one_keeps_its_bold_name() {
    let sophie = entry("sophie-neveu.md", Kind::File);
    let marked = row_segments(&row(&sophie, 0, false, false), false, false, false, false, true, &DARK);
    assert_eq!(text(&marked), "✓  \u{f48a} sophie-neveu.md");
    assert_eq!(marked[0].fg, Some(DARK.accent_soft));
    let both = row_segments(&row(&sophie, 0, false, false), true, false, false, false, true, &DARK);
    assert_eq!(text(&both), "✓  \u{f48a} sophie-neveu.md");
    assert!(both.last().unwrap().bold && both.last().unwrap().fg == Some(DARK.accent));
    let selected = row_segments(&row(&sophie, 0, false, false), true, false, false, false, false, &DARK);
    assert_eq!(text(&selected), "▎  \u{f48a} sophie-neveu.md");
}

#[test]
fn an_ignored_row_is_dimmed_in_its_icon_and_name() {
    let target = entry("target", Kind::Directory);
    let segments = row_segments(&row(&target, 0, true, false), false, true, false, false, false, &DARK);
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
        let segments = row_segments(&row(&e, 0, false, false), false, false, false, false, false, &DARK);
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

#[test]
fn a_text_hit_row_reads_name_line_snippet_and_folder_like_scouter() {
    // The finder's rows carry the path relative to the folder, as Scouter's `relative` does.
    let chapters =
        Entry { name: "notes/chapters.txt".into(), path: "/vault/notes/chapters.txt".into(), kind: Kind::File };
    let snippet = Snippet { before: "Line ".into(), matched: "043".into(), after: ": Teabing".into() };
    let segments = row_segments(&hit_row(&chapters, 43, &snippet), false, false, false, false, false, &DARK);
    assert_eq!(text(&segments), " \u{f016} chapters.txt:43  Line 043: Teabing  notes/");
    let matched = segments.iter().find(|seg| seg.text == "043").unwrap();
    assert_eq!(matched.fg, Some(DARK.accent_soft));
    assert!(matched.bold);
}

#[test]
fn a_finder_row_names_the_file_first_and_its_folder_dimmed_after_it() {
    let marauder = entry("assets/marauder-map.png", Kind::File);
    let narrow = row_segments(&row(&marauder, 0, false, false), false, false, true, false, false, &DARK);
    assert_eq!(narrow.last(), Some(&Seg::new("  assets/", DARK.dim)));
    assert!(text(&narrow).find("marauder-map.png") < text(&narrow).find("assets/"));
    let wide = row_segments(&row(&marauder, 0, false, false), false, false, false, false, false, &DARK);
    assert!(text(&wide).contains("assets/marauder-map.png"), "a row at 80 columns keeps its path");
}

#[test]
fn a_fuzzy_match_colours_its_characters_in_the_name_and_the_folder_and_keeps_the_text() {
    let owls = entry("hogwarts/owls.md", Kind::File);
    let plain = row_segments(&row(&owls, 0, false, false), false, false, true, false, false, &DARK);
    let matched = Row { matched: &[1, 3, 9, 10], ..row(&owls, 0, false, false) };
    let marked = row_segments(&matched, false, false, true, false, false, &DARK);
    assert_eq!(text(&marked), text(&plain), "a match changes the colour, not the text");
    let coloured: Vec<&Seg> = marked
        .iter()
        .filter(|seg| seg.fg == Some(DARK.accent_soft) && seg.text.chars().all(char::is_alphanumeric))
        .collect();
    assert_eq!(coloured.iter().map(|seg| seg.text.as_str()).collect::<Vec<_>>(), ["ow", "o", "w"]);
    assert!(coloured.iter().all(|seg| seg.bold));
}

#[test]
fn a_cut_row_is_muted_even_when_selected() {
    let notes = entry("Notes.md", Kind::File);
    let segments = row_segments(&row(&notes, 0, false, false), true, false, false, true, false, &DARK);
    assert_eq!(segments.last().unwrap().fg, Some(DARK.muted));
}
