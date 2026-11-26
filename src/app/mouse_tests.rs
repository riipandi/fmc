use std::fs;
use std::path::Path;

use fmc::explorer::{Entry, Kind};
use opentui::testing::TestApp;

use super::mouse::Click;
use super::*;
use crate::ui::tree::{LIST_TOP, list_top};

const DOWN: MouseEventKind = MouseEventKind::Down(MouseButton::Left);
const DRAG: MouseEventKind = MouseEventKind::Drag(MouseButton::Left);
const UP: MouseEventKind = MouseEventKind::Up(MouseButton::Left);

#[test]
fn a_click_completes_on_its_release_not_its_press() {
    let mut click = Click::default();
    assert!(!click.feed(DOWN, 5, 3));
    assert!(click.feed(UP, 5, 3));
}

#[test]
fn a_press_dragged_away_is_not_a_click() {
    let mut click = Click::default();
    click.feed(DOWN, 5, 3);
    click.feed(DRAG, 5, 4);
    assert!(!click.feed(UP, 5, 4));
}

#[test]
fn a_swipe_that_returns_to_its_cell_is_not_a_click() {
    let mut click = Click::default();
    click.feed(DOWN, 5, 3);
    click.feed(DRAG, 6, 3);
    click.feed(DRAG, 5, 3);
    assert!(!click.feed(UP, 5, 3));
}

#[test]
fn a_release_at_another_cell_is_not_a_click() {
    let mut click = Click::default();
    click.feed(DOWN, 5, 3);
    assert!(!click.feed(UP, 5, 4));
}

/// Two files listed under `/hogwarts` at 80x24, the first selected, with the list's mouse wired.
fn listing() -> (TestApp, Rc<RefCell<Model>>) {
    listing_at(80, 24)
}

/// Two files listed under `/hogwarts` on a `columns` by `rows` screen, the first selected, with the list's mouse wired.
/// The list's rows start at `LIST_TOP`, so the second file is on the row below the first.
fn listing_at(columns: u16, rows: u16) -> (TestApp, Rc<RefCell<Model>>) {
    let app = TestApp::new(columns, rows);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    {
        let mut m = model.borrow_mut();
        m.browser.listed(
            &root,
            vec![Entry::new(&root, "owls.txt", Kind::File), Entry::new(&root, "hedwig.txt", Kind::File)],
        );
        m.browser.select(&root.join("owls.txt"));
    }
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model)
}

fn selected(model: &Rc<RefCell<Model>>) -> PathBuf {
    model.borrow().browser.selected().to_path_buf()
}

#[test]
fn a_row_acts_when_the_release_ends_the_click() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.mouse(DOWN, 3, row);
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"), "the press alone selects nothing");
    app.mouse(UP, 3, row);
    assert_eq!(selected(&model), Path::new("/hogwarts/hedwig.txt"));
}

#[test]
fn a_drag_across_a_row_acts_on_nothing() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.drag((3, row), (3, row + 1));
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"));
}

#[test]
fn a_swipe_that_returns_to_its_row_acts_on_nothing() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.drag((3, row), (3, row));
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"), "a drag that ends on its cell is still a drag");
}

#[test]
fn a_release_off_the_row_does_not_act() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.mouse(DOWN, 3, row);
    app.mouse(UP, 3, LIST_TOP - 1);
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"));
}

/// The bytes the phone client sent, from the probe log (`/tmp/fmc-probe/touch.log`, 2026-11-27). A tap is a press and
/// its release at one cell (43 taps, none unpaired). A swipe is a run of wheel codes, 64 up and 65 down, with no drag
/// events. These are SGR coordinates, one-based.
const TAP: &str = "\x1b[<0;31;18M\x1b[<0;31;18m";
const SWIPE_DOWN: &str =
    "\x1b[<65;13;26M\x1b[<65;12;24M\x1b[<65;11;23M\x1b[<65;10;23M\x1b[<65;9;22M\x1b[<65;8;21M\x1b[<65;7;20M";

/// A 49-column phone screen (the compact tier) listing sixty files, the first selected, with the mouse wired.
fn phone() -> (TestApp, Rc<RefCell<Model>>) {
    let app = TestApp::new(49, 40);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    {
        let mut m = model.borrow_mut();
        let files = (0..60).map(|i| Entry::new(&root, &format!("hogwarts-{i:02}.txt"), Kind::File)).collect();
        m.browser.listed(&root, files);
        m.browser.select(&root.join("hogwarts-00.txt"));
    }
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model)
}

