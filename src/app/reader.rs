//! The reader on the UI thread: what is shown, the file open that shows it, and the header and footer facts (Scouter
//! `src/app/app.ts` `showDocument`, `src/app/chrome.ts` `viewerHeaderFor`, `viewerBadgesFor`).

use std::path::{Path, PathBuf};

use fmc::diff::Sign;
use fmc::keys::{Focus, key_page_markdown_with};
use fmc::reader::binary::format_size;
use fmc::reader::document::Document;
use fmc::reader::image;
use fmc::reader::jump::Jump;
use fmc::reader::markdown::{View, is_markdown};
use fmc::reader::position::scroll_label;
use fmc::reader::search::Search;
use fmc::text::display_label;

use super::{Model, Ui, gutter, search};
use crate::services::fs::Reread;
use crate::ui::footer::{DocumentInfo, Message};
use crate::ui::header::{Badge, BadgeKind};
use crate::ui::syntax::language_for;

/// What the reader shows. `markdown` is the view of a Markdown document, `None` for any other text.
#[derive(Debug, Default)]
pub enum Shown {
    #[default]
    None,
    Text {
        path: PathBuf,
        document: Document,
        language: Option<&'static str>,
        markdown: Option<View>,
        /// The file's stamp when it was read, which the poll compares (`None` once the file is gone).
        stamp: Option<String>,
        /// The showing this text is, and its git signs (`app/gutter.rs`).
        serial: u64,
        signs: Vec<(u32, Sign)>,
    },
    /// The key page (`?`): generated Markdown, shown as a Markdown document. `back` is what it replaced.
    Keys {
        document: Document,
        markdown: View,
        back: Box<Back>,
    },
    Image {
        path: PathBuf,
        info: image::Info,
    },
    Binary {
        path: PathBuf,
        size: u64,
    },
}

/// The reader before the key page took it: `?` and Esc put it back, with its focus, scroll, and search.
#[derive(Debug)]
pub struct Back {
    pub shown: Shown,
    pub focus: Focus,
    /// The viewer's scroll position, columns then rows.
    pub scroll: (f64, f64),
    pub search: Option<Search>,
}

impl Shown {
    pub fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn shows_keys(&self) -> bool {
        matches!(self, Self::Keys { .. })
    }

    /// Whether the reader holds a document the keys can reach (Scouter `hasDocument`): text, the key page, or an image.
    pub fn has_document(&self) -> bool {
        matches!(self, Self::Text { .. } | Self::Keys { .. } | Self::Image { .. })
    }

    /// The file the reader shows, when it shows one: a text, an image, or the binary notice.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Text { path, .. } | Self::Image { path, .. } | Self::Binary { path, .. } => Some(path),
            _ => None,
        }
    }

    /// The text the reader shows, with its filetype and Markdown view: a file's, or the key page's.
    pub fn text(&self) -> Option<(&Document, Option<&'static str>, Option<View>)> {
        match self {
            Self::Text { document, language, markdown, .. } => Some((document, *language, *markdown)),
            Self::Keys { document, markdown, .. } => Some((document, Some("markdown"), Some(*markdown))),
            _ => None,
        }
    }

    /// Whether the rendered Markdown view shows (Scouter `previewing`).
    pub fn previewing(&self) -> bool {
        matches!(self.text(), Some((_, _, Some(view))) if view.is_rendered())
    }
}

/// Reveals a file the worker read: it is selected. A file that left the tree while the read ran is dropped.
/// The reader takes focus only for the startup file, which Scouter opens with the reader focused; a binary keeps
/// the list, as Scouter's `showBinary` does.
fn reveal(m: &mut Model, path: &Path, focus_reader: bool) -> bool {
    let startup = std::mem::take(&mut m.focus_reader_on_open);
    if m.browser.tree.entry(path).is_none() {
        return false;
    }
    m.browser.select(path);
    m.message = None;
    m.notice = None;
    if startup && focus_reader {
        m.focus = Focus::Reader;
    }
    true
}

