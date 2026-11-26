//! The mouse on the panes: the file list (press, wheel), the changes list, and each pane's press that focuses it. The
//! handlers run on the UI thread like the key capture (`app/capture.rs`).

use super::*;

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

    layout::wire(model, ui);
}
