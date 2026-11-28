use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use fmc::keys::Key;
use opentui::testing::TestApp;

use super::super::{Model, build, new_model};
use super::*;

fn model() -> Rc<RefCell<Model>> {
    let app = TestApp::new(80, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root, None)));
    let _ui = build(app.app(), model.borrow().palette());
    model
}

/// One plain key press: a printable key names itself, as OpenTUI names it.
fn press(m: &mut Model, name: &str, text: &str) -> bool {
    prompt_key(m, Key { name, text, ctrl: false, alt: false, shift: false, cmd: false })
}

fn type_text(m: &mut Model, text: &str) {
    for c in text.chars() {
        let c = c.to_string();
        press(m, &c, &c);
    }
}

#[test]
fn the_quit_words_quit_on_enter() {
    for word in ["q", "q!", "qa", "qa!", "quit"] {
        let model = model();
        let mut m = model.borrow_mut();
        open(&mut m);
        type_text(&mut m, word);
        assert!(press(&mut m, "return", "\r"), ":{word} quits");
        assert!(m.command.is_none());
    }
}

#[test]
fn an_empty_line_cancels_without_a_notice() {
    let model = model();
    let mut m = model.borrow_mut();
    open(&mut m);
    assert!(!press(&mut m, "return", "\r"));
    assert!(m.command.is_none());
    assert!(m.message.is_none());
}

#[test]
fn an_unknown_command_names_itself() {
    let model = model();
    let mut m = model.borrow_mut();
    open(&mut m);
    type_text(&mut m, "w");
    assert!(!press(&mut m, "return", "\r"));
    let message = m.message.as_ref().expect("a notice");
    assert_eq!(message.text, "Not a command: w");
    assert!(message.error);
}

#[test]
fn escape_cancels_and_backspace_edits() {
    let model = model();
    let mut m = model.borrow_mut();
    open(&mut m);
    type_text(&mut m, "qx");
    press(&mut m, "backspace", "");
    assert_eq!(m.command.as_deref(), Some("q"));
    press(&mut m, "escape", "");
    assert!(m.command.is_none());
}
