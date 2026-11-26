//! The sidebar in the app (plan-00002): whether it shows, the width it takes, its edge and mark on the border, and what
//! `Ctrl+/`, a click on the mark, and `Ctrl+←/→` do to it. The rule itself is `fmc::layout`.

use std::cell::RefCell;
use std::rc::Rc;

use fmc::keys::Focus;
use fmc::layout::{Page, Tier, tier};
use fmc::theme::Palette;
use opentui::prelude::*;

use super::{Model, Ui, pages, paint};
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

/// The toggle works where the list can hide: with a document, in the medium and wide tiers (D7).
fn can_hide(m: &Model, columns: u16) -> bool {
    !held(m) && document(m) && tier(columns) != Tier::Compact
}

/// The medium tier's chevron on the list's header: `‹` hides the list while it shows and can hide (plan-00005, record 10).
pub fn list_chevron(m: &Model, columns: u16) -> Option<String> {
    (tier(columns) == Tier::Medium && shown(m, columns) && can_hide(m, columns)).then(|| "‹".into())
}

/// The medium tier's chevron on the reader's header: `›` shows the list while it is hidden.
pub fn reader_chevron(m: &Model, columns: u16) -> Option<String> {
    (tier(columns) == Tier::Medium && !shown(m, columns)).then(|| "›".into())
}

/// Whether the list shows. In compact the top page decides (the list pages show it); otherwise the sidebar rule does.
pub fn shown(m: &Model, columns: u16) -> bool {
    if tier(columns) == Tier::Compact {
        matches!(m.pages.top(), Page::Files | Page::Changes | Page::Find)
    } else {
        m.visibility.shown(columns, document(m), held(m))
    }
}

/// The width the sidebar takes: none while it is hidden, the whole terminal in compact.
pub fn width(m: &Model, columns: u16) -> u16 {
    if !shown(m, columns) {
        0
    } else if tier(columns) == Tier::Compact {
        columns
    } else {
        sidebar::fit(columns, m.sidebar)
    }
}

/// Runs at each paint: the page stack follows the screens, records the width and document for the next crossing, and
/// moves focus off a hidden sidebar.
pub fn sync(m: &mut Model, columns: u16) {
    m.visibility.observe(columns, document(m));
    pages::sync(m);
    if m.focus == Focus::List && !shown(m, columns) {
        m.focus = Focus::Reader;
    }
}

/// `Ctrl+/`, `Ctrl+_`, or a click on the mark: showing the sidebar also focuses it (architecture.md, "Auto-preview in
/// the sidebar").
pub fn toggle_key(m: &mut Model, columns: u16) {
    if tier(columns) == Tier::Compact {
        return pages::switch(m);
    }
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
    if !shown(m, columns) || tier(columns) == Tier::Compact {
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
    let compact = tier(columns) == Tier::Compact;
    let width = width(m, columns);
    ui.browser_panel.visible(visible);
    ui.browser_panel.width(width);
    // In compact the reader is the page when the list is not (plan-00005, D2); the panels are then full width.
    ui.viewer_panel.visible(!compact || !visible);
    ui.handle.visible(visible && !compact);
    ui.handle.position(Edge::Left, width.saturating_sub(1));
    // Compact draws no panel border and one-row headers (architecture.md, record 5).
    let sides = if compact { Sides::NONE } else { Sides::ALL };
    ui.browser_panel.border_sides(sides);
    ui.viewer_panel.border_sides(sides);
    ui.browser_header.compact(compact);
    ui.viewer_header.compact(compact);
    ui.find.compact(compact);
    // The marks are drawn at 80 columns and wider only; below that the header boxes are the touch targets.
    ui.mark.root.visible(tier(columns) == Tier::Wide && can_hide(m, columns));
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

    // The lead text of each header is a button: the compact back text is Back, as `q` is (plan-00005, record 6), and a
    // medium chevron shows or hides the list (record 10). The press stops at the text, so the header's own press does
    // not act on it too.
    {
        let u = ui.borrow();
        for header in [&u.browser_header, &u.viewer_header] {
            header.on_lead_press({
                let (model, ui) = (model.clone(), ui.clone());
                move |cx, event| {
                    if event.kind != MouseEventKind::Down(MouseButton::Left) {
                        return;
                    }
                    let mut m = model.borrow_mut();
                    let columns = cx.app().size().0;
                    if pages::back_label(&m).is_some() {
                        cx.stop_propagation();
                        pages::back(&mut m, &ui.borrow());
                    } else if list_chevron(&m, columns).is_some() || reader_chevron(&m, columns).is_some() {
                        cx.stop_propagation();
                        toggle_key(&mut m, columns);
                    } else {
                        return;
                    }
                    paint(cx.app(), &mut m, &mut ui.borrow_mut());
                }
            });
        }
    }

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
