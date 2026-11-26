use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmc::explorer::{Entry, Kind};
use fmc::reader::document::decode;
use opentui::testing::TestApp;

use super::*;
use crate::app::{build, layout, new_model, paint, tree, wire_mouse};
use crate::services::{Event, fs};

/// A compact screen, 40 columns wide, with two files listed under `/hogwarts` and the first selected. The keys and the
/// mouse are wired, so a press runs as it does in the app.
fn compact() -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(40, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    {
        let mut m = model.borrow_mut();
        let file = |name: &str| Entry::new(&root, name, Kind::File);
        m.browser.listed(&root, vec![file("owls.txt"), file("hedwig.txt")]);
        m.browser.select(&root.join("owls.txt"));
    }
    crate::app::capture::wire(app.app(), &model, &ui);
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

/// Paints the frame that the last change made, and renders it.
fn repaint(app: &TestApp, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
}

/// Enter on `hedwig.txt`, and the worker's answer: the file is shown and the reader takes the focus.
fn open_hedwig(app: &TestApp, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let path = PathBuf::from("/hogwarts/hedwig.txt");
    {
        let mut m = model.borrow_mut();
        m.browser.select(&path);
        tree::activate(&mut m);
    }
    let event = Event::Opened {
        path,
        result: Ok(fs::Opened::Text { document: decode(b"Hedwig\n").unwrap(), stamp: String::new() }),
    };
    tree::apply(&mut model.borrow_mut(), &ui.borrow(), &event);
    repaint(app, model, ui);
}

fn top(model: &Rc<RefCell<Model>>) -> Page {
    model.borrow().pages.top()
}

fn selected(model: &Rc<RefCell<Model>>) -> PathBuf {
    model.borrow().browser.selected().to_path_buf()
}

#[test]
fn the_compact_screen_starts_on_files_with_the_list_full_width() {
    let (app, model, _) = compact();
    assert_eq!(top(&model), Page::Files);
    assert_eq!(model.borrow().focus, Focus::List);
    assert_eq!(layout::width(&model.borrow(), 40), 40, "the list takes the whole screen");
    assert!(!app.render().row(0).starts_with('╭'), "no panel border in compact");
    assert!(!screen(&app).contains("Select a file"), "the welcome screen is not drawn in compact");
}

#[test]
fn enter_on_a_file_pushes_the_reader_and_q_pops_it_keeping_the_selection() {
    let (app, model, ui) = compact();
    open_hedwig(&app, &model, &ui);
    assert_eq!(top(&model), Page::Reader);
    assert_eq!(model.borrow().focus, Focus::Reader);
    assert_eq!(layout::width(&model.borrow(), 40), 0, "the reader takes the whole screen");
    app.press("q");
    repaint(&app, &model, &ui);
    assert_eq!(top(&model), Page::Files, "q goes back a page");
    assert_eq!(model.borrow().focus, Focus::List);
    assert_eq!(selected(&model), Path::new("/hogwarts/hedwig.txt"), "the selection stays on the file");
    assert!(model.borrow().shown.is_some(), "the document stays open for the next visit");
}

#[test]
fn escape_pops_the_reader_as_back_does() {
    let (app, model, ui) = compact();
    open_hedwig(&app, &model, &ui);
    app.press("escape");
    repaint(&app, &model, &ui);
    assert_eq!(top(&model), Page::Files);
}

#[test]
fn a_tap_on_a_file_opens_it_in_compact() {
    let (app, model, _) = compact();
    let row = crate::ui::tree::list_top(true) + 1;
    app.click(3, row);
    assert!(model.borrow().loading, "a tap on a file starts opening it, as Enter does");
}

#[test]
fn find_pushes_its_page_and_escape_closes_it() {
    let (app, model, ui) = compact();
    app.press("/");
    repaint(&app, &model, &ui);
    assert_eq!(top(&model), Page::Find);
    app.press("escape");
    repaint(&app, &model, &ui);
    assert_eq!(top(&model), Page::Files);
}

#[test]
fn the_key_page_pushes_and_q_puts_the_file_back() {
    let (app, model, ui) = compact();
    open_hedwig(&app, &model, &ui);
    app.press("?");
    repaint(&app, &model, &ui);
    assert_eq!(top(&model), Page::Keys);
    app.press("q");
    repaint(&app, &model, &ui);
    assert_eq!(top(&model), Page::Reader, "the file the key page replaced is back");
}

#[test]
fn a_move_in_compact_schedules_no_preview() {
    let (app, model, ui) = compact();
    model.borrow_mut().auto_preview = true;
    app.press("j");
    repaint(&app, &model, &ui);
    let generation = model.borrow().preview_generation;
    assert!(crate::app::preview::due(&model.borrow(), generation).is_none(), "the reader is not on screen");
}

#[test]
fn a_move_schedules_a_preview_at_80_columns() {
    let app = TestApp::new(80, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    {
        let mut m = model.borrow_mut();
        let file = |name: &str| Entry::new(&root, name, Kind::File);
        m.browser.listed(&root, vec![file("owls.txt"), file("hedwig.txt")]);
        m.browser.select(&root.join("owls.txt"));
        m.auto_preview = true;
    }
    crate::app::capture::wire(app.app(), &model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.press("j");
    let generation = model.borrow().preview_generation;
    assert!(crate::app::preview::due(&model.borrow(), generation).is_some(), "wide keeps the auto-preview");
}

#[test]
fn a_focus_in_the_reader_without_a_document_opens_no_page() {
    let (_app, model, _) = compact();
    let mut m = model.borrow_mut();
    m.focus = Focus::Reader;
    sync(&mut m);
    assert_eq!(m.pages.top(), Page::Files, "no document, so no reader page");
}

/// The screen's rows as text.
fn screen(app: &TestApp) -> String {
    let frame = app.render();
    (0..24).map(|y| format!("{}\n", frame.row(y))).collect()
}
