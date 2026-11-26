//! The key capture (Scouter `onKeyPress`): every key reaches the parser here before any widget sees it. A key the
//! parser hands to fmc stops here, so a focused OpenTUI widget cannot act on it as well: the reader's scroll box binds
//! the arrows and h/j/k/l, and a tree arrow must not pan the reader too.

use super::*;

pub fn wire(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let (model, ui) = (model.clone(), ui.clone());
    app.on_key_capture(move |cx, event| {
        let Some(named) = keys::name(event) else { return };
        let mut m = model.borrow_mut();
        // Scouter clears the reader's error notice on every key (`src/app/app.ts:531-534`).
        m.notice = None;
        let state = m.key_state();
        // While the finder is open, its plain keys edit the query before any command (Scouter `onKeyPress`).
        if m.finder.is_some() && named.key().plain() {
            cx.stop_propagation();
            find::prompt_key(&mut m, named.key());
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
            repaint_after_layout(cx.app(), &model, &ui);
            return;
        }
        // While the search prompt is open, its plain keys edit the query before any command (Scouter `onKeyPress`).
        if m.search.as_ref().is_some_and(|search| search.editing) && named.key().plain() {
            cx.stop_propagation();
            search::prompt_key(&mut m, &ui.borrow(), named.key());
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
            repaint_after_layout(cx.app(), &model, &ui);
            return;
        }
        let Step::Done(action) = m.parser.feed(named.key(), state) else {
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
            return;
        };
        // The parser's key is fmc's: the focused OpenTUI widget (the reader's scroll box) does not see it too.
        cx.stop_propagation();
        // A keypress can move the reader; the position it moves to is laid out by the next frame.
        repaint_after_layout(cx.app(), &model, &ui);
        let body = changes::body_shows(&m);
        match action {
            Action::Quit => return cx.quit(),
            Action::Help => reader::toggle_keys(&mut m, &ui.borrow()),
            // `q` on the changes screen goes back to browsing, as `d` does (architecture.md, approved deviation).
            Action::Close if body => changes::leave(&mut m),
            Action::Close => reader::close(&mut m, &ui.borrow()),
            Action::ToggleChanges => changes::toggle(&mut m),
            // The changes keys work in any focus while the body shows (Scouter `handleDiffKey`).
            Action::ToggleSplit if body => {
                let columns = pane_columns(cx.app(), &m);
                changes::toggle_split(&mut m, columns);
            }
            Action::ToggleScope if body => changes::toggle_scope(&mut m),
            Action::ToggleFold if body => changes::toggle_fold(&mut m),
            Action::ToggleContext if body => changes::toggle_context(&mut m),
            Action::OpenChanged if body => changes::open_changed(&mut m, &ui.borrow()),
            Action::Reload if body => changes::reload(&mut m),
            Action::Hunk(step) if body => changes::step_hunk(&mut m, &ui.borrow(), step > 0),
            Action::SwitchPane if body => changes::switch_pane(&mut m),
            Action::FocusDiff => changes::activate(&mut m),
            Action::Escape if cx.app().selected_text().is_some() || cx.app().is_selecting() => {
                cx.app().clear_selection();
            }
            Action::Escape if m.shown.shows_keys() => reader::close_keys(&mut m, &ui.borrow()),
            // Tab moves to the reader only when it shows a document (Scouter `switchPane`).
            Action::SwitchPane if m.focus == Focus::List && m.shown.has_document() => m.focus = Focus::Reader,
            Action::Escape if m.focus == Focus::Reader && search::dismiss(&mut m) => {}
            Action::SwitchPane | Action::Escape => m.focus = Focus::List,
            Action::Resize(step) => layout::resize(&mut m, cx.app().size().0, step),
            Action::ToggleSidebar => layout::toggle_key(&mut m, cx.app().size().0),
            Action::ToggleDotfiles => tree::toggle_dotfiles(&mut m),
            // The changes body has no Markdown view of its own: `m` is for the reader's document.
            Action::ToggleMarkdown if !body => {
                reader::toggle_markdown(&mut m);
                ui.borrow().viewer.scroll_to(0.0, 0.0);
            }
            Action::ToggleWrap if m.shown.has_document() || m.changes.is_some() => m.wrap = !m.wrap,
            Action::Copy if m.shown.is_some() || m.focus == Focus::List || m.changes.is_some() => {
                if scroll::yank(&mut m, cx.app()) {
                    clear_message_later(cx.app(), &model, &ui);
                }
            }
            Action::JumpBack | Action::JumpForward if m.loading => {
                m.message = Some(Message { text: "Still loading. Try again when it finishes.".into(), error: false });
            }
            Action::JumpBack | Action::JumpForward => {
                if let Some(target) = scroll::jump(&mut m, action == Action::JumpBack) {
                    // Leaving the changes screen comes first, as Scouter's `jump` does.
                    changes::leave(&mut m);
                    m.restore = Some((target.path.clone(), target.top));
                    tree::open_entry(&mut m, target.path);
                }
            }
            Action::Scroll(_)
            | Action::Pan(_)
            | Action::Page(_)
            | Action::HalfPage(_)
            | Action::Top
            | Action::Bottom
            | Action::GoToLine(_)
            | Action::Centre
            | Action::Paragraph(_)
                if m.focus == Focus::Reader && (m.shown.has_document() || body) =>
            {
                let ui = ui.borrow();
                scroll::motion(&m, if body { &ui.diff.scroller } else { &ui.viewer }, action)
            }
            // The rendered view has no search or paragraph motion: the footer says which key shows the source.
            Action::Search | Action::Paragraph(_) if m.focus == Focus::Reader && m.shown.previewing() => {
                let text = if action == Action::Search { SEARCH_REFUSED } else { PARAGRAPH_REFUSED };
                reader::refuse(&mut m, text);
            }
            Action::Search if m.focus == Focus::Reader && m.shown.text().is_some() => search::start(&mut m),
            Action::Find if m.focus == Focus::List => find::start(&mut m),
            Action::NextMatch | Action::PreviousMatch if m.focus == Focus::Reader => {
                search::step(&mut m, &ui.borrow(), action == Action::NextMatch)
            }
            Action::Move(_) | Action::MovePage(_) | Action::Top | Action::Bottom
                if m.focus == Focus::List && m.changes.is_some() =>
            {
                changes::move_list(&mut m, action)
            }
            Action::Move(_)
            | Action::MovePage(_)
            | Action::Top
            | Action::Bottom
            | Action::Toggle
            | Action::Expand
            | Action::Collapse
            | Action::GoUp
                if m.focus == Focus::List && m.changes.is_none() =>
            {
                tree::act(&mut m, action)
            }
            _ => {}
        }
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
    });
}

#[cfg(test)]
#[path = "capture_tests.rs"]
mod capture_tests;
