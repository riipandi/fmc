//! The three prompts through the keys: the finder's query, the in-file search, and the `:` line take Option and Cmd
//! edits at the caret, and Ctrl+Left edits the prompt rather than resizing the sidebar. Each edit that changes the text
//! finds again, as typing does.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmc::find::Matching;
use opentui::testing::TestApp;

use super::search_tests::reading_text;
use super::*;

const NOTES: &str = "# Hogwarts\n\nThe *Sorting Hat* waits for **Harry Potter** in the Great Hall.\n\n- Gryffindor: Harry Potter, Hermione Granger\n";

/// A folder with two files, with the finder open on it, the index walked as its worker walks it.
fn finding(dir: &Path) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(100, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), dir.to_path_buf(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    capture::wire(app.app(), &model, &ui);
    model.borrow_mut().matching = Matching::Substring;
    app.press("/");
    {
        let mut m = model.borrow_mut();
        let index = crate::services::find::index(dir, |_| false, fmc::find::MAX_INDEXED);
        let session = m.find_session;
        find::indexed(&mut m, session, index);
    }
    (app, model, ui)
}

fn folder() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("Hogwarts")).unwrap();
    fs::create_dir_all(dir.path().join("Gringotts")).unwrap();
    fs::write(dir.path().join("Hogwarts/owls.md"), "Hedwig\n").unwrap();
    fs::write(dir.path().join("Gringotts/vault.md"), "713\n").unwrap();
    dir
}

/// The finder's query and its caret, as the text before the caret and the text after it.
fn query(model: &Rc<RefCell<Model>>) -> (String, String) {
    let m = model.borrow();
    let line = &m.finder.as_ref().expect("the finder is open").query;
    (line.before().to_owned(), line.after().to_owned())
}

fn finder_status(model: &Rc<RefCell<Model>>) -> String {
    model.borrow().finder.as_ref().map(|finder| finder.status.clone()).unwrap_or_default()
}

#[test]
fn option_left_moves_the_finder_caret_and_typing_inserts_there() {
    let dir = folder();
    let (app, model, ui) = finding(dir.path());
    app.type_text("vault");
    assert_eq!(finder_status(&model), "1 match");
    app.press("alt+left");
    assert_eq!(query(&model), (String::new(), "vault".into()), "Option+Left goes to the start of the word");
    app.type_text("Gringotts/");
    assert_eq!(query(&model), ("Gringotts/".into(), "vault".into()));
    assert_eq!(finder_status(&model), "1 match", "the list is found again for the new query");
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    assert!(app.render().text().contains("Gringotts/▎vault"), "the caret is drawn where it is");
}

#[test]
fn typing_in_the_middle_of_the_finder_query_finds_again() {
    let dir = folder();
    let (app, model, _ui) = finding(dir.path());
    app.type_text("owls");
    app.press("left");
    app.press("left");
    app.press("left");
    assert_eq!(query(&model), ("o".into(), "wls".into()), "a plain arrow moves one character");
    app.type_text("z");
    assert_eq!(query(&model), ("oz".into(), "wls".into()));
    assert_eq!(finder_status(&model), "no matches");
    app.press("backspace");
    assert_eq!(query(&model), ("o".into(), "wls".into()), "Backspace deletes before the caret");
    assert_eq!(finder_status(&model), "1 match");
}

#[test]
fn ctrl_left_edits_the_finder_query_and_leaves_the_sidebar_alone() {
    let dir = folder();
    let (app, model, _ui) = finding(dir.path());
    app.type_text("Gringotts/vault");
    app.press("ctrl+left");
    assert_eq!(query(&model), ("Gringotts/".into(), "vault".into()), "Ctrl+Left is a word step in the prompt");
    assert_eq!(model.borrow().sidebar, None, "the sidebar keeps its width while the finder is open");
}

#[test]
fn ctrl_left_resizes_the_sidebar_when_no_prompt_is_open() {
    let app = TestApp::new(100, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), PathBuf::from("/hogwarts"), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    capture::wire(app.app(), &model, &ui);
    app.press("ctrl+left");
    assert!(model.borrow().sidebar.is_some(), "without a prompt, Ctrl+Left resizes the sidebar");
}

#[test]
fn cmd_arrows_and_ctrl_a_e_move_the_finder_caret_to_the_ends() {
    let dir = folder();
    let (app, model, _ui) = finding(dir.path());
    app.type_text("vault");
    app.feed(b"\x1b[1;9D"); // Cmd+Left, as Rex sends it
    assert_eq!(query(&model), (String::new(), "vault".into()));
    app.type_text("v");
    app.feed(b"\x05"); // Ctrl+E, the byte Ghostty sends for Cmd+Right
    app.type_text("!");
    assert_eq!(query(&model), ("vvault!".into(), String::new()));
    app.feed(b"\x01"); // Ctrl+A
    assert_eq!(query(&model), (String::new(), "vvault!".into()));
}

