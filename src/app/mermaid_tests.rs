//! A Mermaid fence in the rendered Markdown view (plan-00006 phase 2): drawn as a diagram, framed when it cannot be,
//! measured again on a resize, and left as text in the source view (D1).

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use fmc::explorer::{Entry, Kind};
use fmc::reader::document::decode;
use opentui::testing::TestApp;

use super::*;

/// Narrower than the reader at 100 columns, where the sidebar takes its share.
const SMALL_FLOWCHART: &str = "flowchart LR\n    A[Harry Potter] --> B[Albus Dumbledore]\n";

const HOGWARTS_FLOWCHART: &str = "flowchart LR\n    subgraph Hogwarts\n        A[Harry Potter] --> B{Sorting Hat}\n        B -->|Gryffindor| C[Gryffindor Tower]\n        B -->|Slytherin| D[Slytherin Dungeon]\n    end\n    C --> E[Albus Dumbledore]\n    D --> F[Severus Snape]\n";

/// `notes.md` open in the reader, its rendered view shown, on a screen `columns` wide.
fn reading_markdown(columns: u16, markdown: &str) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 40);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    let notes = root.join("notes.md");
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&root, vec![Entry::new(&root, "notes.md", Kind::File)]);
        m.browser.select(&notes);
    }
    let document = decode(markdown.as_bytes()).expect("a text document");
    reader::show_text(&mut model.borrow_mut(), &ui.borrow(), &notes, document, String::new());
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

fn fenced(diagram: &str) -> String {
    format!("# Hogwarts\n\n```mermaid\n{diagram}```\n")
}

#[test]
fn a_mermaid_fence_is_drawn_as_a_diagram_in_the_preview() {
    let (app, _model, _ui) = reading_markdown(100, &fenced(SMALL_FLOWCHART));
    let text = app.text();
    assert!(text.contains('┌') && text.contains('│'), "{text}");
    assert!(text.contains("Harry Potter") && text.contains("Albus Dumbledore"), "{text}");
    assert!(!text.contains("flowchart LR"), "the diagram replaces its source: {text}");
}

#[test]
fn a_diagram_wider_than_the_reader_is_framed_with_its_reason() {
    let (app, _model, _ui) = reading_markdown(60, &fenced(HOGWARTS_FLOWCHART));
    let text = app.text();
    assert!(text.contains("too wide: ") && text.contains("the reader has"), "{text}");
    assert!(text.contains("Sorting Hat"), "the source stays readable: {text}");
}

#[test]
fn an_unsupported_diagram_is_framed_with_its_keyword() {
    let (app, _model, _ui) = reading_markdown(100, &fenced("this is not a diagram\n"));
    let text = app.text();
    assert!(text.contains("unsupported diagram: this"), "{text}");
    assert!(text.contains("this is not a diagram"), "{text}");
}

#[test]
fn a_resize_measures_the_diagram_again() {
    let (app, _model, _ui) = reading_markdown(160, &fenced(HOGWARTS_FLOWCHART));
    assert!(app.text().contains("Sorting Hat"), "drawn at 160 columns: {}", app.text());
    app.resize(60, 40);
    app.render();
    assert!(app.text().contains("too wide"), "measured again at 60 columns: {}", app.text());
}

#[test]
fn the_source_view_keeps_the_fence_as_text() {
    let (app, model, ui) = reading_markdown(100, &fenced(HOGWARTS_FLOWCHART));
    reader::toggle_markdown(&mut model.borrow_mut());
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    let text = app.text();
    assert!(text.contains("```mermaid") && text.contains("flowchart LR"), "{text}");
    assert!(!text.contains('┌'), "the source view draws no diagram: {text}");
}

/// The top-left corner of the first diagram box on screen, and the top-right corner of the rightmost box on its row,
/// as (column, row). The fill covers the whole diagram, arrows between boxes included, so the rightmost box is its edge.
fn box_corners(app: &TestApp) -> ((u32, u32), (u32, u32)) {
    let capture = app.render();
    let (left, top) = (0..capture.height)
        .flat_map(|y| (0..capture.width).map(move |x| (x, y)))
        .find(|&(x, y)| capture.cell(x, y).is_some_and(|cell| cell.symbol == "┌"))
        .unwrap_or_else(|| panic!("no diagram on screen: {}", app.text()));
    let right = (0..capture.width)
        .rev()
        .find(|&x| capture.cell(x, top).is_some_and(|cell| cell.symbol == "┐"))
        .unwrap_or_else(|| panic!("no box corner on row {top}: {}", app.text()));
    ((left, top), (right, top))
}

#[test]
fn a_diagram_sits_on_the_panel_tone_at_each_width_and_follows_a_resize() {
    for columns in [49, 70, 120] {
        let (app, model, ui) = reading_markdown(columns, &fenced(SMALL_FLOWCHART));
        // Below 60 columns the list is its own page, and the reader's page shows only with the focus in the reader.
        if columns < 60 {
            model.borrow_mut().focus = fmc::keys::Focus::Reader;
            paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
        }
        let panel = crate::ui::styled::color(model.borrow().palette().fallback_background);
        let ((left, top), (right, _)) = box_corners(&app);
        let capture = app.render();
        assert_eq!(capture.cell(left + 1, top + 1).map(|cell| cell.bg), Some(panel), "inside the box at {columns}");
        assert_fill_spans(&capture, panel, right, top + 1, u32::from(columns), columns);
    }
    let (app, model, _ui) = reading_markdown(120, &fenced(SMALL_FLOWCHART));
    let panel = crate::ui::styled::color(model.borrow().palette().fallback_background);
    app.resize(70, 40);
    app.render();
    let ((left, top), (right, _)) = box_corners(&app);
    let capture = app.render();
    assert_eq!(capture.cell(left + 1, top + 1).map(|cell| cell.bg), Some(panel), "inside after the resize");
    assert_fill_spans(&capture, panel, right, top + 1, 70, "the resize");
}

/// The fill runs past the diagram's right edge to the reader's text edge: the two padding cells and the border stay
/// clear (owner request: a full-width diagram block).
fn assert_fill_spans(
    capture: &opentui::testing::Capture,
    panel: Color,
    right: u32,
    row: u32,
    columns: u32,
    at: impl std::fmt::Display,
) {
    let last = (0..capture.width)
        .rev()
        .find(|&x| capture.cell(x, row).is_some_and(|cell| cell.bg == panel))
        .unwrap_or_else(|| panic!("no fill on row {row} at {at}"));
    assert!(last > right, "the fill runs past the box at {at}");
    assert_eq!(last, columns - 4, "the fill stops at the reader's padding at {at}");
}
