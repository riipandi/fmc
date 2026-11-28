use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmc::explorer::{Entry, Kind};
use fmc::reader::document::decode;
use opentui::testing::TestApp;

use super::super::{Model, Ui, build, new_model, paint};
use super::*;

/// A model browsing `/hogwarts`: two files and a folder, with `owls.txt` selected, on an 80-column screen.
fn browsing() -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    browsing_at(80)
}

/// The same browsing on a screen `columns` wide.
fn browsing_at(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    {
        let mut m = model.borrow_mut();
        m.browser.listed(
            &root,
            vec![
                Entry::new(&root, "owls.txt", Kind::File),
                Entry::new(&root, "hedwig.txt", Kind::File),
                Entry::new(&root, "hogsmeade", Kind::Directory),
            ],
        );
        m.browser.select(&root.join("owls.txt"));
    }
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

fn text(name: &str) -> Result<fs::Opened, String> {
    Ok(fs::Opened::Text { document: decode(format!("{name}\n").as_bytes()).unwrap(), stamp: String::new() })
}

#[test]
fn only_the_newest_move_is_read() {
    let (app, model, _ui) = browsing();
    moved(app.app(), &model);
    moved(app.app(), &model);
    let m = model.borrow();
    assert_eq!(due(&m, 1), None, "an older move is not read");
    assert_eq!(due(&m, 2), Some(PathBuf::from("/hogwarts/owls.txt")));
}

#[test]
fn a_folder_selection_has_nothing_to_read() {
    let (_app, model, _ui) = browsing();
    let mut m = model.borrow_mut();
    m.browser.select(Path::new("/hogwarts/hogsmeade"));
    m.preview_generation = 1;
    assert_eq!(due(&m, 1), None);
}

#[test]
fn a_file_the_reader_shows_is_not_read_again() {
    let (_app, model, _ui) = browsing();
    let mut m = model.borrow_mut();
    m.preview_generation = 1;
    m.shown = reader::Shown::Text {
        path: PathBuf::from("/hogwarts/owls.txt"),
        document: decode(b"Hedwig\n").unwrap(),
        language: None,
        markdown: None,
        stamp: Some(String::new()),
        serial: 0,
        signs: Vec::new(),
    };
    assert_eq!(due(&m, 1), None);
}

#[test]
fn the_list_focus_and_the_setting_gate_a_preview() {
    let (_app, model, _ui) = browsing();
    let mut m = model.borrow_mut();
    m.preview_generation = 1;
    m.focus = Focus::Reader;
    assert_eq!(due(&m, 1), None, "the reader has the focus");
    m.focus = Focus::List;
    m.auto_preview = false;
    assert_eq!(due(&m, 1), None, "auto-preview is off");
}

#[test]
fn a_preview_keeps_the_focus_and_records_no_jump() {
    let (_app, model, ui) = browsing();
    read(&mut model.borrow_mut(), &ui.borrow(), Path::new("/hogwarts/owls.txt"), &text("Hedwig"));
    let m = model.borrow();
    assert!(m.shown.text().is_some(), "the file shows");
    assert_eq!(m.focus, Focus::List);
    assert!(!m.jumps.can_go_back());
}

#[test]
fn a_read_of_a_file_the_list_has_left_is_dropped() {
    let (_app, model, ui) = browsing();
    read(&mut model.borrow_mut(), &ui.borrow(), Path::new("/hogwarts/hedwig.txt"), &text("Hedwig"));
    assert!(matches!(model.borrow().shown, reader::Shown::None));
}

#[test]
fn p_turns_auto_preview_off_and_on_and_says_so() {
    let (_app, model, _ui) = browsing();
    toggle(&mut model.borrow_mut());
    {
        let m = model.borrow();
        assert!(!m.auto_preview);
        assert_eq!(m.message.as_ref().map(|message| message.text.as_str()), Some("Auto-preview off."));
    }
    toggle(&mut model.borrow_mut());
    assert!(model.borrow().auto_preview);
}

#[test]
fn compact_schedules_no_preview_because_the_reader_is_a_page() {
    let (app, model, ui) = browsing_at(40);
    model.borrow_mut().focus = Focus::List;
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    let mut m = model.borrow_mut();
    m.preview_generation = 1;
    assert_eq!(due(&m, 1), None, "the reader page is not on screen while the list is");
}