#[test]
fn option_backspace_deletes_the_word_before_the_finder_caret() {
    let dir = folder();
    let (app, model, _ui) = finding(dir.path());
    app.type_text("Gringotts/vault");
    app.feed(b"\x1b\x7f"); // Option+Backspace, as Rex sends it (ESC DEL)
    assert_eq!(query(&model), ("Gringotts/".into(), String::new()));
    assert_eq!(finder_status(&model), "1 match", "the query holds one folder's name");
}

#[test]
fn option_left_then_typing_inserts_in_the_reader_search_and_finds_again() {
    let (app, model, _ui) = reading_text(100, NOTES);
    app.press("/");
    app.type_text("harry potter");
    assert_eq!(model.borrow().search.as_ref().map(|s| s.message()), Some("/harry potter  1 of 2".into()));
    app.press("alt+left");
    app.type_text("x");
    let search = model.borrow();
    let search = search.search.as_ref().expect("the prompt is open");
    assert_eq!((search.query.before(), search.query.after()), ("harry x", "potter"));
    assert_eq!(search.message(), "/harry xpotter  no matches");
    assert!(app.render().text().contains("/ harry x▎potter"), "the search prompt draws its caret in place");
}

#[test]
fn ctrl_left_and_cmd_keys_edit_the_reader_search_and_leave_the_sidebar() {
    let (app, model, _ui) = reading_text(100, NOTES);
    app.press("/");
    app.type_text("harry potter");
    app.press("ctrl+left");
    assert_eq!(model.borrow().search.as_ref().map(|s| s.query.before().to_owned()), Some("harry ".into()));
    assert_eq!(model.borrow().sidebar, None, "Ctrl+Left edits the prompt, not the sidebar");
    app.feed(b"\x1b[1;9D"); // Cmd+Left
    app.type_text("the ");
    assert_eq!(model.borrow().search.as_ref().map(|s| s.query.text().to_owned()), Some("the harry potter".into()));
    app.feed(b"\x15"); // Ctrl+U deletes to the start, at the caret after "the "
    assert_eq!(model.borrow().search.as_ref().map(|s| s.query.text().to_owned()), Some("harry potter".into()));
    app.feed(b"\x1b\x7f"); // Option+Backspace at the start deletes nothing
    assert_eq!(model.borrow().search.as_ref().map(|s| s.query.text().to_owned()), Some("harry potter".into()));
}

#[test]
fn a_caret_move_in_the_search_keeps_the_hit_where_it_is() {
    let (app, model, _ui) = reading_text(100, NOTES);
    app.press("/");
    app.type_text("harry potter");
    let at = model.borrow().search.as_ref().map(|s| s.at);
    app.press("alt+left");
    app.press("left");
    assert_eq!(model.borrow().search.as_ref().map(|s| s.at), at, "a caret move does not step to another hit");
    assert_eq!(model.borrow().search.as_ref().map(|s| s.message()), Some("/harry potter  1 of 2".into()));
}

#[test]
fn option_left_edits_the_command_line_at_the_caret() {
    let app = TestApp::new(80, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), PathBuf::from("/hogwarts"), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    capture::wire(app.app(), &model, &ui);
    app.press(":");
    app.type_text("qx");
    app.feed(b"\x1b[1;3D"); // Option+Left, as Rex sends it
    app.type_text("w");
    assert_eq!(
        model.borrow().command.as_ref().map(|line| (line.before().to_owned(), line.after().to_owned())),
        Some(("w".into(), "qx".into()))
    );
    app.press("return");
    let message = model.borrow().message.clone().expect("a notice");
    assert_eq!(message.text, "Not a command: wqx");
    assert!(model.borrow().command.is_none());
}

#[test]
fn ctrl_k_cuts_the_command_line_from_the_caret_and_escape_cancels_it() {
    let app = TestApp::new(80, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), PathBuf::from("/hogwarts"), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    capture::wire(app.app(), &model, &ui);
    app.press(":");
    app.type_text("qa!");
    app.press("left");
    app.feed(b"\x0b"); // Ctrl+K deletes to the end
    assert_eq!(model.borrow().command.as_ref().map(|line| line.text().to_owned()), Some("qa".into()));
    app.press("escape");
    assert!(model.borrow().command.is_none());
}
