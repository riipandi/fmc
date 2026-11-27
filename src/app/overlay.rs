//! The overlays over the page (plan-00012, D1): a context menu and a dialog. While one is open it takes every key and
//! every click, so the page under it does not act: `capture.rs` hands the keys here, and a catcher beneath the overlay
//! takes its clicks. What a row or a button does is the write layer's (`app/write.rs`); a press outside a menu closes it.

use fmc::keys::Key;
use fmc::layout::tier;
#[cfg(any(debug_assertions, test))]
use fmc::overlay::Dialog;
use fmc::overlay::{FieldEdit, Outcome, Overlay, Stop};
use opentui::prelude::*;

use super::*;
use crate::ui::dialog::{self, DialogView, Scrim};
use crate::ui::menu::{self, MenuView};

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
            if !matches!(event.kind, MouseEventKind::Down(_)) {
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
        row.on_mouse(hover(model, ui, move |m| match &mut m.overlay {
            Some(Overlay::Menu(menu)) => menu.point(index),
            _ => false,
        }));
    }
    for (index, button) in views.dialog.buttons.iter().enumerate() {
        button.on_mouse(press(model, ui, move |m| match &m.overlay {
            Some(Overlay::Dialog(dialog)) => dialog.pick(index),
            _ => Outcome::Stay,
        }));
        button.on_hover(hover_button(model, ui, index));
    }
}

/// A pointer that moves over a menu row selects it, as a key does: the keys and the pointer share one selection. `point`
/// says whether the selection changed; the frame is painted only then.
fn hover(
    model: &Rc<RefCell<Model>>,
    ui: &Rc<RefCell<Ui>>,
    point: impl Fn(&mut Model) -> bool + 'static,
) -> impl FnMut(&mut Cx<'_>, &MouseEvent) + 'static {
    let (model, ui) = (model.clone(), ui.clone());
    move |cx, event| {
        if event.kind != MouseEventKind::Moved {
            return;
        }
        let mut m = model.borrow_mut();
        if point(&mut m) {
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    }
}

/// A pointer that enters or leaves a dialog button highlights it. The focus stays where the keys put it, so Return still
/// picks the focused button.
fn hover_button(
    model: &Rc<RefCell<Model>>,
    ui: &Rc<RefCell<Ui>>,
    index: usize,
) -> impl FnMut(&mut Cx<'_>, bool) + 'static {
    let (model, ui) = (model.clone(), ui.clone());
    move |cx, entered| {
        let mut m = model.borrow_mut();
        let changed = match &mut m.overlay {
            Some(Overlay::Dialog(dialog)) => dialog.hover(index, entered),
            _ => false,
        };
        if changed {
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
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
        apply(cx.app(), &mut m, &ui.borrow(), outcome);
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
    }
}

/// A key while an overlay is open. `capture.rs` sends every key here except Ctrl+q and Ctrl+c, which quit.
pub fn key(app: &App, m: &mut Model, ui: &Ui, key: Key<'_>) {
    let Some(overlay) = m.overlay.as_mut() else { return };
    let outcome = overlay.key(key);
    apply(app, m, ui, outcome);
}

/// Carries out what a key or a press asked of the open overlay. A pick is for the write layer: a menu's act starts, and
/// a dialog's button may keep the dialog open.
fn apply(app: &App, m: &mut Model, ui: &Ui, outcome: Outcome) {
    match outcome {
        Outcome::Stay => {}
        Outcome::Close => close(app, m),
        Outcome::Pick(index) => write::pick(app, m, ui, index),
        Outcome::Insert(text) => {
            ui.overlay.dialog.field.insert(text);
            clear_error(m);
        }
        Outcome::Edit(edit) => {
            ui.overlay.dialog.field.perform(editor_action(edit));
            clear_error(m);
        }
    }
}

/// The OpenTUI action of a field edit: `Input::default_keymap` binds the same keys to it (`app/overlay_tests.rs` checks
/// that). OpenTUI binds Cmd+Left and Cmd+Right to its visual-line actions; on a one-line field those are the line's ends.
fn editor_action(edit: FieldEdit) -> EditorAction {
    match edit {
        FieldEdit::Backspace => EditorAction::Backspace,
        FieldEdit::DeleteWordBackward => EditorAction::DeleteWordBackward,
        FieldEdit::DeleteWordForward => EditorAction::DeleteWordForward,
        FieldEdit::DeleteToLineStart => EditorAction::DeleteToLineStart,
        FieldEdit::DeleteToLineEnd => EditorAction::DeleteToLineEnd,
        FieldEdit::WordBackward => EditorAction::WordBackward,
        FieldEdit::WordForward => EditorAction::WordForward,
        FieldEdit::LineHome => EditorAction::LineHome,
        FieldEdit::LineEnd => EditorAction::LineEnd,
    }
}

/// Typing clears a dialog's error line, which named the last attempt.
fn clear_error(m: &mut Model) {
    if let Some(Overlay::Dialog(dialog)) = m.overlay.as_mut() {
        dialog.error = None;
    }
}

/// Closes the overlay and gives the keyboard back: a dialog's field may hold the focus, and a key the parser leaves
/// would otherwise type into it.
pub fn close(app: &App, m: &mut Model) {
    m.overlay = None;
    m.overlay_task = None;
    app.blur();
}

/// Opens a dialog over the page with its field empty.
#[cfg(any(debug_assertions, test))]
fn open_dialog(m: &mut Model, ui: &Ui, dialog: Dialog) {
    ui.overlay.dialog.field.set_value("");
    m.overlay = Some(Overlay::Dialog(dialog));
}

/// Opens a dialog at start, so a live run can capture one: `FMC_OVERLAY=confirm` or `FMC_OVERLAY=name`. Debug builds
/// only, as `FMC_PANIC_AFTER_START` is.
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
    // The Settings page is modal too: the catcher takes its clicks, so the panes beneath it do not act (plan-00015, A21).
    views.catcher.visible(open.is_some() || m.settings.page.is_some());
    views.scrim.visible(dialog_open);
    views.menu.root.visible(matches!(open, Some(Overlay::Menu(_))));
    views.dialog.root.visible(dialog_open);
    match open {
        Some(Overlay::Menu(menu)) => {
            let rect = menu::frame(menu.items(), m.overlay_at, screen, tier(columns));
            views.menu.show(menu, rect, p, tint);
        }
        Some(Overlay::Dialog(dialog)) => {
            let rect = dialog::frame(screen, tier(columns), &dialog.message, dialog.field);
            views.dialog.show(dialog, rect, p, tint);
        }
        None => {}
    }
    // The field holds the keyboard, and so the terminal cursor, while the dialog's focus is on it. A focus on a button
    // (Tab, or a pointer) takes it away. Nothing else is focused by fmc, so a field that lost focus is the only one to blur.
    let on_field = matches!(open, Some(Overlay::Dialog(dialog)) if dialog.focus == Stop::Field);
    if on_field {
        app.focus(&views.dialog.field);
    } else if app.focused().is_some_and(|widget| widget.is(&views.dialog.field)) {
        app.blur();
    }
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;
