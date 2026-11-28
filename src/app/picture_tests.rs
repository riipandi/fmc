//! The rendered view's image boxes in the app: `Enter` and a click open a box's picture, a box with a reason says why,
//! and `Ctrl+o` returns to the Markdown at its scroll. The worker's answers are applied by hand, as the tree tests do.

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use fmc::explorer::{Entry, Kind};
use fmc::reader::document::decode;
use fmc::reader::images::Header;
use opentui::testing::TestApp;

use crate::app::{Focus, Model, Ui, build, capture, new_model, paint, reader, tree, wire_mouse};
use crate::services::{self, Event, picture};
use crate::ui::markdown_images;

const DOCUMENT: &str = "# Hogwarts\n\n![Marauder map](../maps/marauder.png)\n\nThe map is old.\n\n![Mona Lisa](../Louvre/mona.png)\n\n![Outside](../../../etc/passwd.png)\n";

/// Bytes of a PNG header: the signature and an IHDR with the size, which the header read takes.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 2, 0, 0, 0, 0, 0, 0, 0]);
    bytes
}

struct Hogwarts {
    _dir: tempfile::TempDir,
    root: PathBuf,
    notes: PathBuf,
    document: PathBuf,
}

/// A Hogwarts folder with `notes/hogwarts.md`, two pictures, and the tree listed down to the notes.
fn hogwarts() -> Hogwarts {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let root = dir.path().join("hogwarts");
    let notes = root.join("notes");
    fs::create_dir_all(root.join("maps")).expect("maps");
    fs::create_dir_all(root.join("Louvre")).expect("Louvre");
    fs::create_dir_all(&notes).expect("notes");
    fs::write(root.join("maps/marauder.png"), png(480, 270)).expect("the marauder map");
    fs::write(root.join("Louvre/mona.png"), png(640, 480)).expect("the portrait");
    let document = notes.join("hogwarts.md");
    fs::write(&document, DOCUMENT).expect("the notes");
    Hogwarts { _dir: dir, root, notes, document }
}

/// The reader on the notes, its rendered view shown with each box's header read as the worker would read it.
fn reading(fixture: &Hogwarts) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(80, 40);
    let model = Rc::new(RefCell::new(new_model(app.app(), fixture.root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    wire_mouse(&model, &ui);
    capture::wire(app.app(), &model, &ui);
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&fixture.root, vec![Entry::new(&fixture.root, "notes", Kind::Directory)]);
        m.browser.listed(&fixture.notes, vec![Entry::new(&fixture.notes, "hogwarts.md", Kind::File)]);
        m.browser.select(&fixture.document);
    }
    let document = decode(DOCUMENT.as_bytes()).expect("a text document");
    reader::show_text(&mut model.borrow_mut(), &ui.borrow(), &fixture.document, document, String::new());
    let targets = markdown_images::targets(DOCUMENT);
    let headers = picture::headers(&model.borrow().pictures, &fixture.root, &fixture.notes, &targets);
    ui.borrow().markdown.set_headers(headers);
    model.borrow_mut().focus = Focus::Reader;
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

/// Scrolls the reader so the box at `index` is the first row in view, then paints.
fn show_box(app: &TestApp, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, index: usize) {
    let (top, _) = ui.borrow().markdown.boxes()[index];
    ui.borrow().viewer.scroll_to(0.0, f64::from(top));
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
}

/// The screen row and column where the box named `label` starts, for a click on it.
fn box_at(app: &TestApp, label: &str) -> (u16, u16) {
    let capture = app.render();
    (0..capture.height)
        .find_map(|y| {
            let row = capture.row(y);
            row.find(label).map(|byte| (row[..byte].chars().count() as u16, y as u16))
        })
        .unwrap_or_else(|| panic!("{label} on screen: {}", app.text()))
}

#[test]
fn enter_opens_the_first_box_in_view_and_the_tree_keeps_its_selection() {
    let fixture = hogwarts();
    let (app, model, ui) = reading(&fixture);
    app.press("enter");
    assert!(model.borrow().loading, "the picture is on its way: {}", app.text());
    let header = ui.borrow().markdown.boxes().into_iter().next().and_then(|(_, header)| header);
    let Some(Header::Picture { path, .. }) = header else { panic!("the first box has a picture") };
    apply_picture(&model, &ui, &path);
    assert!(matches!(model.borrow().shown, reader::Shown::Image { .. }), "the picture shows");
    assert_eq!(model.borrow().browser.selected(), fixture.document.as_path(), "the tree does not move");
}

/// The worker's answer for `path`, applied as the event loop applies it.
fn apply_picture(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, path: &std::path::Path) {
    let result = services::fs::open(path, &[]);
    let mut m = model.borrow_mut();
    tree::apply(&mut m, &ui.borrow(), &Event::Picture { path: path.to_path_buf(), result });
}

#[test]
fn a_click_on_a_box_opens_its_picture() {
    let fixture = hogwarts();
    let (app, model, ui) = reading(&fixture);
    show_box(&app, &model, &ui, 1);
    let (x, y) = box_at(&app, "Mona Lisa");
    app.click(x + 2, y + 1);
    assert!(model.borrow().loading, "a release over the box opens it: {}", app.text());
    let mona = fs::canonicalize(fixture.root.join("Louvre/mona.png")).expect("the real path");
    apply_picture(&model, &ui, &mona);
    assert!(matches!(&model.borrow().shown, reader::Shown::Image { path, .. } if *path == mona));
}

#[test]
fn a_box_with_a_reason_does_nothing_and_says_why() {
    let fixture = hogwarts();
    let (app, model, ui) = reading(&fixture);
    show_box(&app, &model, &ui, 2);
    let (x, y) = box_at(&app, "Outside");
    app.click(x + 2, y + 1);
    assert_eq!(model.borrow().message.as_ref().map(|m| m.text.as_str()), Some("Cannot open: outside the folder."));
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    let text = app.text();
    assert!(text.contains("Cannot open: outside the folder."), "{text}");
    assert!(!model.borrow().loading && matches!(model.borrow().shown, reader::Shown::Text { .. }));
}

#[test]
fn ctrl_o_returns_to_the_markdown_at_the_scroll_it_was_left_at() {
    let fixture = hogwarts();
    let (app, model, ui) = reading(&fixture);
    // The box under the view's top row is the one Enter opens: scroll so its row is the first one in view.
    let (box_row, _) = ui.borrow().markdown.boxes().into_iter().next().expect("the first box");
    ui.borrow().viewer.scroll_to(0.0, f64::from(box_row));
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    let left = ui.borrow().viewer.position().1;
    app.press("enter");
    let header = ui.borrow().markdown.boxes().into_iter().next().and_then(|(_, header)| header);
    let Some(Header::Picture { path, .. }) = header else { panic!("the first box has a picture") };
    apply_picture(&model, &ui, &path);
    app.press("ctrl+o");
    let document = fixture.document.clone();
    let result = services::fs::open(&document, &[]);
    {
        let mut m = model.borrow_mut();
        tree::apply(&mut m, &ui.borrow(), &Event::Opened { path: document.clone(), result });
    }
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    assert!(matches!(model.borrow().shown, reader::Shown::Text { .. }), "the Markdown is back");
    assert_eq!(ui.borrow().viewer.position().1, left, "the same scroll");
}

#[test]
fn a_box_whose_header_has_not_come_back_does_nothing() {
    let fixture = hogwarts();
    let (app, model, ui) = reading(&fixture);
    ui.borrow().markdown.set_headers(Vec::new());
    app.press("enter");
    assert!(!model.borrow().loading, "nothing is read before the header is");
}
