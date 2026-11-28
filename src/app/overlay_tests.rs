use std::path::{Path, PathBuf};

use fmc::explorer::{Entry, Kind};
use fmc::overlay::{Dialog, FieldEdit, Stop};
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
    for _ in 0..5 {
        app.press("down");
    }
    assert_eq!(
        selected(&model),
        Some(5),
        "Rename is the sixth row, after Open, Open With, Show in Finder, and the new items"
    );
    for _ in 0..2 {
        app.press("down");
    }
    assert_eq!(selected(&model), Some(7), "Cut is the eighth row");
    app.press("down");
    assert_eq!(selected(&model), Some(9), "Paste is disabled with nothing copied, so the selection skips it");
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
    // The menu sits at the list's first row (column 2, row 3): Open is on row 4, and Paste is the ninth, below the
    // rules that open the second and third groups.
    app.click(5, 14);
    assert!(is_open(&model), "Paste is disabled");
    app.click(5, 8);
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

/// A name dialog over the page, its field holding `text` with the caret at the end. The bytes in the tests below are
/// what Rex sends for each chord (`rex send-key`, legacy and with the kitty keyboard protocol), so each test feeds
/// them through the parser.
fn name_field(text: &str) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let (app, model, ui) = screen(120, 40);
    open_dialog(&mut model.borrow_mut(), &ui.borrow(), Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1));
    ui.borrow().overlay.dialog.field.set_value(text);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

/// The field's text and caret (in bytes; the test names are ASCII).
fn field(ui: &Rc<RefCell<Ui>>) -> (String, u32) {
    let ui = ui.borrow();
    (ui.overlay.dialog.field.value(), ui.overlay.dialog.field.cursor_offset())
}

#[test]
fn option_and_ctrl_arrows_move_the_caret_by_word() {
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b[1;3D"); // Option+Left, as Rex sends it
    assert_eq!(field(&ui), ("sophie neveu".into(), 7));
    app.feed(b"\x1bb"); // Option+Left as ESC b
    assert_eq!(field(&ui), ("sophie neveu".into(), 0));
    app.feed(b"\x1b[1;3C"); // Option+Right: OpenTUI stops at the start of the next word
    assert_eq!(field(&ui), ("sophie neveu".into(), 7));
    app.feed(b"\x1b[98;3u"); // Option+B with the kitty protocol
    assert_eq!(field(&ui), ("sophie neveu".into(), 0));
    app.feed(b"\x1b[102;3u"); // Option+F with the kitty protocol
    assert_eq!(field(&ui), ("sophie neveu".into(), 7));
    app.feed(b"\x1bf"); // Option+Right as ESC f, past the last word
    assert_eq!(field(&ui), ("sophie neveu".into(), 12));
    app.feed(b"\x1b[1;5D"); // Ctrl+Left
    assert_eq!(field(&ui), ("sophie neveu".into(), 7));
}

#[test]
fn cmd_arrows_and_ctrl_a_e_move_to_the_ends_of_the_line() {
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b[1;9D"); // Cmd+Left, as Rex sends it
    assert_eq!(field(&ui), ("sophie neveu".into(), 0));
    app.feed(b"\x1b[1;9C"); // Cmd+Right
    assert_eq!(field(&ui), ("sophie neveu".into(), 12));
    app.feed(b"\x01"); // Ctrl+A, the byte Ghostty sends for Cmd+Left
    assert_eq!(field(&ui), ("sophie neveu".into(), 0));
    app.feed(b"\x05"); // Ctrl+E, the byte Ghostty sends for Cmd+Right
    assert_eq!(field(&ui), ("sophie neveu".into(), 12));
}

#[test]
fn option_backspace_deletes_the_word_before_the_caret() {
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b\x7f"); // Option+Backspace, as Rex sends it (ESC DEL)
    assert_eq!(field(&ui), ("sophie ".into(), 7));
    app.feed(b"\x17"); // Ctrl+W
    assert_eq!(field(&ui), ("".into(), 0));
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b[127;3u"); // Option+Backspace with the kitty protocol
    assert_eq!(field(&ui), ("sophie ".into(), 7));
}

#[test]
fn option_delete_deletes_the_word_after_the_caret() {
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b[1;9D"); // Cmd+Left: the caret goes to the start
    app.feed(b"\x1b[3;3~"); // Option+Delete, as Rex sends it: the word and the space after it
    assert_eq!(field(&ui), ("neveu".into(), 0));
    app.feed(b"\x1bd"); // Option+D: the last word
    assert_eq!(field(&ui), ("".into(), 0));
}

#[test]
fn cmd_backspace_and_ctrl_u_delete_to_the_start_of_the_line() {
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b[1;3D"); // the caret to the start of "neveu"
    app.feed(b"\x15"); // Ctrl+U, the byte Ghostty sends for Cmd+Backspace
    assert_eq!(field(&ui), ("neveu".into(), 0));
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b[1;3D");
    app.feed(b"\x1b[127;9u"); // Cmd+Backspace with the kitty protocol
    assert_eq!(field(&ui), ("neveu".into(), 0));
}

#[test]
fn ctrl_k_deletes_to_the_end_and_typing_goes_at_the_caret() {
    let (app, _model, ui) = name_field("sophie neveu");
    app.feed(b"\x1b[1;9D"); // Cmd+Left
    app.feed(b"\x1bf"); // Option+Right: the caret at the start of "neveu"
    app.feed(b"\x0b"); // Ctrl+K
    assert_eq!(field(&ui), ("sophie ".into(), 7));
    app.feed(b"\x1b[1;9D");
    app.type_text("X");
    assert_eq!(field(&ui), ("Xsophie ".into(), 1));
}

#[test]
fn opentui_input_binds_each_field_edit_to_the_same_action() {
    let keymap = Input::default_keymap();
    let cases = [
        ("alt+left", FieldEdit::WordBackward),
        ("alt+b", FieldEdit::WordBackward),
        ("ctrl+left", FieldEdit::WordBackward),
        ("alt+right", FieldEdit::WordForward),
        ("alt+f", FieldEdit::WordForward),
        ("ctrl+right", FieldEdit::WordForward),
        ("alt+backspace", FieldEdit::DeleteWordBackward),
        ("ctrl+w", FieldEdit::DeleteWordBackward),
        ("ctrl+backspace", FieldEdit::DeleteWordBackward),
        ("alt+delete", FieldEdit::DeleteWordForward),
        ("alt+d", FieldEdit::DeleteWordForward),
        ("ctrl+delete", FieldEdit::DeleteWordForward),
        ("ctrl+a", FieldEdit::LineHome),
        ("ctrl+e", FieldEdit::LineEnd),
        ("super+backspace", FieldEdit::DeleteToLineStart),
        ("ctrl+u", FieldEdit::DeleteToLineStart),
        ("ctrl+k", FieldEdit::DeleteToLineEnd),
        ("backspace", FieldEdit::Backspace),
    ];
    for (shortcut, edit) in cases {
        assert_eq!(keymap.lookup(shortcut), Some(editor_action(edit)), "{shortcut}");
    }
}
