//! The pictures of the box fences in the rendered view: the columns and rows a picture takes, the row cap, the decode cache,
//! the theme's source, a decode that fails, and the cache's eviction. The logo is the repository's own asset.

use std::path::Path;

use fmc::reader::image::Info;
use fmc::reader::images::Header;
use fmc::theme::DARK;
use opentui::testing::TestApp;

use super::*;
use crate::ui::markdown::MarkdownView;

/// The README's logo: 1176 by 384 pixels, so 6.125 columns to a row at the 2:1 cell aspect.
const LOGO: &[u8] = include_bytes!("../../assets/brand/png/logo-light@2x.png");
const LOGO_RATIO: f32 = 1176.0 / 384.0 * 2.0;

const SOURCE: &str = "# Hogwarts\n\n![Marauder map](../assets/brand/png/logo-light@2x.png)\n\nThe map is old.\n";
const PATH: &str = "/hogwarts/logo-light@2x.png";

fn logo_header() -> Header {
    Header::Picture { path: PATH.into(), info: Info { width: 1176, height: 384, format: "png" }, modified: None }
}

/// The rendered view of `source` with `headers`, on a screen `columns` wide, with the viewer `viewer_rows` tall.
fn drawn(columns: u16, source: &str, headers: Vec<(String, Header)>, viewer_rows: usize) -> (TestApp, MarkdownView) {
    let app = TestApp::new(columns, 60);
    let view = MarkdownView::new(app.app(), &DARK, true);
    app.app().root().add(&view.widget());
    view.visible(true);
    view.fit(usize::from(columns) - 4, viewer_rows);
    view.show(source);
    view.set_headers(headers);
    app.render();
    (app, view)
}

/// The rows the picture's block takes on screen: the second block, after the heading.
fn picture_rows(view: &MarkdownView) -> u32 {
    let rows = view.widget().block_rows();
    let block = &rows[1];
    block.end - block.start
}

#[test]
fn a_decoded_picture_takes_the_columns_its_width_gives_and_the_rows_of_its_ratio() {
    for columns in [40u16, 80, 120] {
        let (app, view) =
            drawn(columns, SOURCE, vec![("../assets/brand/png/logo-light@2x.png".into(), logo_header())], 40);
        view.decode(Path::new(PATH), None, LOGO);
        app.render();
        // The pane is the screen less the view's two cells of padding on each side.
        let pane = f32::from(columns - 4).min(147.0);
        let expected = (pane / LOGO_RATIO).clamp(ROWS as f32, 24.0);
        let rows = picture_rows(&view) as f32;
        assert!((rows - expected).abs() <= 1.0, "{columns} columns: {rows} rows, expected about {expected}");
    }
}

#[test]
fn an_html_width_sets_the_columns_before_the_pane_does() {
    // 392 pixels at 8 a cell is 49 columns: narrower than the pane at 120 columns, so the picture keeps its own width.
    let source = "<img src=\"../assets/brand/png/logo-light@2x.png\" alt=\"fmc\" width=\"392\">\n\nAfter.\n";
    let (app, view) = drawn(120, source, vec![("../assets/brand/png/logo-light@2x.png".into(), logo_header())], 40);
    view.decode(Path::new(PATH), None, LOGO);
    app.render();
    let rows = view.widget().block_rows()[0].clone();
    let expected = 49.0 / LOGO_RATIO;
    assert!(((rows.end - rows.start) as f32 - expected).abs() <= 1.0, "49 columns is about {expected} rows: {rows:?}");
}

#[test]
fn a_picture_takes_no_more_rows_than_the_row_cap() {
    let (app, view) = drawn(120, SOURCE, vec![("../assets/brand/png/logo-light@2x.png".into(), logo_header())], 10);
    view.decode(Path::new(PATH), None, LOGO);
    app.render();
    assert_eq!(picture_rows(&view), 6, "three fifths of ten viewer rows");
}

#[test]
fn a_box_keeps_the_rows_of_its_picture_until_the_picture_is_decoded() {
    let (app, view) = drawn(80, SOURCE, vec![("../assets/brand/png/logo-light@2x.png".into(), logo_header())], 40);
    let reading = picture_rows(&view);
    assert!(reading > ROWS as u32, "the box is as tall as the picture it stands for: {reading}");
    view.decode(Path::new(PATH), None, LOGO);
    app.render();
    assert_eq!(picture_rows(&view), reading, "the lines below do not move when the picture lands");
}

