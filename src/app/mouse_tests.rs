use super::mouse::HeaderClick;
use super::*;

const DOWN: MouseEventKind = MouseEventKind::Down(MouseButton::Left);
const DRAG: MouseEventKind = MouseEventKind::Drag(MouseButton::Left);
const UP: MouseEventKind = MouseEventKind::Up(MouseButton::Left);

#[test]
fn a_click_folds_on_its_release_not_its_press() {
    let mut click = HeaderClick::default();
    assert!(!click.feed(DOWN, 5, 3));
    assert!(click.feed(UP, 5, 3));
}

#[test]
fn a_press_dragged_away_does_not_fold() {
    let mut click = HeaderClick::default();
    click.feed(DOWN, 5, 3);
    click.feed(DRAG, 5, 4);
    assert!(!click.feed(UP, 5, 4));
}

#[test]
fn a_swipe_that_returns_to_its_cell_does_not_fold() {
    let mut click = HeaderClick::default();
    click.feed(DOWN, 5, 3);
    click.feed(DRAG, 6, 3);
    click.feed(DRAG, 5, 3);
    assert!(!click.feed(UP, 5, 3));
}

#[test]
fn a_release_at_another_cell_does_not_fold() {
    let mut click = HeaderClick::default();
    click.feed(DOWN, 5, 3);
    assert!(!click.feed(UP, 5, 4));
}
