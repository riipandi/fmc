//! A Mermaid fence in the rendered Markdown view (plan-00006 phase 2): drawn as a diagram, framed when it cannot be,
//! measured again on a resize, and left as text in the source view (D1).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
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
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
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
