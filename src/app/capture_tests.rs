use std::path::{Path, PathBuf};

use fmc::reader::document::decode;
use opentui::testing::TestApp;

use super::*;

/// The panes wired with the key capture, and a long unwrapped line open in the reader, on a screen `columns` wide.
fn reading(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root, None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    wire_mouse(&model, &ui);
    capture::wire(app.app(), &model, &ui);
    let document = decode(format!("{}\n", "Expelliarmus! ".repeat(40)).as_bytes()).expect("a text document");
    ui.borrow().reader.show(&document, None);
    model.borrow_mut().shown = reader::Shown::Text {
        path: PathBuf::from("/hogwarts/spells.txt"),
        document,
        language: None,
        markdown: None,
        stamp: Some(String::new()),
    };
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

#[test]
fn an_arrow_the_list_takes_does_not_pan_the_reader() {
    let (app, model, ui) = reading(80);
    // A click in the reader gives OpenTUI's scroll box the keyboard focus; Tab then gives the list fmc's focus.
    app.click(40, 10);
    app.press("tab");
    assert_eq!(model.borrow().focus, Focus::List);
    // The headless layout allows a pan of about four columns, which a stray arrow would take at once.
    assert_eq!(ui.borrow().viewer.position(), (0.0, 0.0));
    app.press("right");
    assert_eq!(ui.borrow().viewer.position(), (0.0, 0.0));
}

#[test]
fn ctrl_slash_that_shows_the_sidebar_focuses_the_list() {
    let (app, model, _) = reading(79);
    assert!(!layout::shown(&model.borrow(), 79), "a document under 80 columns hides the sidebar");
    app.click(40, 10);
    assert_eq!(model.borrow().focus, Focus::Reader);
    app.press("ctrl+/");
    assert!(layout::shown(&model.borrow(), 79));
    assert_eq!(model.borrow().focus, Focus::List);
}