/// Puts a shown text into the views that draw it: the code view, and the rendered view for Markdown.
fn present(ui: &Ui, shown: &Shown) {
    let Some((document, language, markdown)) = shown.text() else { return };
    ui.reader.show(document, language);
    if markdown.is_some() {
        ui.markdown.show(&document.text);
    }
}

/// Shows a text document the worker read. A key page's search does not carry over to the file.
pub fn show_text(m: &mut Model, ui: &Ui, path: &Path, document: Document, stamp: String) {
    if !reveal(m, path, true) {
        return;
    }
    let language = language_for(&m.associations, &m.opened, path);
    let markdown = language.filter(|filetype| is_markdown(filetype)).map(|_| View::default());
    m.serial += 1;
    let shown = Shown::Text {
        path: path.to_path_buf(),
        document,
        language,
        markdown,
        stamp: Some(stamp),
        serial: m.serial,
        signs: Vec::new(),
    };
    present(ui, &shown);
    if m.shown.shows_keys() {
        m.search = None;
    }
    m.shown = shown;
    gutter::request(m);
    super::picture::request(m);
    search::follow(m, ui);
    reset_scroll(m, ui, path);
    super::find::arrive(m, path);
}

/// A poll's answer for the open text document at `path` (Scouter `refreshDocument`): a changed file replaces the text
/// in place and keeps the scroll; a missing one keeps its text and says so, and is not polled again.
pub fn reread(m: &mut Model, ui: &Ui, path: &Path, result: Reread) {
    let Shown::Text { path: shown, document, stamp, .. } = &mut m.shown else { return };
    if shown.as_path() != path {
        return;
    }
    match result {
        Reread::Unchanged => {}
        Reread::Missing => {
            *stamp = None;
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let text = format!("{} is no longer on disk.", display_label(&name));
            super::tree::show_error(m, &text);
        }
        Reread::Changed { document: next, stamp: now } => {
            *stamp = Some(now);
            if next.text == document.text && next.truncated == document.truncated {
                return;
            }
            *document = next;
            present(ui, &m.shown);
            super::picture::request(m);
            search::follow(m, ui);
        }
    }
}

/// A shown file starts at its top, as Scouter's `revealViewer` does; a jump opens at the row it was left at.
fn reset_scroll(m: &mut Model, ui: &Ui, path: &Path) {
    let top = match m.restore.take() {
        Some((restored, top)) if restored == path => top,
        _ => 0,
    };
    ui.viewer.scroll_to(0.0, top as f64);
}

/// Shows an image: the picture is decoded by the view, and the header facts go to the footer.
pub fn show_image(m: &mut Model, ui: &Ui, path: &Path, info: image::Info, bytes: &[u8]) {
    if !reveal(m, path, true) {
        return;
    }
    ui.picture.show(bytes);
    m.shown = Shown::Image { path: path.to_path_buf(), info };
    m.search = None;
    reset_scroll(m, ui, path);
}

/// Shows a binary file's notice. Focus stays on the list, as in Scouter; the compact tier opens the reader page for the
/// notice instead, since a list tap there would otherwise show nothing.
pub fn show_binary(m: &mut Model, path: &Path, size: u64) {
    if !reveal(m, path, false) {
        return;
    }
    m.focus = if m.compact() { Focus::Reader } else { Focus::List };
    m.shown = Shown::Binary { path: path.to_path_buf(), size };
    m.restore = None;
    m.search = None;
}

/// The place the reader is at, for the jump history (Scouter `here`): a text's row, and the top of an image or binary.
/// The key page is not a place.
pub fn here(m: &Model) -> Option<Jump> {
    match &m.shown {
        Shown::None | Shown::Keys { .. } => None,
        Shown::Text { path, .. } => Some(Jump { path: path.clone(), top: m.top.floor() as usize }),
        Shown::Image { path, .. } | Shown::Binary { path, .. } => Some(Jump { path: path.clone(), top: 0 }),
    }
}

