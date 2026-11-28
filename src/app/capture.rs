//! The key capture (Scouter `onKeyPress`): every key reaches the parser here before any widget sees it. A key the
//! parser hands to fmc stops here, so a focused OpenTUI widget cannot act on it as well: the reader's scroll box binds
//! the arrows and h/j/k/l, and a tree arrow must not pan the reader too.

use fmc::line_edit::edit_of;

use super::*;

pub fn wire(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let (model, ui) = (model.clone(), ui.clone());
    app.on_key_capture(move |cx, event| {
        let Some(named) = keys::name(event) else { return };
        press_key(cx, &model, &ui, &named);
    });
}

/// Whether an open prompt takes the key: a plain key, or a field edit such as Option+Left or Ctrl+Left. The edit keys
/// come first, so Ctrl+Left edits the query and does not resize the sidebar.
fn prompt_takes(key: fmc::keys::Key<'_>) -> bool {
    key.plain() || edit_of(key).is_some()
}

/// One key press, from the keyboard or from a tap on a footer hint (`app/mouse.rs`). A tap sends the hint's own
/// keys through here, so it runs the same Action as the key.
pub fn press_key(cx: &mut Cx<'_>, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, named: &keys::Named) {
    let mut m = model.borrow_mut();
    // Scouter clears the reader's error notice on every key (`src/app/app.ts:531-534`).
    m.notice = None;
    let state = m.key_state();
    // An open menu or dialog takes every key, so the page under it does not act; Ctrl+q and Ctrl+c still quit.
    let key = named.key();
    if m.overlay.is_some() && !(key.ctrl && matches!(key.name, "q" | "c")) {
        cx.stop_propagation();
        overlay::key(cx.app(), &mut m, &ui.borrow(), key);
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
        return;
    }
    // The Settings page takes every key while it is open, as an overlay does; Ctrl+q and Ctrl+c still quit.
    if m.settings.page.is_some() && !(key.ctrl && matches!(key.name, "q" | "c")) {
        cx.stop_propagation();
        settings::key(cx.app(), &mut m, &mut ui.borrow_mut(), key);
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
        return;
    }
    // While the finder is open, its plain keys and field edits edit the query before any command (Scouter
    // `onKeyPress`).
    if m.finder.is_some() && prompt_takes(named.key()) {
        cx.stop_propagation();
        find::prompt_key(&mut m, named.key());
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
        repaint_after_layout(cx.app(), model, ui);
        return;
    }
    // While the command line is open, its plain keys and field edits edit it; Enter runs it and `:q` quits.
    if m.command.is_some() && prompt_takes(named.key()) {
        cx.stop_propagation();
        if command::prompt_key(&mut m, named.key()) {
            return cx.quit();
        }
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
        repaint_after_layout(cx.app(), model, ui);
        return;
    }
    // While the search prompt is open, its plain keys and field edits edit the query before any command (Scouter
    // `onKeyPress`).
    if m.search.as_ref().is_some_and(|search| search.editing) && prompt_takes(named.key()) {
        cx.stop_propagation();
        search::prompt_key(&mut m, &ui.borrow(), named.key());
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
        repaint_after_layout(cx.app(), model, ui);
        return;
    }
    let Step::Done(action) = m.parser.feed(named.key(), state) else {
        paint(cx.app(), &mut m, &mut ui.borrow_mut());
        return;
    };
    // The parser's key is fmc's: the focused OpenTUI widget (the reader's scroll box) does not see it too.
    cx.stop_propagation();
    // A keypress can move the reader; the position it moves to is laid out by the next frame.
    repaint_after_layout(cx.app(), model, ui);
    let body = changes::body_shows(&m);
    match action {
        Action::Quit => return cx.quit(),
        Action::Help => reader::toggle_keys(&mut m, &ui.borrow()),
        Action::OpenSettings => settings::open(&mut m),
        // In compact `q` goes back a page; the list page at the root has nothing to go back to.
        Action::Close if m.compact() && m.pages.top() != Page::Files => pages::back(&mut m, &ui.borrow()),
        // `q` on the changes screen goes back to browsing, as `d` does, where Scouter's `q` quits.
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
        Action::Escape if m.compact() && m.pages.top() != Page::Files => pages::back(&mut m, &ui.borrow()),
        // Esc in the list drops a copy or cut; with nothing to drop it is the list's own Esc.
        Action::Escape if m.focus == Focus::List && m.clipboard.is_some() && m.changes.is_none() => m.clipboard = None,
        // Esc drops the marks once the clipboard is empty.
        Action::Escape if m.focus == Focus::List && !m.browser.marked().is_empty() && m.changes.is_none() => {
            m.browser.clear_marks()
        }
        Action::SwitchPane | Action::Escape => m.focus = Focus::List,
        Action::Resize(step) => layout::resize(&mut m, cx.app().size().0, step),
        Action::ToggleSidebar => layout::toggle_key(&mut m, cx.app().size().0),
        Action::ToggleDotfiles => tree::toggle_dotfiles(&mut m),
        Action::ToggleFuzzy => find::toggle_fuzzy(&mut m),
        // The changes body has no Markdown view of its own: `m` is for the reader's document.
        Action::ToggleMarkdown if !body => {
            reader::toggle_markdown(&mut m);
            search::clear(&mut m);
            ui.borrow().viewer.scroll_to(0.0, 0.0);
        }
        Action::ToggleWrap if m.shown.has_document() || m.changes.is_some() => m.wrap = !m.wrap,
        Action::OpenDefault if m.changes.is_none() => open::default_app(&mut m),
        Action::OpenAsHex if m.changes.is_none() => hex::open_shown(&mut m),
        Action::OpenWith if m.changes.is_none() => open::with(&mut m),
        Action::OpenImage if m.focus == Focus::Reader && m.shown.previewing() => {
            picture::open_first(&mut m, &ui.borrow())
        }
        Action::OpenInEditor if m.changes.is_none() => open::editor(&mut m),
        Action::RevealInFinder if m.changes.is_none() => open::reveal(&mut m),
        Action::Copy if m.shown.is_some() || m.focus == Focus::List || m.changes.is_some() => {
            if scroll::yank(&mut m, cx.app()) {
                clear_message_later(cx.app(), model, ui);
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
        Action::Scroll(_) | Action::Page(_) | Action::HalfPage(_) | Action::Top | Action::Bottom
            if m.focus == Focus::Reader && m.shown.is_hex() && !body =>
        {
            hex::motion(&mut m, action)
        }
        Action::Jump(step) if m.focus == Focus::Reader && m.shown.is_hex() && !body => {
            let rows = i32::try_from(m.reader_jump).unwrap_or(i32::MAX) * i32::from(step);
            hex::motion(&mut m, Action::Scroll(rows))
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
        // The rendered view has no paragraph motion: the footer says which key shows the source.
        Action::Paragraph(_) if m.focus == Focus::Reader && m.shown.previewing() => {
            reader::refuse(&mut m, PARAGRAPH_REFUSED);
        }
        Action::Search if m.focus == Focus::Reader && (m.shown.text().is_some() || m.shown.is_hex()) => {
            search::start(&mut m)
        }
        Action::Find if m.focus == Focus::List => find::start(&mut m),
        Action::ContextMenu if m.focus == Focus::List && m.changes.is_none() => write::open_menu(&mut m),
        Action::NewFile | Action::NewFolder | Action::Rename | Action::Duplicate | Action::Delete
            if m.focus == Focus::List && m.changes.is_none() =>
        {
            write::key(&mut m, &ui.borrow(), action)
        }
        Action::CopyFile | Action::CutFile if m.focus == Focus::List && m.changes.is_none() => {
            write::take(&mut m, action == Action::CutFile)
        }
        Action::PasteFile if m.focus == Focus::List && m.changes.is_none() => write::paste(&mut m),
        Action::ToggleMark if m.focus == Focus::List && m.changes.is_none() => tree::toggle_mark(&mut m),
        Action::ClearMarks if m.focus == Focus::List && m.changes.is_none() => m.browser.clear_marks(),
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
        Action::TogglePreview if m.focus == Focus::List && m.changes.is_none() => preview::toggle(&mut m),
        Action::OpenCommand => command::open(&mut m),
        Action::Jump(step) if m.focus == Focus::List && m.changes.is_none() => tree::jump(&mut m, step),
        Action::Jump(step) if m.focus == Focus::Reader && m.shown.has_document() => {
            let lines = i32::try_from(m.reader_jump).unwrap_or(i32::MAX) * i32::from(step);
            let ui = ui.borrow();
            scroll::motion(&m, &ui.viewer, Action::Scroll(lines))
        }
        _ => {}
    }
    paint(cx.app(), &mut m, &mut ui.borrow_mut());
    // A list move or a toggle schedules the preview; the debounce's end checks the focus and the selection.
    let previews = m.focus == Focus::List
        && m.changes.is_none()
        && matches!(
            action,
            Action::Move(_)
                | Action::MovePage(_)
                | Action::Top
                | Action::Bottom
                | Action::Toggle
                | Action::Expand
                | Action::Collapse
                | Action::GoUp
                | Action::Jump(_)
                | Action::TogglePreview
                | Action::ToggleMark
        );
    drop(m);
    if previews {
        preview::moved(cx.app(), model);
    }
}

#[cfg(test)]
#[path = "capture_tests.rs"]
mod capture_tests;
