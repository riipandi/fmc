use std::path::{Path, PathBuf};

use fmc::explorer::{Entry, Kind};
use fmc::overlay::{Dialog, Stop};
use opentui::testing::TestApp;

use super::*;

/// The panes wired as the app wires them, on a screen `columns` by `rows`, with the list focused.
fn screen(columns: u16, rows: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, rows);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    {
        // One file listed and selected, so the menu has a row to act on.
        let mut m = model.borrow_mut();
        m.browser.listed(&root, vec![Entry::new(&root, "owls.txt", Kind::File)]);
        m.browser.select(&root.join("owls.txt"));
    }
    wire_mouse(&model, &ui);
    capture::wire(app.app(), &model, &ui);
    wire_views(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

fn selected(model: &Rc<RefCell<Model>>) -> Option<usize> {
    match &model.borrow().overlay {
        Some(Overlay::Menu(menu)) => Some(menu.selected()),
        _ => None,
    }
}

fn is_open(model: &Rc<RefCell<Model>>) -> bool {
    model.borrow().overlay.is_some()
}

#[test]
fn space_opens_the_menu_and_the_page_gets_no_key_while_it_is_open() {
    let (app, model, _) = screen(120, 40);
    app.press("space");
    assert_eq!(selected(&model), Some(0));
    model.borrow_mut().focus = Focus::Reader;
    app.press("tab");
    app.press("j");
    assert_eq!(model.borrow().focus, Focus::Reader, "Tab is swallowed");
    assert_eq!(selected(&model), Some(1), "j moves the menu; the parser never sees it");
}

#[test]
fn the_menu_skips_the_disabled_rows_and_escape_closes_it() {
    let (app, model, _) = screen(120, 40);
    app.press("space");
    for _ in 0..2 {
        app.press("down");
    }
    assert_eq!(selected(&model), Some(2), "Rename is the third row");
    app.press("down");
    assert_eq!(selected(&model), Some(6), "Copy, Cut, and Paste are disabled and skipped");
    app.press("escape");
    assert!(!is_open(&model));
}

#[test]
fn a_press_outside_the_menu_closes_it() {
    let (app, model, _) = screen(120, 40);
    app.press("space");
    app.render();
    app.click(100, 30);
    assert!(!is_open(&model));
}

#[test]
fn a_press_on_a_disabled_row_keeps_the_menu_and_on_an_enabled_row_picks_it() {
    let (app, model, _) = screen(120, 40);
    app.press("space");
    app.render();
    // The menu sits at the list's first row (column 2, row 3): its items start on row 4, and Paste is the sixth.
    app.click(5, 9);
    assert!(is_open(&model), "Paste is disabled");
    app.click(5, 4);
    assert!(
        matches!(model.borrow().overlay, Some(Overlay::Dialog(_))),
        "New file is picked: the menu closes and its name dialog opens"
    );
}

#[test]
fn a_dialog_takes_typing_into_its_field_and_return_closes_it() {
    let (app, model, ui) = screen(120, 40);
    let dialog = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
    open_dialog(&mut model.borrow_mut(), &ui.borrow(), dialog);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    for key in ["r", "o", "n"] {
        app.press(key);
    }
    assert_eq!(ui.borrow().overlay.dialog.field.value(), "ron");
    app.press("backspace");
    assert_eq!(ui.borrow().overlay.dialog.field.value(), "ro");
    app.press("return");
    assert!(!is_open(&model));
}

#[test]
fn tab_moves_a_confirm_dialogs_focus_and_escape_closes_it() {
    let (app, model, ui) = screen(120, 40);
    let dialog = Dialog::new("Move to Trash?", "Gone.", false, &["Cancel", "Delete"], 0);
    open_dialog(&mut model.borrow_mut(), &ui.borrow(), dialog);
    app.press("tab");
    let focus = match &model.borrow().overlay {
        Some(Overlay::Dialog(dialog)) => dialog.focus,
        _ => panic!("the dialog closed on Tab"),
    };
    assert_eq!(focus, Stop::Button(1));
    app.press("escape");
    assert!(!is_open(&model));
}
