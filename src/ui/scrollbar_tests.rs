use std::cell::RefCell;
use std::rc::Rc;

use fmc::theme::DARK;
use opentui::testing::TestApp;

use super::*;

/// A ten-row bar beside `total` rows of text, shown from row `top`, with every value its track reports.
fn bar(total: usize, top: usize) -> (TestApp, Rc<RefCell<Vec<f64>>>) {
    let app = TestApp::new(4, 10);
    let row = View::new(app.app()).row().height(10);
    let bar = ScrollBar::new(app.app(), 0);
    row.add(&bar.root());
    app.app().root().add(&row);
    bar.paint(app.app(), total, 10, top, true, &DARK);
    let values = Rc::new(RefCell::new(Vec::new()));
    let log = values.clone();
    bar.track().on_change(move |_, value| log.borrow_mut().push(value));
    app.render();
    (app, values)
}

#[test]
fn a_track_click_moves_to_the_row_pressed() {
    // 30 rows of text, 10 shown: the range is 20, and row 9 of a 10-row track is 9/10 of it.
    let (app, values) = bar(30, 0);
    app.click(0, 9);
    assert_eq!(values.borrow().last().copied(), Some(18.0));
}

#[test]
fn a_track_click_uses_the_text_height() {
    // 100 rows of text: the range is 90, so the same row moves further.
    let (app, values) = bar(100, 0);
    app.click(0, 9);
    assert_eq!(values.borrow().last().copied(), Some(81.0));
}

#[test]
fn a_drag_on_the_thumb_moves_by_the_rows_dragged() {
    // The crate's thumb is 6 half-cells (10/30 of the 20 half-cells, rounded down) at start 0. A press on row 1
    // grabs it at half-cell 2, so a drag to row 5 puts the thumb's start at half-cell 8 of the 14 it can take.
    let (app, values) = bar(30, 0);
    app.drag((0, 1), (0, 5));
    let last = values.borrow().last().copied().expect("the drag moved the bar");
    assert!((last - 8.0 / 14.0 * 20.0).abs() < 1e-9, "got {last}");
}

#[test]
fn a_press_on_the_thumb_does_not_jump() {
    let (app, values) = bar(30, 0);
    app.mouse(MouseEventKind::Down(MouseButton::Left), 0, 1);
    assert!(values.borrow().is_empty());
}

#[test]
fn the_blank_columns_before_a_thumb_are_not_the_track() {
    // A bar with two blank columns (the text's bar) is a track only in its last column, the thumb's.
    let app = TestApp::new(4, 10);
    let row = View::new(app.app()).row().height(10);
    let bar = ScrollBar::new(app.app(), 2);
    row.add(&bar.root());
    app.app().root().add(&row);
    bar.paint(app.app(), 30, 10, 0, true, &DARK);
    let values = Rc::new(RefCell::new(Vec::new()));
    let log = values.clone();
    bar.track().on_change(move |_, value| log.borrow_mut().push(value));
    app.render();
    app.click(0, 9);
    app.click(1, 9);
    assert!(values.borrow().is_empty());
    app.click(2, 9);
    assert_eq!(values.borrow().last().copied(), Some(18.0));
}

#[test]
fn the_thumb_reaches_the_bottom_after_the_bar_gets_shorter() {
    // Painted 20 rows tall first, then 10: the cells from the taller frame must not squeeze the shorter column.
    let app = TestApp::new(4, 20);
    let row = View::new(app.app()).row().height(20);
    let bar = ScrollBar::new(app.app(), 0);
    row.add(&bar.root());
    app.app().root().add(&row);
    bar.paint(app.app(), 60, 20, 40, true, &DARK);
    app.render();
    row.height(10);
    bar.paint(app.app(), 30, 10, 20, true, &DARK);
    app.render();
    let text = app.text();
    let rows: Vec<&str> = text.lines().collect();
    assert!(rows[9].contains('▐'), "the last row holds the thumb: {rows:?}");
    assert!(!rows[0].contains('▐'), "the thumb sits at the bottom: {rows:?}");
}
