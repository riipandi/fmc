use std::path::Path;

use fmc::explorer::{Entry, Kind};
use opentui::testing::TestApp;

use super::*;
use crate::ui::tree::LIST_TOP;

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

#[test]
fn a_press_from_another_pane_only_focuses_the_list() {
    let (app, model) = panes();
    let root = PathBuf::from("/hogwarts");
    {
        let mut m = model.borrow_mut();
        m.browser.listed(
            &root,
            vec![Entry::new(&root, "owls.txt", Kind::File), Entry::new(&root, "hedwig.txt", Kind::File)],
        );
        m.browser.select(&root.join("hedwig.txt"));
        m.focus = Focus::Reader;
    }
    app.click(5, LIST_TOP);
    assert_eq!(model.borrow().focus, Focus::List);
    assert_eq!(model.borrow().browser.selected(), root.join("hedwig.txt"), "the selection does not move");
    app.click(5, LIST_TOP);
    assert_eq!(model.borrow().browser.selected(), root.join("owls.txt"), "the next press selects the row");
}
