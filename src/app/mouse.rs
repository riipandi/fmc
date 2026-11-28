//! The mouse on the panes: the file list (press, wheel), the changes list, and each pane's press that focuses it. The
//! handlers run on the UI thread like the key capture (`app/capture.rs`).

use super::*;
use crate::ui::viewer::Viewer;

/// Mouse on the list (click to select and open, wheel to scroll) and on the sidebar's edge (drag to resize).
pub fn wire_mouse(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let list = ui.borrow().tree.root.clone();
    list.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        // The row a press took while the list had the focus; its release acts on it.
        let mut click = Click::default();
        let mut pressed: Option<usize> = None;
        move |cx, event| {
            let mut m = model.borrow_mut();
            let done = click.feed(event.kind, event.x, event.y);
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    let acts = press_acts(&m, cx.app().size().0);
                    m.focus = Focus::List;
                    pressed = line_at(event.y, LIST_TOP, m.browser.height())
                        .and_then(|line| m.browser.row_at(line))
                        .filter(|_| acts);
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    if let Some(index) = pressed.take().filter(|_| done) {
                        act_on_row(&mut m, index, cx.app().size().0);
                    }
                }
                MouseEventKind::Down(MouseButton::Right) => {
                    // A right press on a row selects it and opens its context menu at the pointer.
                    let row = line_at(event.y, LIST_TOP, m.browser.height()).and_then(|line| m.browser.row_at(line));
                    let Some(index) = row else { return };
                    m.focus = Focus::List;
                    m.browser.select_index(index);
                    write::open_menu_at(&mut m, (event.x, event.y));
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
        let mut click = Click::default();
        let mut pressed: Option<usize> = None;
        move |cx, event| {
            let mut m = model.borrow_mut();
            let done = click.feed(event.kind, event.x, event.y);
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    // The row's action may move the focus to the body, so the list's press stops here (the pane's own
                    // focus handler is further up the path).
                    let acts = press_acts(&m, cx.app().size().0);
                    m.focus = Focus::List;
                    let line = line_at(event.y, LIST_TOP, m.change_list.height());
                    pressed = line.filter(|_| acts).and_then(|line| m.change_list.row_at(line));
                    cx.stop_propagation();
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    if let Some(index) = pressed.take().filter(|_| done) {
                        changes::click_row(&mut m, index, cx.app().size().0);
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

    // A press anywhere in a pane focuses it, its header and its bar included, where Scouter only takes a click on a row.
    // A row press has already set the focus and stopped here, so the row's own action is what the focus ends on.
    let (sidebar, viewer) = {
        let u = ui.borrow();
        (u.browser_panel.clone(), u.viewer_panel.clone())
    };
    focus_on_press(model, ui, &sidebar, Focus::List);
    focus_on_press(model, ui, &viewer, Focus::Reader);
    // A drag's release ends a text selection, which the footer's `Copy` hint shows. The selection is
    // settled after the handlers, so the frame repaints shortly after.
    viewer.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            if event.kind == MouseEventKind::Up(MouseButton::Left) {
                // A box under the pointer picks its header on the release; the box opens as `Enter` on it would.
                let picked = ui.borrow().markdown.take_pick();
                if let Some(header) = picked {
                    picture::clicked(&mut model.borrow_mut(), &ui.borrow(), header);
                }
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

    // A tap on a footer hint presses the hint's key through the key path, so it runs the same Action.
    // `^q` has no tap.
    ui.borrow().footer.on_hint_press({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            // A phone rule: at 80 columns and wider the hints are not buttons.
            if event.kind != MouseEventKind::Down(MouseButton::Left)
                || fmc::layout::tier(cx.app().size().0) == fmc::layout::Tier::Wide
            {
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

/// Whether a press on a row acts, or only focuses its list. At 80 columns and wider a press from another pane only focuses
/// the list (the next click selects); below 80 there is no hover or visible focus to aim with, so the first tap acts.
fn press_acts(m: &Model, columns: u16) -> bool {
    m.focus == Focus::List || tier(columns) != Tier::Wide
}

/// A row's action once its click completes: the finder picks it, a file under a drawer opens, and anything else
/// is a click on the tree (a folder toggles, a file is selected).
fn act_on_row(m: &mut Model, index: usize, columns: u16) {
    m.browser.select_index(index);
    // A click on a finder row picks it, as Enter does (Scouter `onExplorerActivate`).
    if m.finder.is_some() {
        find::pick(m);
    } else if tree::is_file(m, m.browser.selected()) && tier(columns) == Tier::Compact {
        // In compact a tap on a file opens it, as Enter does: the reader page takes the focus. A preview would not show
        // it, since the list is not on screen once the reader is.
        tree::activate(m);
    } else {
        tree::click(m);
    }
}

/// A click completes on the release that ends a press at one cell, with no drag between.
/// A header folds on it, where Scouter folds on the press, and so does a list row. A swipe that starts on a row is a drag,
/// so it acts on nothing.
#[derive(Default)]
pub struct Click(Option<(u16, u16)>);

impl Click {
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
