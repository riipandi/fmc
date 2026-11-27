//! The overlays over the page (plan-00012, D1): a context menu and a dialog. While one is open it takes every key and
//! every click, so the page under it does not act: `capture.rs` hands the keys here, and a catcher beneath the overlay
//! takes its clicks. This phase draws them and carries nothing out: a row or a button closes the overlay.

use fmc::keys::Key;
use fmc::layout::tier;
use fmc::overlay::{Dialog, Item, Menu, Outcome, Overlay};
use opentui::prelude::*;

use super::*;
use crate::ui::dialog::{self, DialogView, Scrim};
use crate::ui::menu::{self, MenuView};
use crate::ui::tree::LIST_TOP;

/// The views above the page. The catcher takes the clicks an overlay leaves; the scrim dims the page under a dialog.
pub struct Views {
    catcher: View,
    scrim: Custom<Scrim>,
    pub menu: MenuView,
    pub dialog: DialogView,
}

/// Builds the overlay views above the page. Each is hidden until an overlay opens.
pub fn build_views(app: &App) -> Views {
    // The catcher and the scrim cover the screen; the menu and the dialog sit above them.
    let catcher = cover(View::new(app).absolute().z_index(i32::MAX - 1).visible(false));
    let scrim = cover(Custom::new(app, Scrim).absolute().z_index(i32::MAX - 1).visible(false));
    Views { catcher, scrim, menu: MenuView::new(app), dialog: DialogView::new(app) }
}

/// Stretches an absolute widget over its parent, from each edge.
fn cover<W: Widget>(widget: W) -> W {
    widget.position(Edge::Top, 0).position(Edge::Left, 0).position(Edge::Right, 0).position(Edge::Bottom, 0);
    widget
}

/// Adds the overlay views to the root, beneath the menu and the dialog in paint order.
pub fn mount(app: &App, views: &Views) {
    app.root().add(&views.catcher).add(&views.scrim).add(&views.menu.root).add(&views.dialog.root);
}

