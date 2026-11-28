use std::path::PathBuf;

use fmc::reader::document::decode;
use opentui::testing::TestApp;

use super::*;

/// The panes wired with the key capture, and a long unwrapped line open in the reader, on a screen `columns` wide.
fn reading(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    reading_text(columns, &format!("{}\n", "Expelliarmus! ".repeat(40)))
}

/// The same panes with `text` open in the reader.
fn reading_text(columns: u16, text: &str) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root, None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    wire_mouse(&model, &ui);
    capture::wire(app.app(), &model, &ui);
    let document = decode(text.as_bytes()).expect("a text document");
    ui.borrow().reader.show(&document, None);
    model.borrow_mut().shown = reader::Shown::Text {
        path: PathBuf::from("/hogwarts/spells.txt"),
        document,
        language: None,
        markdown: None,
        stamp: Some(String::new()),
        serial: 0,
        signs: Vec::new(),
        lossy: None,
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

#[test]
fn brackets_scroll_the_reader_by_the_reader_jump() {
    let (app, model, ui) = reading_text(80, &"Hedwig\n".repeat(80));
    app.click(40, 10);
    assert_eq!(model.borrow().focus, Focus::Reader);
    app.press("]");
    assert_eq!(ui.borrow().viewer.position().1, 5.0);
    app.press("[");
    assert_eq!(ui.borrow().viewer.position().1, 0.0);
}

/// Thirty numbered lines, each longer than the reader at 80 columns, for the gutter tests.
fn panning_text() -> String {
    (0..30).map(|i| format!("line {i:02} {}\n", "0123456789".repeat(8))).collect()
}

/// A screen row of the reader at 80 columns: from its first column, past the list and its border.
fn reader_row(app: &TestApp, y: u32) -> String {
    app.render().row(y).chars().skip(29).take(50).collect()
}

#[test]
fn the_line_numbers_stay_put_when_the_reader_pans() {
    let (app, _model, _ui) = reading_text(80, &panning_text());
    app.click(40, 10);
    assert!(reader_row(&app, 3).starts_with("  1  line 00"));
    app.press("l");
    assert!(reader_row(&app, 3).starts_with("  1  "), "the number keeps its column");
    assert!(!reader_row(&app, 3).starts_with("  1  line"), "the text moved");
    assert!(reader_row(&app, 4).starts_with("  2  "), "every row keeps its number");
}

#[test]
fn a_sideways_wheel_pans_the_text_and_not_the_numbers() {
    let (app, _model, _ui) = reading_text(80, &panning_text());
    app.click(40, 10);
    app.mouse(opentui::input::MouseEventKind::ScrollRight, 40, 10);
    assert!(reader_row(&app, 3).starts_with("  1  "));
    assert!(!reader_row(&app, 3).starts_with("  1  line"), "the wheel moved the text");
}

#[test]
fn a_pan_moves_the_text_by_the_pan_step() {
    let (app, _model, _ui) = reading_text(80, &panning_text());
    app.click(40, 10);
    app.press("l");
    // `l` pans four columns, so "line 00" starts four characters earlier, after the five-column gutter.
    assert!(reader_row(&app, 3).starts_with("  1   00 0123"), "{}", reader_row(&app, 3));
}

#[test]
fn a_selection_after_a_pan_copies_the_visible_text_and_no_number() {
    let (app, _model, _ui) = reading_text(80, &panning_text());
    app.click(40, 10);
    app.press("l");
    let visible: String = reader_row(&app, 3).chars().skip(5).take(4).collect();
    app.drag((34, 3), (37, 3));
    assert_eq!(app.app().selected_text().as_deref(), Some(visible.as_str()));
}

#[test]
fn wrapping_keeps_the_numbers_beside_the_text() {
    let (app, _model, _ui) = reading_text(80, &panning_text());
    app.click(40, 10);
    app.press("w");
    assert!(reader_row(&app, 3).starts_with("  1  line 00"), "{}", reader_row(&app, 3));
}

#[test]
fn the_numbers_follow_a_vertical_scroll() {
    let (app, _model, _ui) = reading_text(80, &panning_text());
    app.click(40, 10);
    app.press("j");
    app.press("j");
    assert!(reader_row(&app, 3).starts_with("  3  line 02"), "{}", reader_row(&app, 3));
    app.press("l");
    assert!(reader_row(&app, 3).starts_with("  3  "), "the numbers keep their rows after a pan too");
}

#[test]
fn a_wheel_over_the_gutter_scrolls_the_text_beside_it() {
    let (app, _model, _ui) = reading_text(80, &panning_text());
    app.click(40, 10);
    app.mouse(opentui::input::MouseEventKind::ScrollDown, 30, 10);
    app.advance(std::time::Duration::from_millis(50));
    assert!(reader_row(&app, 3).starts_with("  2  line 01"), "{}", reader_row(&app, 3));
}

#[test]
fn a_key_from_the_keymap_file_runs_its_action_and_the_default_key_does_nothing() {
    let (app, model, _ui) = reading(120);
    let keymap = r#"{"everywhere.ctrl-,": null, "everywhere.ctrl-s": "open_settings"}"#;
    model.borrow_mut().parser = Parser::new(fmc::keys::keymap::parse(keymap).keymap);
    app.key(KeyEvent::new(KeyCode::Char(','), Modifiers::CTRL));
    assert!(model.borrow().settings.page.is_none(), "the default Ctrl+, is unbound");
    app.key(KeyEvent::new(KeyCode::Char('s'), Modifiers::CTRL));
    assert!(model.borrow().settings.page.is_some(), "the rebound Ctrl+s opens the Settings page");
}
