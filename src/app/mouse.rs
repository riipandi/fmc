//! The mouse on the panes: the file list (press, wheel), the changes list, and each pane's press that focuses it. The
//! handlers run on the UI thread like the key capture (`app/capture.rs`).

use super::*;
use crate::ui::viewer::Viewer;

/// Mouse on the list (click to select and open, wheel to scroll) and on the sidebar's edge (drag to resize).
pub fn wire_mouse(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let list = ui.borrow().tree.root.clone();
    list.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let mut m = model.borrow_mut();
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    // A press from another pane only focuses the list; the next press selects the row.
                    let focused = m.focus == Focus::List;
                    m.focus = Focus::List;
                    let row = line_at(event.y, LIST_TOP, m.browser.height())
                        .and_then(|line| m.browser.row_at(line))
                        .filter(|_| focused);
                    if let Some(index) = row {
                        m.browser.select_index(index);
                        // A click on a finder row picks it, as Enter does (Scouter `onExplorerActivate`).
                        if m.finder.is_some() {
                            find::pick(&mut m);
                        } else if tree::is_file(&m, m.browser.selected()) && layout::drawer(&m, cx.app().size().0) {
                            // Under a drawer a tap on a file opens it, as Enter does: the reader takes the focus, and
                            // the drawer closes (P3). A preview would not show it, since the focus leaves the list.
                            tree::activate(&mut m);
                        } else {
                            tree::click(&mut m);
                        }
                    }
                }
                MouseEventKind::ScrollUp => m.browser.scroll(-1),
                MouseEventKind::ScrollDown => m.browser.scroll(1),
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
            drop(m);
            preview::moved(cx.app(), &model);
        }
    });

    // The changes list: a click selects its row and opens it (Scouter `ChangeList`), and the wheel scrolls it.
    let changes_list = ui.borrow().changes.root.clone();
    changes_list.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let mut m = model.borrow_mut();
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    // The row's action may move the focus to the body, so the list's press stops here (the pane's own
                    // focus handler is further up the path).
                    let focused = m.focus == Focus::List;
                    m.focus = Focus::List;
                    let line = line_at(event.y, LIST_TOP, m.change_list.height());
                    if let Some(index) = line.filter(|_| focused).and_then(|line| m.change_list.row_at(line)) {
                        changes::click_row(&mut m, index);
                        layout::close_drawer(&mut m, cx.app().size().0);
                    }
                    cx.stop_propagation();
                }
                MouseEventKind::ScrollUp => m.change_list.scroll(-1),
                MouseEventKind::ScrollDown => m.change_list.scroll(1),
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });

    // A press anywhere in a pane focuses it, its header and its bar included (approved deviation, architecture.md). A row
    // press has already set the focus and stopped here, so the row's own action is what the focus ends on.
    let (sidebar, viewer) = {
        let u = ui.borrow();
        (u.browser_panel.clone(), u.viewer_panel.clone())
    };
    focus_on_press(model, ui, &sidebar, Focus::List);
    focus_on_press(model, ui, &viewer, Focus::Reader);
    // A drag's release ends a text selection, which the footer's `Copy` hint shows (plan-00004, P8). The selection is
    // settled after the handlers, so the frame repaints shortly after.
    viewer.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            if event.kind == MouseEventKind::Up(MouseButton::Left) {
                repaint_after_layout(cx.app(), &model, &ui);
            }
        }
    });

    // The bars: a drag, a track click, and a wheel step move the pane each bar belongs to (Scouter's scroll box bars).
    let tree_track = ui.borrow().tree.track();
    wire_bar(
        model,
        ui,
        &tree_track,
        Focus::List,
        |m, _, rows| m.browser.scroll(rows),
        |m, _, row| m.browser.scroll(row as isize - m.browser.top() as isize),
    );
    let changes_track = ui.borrow().changes.track();
    wire_bar(
        model,
        ui,
        &changes_track,
        Focus::List,
        |m, _, rows| m.change_list.scroll(rows),
        |m, _, row| m.change_list.scroll(row as isize - m.change_list.top() as isize),
    );
    let (text_track, rendered_track) = {
        let u = ui.borrow();
        (u.bar.track(), u.narrow_bar.track())
    };
    for track in [&text_track, &rendered_track] {
        wire_bar(
            model,
            ui,
            track,
            Focus::Reader,
            |m, u, rows| {
                bar_box(m, u).scroll_by(0.0, rows as f64);
            },
            |m, u, row| {
                bar_box(m, u).scroll_to(0.0, row as f64);
            },
        );
    }

    // A tap on a footer hint presses the hint's key through the key path, so it runs the same Action (plan-00004, D2).
    // `^q` has no tap (P2).
    ui.borrow().footer.on_hint_press({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            // A phone rule: at 80 columns and wider the hints are not buttons (plan-00004, P2).
            if event.kind != MouseEventKind::Down(MouseButton::Left) || cx.app().size().0 >= fmc::layout::BREAKPOINT {
                return;
            }
            let Some(key) = ui.borrow().footer.key_at(usize::from(event.x)) else { return };
            let Some(presses) = keys::hint_presses(key) else { return };
            for press in &presses {
                capture::press_key(cx, &model, &ui, press);
            }
        }
    });

    // The gutters sit outside their scrolling boxes, so a wheel over one scrolls the text it numbers (`ui/reader.rs`).
    let (reader_gutter, preview_gutter) = {
        let u = ui.borrow();
        (u.reader.gutter(), u.preview.gutter())
    };
    wire_gutter(model, ui, &reader_gutter, |u| &u.viewer);
    wire_gutter(model, ui, &preview_gutter, |u| &u.preview_viewer);

    layout::wire(model, ui);
}

/// A wheel over a gutter scrolls the box that holds its text, one row or one column a step.
fn wire_gutter(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, gutter: &View, target: fn(&Ui) -> &Viewer) {
    gutter.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let (columns, rows) = match event.kind {
                MouseEventKind::ScrollUp => (0.0, -1.0),
                MouseEventKind::ScrollDown => (0.0, 1.0),
                MouseEventKind::ScrollLeft => (-1.0, 0.0),
                MouseEventKind::ScrollRight => (1.0, 0.0),
                _ => return,
            };
            target(&ui.borrow()).scroll_by(columns, rows);
            repaint_after_layout(cx.app(), &model, &ui);
        }
    });
}

/// A header folds on the release that ends a click: a press and a release at one cell, with no drag between (approved
/// deviation, architecture.md). A swipe that starts on a header is a drag, so it folds nothing.
#[derive(Default)]
pub struct HeaderClick(Option<(u16, u16)>);

impl HeaderClick {
    /// Feeds one mouse event; `true` when it is the release that completes a click.
    pub fn feed(&mut self, kind: MouseEventKind, x: u16, y: u16) -> bool {
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.0 = Some((x, y));
                false
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.0 = None;
                false
            }
            MouseEventKind::Up(MouseButton::Left) => self.0.take() == Some((x, y)),
            _ => false,
        }
    }
}