#[test]
fn a_decoded_picture_stays_across_a_rebuild_and_is_not_read_again() {
    let (app, view) = drawn(80, SOURCE, vec![("../assets/brand/png/logo-light@2x.png".into(), logo_header())], 40);
    assert_eq!(view.wanted(), vec![(Path::new(PATH).to_path_buf(), None)]);
    view.decode(Path::new(PATH), None, LOGO);
    assert!(view.wanted().is_empty());
    // A rebuild: a new row cap rebuilds every block, and the picture comes from the cache.
    view.fit(76, 41);
    app.render();
    assert!(view.wanted().is_empty(), "the decoded picture is kept, so nothing is read again");
    assert!(picture_rows(&view) > ROWS as u32);
}

#[test]
fn a_picture_that_does_not_decode_keeps_its_box_with_the_reason() {
    let (app, view) = drawn(80, SOURCE, vec![("../assets/brand/png/logo-light@2x.png".into(), logo_header())], 40);
    view.decode(Path::new(PATH), None, b"not a picture");
    app.render();
    let text = app.text();
    assert!(text.contains("could not be decoded"), "{text}");
    assert!(!text.contains("Enter opens it"), "{text}");
    assert!(view.wanted().is_empty(), "a failed decode is not read again");
}

#[test]
fn a_picture_that_cannot_be_read_keeps_its_box_with_the_reason() {
    let (app, view) = drawn(80, SOURCE, vec![("../assets/brand/png/logo-light@2x.png".into(), logo_header())], 40);
    view.unreadable(Path::new(PATH), None, fmc::reader::images::UNREADABLE);
    app.render();
    assert!(app.text().contains("could not be read"), "{}", app.text());
}

const PICTURE: &str = "<picture>\n  <source media=\"(prefers-color-scheme: dark)\" srcset=\"logo-dark.png\">\n  <img src=\"logo-light.png\" alt=\"fmc\">\n</picture>\n\nAfter.\n";

#[test]
fn a_picture_takes_its_dark_source_in_the_dark_theme_and_its_light_source_otherwise() {
    let headers = vec![
        ("logo-light.png".to_owned(), Header::Refused(fmc::reader::images::UNREADABLE)),
        ("logo-dark.png".to_owned(), Header::Refused(fmc::reader::images::NOT_FOUND)),
    ];
    let app = TestApp::new(80, 30);
    let light = MarkdownView::new(app.app(), &DARK, false);
    app.app().root().add(&light.widget());
    light.visible(true);
    light.show(PICTURE);
    light.set_headers(headers.clone());
    app.render();
    assert!(app.text().contains("logo-light.png"), "{}", app.text());
    light.restyle(&DARK, true);
    app.render();
    assert!(app.text().contains("logo-dark.png"), "{}", app.text());
    assert!(!app.text().contains("logo-light.png"), "{}", app.text());
}

#[test]
fn a_centred_image_sits_in_the_middle_of_its_row() {
    let source = "<p align=\"center\"><img src=\"logo-light.png\" alt=\"fmc\" width=\"160\"></p>\n\nAfter.\n";
    let (app, _view) =
        drawn(80, source, vec![("logo-light.png".into(), Header::Refused(fmc::reader::images::UNREADABLE))], 40);
    let text = app.text();
    let column = text.lines().find_map(|row| row.find('┌').map(|at| row[..at].chars().count())).expect("a box");
    assert!(column >= 20, "the box starts at column {column}, past the left margin: {text}");
}

#[test]
fn the_cache_keeps_the_most_recent_pictures_and_drops_the_least_recent() {
    let slot = || Slot::Failed(fmc::reader::images::UNREADABLE);
    let mut pictures = Pictures::default();
    for n in 0..CAPACITY {
        pictures.insert(Path::new(&format!("/hogwarts/{n}.png")), None, slot());
    }
    assert!(pictures.get(Path::new("/hogwarts/0.png"), None).is_some());
    pictures.insert(Path::new("/hogwarts/extra.png"), None, slot());
    assert!(pictures.contains(Path::new("/hogwarts/0.png"), None), "drawn last, so it stays");
    assert!(!pictures.contains(Path::new("/hogwarts/1.png"), None), "the least recently drawn goes");
    assert!(pictures.contains(Path::new("/hogwarts/extra.png"), None));
}

#[test]
fn the_row_cap_is_three_fifths_of_the_viewer_and_never_less_than_a_box() {
    assert_eq!(row_cap(40), 24);
    assert_eq!(row_cap(10), 6);
    assert_eq!(row_cap(0), ROWS as u32);
}
