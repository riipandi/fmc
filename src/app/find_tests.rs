//! The finder's fuzzy matching through the keys: `find_mode` sets the start, `Ctrl+F` flips it for
//! the session, and the footer names fuzzy while it is on. The index is walked at once, as the finder's worker walks it.

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use fmc::find::Matching;
use opentui::testing::TestApp;

use super::*;

fn finding(dir: &Path, matching: Matching) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(80, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), dir.to_path_buf(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    capture::wire(app.app(), &model, &ui);
    model.borrow_mut().matching = matching;
    app.press("/");
    {
        let mut m = model.borrow_mut();
        let index = crate::services::find::index(dir, |_| false, fmc::find::MAX_INDEXED);
        let session = m.find_session;
        find::indexed(&mut m, session, index);
    }
    (app, model, ui)
}

/// A folder with two files. `hgw` matches the folder `Hogwarts` and `Hogwarts/owls.md` fuzzily; no substring holds it.
fn folder() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("Hogwarts")).unwrap();
    fs::create_dir_all(dir.path().join("Gringotts")).unwrap();
    fs::write(dir.path().join("Hogwarts/owls.md"), "Hedwig\n").unwrap();
    fs::write(dir.path().join("Gringotts/vault.md"), "713\n").unwrap();
    dir
}

fn status(model: &Rc<RefCell<Model>>) -> String {
    model.borrow().finder.as_ref().map(|finder| finder.status.clone()).unwrap_or_default()
}

#[test]
fn ctrl_f_flips_the_match_and_the_footer_names_fuzzy_while_it_is_on() {
    let dir = folder();
    let (app, model, ui) = finding(dir.path(), Matching::Substring);
    app.type_text("hgw");
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    assert_eq!(status(&model), "no matches");

    app.press("ctrl+f");
    assert_eq!(model.borrow().matching, Matching::Fuzzy);
    assert_eq!(status(&model), "fuzzy, 2 matches");

    app.press("ctrl+f");
    assert_eq!(model.borrow().matching, Matching::Substring);
    assert_eq!(status(&model), "no matches");
}

#[test]
fn a_fuzzy_start_mode_matches_from_the_first_letter_and_the_flip_keeps_the_query() {
    let dir = folder();
    let (app, model, _ui) = finding(dir.path(), Matching::Fuzzy);
    app.type_text("hgw");
    assert_eq!(status(&model), "fuzzy, 2 matches");
    assert_eq!(model.borrow().finder.as_ref().map(|finder| finder.query.clone()), Some("hgw".to_string()));
}

#[test]
fn ctrl_f_outside_the_finder_changes_nothing() {
    let dir = folder();
    let app = TestApp::new(80, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), dir.path().to_path_buf(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    capture::wire(app.app(), &model, &ui);
    app.press("ctrl+f");
    assert_eq!(model.borrow().matching, Matching::Substring);
    assert!(model.borrow().finder.is_none());
}

/// The bytes a terminal sends for Ctrl+F: the legacy `0x06`, and the kitty protocol's CSI u form.
#[test]
fn raw_ctrl_f_from_the_terminal_flips_the_match_in_either_encoding() {
    for bytes in [&b"\x06"[..], &b"\x1b[102;5u"[..]] {
        let dir = folder();
        let (app, model, _ui) = finding(dir.path(), Matching::Substring);
        app.type_text("hgw");
        app.feed(bytes);
        assert_eq!(model.borrow().matching, Matching::Fuzzy, "bytes {bytes:?}");
        assert_eq!(status(&model), "fuzzy, 2 matches", "bytes {bytes:?}");
    }
}

/// Ctrl+F before the first letter has no match count to change, so the prompt's hint must show the mode.
#[test]
fn raw_ctrl_f_on_an_empty_prompt_shows_the_mode_in_the_hint() {
    let dir = folder();
    let (app, model, _ui) = finding(dir.path(), Matching::Substring);
    app.feed(b"\x06");
    assert_eq!(status(&model), "fuzzy, type to find a file, Tab to search inside files");
    app.feed(b"\x06");
    assert_eq!(status(&model), "type to find a file, Tab to search inside files");
}
