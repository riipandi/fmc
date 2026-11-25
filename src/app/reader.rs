//! The reader on the UI thread: what is shown, the file open that shows it, and the header and footer facts (Scouter
//! `src/app/app.ts` `showDocument`, `src/app/chrome.ts` `viewerHeaderFor`, `viewerBadgesFor`).

use std::path::{Path, PathBuf};

use fmc::keys::Focus;
use fmc::reader::binary::format_size;
use fmc::reader::document::Document;
use fmc::reader::image;
use fmc::reader::jump::Jump;
use fmc::reader::markdown::{View, is_markdown};
use fmc::reader::position::scroll_label;
use fmc::text::display_label;

use super::{Model, Ui};
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

impl Shown {
    pub fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the reader holds a document the keys can reach (Scouter `hasDocument`): text or an image.
    pub fn has_document(&self) -> bool {
        matches!(self, Self::Text { .. } | Self::Image { .. })
    }

    /// Whether the rendered Markdown view shows (Scouter `previewing`).
    pub fn previewing(&self) -> bool {
        matches!(self, Self::Text { markdown: Some(view), .. } if view.is_rendered())
    }
}

/// Reveals a file the worker read: it is selected, and the footer shows its path. A file that left the tree while
/// the read ran is dropped (`rust.md`, "Workers answer through events"). The reader takes focus only for the startup
/// file, which Scouter opens with the reader focused; a binary keeps the list, as Scouter's `showBinary` does.
fn reveal(m: &mut Model, path: &Path, focus_reader: bool) -> bool {
    let startup = std::mem::take(&mut m.focus_reader_on_open);
    if m.browser.tree.entry(path).is_none() {
        return false;
    }
    m.browser.select(path);
    m.location = path.to_string_lossy().into_owned();
    m.message = None;
    m.notice = None;
    if startup && focus_reader {
        m.focus = Focus::Reader;
    }
    true
}

/// Shows a text document the worker read.
pub fn show_text(m: &mut Model, ui: &Ui, path: &Path, document: Document) {
    if !reveal(m, path, true) {
        return;
    }
    let language = language_for(path);
    let markdown = language.filter(|filetype| is_markdown(filetype)).map(|_| View::default());
    ui.reader.show(&document, language);
    if markdown.is_some() {
        ui.markdown.show(&document.text);
    }
    m.shown = Shown::Text { path: path.to_path_buf(), document, language, markdown };
    reset_scroll(m, ui, path);
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
    reset_scroll(m, ui, path);
}

/// Shows a binary file's notice. Focus stays on the list, as in Scouter.
pub fn show_binary(m: &mut Model, path: &Path, size: u64) {
    if !reveal(m, path, false) {
        return;
    }
    m.focus = Focus::List;
    m.shown = Shown::Binary { path: path.to_path_buf(), size };
    m.restore = None;
}

/// The place the reader is at, for the jump history (Scouter `here`): a text's row, and the top of an image or binary.
pub fn here(m: &Model) -> Option<Jump> {
    match &m.shown {
        Shown::None => None,
        Shown::Text { path, .. } => Some(Jump { path: path.clone(), top: m.top.floor() as usize }),
        Shown::Image { path, .. } | Shown::Binary { path, .. } => Some(Jump { path: path.clone(), top: 0 }),
    }
}

/// `m`: switches a Markdown document between its rendered view and its source (Scouter `toggleMarkdownSource`).
pub fn toggle_markdown(m: &mut Model) {
    if let Shown::Text { markdown: Some(view), .. } = &mut m.shown {
        *view = view.toggled();
    }
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
    if wrap && matches!(shown, Shown::Text { .. }) && !shown.previewing() {
        badges.push(Badge { text: "wrap".into(), kind: BadgeKind::Pill });
    }
    badges
}

/// The rows the shown text takes: its lines, or the rendered Markdown's rows (`rendered_rows`). An image fills the
/// viewport, so it takes none.
pub fn content_rows(shown: &Shown, rendered_rows: usize, text_rows: usize) -> usize {
    match shown {
        Shown::Text { .. } if shown.previewing() => rendered_rows.max(1),
        Shown::Text { .. } => text_rows.max(1),
        _ => 0,
    }
}

/// The footer's facts for what is shown (Scouter `documentInfo`, `setDocument`), and its scroll position in a
/// viewport of `viewport` rows. A binary has no position: Scouter's stale `All` is not copied (`plan-00001` phase 13).
pub fn footer_facts(
    shown: &Shown,
    viewport: usize,
    rendered_rows: usize,
    text_rows: usize,
    top: f64,
) -> Option<(DocumentInfo, Option<String>)> {
    match shown {
        Shown::None => None,
        Shown::Text { document, language, markdown, .. } => {
            let info = DocumentInfo::Text {
                lines: document.lines,
                language: language.unwrap_or("plain text").into(),
                truncated: document.truncated,
                rendered: markdown.map(|view| view.is_rendered()),
            };
            Some((info, Some(scroll_label(content_rows(shown, rendered_rows, text_rows), viewport, top))))
        }
        Shown::Image { info, .. } => {
            let info = DocumentInfo::Image { width: info.width, height: info.height, format: info.format.into() };
            Some((info, Some(scroll_label(viewport, viewport, 0.0))))
        }
        Shown::Binary { size, .. } => Some((DocumentInfo::Binary { size: format_size(*size) }, None)),
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
    fn only_a_rendered_markdown_document_previews() {
        let document = Document { text: "# Hogwarts\n".into(), lines: 1, truncated: false };
        let shown = |markdown| Shown::Text {
            path: "a.md".into(),
            document: document.clone(),
            language: Some("markdown"),
            markdown,
        };
        assert!(shown(Some(View::Rendered)).previewing());
        assert!(!shown(Some(View::Source)).previewing());
        assert!(!shown(None).previewing());
        assert_eq!(badges(&shown(Some(View::Rendered)), false).len(), 2);
        assert_eq!(badges(&shown(Some(View::Source)), true).last().map(|b| b.text.as_str()), Some("wrap"));
    }
}
