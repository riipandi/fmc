//! A lone image's box in the rendered view: its rows at each width, each reason, and the search over its name.

use fmc::reader::image::Info;
use fmc::reader::images::{self, Header};
use fmc::reader::rendered_search::Slot;
use fmc::theme::DARK;
use opentui::prelude::*;
use opentui::testing::TestApp;

use super::ROWS;
use crate::ui::markdown::MarkdownView;

const TARGET: &str = "../maps/marauder.png";

/// A heading, the box, and a paragraph that also says "map": the box is block 1, its lines 2 and 3.
const SOURCE: &str = "# Hogwarts\n\n![Marauder map](../maps/marauder.png)\n\nThe map is old.\n";

fn picture() -> Header {
    Header::Picture {
        path: "/hogwarts/maps/marauder.png".into(),
        info: Info { width: 480, height: 270, format: "png" },
        modified: None,
    }
}

/// The rendered view of `SOURCE` with `headers` set, on a screen `columns` wide.
fn drawn(columns: u16, headers: Vec<(String, Header)>) -> (TestApp, MarkdownView) {
    let app = TestApp::new(columns, 30);
    let view = MarkdownView::new(app.app(), &DARK, true);
    app.app().root().add(&view.widget());
    view.visible(true);
    view.fit(usize::from(columns) - 4, 40);
    view.show(SOURCE);
    view.set_headers(headers);
    app.render();
    (app, view)
}

/// The screen rows that hold the box's top and bottom borders, as (top, bottom).
fn frame(app: &TestApp) -> Option<(usize, usize)> {
    let text = app.text();
    let rows: Vec<&str> = text.lines().collect();
    let top = rows.iter().position(|row| row.contains('┌'))?;
    let bottom = rows.iter().position(|row| row.contains('└'))?;
    Some((top, bottom))
}

#[test]
fn a_picture_box_names_the_image_its_path_its_size_and_format_and_the_open_hint() {
    for columns in [40, 80, 120] {
        let (app, _view) = drawn(columns, vec![(TARGET.into(), picture())]);
        let text = app.text();
        assert!(text.contains("▣ Marauder map"), "{columns} columns: {text}");
        assert!(text.contains("Enter opens it"), "{columns} columns: {text}");
        let (top, bottom) = frame(&app).unwrap_or_else(|| panic!("a frame at {columns} columns: {text}"));
        assert!(bottom - top >= ROWS - 1, "at least three text rows and two borders at {columns} columns: {text}");
    }
}

#[test]
fn a_box_at_a_narrow_width_cuts_its_path_and_keeps_its_hint() {
    let (app, _view) = drawn(40, vec![(TARGET.into(), picture())]);
    let text = app.text();
    assert!(text.contains('…'), "the path is cut at 40 columns: {text}");
    assert!(text.contains("Enter opens it"), "{text}");
}

#[test]
fn a_box_at_a_wide_width_shows_the_whole_path_and_size() {
    let (app, _view) = drawn(120, vec![(TARGET.into(), picture())]);
    assert!(app.text().contains("../maps/marauder.png · 480×270 · png"), "{}", app.text());
}

#[test]
fn each_reason_a_box_cannot_open_is_its_last_row_and_no_open_hint_shows() {
    let reasons = [
        (images::REMOTE, "remote"),
        (images::SVG, "SVG"),
        (images::OUTSIDE, "outside the folder"),
        (images::NOT_FOUND, "not found"),
        (images::NOT_SHOWN, "not an image fmc shows"),
        (images::UNREADABLE, "could not be read"),
        (images::TOO_LARGE, "too large"),
    ];
    for (reason, words) in reasons {
        let (app, _view) = drawn(80, vec![(TARGET.into(), Header::Refused(reason))]);
        let text = app.text();
        assert!(text.contains(words), "{words}: {text}");
        assert!(!text.contains("Enter opens it"), "{words}: {text}");
    }
}

#[test]
fn a_box_whose_header_has_not_come_back_says_it_is_reading() {
    let (app, _view) = drawn(80, Vec::new());
    let text = app.text();
    assert!(text.contains("reading the image…") && text.contains("▣ Marauder map"), "{text}");
}

#[test]
fn search_finds_a_box_by_its_name_and_counts_it_once() {
    let app = TestApp::new(80, 30);
    let view = MarkdownView::new(app.app(), &DARK, true);
    let hits = view.index(SOURCE).hits("marauder");
    assert_eq!(hits.len(), 1, "the box's name only: {hits:?}");
    assert_eq!((hits[0].block, hits[0].slot), (1, Some(Slot::Code(0))));
    // "map" is in the box's name and in the paragraph after it.
    let blocks: Vec<usize> = view.index(SOURCE).hits("map").iter().map(|hit| hit.block).collect();
    assert_eq!(blocks, vec![1, 2]);
}

#[test]
fn a_box_owns_the_lines_of_its_fence_including_the_closing_one() {
    let app = TestApp::new(80, 30);
    let view = MarkdownView::new(app.app(), &DARK, true);
    let blocks = view.block_lines(SOURCE);
    assert!(blocks.contains(&(2..4)), "the box is lines 2 and 3, its closing line included: {blocks:?}");
}
