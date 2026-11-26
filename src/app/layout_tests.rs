use std::path::Path;

use opentui::testing::TestApp;

use super::*;

/// The panes, wired and painted on a headless screen `columns` wide, with nothing open.
fn panes(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root, None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    wire_mouse(&model, &ui);
    app.render();
    (app, model, ui)
}

/// Opens the key page, which is a document for the 80-column rule.
fn open_document(app: &TestApp, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let mut m = model.borrow_mut();
    reader::toggle_keys(&mut m, &ui.borrow());
    paint(app.app(), &mut m, &mut ui.borrow_mut());
    app.render();
}

#[test]
fn the_welcome_screen_keeps_the_sidebar_at_any_width() {
    let (app, model, _) = panes(60);
    assert!(layout::shown(&model.borrow(), 60));
    assert_eq!(app.render().row(0).chars().nth(26), Some('─'), "no mark on the welcome screen");
}

#[test]
fn a_document_under_80_columns_hides_the_sidebar_and_the_mark_brings_it_back() {
    let (app, model, ui) = panes(79);
    open_document(&app, &model, &ui);
    assert!(!layout::shown(&model.borrow(), 79));
    assert_eq!(app.render().row(0).chars().next(), Some('»'));
    app.click(0, 0);
    assert!(layout::shown(&model.borrow(), 79));
    assert_eq!(app.render().row(0).chars().nth(26), Some('«'));
}

#[test]
fn the_mark_that_shows_the_sidebar_focuses_it() {
    let (app, model, ui) = panes(79);
    open_document(&app, &model, &ui);
    model.borrow_mut().focus = Focus::Reader;
    app.click(0, 0);
    assert_eq!(model.borrow().focus, Focus::List);
}

#[test]
fn a_hidden_sidebar_does_not_keep_the_focus() {
    let (app, model, ui) = panes(79);
    open_document(&app, &model, &ui);
    model.borrow_mut().focus = Focus::List;
    layout::sync(&mut model.borrow_mut(), 79);
    assert_eq!(model.borrow().focus, Focus::Reader);
}

#[test]
fn ctrl_arrows_do_not_resize_a_hidden_sidebar() {
    let (app, model, ui) = panes(79);
    open_document(&app, &model, &ui);
    let before = model.borrow().sidebar;
    layout::resize(&mut model.borrow_mut(), 79, 1);
    assert_eq!(model.borrow().sidebar, before);
}
