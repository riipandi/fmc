use std::path::Path;

use opentui::testing::TestApp;

use super::*;

/// The panes, wired and painted, on a headless 80x24 screen with nothing open.
fn panes() -> (TestApp, Rc<RefCell<Model>>) {
    let app = TestApp::new(80, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root, None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    wire_mouse(&model, &ui);
    app.render();
    (app, model)
}

#[test]
fn a_press_in_the_reader_focuses_the_reader() {
    let (app, model) = panes();
    assert_eq!(model.borrow().focus, Focus::List);
    app.click(40, 10);
    assert_eq!(model.borrow().focus, Focus::Reader);
}

#[test]
fn a_press_in_the_list_focuses_the_list() {
    let (app, model) = panes();
    model.borrow_mut().focus = Focus::Reader;
    app.click(5, 10);
    assert_eq!(model.borrow().focus, Focus::List);
}

#[test]
fn a_press_on_a_pane_header_focuses_that_pane() {
    let (app, model) = panes();
    app.click(40, 1);
    assert_eq!(model.borrow().focus, Focus::Reader);
    app.click(5, 1);
    assert_eq!(model.borrow().focus, Focus::List);
}
