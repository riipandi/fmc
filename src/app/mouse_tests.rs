use std::path::Path;

use fmc::explorer::{Entry, Kind};
use opentui::testing::TestApp;

use super::mouse::Click;
use super::*;
use crate::ui::tree::LIST_TOP;

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

/// Two files listed under `/hogwarts` at 80x24, the first selected, with the list's mouse wired. The list's rows start
/// at `LIST_TOP`, so the second file is on the row below the first.
fn listing() -> (TestApp, Rc<RefCell<Model>>) {
    let app = TestApp::new(80, 24);
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
