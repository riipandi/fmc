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

#[test]
fn a_tap_on_the_reader_header_shows_a_hidden_sidebar_at_every_phone_width() {
    for columns in [40, 59, 60, 79] {
        let (app, model, ui) = panes(columns);
        open_document(&app, &model, &ui);
        assert!(!layout::shown(&model.borrow(), columns), "{columns}: a document hides the sidebar");
        app.click(columns / 2, 1);
        let m = model.borrow();
        assert!(layout::shown(&m, columns), "{columns}: the header tap shows the sidebar");
        assert_eq!(m.focus, Focus::List, "{columns}: and focuses it");
        assert_eq!(layout::drawer(&m, columns), columns < 60, "{columns}: a drawer below 60 columns");
        let width = if columns < 60 { columns } else { 28 };
        assert_eq!(layout::width(&m, columns), width, "{columns}: the drawer is full width, a sidebar 28 wide");
    }
}

#[test]
fn a_tap_on_the_reader_header_does_nothing_at_80_columns() {
    let (app, model, ui) = panes(80);
    open_document(&app, &model, &ui);
    app.click(40, 1);
    let m = model.borrow();
    assert!(layout::shown(&m, 80));
    assert_eq!(m.focus, Focus::Reader, "the press focuses the reader, as before");
}

#[test]
fn a_tap_on_the_list_header_closes_a_drawer_below_60_columns() {
    let (app, model, ui) = panes(40);
    open_document(&app, &model, &ui);
    app.click(20, 1);
    assert!(layout::shown(&model.borrow(), 40));
    app.click(5, 1);
    assert!(!layout::shown(&model.borrow(), 40));
    assert_eq!(model.borrow().focus, Focus::Reader);
}

#[test]
fn a_drawer_covers_the_reader_until_the_focus_reaches_the_reader() {
    let (app, model, ui) = panes(40);
    open_document(&app, &model, &ui);
    app.click(20, 1);
    assert!(!app.render().row(1).contains("Reader"), "the list takes the full width");
    model.borrow_mut().focus = Focus::Reader;
    layout::sync(&mut model.borrow_mut(), 40);
    assert!(!layout::shown(&model.borrow(), 40));
}

#[test]
fn the_welcome_screen_is_a_full_width_drawer_below_60_columns() {
    let (_narrow_app, narrow, _) = panes(40);
    assert!(layout::drawer(&narrow.borrow(), 40));
    assert_eq!(layout::width(&narrow.borrow(), 40), 40);
    let (_wide_app, wide, _) = panes(60);
    assert!(!layout::drawer(&wide.borrow(), 60));
    assert_eq!(layout::width(&wide.borrow(), 60), 28);
}

#[test]
fn closing_a_drawer_leaves_a_sidebar_beside_the_reader_alone() {
    let (app, model, ui) = panes(60);
    open_document(&app, &model, &ui);
    app.click(0, 0);
    layout::close_drawer(&mut model.borrow_mut(), 60);
    assert!(layout::shown(&model.borrow(), 60));
}
