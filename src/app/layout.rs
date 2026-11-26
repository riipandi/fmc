//! The sidebar in the app (plan-00002): whether it shows, the width it takes, its edge and mark on the border, and what
//! `Ctrl+/`, a click on the mark, and `Ctrl+←/→` do to it. The rule itself is `fmc::layout`.

use std::cell::RefCell;
use std::rc::Rc;

use fmc::keys::Focus;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::{Model, Ui, paint};
use crate::ui::sidebar;
use crate::ui::styled::color;

/// A document is open, or the changes screen's body is: the 80-column rule applies (D7).
fn document(m: &Model) -> bool {
    m.shown.is_some() || m.changes.is_some()
}

/// The finder's prompt is in the sidebar, so the sidebar cannot hide while it is open.
fn held(m: &Model) -> bool {
    m.finder.is_some()
}

/// The mark is drawn and the toggle works only where the list can hide: with a document, and on the welcome screen
/// below `DRAWER` columns, where its list starts hidden (D7, amended).
fn can_hide(m: &Model, columns: u16) -> bool {
    !held(m) && (document(m) || columns < fmc::layout::DRAWER)
}

/// Whether the drawer covers the reader as of the last paint, so auto-preview waits (architecture.md, amended).
pub fn covered(m: &Model) -> bool {
    m.visibility.covers(document(m), held(m))
}

pub fn shown(m: &Model, columns: u16) -> bool {
    m.visibility.shown(columns, document(m), held(m))
}

/// The sidebar covers the reader below `DRAWER` columns (P3).
pub fn drawer(m: &Model, columns: u16) -> bool {
    fmc::layout::is_drawer(columns, shown(m, columns))
}

/// The width the sidebar takes: none while it is hidden, the whole terminal as a drawer.
pub fn width(m: &Model, columns: u16) -> u16 {
    if !shown(m, columns) {
        0
    } else if drawer(m, columns) {
        columns
    } else {
        sidebar::fit(columns, m.sidebar)
    }
}

/// Closes a drawer when a file opens from it (P3). A sidebar beside the reader is left as it is.
pub fn close_drawer(m: &mut Model, columns: u16) {
    if drawer(m, columns) {
        m.visibility.close();
    }
}

/// Runs at each paint: records the width and document for the next crossing, closes a drawer the reader's focus has
/// reached, and moves focus off a hidden sidebar.
pub fn sync(m: &mut Model, columns: u16) {
    m.visibility.observe(columns, document(m));
    if m.focus == Focus::Reader {
        close_drawer(m, columns);
    }
    if m.focus == Focus::List && !shown(m, columns) {
        m.focus = Focus::Reader;
    }
}

/// `Ctrl+/`, `Ctrl+_`, or a click on the mark: showing the sidebar also focuses it (architecture.md, "Auto-preview in
/// the sidebar").
pub fn toggle_key(m: &mut Model, columns: u16) {
    if !can_hide(m, columns) {
        return;
    }
    let hidden = !shown(m, columns);
    m.visibility.toggle(columns, true, false);
    if hidden && shown(m, columns) {
        m.focus = Focus::List;
    }
}

/// Ctrl+←/→: two columns narrower or wider, kept for the session (Scouter `nudge`). A hidden sidebar takes no key.
pub fn resize(m: &mut Model, columns: u16, step: i8) {
    if !shown(m, columns) || drawer(m, columns) {
        return;
    }
    let width = i64::from(sidebar::fit(columns, m.sidebar)) + 2 * i64::from(step);
    m.sidebar = Some(i64::from(sidebar::constrain(columns, width)));
}

/// The sidebar's right edge: a one-column strip over the panel's border, between its top and bottom rows.
pub fn handle(app: &App) -> View {
    View::new(app)
        .absolute()
        .width(1)
        .position(Edge::Top, 1)
        .position(Edge::Bottom, 1)
        .border_sides(Sides::LEFT)
        .border_color(Color::TRANSPARENT)
        .mouse_pointer(MousePointer::EW_RESIZE)
}

/// Lays out the sidebar, its edge, and its mark for this frame, and returns the width the sidebar takes.
pub fn lay_out(ui: &Ui, m: &Model, columns: u16, p: &Palette) -> u16 {
    let visible = shown(m, columns);
    let covered = drawer(m, columns);
    let width = width(m, columns);
    ui.browser_panel.visible(visible);
    ui.browser_panel.width(width);
    ui.viewer_panel.visible(!covered);
    ui.handle.visible(visible && !covered);
    ui.handle.position(Edge::Left, width.saturating_sub(1));
    ui.mark.root.visible(can_hide(m, columns));
    ui.mark.paint(visible, if visible { width.saturating_sub(2) } else { 0 }, p);
    width
}

/// The sidebar's edge (a drag resizes it) and its mark (a click shows or hides it).
pub fn wire(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let mark = ui.borrow().mark.root.clone();
    mark.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            if event.kind != MouseEventKind::Down(MouseButton::Left) {
                return;
            }
            // The `»` mark sits on the reader's corner; the reader's press would take the focus back.
            cx.stop_propagation();
            let mut m = model.borrow_mut();
            toggle_key(&mut m, cx.app().size().0);
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });

    // A tap on the reader's header row shows a hidden sidebar below 80 columns (P1). The press stops here, or the
    // reader's own press takes the focus back. A tap on the list's header row closes a drawer (P3).
    let (reader_header, list_header) = {
        let u = ui.borrow();
        (u.viewer_header.root.clone(), u.browser_header.root.clone())
    };
    reader_header.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            if event.kind != MouseEventKind::Down(MouseButton::Left) {
                return;
            }
            let mut m = model.borrow_mut();
            let columns = cx.app().size().0;
            if columns >= fmc::layout::BREAKPOINT || shown(&m, columns) {
                return;
            }
            cx.stop_propagation();
            toggle_key(&mut m, columns);
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
    list_header.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            if event.kind != MouseEventKind::Down(MouseButton::Left) {
                return;
            }
            let mut m = model.borrow_mut();
            let columns = cx.app().size().0;
            if !drawer(&m, columns) {
                return;
            }
            cx.stop_propagation();
            toggle_key(&mut m, columns);
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });

    let handle = ui.borrow().handle.clone();
    handle.on_hover({
        let (model, ui) = (model.clone(), ui.clone());
        move |_, hovered| {
            let p = model.borrow().palette();
            let edge = if hovered { color(p.accent_soft) } else { Color::TRANSPARENT };
            ui.borrow().handle.border_color(edge);
        }
    });
    handle.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let mut m = model.borrow_mut();
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    let width = sidebar::fit(cx.app().size().0, m.sidebar);
                    m.drag = Some((event.x, width));
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    let Some((start_x, start_width)) = m.drag else { return };
                    m.sidebar = Some(sidebar::dragged(start_width, start_x, event.x));
                }
                MouseEventKind::Up(MouseButton::Left) => m.drag = None,
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
}
