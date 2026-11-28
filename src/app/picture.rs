//! The rendered view's image boxes: each box's header is read on a worker when a Markdown text is shown, and an answer is
//! kept only for the showing it was asked for, as the git signs are (`app/gutter.rs`).

use std::path::Path;

use fmc::reader::image;
use fmc::reader::images::Header;

use super::reader::{self, Shown};
use super::{Model, Ui, tree};
use crate::services::{self, Event, fs};
use crate::ui::footer::Message;
use crate::ui::markdown_images;

/// Asks for the headers of the shown rendered document's boxes. A document with no box, or the source view, asks for none.
pub fn request(m: &Model) {
    let Shown::Text { path, document, serial, markdown: Some(view), .. } = &m.shown else { return };
    if !view.is_rendered() {
        return;
    }
    let targets = markdown_images::targets(&document.text);
    if targets.is_empty() {
        return;
    }
    let folder = path.parent().unwrap_or(path).to_path_buf();
    let (path, serial) = (path.clone(), *serial);
    let (cache, root) = (m.pictures.clone(), m.browser.tree.root().to_path_buf());
    services::spawn(&m.proxy, move || Event::Headers {
        path: path.clone(),
        serial,
        headers: services::picture::headers(&cache, &root, &folder, &targets),
    });
}

/// Keeps an answer for the text shown as `serial`: the boxes draw the headers the worker read for them.
pub fn apply(m: &Model, ui: &Ui, path: &Path, serial: u64, headers: &[(String, Header)]) {
    if let Shown::Text { path: shown, serial: shown_serial, .. } = &m.shown
        && shown == path
        && *shown_serial == serial
    {
        ui.markdown.set_headers(headers.to_vec());
    }
}

/// `Enter` in the rendered view: opens the first box whose top row is in view, and a box with no picture says why.
pub fn open_first(m: &mut Model, ui: &Ui) {
    let scroll = ui.viewer.position().1.max(0.0);
    let top = scroll as u32;
    let rows = m.viewport as u32;
    let first = ui.markdown.boxes().into_iter().find(|(row, _)| (top..top.saturating_add(rows)).contains(row));
    if let Some((_, header)) = first {
        open(m, scroll, header);
    }
}

/// A release over a box: the same as `Enter` on it, with the reader's scroll as the viewer has it now.
pub fn clicked(m: &mut Model, ui: &Ui, header: Header) {
    open(m, ui.viewer.position().1, Some(header));
}

/// Opens a box's picture on a worker, and records the Markdown place at `scroll` for `Ctrl+o`. A box with no picture says
/// why, and a box whose header has not come back does nothing. `scroll` is the viewer's row now: the paint's `m.top`
/// lags a key press, and the place is what the reader shows at this moment.
fn open(m: &mut Model, scroll: f64, header: Option<Header>) {
    match header {
        None => {}
        Some(Header::Refused(reason)) => {
            m.message = Some(Message { text: format!("Cannot open: {reason}."), error: false });
        }
        Some(Header::Picture { path, .. }) => {
            if m.loading {
                m.message = Some(Message { text: "Still loading. Try again when it finishes.".into(), error: false });
                return;
            }
            m.top = scroll;
            let here = reader::here(m);
            m.jumps.leave(here);
            m.loading = true;
            let work = path.clone();
            services::spawn(&m.proxy, move || Event::Picture { path: work.clone(), result: fs::open(&work, &[]) });
        }
    }
}

/// The picture a box opened: shown in the image view, and the tree keeps its selection.
pub fn opened(m: &mut Model, ui: &Ui, path: &Path, result: &Result<fs::Opened, String>) {
    m.loading = false;
    match result {
        Ok(fs::Opened::Image { info, bytes }) => reader::show_picture(m, ui, path, info.clone(), bytes),
        Ok(_) => tree::show_error(m, image::UNREADABLE),
        Err(text) => tree::show_error(m, text),
    }
}

#[cfg(test)]
#[path = "picture_tests.rs"]
mod tests;
