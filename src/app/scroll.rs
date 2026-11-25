//! The reader's keys after the parser has named them (Scouter `handleViewerKey`, `src/app/app.ts`), its copy (`yank`)
//! and its jumps. The scroll position is the viewer's; the app keeps the last one it painted for the jump history.

use std::path::{Path, PathBuf};

use fmc::keys::{Action, Focus};
use fmc::reader::motion::{self, HALF_PAGE, PAGE};
use fmc::text::copied_summary;
use opentui::prelude::App;

use super::reader::{Shown, here};
use super::{Model, Ui};
use crate::ui::footer::Message;

/// What `y` copies with nothing selected is the current path (Scouter `currentPath`).
pub const COPIED_PATH: &str = "Copied path.";
pub const COPY_FAILED: &str = "Could not copy to clipboard.";
pub const NO_EARLIER: &str = "No earlier place to go back to.";
pub const NO_LATER: &str = "No later place to go to.";

/// Scroll and paragraph motions in the reader. A paragraph in the rendered view is refused by the app before this.
pub fn motion(m: &Model, ui: &Ui, action: Action) {
    let viewport = m.viewport;
    match action {
        Action::Scroll(rows) => ui.viewer.scroll_by(0.0, f64::from(rows)),
        Action::Pan(columns) => ui.viewer.scroll_by(f64::from(columns), 0.0),
        Action::Page(direction) => ui.viewer.scroll_by(0.0, f64::from(direction) * PAGE * viewport as f64),
        Action::HalfPage(direction) => ui.viewer.scroll_by(0.0, f64::from(direction) * HALF_PAGE * viewport as f64),
        Action::Top => ui.viewer.scroll_to(0.0, 0.0),
        Action::Bottom => ui.viewer.scroll_to_bottom(),
        Action::GoToLine(line) => ui.viewer.scroll_to(0.0, motion::line_top(line as usize - 1, viewport) as f64),
        Action::Centre => {
            let line = motion::centre_line(m.top.floor() as usize, viewport);
            ui.viewer.scroll_to(0.0, motion::line_top(line, viewport) as f64);
        }
        Action::Paragraph(direction) => {
            let Shown::Text { document, .. } = &m.shown else { return };
            let blanks = motion::blank_lines(&document.text);
            let forward = direction > 0;
            match motion::paragraph_target(&blanks, m.top.floor() as usize, forward) {
                Some(line) => ui.viewer.scroll_to(0.0, line as f64),
                None if forward => ui.viewer.scroll_to_bottom(),
                None => ui.viewer.scroll_to(0.0, 0.0),
            }
        }
        _ => {}
    }
}

/// The path `y` copies when nothing is selected: the document the reader shows, or else the list's entry (Scouter
/// `currentPath`).
pub fn current_path(m: &Model) -> PathBuf {
    if let (Focus::Reader, Shown::Text { path, .. } | Shown::Image { path, .. }) = (m.focus, &m.shown) {
        return path.clone();
    }
    m.browser.selected().to_path_buf()
}

/// What `y` copies and the notice it shows: the selection, with Scouter's count, or else the path, as Scouter's `yank`.
pub fn copy_plan(selected: Option<&str>, path: &Path) -> (String, String) {
    match selected {
        Some(text) => (text.to_owned(), copied_summary(text)),
        None => (path.to_string_lossy().into_owned(), COPIED_PATH.into()),
    }
}

/// `y`: copies the mouse selection, or the path, through OpenTUI's clipboard (OSC 52 and the system clipboard). The
/// selection is cleared once copied. The notice is Scouter's; the caller clears a successful one after two seconds.
pub fn yank(m: &mut Model, app: &App) -> bool {
    let selected = app.selected_text();
    let (text, summary) = copy_plan(selected.as_deref(), &current_path(m));
    if selected.is_some() {
        app.clear_selection();
    }
    let ok = app.copy_to_clipboard(&text);
    m.message = Some(if ok {
        Message { text: summary, error: false }
    } else {
        Message { text: COPY_FAILED.into(), error: true }
    });
    ok
}

/// Ctrl+o and Alt+i: the earlier or later place, as Scouter's `jump`. Returns the place to open, which keeps the
/// history as it is, or `None` after a message.
pub fn jump(m: &mut Model, back: bool) -> Option<fmc::reader::jump::Jump> {
    let here = here(m);
    let target = if back { m.jumps.back(here) } else { m.jumps.forward(here) };
    if target.is_none() {
        let text = if back { NO_EARLIER } else { NO_LATER };
        m.message = Some(Message { text: text.into(), error: false });
    }
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_selection_is_copied_with_scouters_count_and_no_selection_copies_the_path() {
        let path = Path::new("/vault/Gringotts/vault.c");
        assert_eq!(copy_plan(Some("Hermione"), path), ("Hermione".into(), "Copied 8 characters.".into()));
        assert_eq!(copy_plan(Some("a\nb"), path), ("a\nb".into(), "Copied 2 lines.".into()));
        assert_eq!(copy_plan(None, path), ("/vault/Gringotts/vault.c".into(), "Copied path.".into()));
    }
}