#[test]
fn a_logged_tap_acts_on_the_row_it_lands_on() {
    let (app, model) = phone();
    app.feed(TAP.as_bytes());
    // Row 18 on the screen is list line 16 (the list starts on row 1), and nothing has scrolled.
    assert_eq!(selected(&model), Path::new("/hogwarts/hogwarts-16.txt"));
    assert!(model.borrow().loading, "compact: a tap on a file opens it");
}

#[test]
fn a_logged_swipe_scrolls_the_list_and_opens_nothing() {
    let (app, model) = phone();
    app.feed(SWIPE_DOWN.as_bytes());
    let m = model.borrow();
    assert_eq!(m.browser.top(), 7, "seven wheel steps, one row each");
    assert_eq!(m.browser.selected(), Path::new("/hogwarts/hogwarts-00.txt"));
    assert!(!m.loading, "a swipe is no tap");
}

#[test]
fn a_tap_after_a_logged_swipe_acts_on_the_row_under_it_once_scrolled() {
    let (app, model) = phone();
    app.feed(SWIPE_DOWN.as_bytes());
    app.feed(TAP.as_bytes());
    assert_eq!(selected(&model), Path::new("/hogwarts/hogwarts-23.txt"), "list line 16 after seven rows of scroll");
}

#[test]
fn a_first_tap_on_a_row_acts_in_medium_with_the_reader_focused() {
    for columns in [60, 70, 79] {
        let (app, model) = listing_at(columns, 24);
        model.borrow_mut().focus = Focus::Reader;
        app.click(3, LIST_TOP + 1);
        assert_eq!(selected(&model), Path::new("/hogwarts/hedwig.txt"), "{columns}: the first tap selects the row");
        assert_eq!(model.borrow().focus, Focus::List, "{columns}: and the list has the focus");
    }
}

#[test]
fn a_first_tap_on_a_file_opens_it_in_compact_with_the_reader_focused() {
    let (app, model) = phone();
    model.borrow_mut().focus = Focus::Reader;
    app.feed(TAP.as_bytes());
    assert_eq!(selected(&model), Path::new("/hogwarts/hogwarts-16.txt"));
    assert!(model.borrow().loading, "compact: the first tap opens the file");
}

/// A git repository with two tracked files, both changed, on the changes screen at `columns` wide, with the mouse wired.
fn changes_at(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "dumbledore@hogwarts.test"]);
    git(root, &["config", "user.name", "Albus Dumbledore"]);
    for name in ["hedwig.txt", "owls.txt"] {
        fs::write(root.join(name), "Hogwarts\n").unwrap();
    }
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "Hogwarts"]);
    for name in ["hedwig.txt", "owls.txt"] {
        fs::write(root.join(name), "Hogwarts, changed\n").unwrap();
    }
    let app = TestApp::new(columns, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), root, root.to_path_buf(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    {
        let mut m = model.borrow_mut();
        m.changes_load = Some(1);
        changes::listed(&mut m, 1, crate::services::changes::list(root));
    }
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui, dir)
}

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

fn selected_change(model: &Rc<RefCell<Model>>) -> Option<String> {
    model.borrow().change_list.selected().map(|change| change.path.clone())
}

#[test]
fn a_first_tap_on_a_change_acts_below_80_columns_with_the_reader_focused() {
    for columns in [49, 60, 70, 79] {
        let (app, model, ui, _dir) = changes_at(columns);
        if !layout::shown(&model.borrow(), columns) {
            layout::toggle_key(&mut model.borrow_mut(), columns);
            paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
            app.render();
        }
        let before = selected_change(&model);
        model.borrow_mut().focus = Focus::Reader;
        let row = list_top(model.borrow().compact()) + 1;
        app.click(3, row);
        assert_ne!(selected_change(&model), before, "{columns}: the first tap selects the change");
        assert_eq!(model.borrow().focus, Focus::List, "{columns}: and the list has the focus");
    }
}

#[test]
fn at_80_columns_the_first_tap_on_a_change_only_focuses_the_list() {
    let (app, model, _ui, _dir) = changes_at(80);
    model.borrow_mut().focus = Focus::Reader;
    let before = selected_change(&model);
    app.click(3, LIST_TOP + 1);
    assert_eq!(selected_change(&model), before, "the first press only focuses the list");
    assert_eq!(model.borrow().focus, Focus::List);
    app.click(3, LIST_TOP + 1);
    assert_ne!(selected_change(&model), before, "the next click selects the row");
}