/// Wires the catcher and the menu's and dialog's buttons to the model. The catcher takes every event: a press outside a
/// menu closes it, and a dialog is modal, so a press on the page beneath it does nothing.
pub fn wire_views(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let ui_ref = ui.borrow();
    let views = &ui_ref.overlay;
    views.catcher.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            cx.stop_propagation();
            if event.kind != MouseEventKind::Down(MouseButton::Left) {
                return;
            }
            let mut m = model.borrow_mut();
            if matches!(m.overlay, Some(Overlay::Menu(_))) {
                close(cx.app(), &mut m);
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
    for (index, row) in views.menu.rows.iter().enumerate() {
        row.on_mouse(press(model, ui, move |m| match &m.overlay {
            Some(Overlay::Menu(menu)) => menu.pick(index),
            _ => Outcome::Stay,
        }));
    }
    for (index, button) in views.dialog.buttons.iter().enumerate() {
        button.on_mouse(press(model, ui, move |m| match &m.overlay {
            Some(Overlay::Dialog(dialog)) => dialog.pick(index),
            _ => Outcome::Stay,
        }));
    }
}

/// A left press on a widget of the overlay: `answer` says what the press asks, and the app carries it out.
fn press(
    model: &Rc<RefCell<Model>>,
    ui: &Rc<RefCell<Ui>>,
    answer: impl Fn(&Model) -> Outcome + 'static,
) -> impl FnMut(&mut Cx<'_>, &MouseEvent) + 'static {
    let (model, ui) = (model.clone(), ui.clone());
    move |cx, event| {
        if event.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        cx.stop_propagation();
        let mut m = model.borrow_mut();
        let outcome = answer(&m);
        if matches!(outcome, Outcome::Pick(_) | Outcome::Close) {
            close(cx.app(), &mut m);
        }
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
    }
}

/// A key while an overlay is open. `capture.rs` sends every key here except Ctrl+q and Ctrl+c, which quit.
pub fn key(app: &App, m: &mut Model, ui: &Ui, key: Key<'_>) {
    let Some(overlay) = m.overlay.as_mut() else { return };
    match overlay.key(key) {
        Outcome::Stay => {}
        Outcome::Close | Outcome::Pick(_) => close(app, m),
        Outcome::Insert(text) => {
            ui.overlay.dialog.field.insert(text);
        }
        Outcome::Backspace => {
            ui.overlay.dialog.field.perform(EditorAction::Backspace);
        }
    }
}

/// Closes the overlay and gives the keyboard back: a dialog's field may hold the focus, and a key the parser leaves
/// would otherwise type into it.
fn close(app: &App, m: &mut Model) {
    m.overlay = None;
    app.blur();
}

/// `Space` on a tree row: the context menu, anchored at the selected row's cell (plan-00012, D3).
pub fn open_menu(m: &mut Model) {
    let row = m.browser.selected_index().saturating_sub(m.browser.top());
    m.overlay_at = (2, LIST_TOP + row as u16);
    m.overlay = Some(Overlay::Menu(Menu::new(sample_items())));
}

/// The menu's rows until the file actions land: the labels the actions will have, and one that is disabled.
fn sample_items() -> Vec<Item> {
    let rows = [
        ("New file…", true),
        ("New folder…", true),
        ("Rename…", true),
        ("Copy", true),
        ("Cut", true),
        ("Paste", false),
        ("Delete…", true),
    ];
    rows.into_iter().map(|(label, enabled)| Item { label: label.into(), enabled }).collect()
}

/// Opens a dialog over the page with its field empty.
#[cfg(debug_assertions)]
fn open_dialog(m: &mut Model, ui: &Ui, dialog: Dialog) {
    ui.overlay.dialog.field.set_value("");
    m.overlay = Some(Overlay::Dialog(dialog));
}

/// Opens a dialog at start, so a live run can capture one: `FMC_OVERLAY=confirm` or `FMC_OVERLAY=name`. Debug builds
/// only, as `FMC_PANIC_AFTER_START` is; no key opens a dialog in this phase.
#[cfg(debug_assertions)]
pub fn debug_open(m: &mut Model, ui: &Ui) {
    let dialog = match std::env::var("FMC_OVERLAY").as_deref() {
        Ok("confirm") => Dialog::new(
            "Move to Trash?",
            "Hermione Granger's essay on house-elves goes to the Trash. Finder's Put Back restores it.",
            false,
            &["Cancel", "Delete"],
            0,
        ),
        Ok("name") => {
            let mut dialog =
                Dialog::new("New file", "Name it. A name ending in / makes a folder.", true, &["Cancel", "OK"], 1);
            dialog.error = Some("robert-langdon.md is already here".into());
            dialog
        }
        _ => return,
    };
    open_dialog(m, ui, dialog);
}

/// Draws the open overlay for the screen's size, and hides the views of the others. `paint` calls it last.
pub fn draw(app: &App, m: &Model, ui: &Ui) {
    let (columns, rows) = app.size();
    let screen = (columns, rows);
    let (p, tint) = (m.palette(), m.tint());
    let views = &ui.overlay;
    let open = m.overlay.as_ref();
    let dialog_open = matches!(open, Some(Overlay::Dialog(_)));
    views.catcher.visible(open.is_some());
    views.scrim.visible(dialog_open);
    views.menu.root.visible(matches!(open, Some(Overlay::Menu(_))));
    views.dialog.root.visible(dialog_open);
    match open {
        Some(Overlay::Menu(menu)) => {
            let rect = menu::frame(menu.items(), m.overlay_at, screen, tier(columns));
            views.menu.show(menu, rect, p, &tint);
        }
        Some(Overlay::Dialog(dialog)) => {
            let rect = dialog::frame(screen, tier(columns), &dialog.message, dialog.field);
            views.dialog.show(dialog, rect, p, &tint);
        }
        None => {}
    }
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;
