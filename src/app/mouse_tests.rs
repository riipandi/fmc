use std::path::Path;

use fmc::explorer::{Entry, Kind};
use opentui::testing::TestApp;

use super::mouse::Click;
use super::*;

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