/// `m`: switches a Markdown document, or the key page, between its rendered view and its source (Scouter
/// `toggleMarkdownSource`).
pub fn toggle_markdown(m: &mut Model) {
    match &mut m.shown {
        Shown::Text { markdown: Some(view), .. } | Shown::Keys { markdown: view, .. } => *view = view.toggled(),
        _ => {}
    }
}

/// `?`: shows the key page, or puts the document back when the page is shown. The page takes focus, so the reader keys
/// scroll it; the document returns with its focus, scroll position, and search.
pub fn toggle_keys(m: &mut Model, ui: &Ui) {
    if m.shown.shows_keys() {
        return close_keys(m, ui);
    }
    let text = key_page_markdown_with(m.parser.keymap());
    let document = Document { lines: text.lines().count(), text, truncated: false };
    let back = Back {
        shown: std::mem::take(&mut m.shown),
        focus: m.focus,
        scroll: ui.viewer.position(),
        search: m.search.take(),
    };
    m.focus = Focus::Reader;
    m.message = None;
    let shown = Shown::Keys { document, markdown: View::default(), back: Box::new(back) };
    present(ui, &shown);
    m.shown = shown;
    ui.viewer.scroll_to(0.0, 0.0);
}

/// `q`: closes the document, so the welcome screen returns and focus goes to the tree; the file's row stays selected
/// (Scouter's `q` quits; here `Ctrl+q` quits). On the key page it puts back what the page replaced, as `?` does.
pub fn close(m: &mut Model, ui: &Ui) {
    if m.shown.shows_keys() {
        return close_keys(m, ui);
    }
    if !m.shown.is_some() {
        return;
    }
    m.shown = Shown::None;
    m.focus = Focus::List;
    m.search = None;
    m.restore = None;
    m.notice = None;
    m.message = None;
}

/// Puts the document back that the key page replaced. Only called while the key page shows.
pub fn close_keys(m: &mut Model, ui: &Ui) {
    let Shown::Keys { back, .. } = std::mem::take(&mut m.shown) else { return };
    let Back { shown, focus, scroll: (left, top), search } = *back;
    present(ui, &shown);
    m.shown = shown;
    super::picture::request(m);
    m.focus = focus;
    m.search = search;
    m.message = None;
    ui.viewer.scroll_to(left, top);
}

/// A key the rendered view refuses: the footer says which key shows the source (Scouter `showMessage`, not an error).
pub fn refuse(m: &mut Model, text: &str) {
    m.message = Some(Message { text: text.into(), error: false });
}

/// The title and the dim parent folder of a path under `root` (Scouter `titleParts`).
pub fn title_parts(root: &Path, path: &Path) -> (String, Option<String>) {
    let title = display_label(&path.file_name().unwrap_or_default().to_string_lossy());
    let Ok(rest) = path.strip_prefix(root) else { return (title, None) };
    let folder = rest.parent().filter(|folder| !folder.as_os_str().is_empty());
    (title, folder.map(|folder| format!("{}/", display_label(&folder.to_string_lossy()))))
}

/// The viewer header's badges for what is shown (Scouter `viewerBadgesFor`): the read-only mark, the rendered view,
/// the image or binary kind, then the cap.
pub fn badges(shown: &Shown, wrap: bool) -> Vec<Badge> {
    let mut badges = vec![Badge::quiet("read-only")];
    if shown.previewing() {
        badges.push(Badge::quiet("rendered"));
    }
    match shown {
        Shown::Image { .. } => badges.push(Badge::quiet("image")),
        Shown::Binary { .. } => badges.push(Badge::quiet("binary")),
        _ => {}
    }
    if let Shown::Text { document, .. } = shown
        && document.truncated
    {
        badges.push(Badge { text: "first 1 MiB".into(), kind: BadgeKind::Pill });
    }
    // Wrapping has no effect on the rendered view, and the image and binary have no text (Scouter `viewerBadgesFor`).
    if wrap && shown.text().is_some() && !shown.previewing() {
        badges.push(Badge { text: "wrap".into(), kind: BadgeKind::Pill });
    }
    badges
}

