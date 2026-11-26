use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use fmc::explorer::{Entry, Expand, Kind};
use fmc::reader::document::decode;
use opentui::testing::TestApp;

use super::super::{Model, Ui, build, new_model, paint};
use super::*;

/// A model browsing `/hogwarts`, which lists two files, with the first one selected.
fn browsing() -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(80, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&root, vec![file(&root, "owls.txt"), file(&root, "hedwig.txt")]);
        m.browser.select(&root.join("owls.txt"));
    }
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

fn file(folder: &Path, name: &str) -> Entry {
    Entry::new(folder, name, Kind::File)
}

#[test]
fn enter_on_a_file_opens_it_and_focuses_the_reader() {
    let (_app, model, ui) = browsing();
    let path = PathBuf::from("/hogwarts/owls.txt");
    activate(&mut model.borrow_mut());
    assert!(model.borrow().loading, "the file is read off the UI thread");
    let event = Event::Opened {
        path: path.clone(),
        result: Ok(fs::Opened::Text { document: decode(b"Hedwig\n").unwrap(), stamp: String::new() }),
    };
    apply(&mut model.borrow_mut(), &ui.borrow(), &event);
    assert!(model.borrow().shown.text().is_some(), "the file is shown");
    assert_eq!(model.borrow().focus, Focus::Reader);
}

#[test]
fn right_on_a_file_does_nothing() {
    let (_app, model, _ui) = browsing();
    act(&mut model.borrow_mut(), Action::Expand);
    let m = model.borrow();
    assert!(!m.loading, "only Enter opens a file");
    assert!(!m.focus_reader_on_open);
    assert_eq!(m.focus, Focus::List);
}

#[test]
fn a_click_on_a_file_only_selects_it() {
    let (_app, model, _ui) = browsing();
    click(&mut model.borrow_mut());
    let m = model.borrow();
    assert!(!m.loading);
    assert!(matches!(m.shown, reader::Shown::None));
}

#[test]
fn a_click_on_a_folder_toggles_it() {
    let (_app, model, _ui) = browsing();
    let root = PathBuf::from("/hogwarts");
    let hogsmeade = root.join("hogsmeade");
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&root, vec![Entry::new(&root, "hogsmeade", Kind::Directory)]);
        m.browser.listed(&hogsmeade, Vec::new());
        m.browser.select(&hogsmeade);
        assert_eq!(m.browser.tree.expand(&hogsmeade), Expand::Ready);
    }
    click(&mut model.borrow_mut());
    assert!(!model.borrow().browser.tree.is_expanded(&hogsmeade));
}

#[test]
fn brackets_move_the_list_by_the_sidebar_jump() {
    let (_app, model, _ui) = browsing();
    let root = PathBuf::from("/hogwarts");
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
        m.sidebar_jump = 2;
    }
    jump(&mut model.borrow_mut(), 1);
    assert_eq!(model.borrow().browser.selected(), root.join("hogsmeade"));
    jump(&mut model.borrow_mut(), -1);
    assert_eq!(model.borrow().browser.selected(), root.join("owls.txt"));
}
