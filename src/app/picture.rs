//! The rendered view's image boxes: each box's header is read on a worker when a Markdown text is shown, and an answer is
//! kept only for the showing it was asked for, as the git signs are (`app/gutter.rs`).

use std::path::Path;

use fmc::reader::images::Header;

use super::reader::Shown;
use super::{Model, Ui};
use crate::services::{self, Event};
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
