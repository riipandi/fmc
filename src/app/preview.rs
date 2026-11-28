//! Auto-preview: while the list has the focus, the
//! file it selects shows in the reader after a short debounce. A preview replaces the document as an open does, but it
//! neither moves the focus nor records a jump, and a folder selection leaves the reader as it is.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use fmc::keys::Focus;
use opentui::prelude::*;

use super::reader;
use super::{Model, Ui, tree};
use crate::services::{self, Event, fs};
use crate::ui::footer::Message;

/// A held key reads only the file it stops on: each move restarts the wait.
pub const DEBOUNCE: Duration = Duration::from_millis(100);

/// The list moved: the file it now selects is read after `DEBOUNCE`, unless another move comes first.
pub fn moved(app: &App, model: &Rc<RefCell<Model>>) {
    let generation = {
        let mut m = model.borrow_mut();
        m.preview_generation += 1;
        m.preview_generation
    };
    let model = model.clone();
    app.set_timeout(DEBOUNCE, move |_| {
        let m = model.borrow();
        let Some(path) = due(&m, generation) else { return };
        let parents = m.browser.tree.ancestors(&path);
        let work = path.clone();
        services::spawn(&m.proxy, move || Event::Selected { path: work.clone(), result: fs::open(&work, &parents) });
    });
}

/// The file a preview of `generation` should read now: it is still the newest move, the list still has the focus, the
/// selection is a file, and the reader does not show it yet.
pub(super) fn due(m: &Model, generation: u64) -> Option<std::path::PathBuf> {
    if generation != m.preview_generation || !wanted(m) {
        return None;
    }
    let path = m.browser.selected();
    let file = tree::is_file(m, path);
    (file && shown_path(m) != Some(path)).then(|| path.to_path_buf())
}

/// Whether a preview may change the reader now: auto-preview is on, the list has the focus, no finder or changes
/// screen is open (the finder previews its own hits, and the changes screen shows its diffs), and the drawer does not
/// cover the reader (a preview would close the drawer, and the reader is not visible).
fn wanted(m: &Model) -> bool {
    m.auto_preview && m.focus == Focus::List && m.finder.is_none() && m.changes.is_none() && !m.compact()
}

fn shown_path(m: &Model) -> Option<&Path> {
    m.shown.path()
}

/// A selected file's read arrives: it shows as an open shows it (its views and notices), unless the selection has moved.
pub fn read(m: &mut Model, ui: &Ui, path: &Path, result: &Result<fs::Opened, String>) {
    if !wanted(m) || m.browser.selected() != path {
        return;
    }
    match result {
        Ok(fs::Opened::Text { document, stamp }) => reader::show_text(m, ui, path, document.clone(), stamp.clone()),
        Ok(fs::Opened::Image { info, bytes }) => reader::show_image(m, ui, path, info.clone(), bytes),
        Ok(fs::Opened::Binary { size }) => reader::show_binary(m, path, *size),
        Ok(fs::Opened::Folder(_)) => {}
        Err(text) => tree::show_error(m, text),
    }
}

/// `p` in the list: turns auto-preview on or off for this session. The caller schedules a preview after a move.
pub fn toggle(m: &mut Model) {
    m.auto_preview = !m.auto_preview;
    let text = if m.auto_preview { "Auto-preview on." } else { "Auto-preview off." };
    m.message = Some(Message { text: text.into(), error: false });
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