/// The rows the shown text takes: its lines, or the rendered Markdown's rows (`rendered_rows`). An image fills the
/// viewport, so it takes none.
pub fn content_rows(shown: &Shown, rendered_rows: usize, text_rows: usize) -> usize {
    if shown.previewing() {
        rendered_rows.max(1)
    } else if shown.text().is_some() {
        text_rows.max(1)
    } else {
        0
    }
}

/// The footer's facts for what is shown (Scouter `documentInfo`, `setDocument`), and its scroll position in a
/// viewport of `viewport` rows. A binary has no position: Scouter's stale `All` is not copied.
pub fn footer_facts(
    shown: &Shown,
    viewport: usize,
    rendered_rows: usize,
    text_rows: usize,
    top: f64,
) -> Option<(DocumentInfo, Option<String>)> {
    if let Some((document, language, markdown)) = shown.text() {
        let info = DocumentInfo::Text {
            lines: document.lines,
            language: language.unwrap_or("plain text").into(),
            truncated: document.truncated,
            rendered: markdown.map(|view| view.is_rendered()),
        };
        return Some((info, Some(scroll_label(content_rows(shown, rendered_rows, text_rows), viewport, top))));
    }
    match shown {
        Shown::Image { info, .. } => {
            let info = DocumentInfo::Image { width: info.width, height: info.height, format: info.format.into() };
            Some((info, Some(scroll_label(viewport, viewport, 0.0))))
        }
        Shown::Binary { size, .. } => Some((DocumentInfo::Binary { size: format_size(*size) }, None)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_keeps_its_folder_as_the_parent() {
        let root = Path::new("/vault");
        assert_eq!(
            title_parts(root, Path::new("/vault/Gringotts/key.rs")),
            ("key.rs".into(), Some("Gringotts/".into()))
        );
        assert_eq!(title_parts(root, Path::new("/vault/README.md")), ("README.md".into(), None));
        assert_eq!(title_parts(root, Path::new("/elsewhere/Louvre.txt")), ("Louvre.txt".into(), None));
    }

    #[test]
    fn the_key_page_is_a_rendered_markdown_document() {
        let text = fmc::keys::key_page_markdown();
        let back = Back { shown: Shown::None, focus: Focus::List, scroll: (0.0, 0.0), search: None };
        let keys = Shown::Keys {
            document: Document { lines: text.lines().count(), text, truncated: false },
            markdown: View::Rendered,
            back: Box::new(back),
        };
        assert!(keys.has_document() && keys.shows_keys());
        assert!(keys.previewing(), "`/` is refused until `m` shows the source");
        let (_, language, markdown) = keys.text().expect("the page is text");
        assert_eq!((language, markdown), (Some("markdown"), Some(View::Rendered)));
        assert!(badges(&keys, true).iter().all(|badge| badge.text != "wrap"));
    }

    #[test]
    fn only_a_rendered_markdown_document_previews() {
        let document = Document { text: "# Hogwarts\n".into(), lines: 1, truncated: false };
        let shown = |markdown| Shown::Text {
            path: "a.md".into(),
            document: document.clone(),
            language: Some("markdown"),
            markdown,
            stamp: None,
            serial: 0,
            signs: Vec::new(),
        };
        assert!(shown(Some(View::Rendered)).previewing());
        assert!(!shown(Some(View::Source)).previewing());
        assert!(!shown(None).previewing());
        assert_eq!(badges(&shown(Some(View::Rendered)), false).len(), 2);
        assert_eq!(badges(&shown(Some(View::Source)), true).last().map(|b| b.text.as_str()), Some("wrap"));
    }
}
