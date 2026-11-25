//! The reader on the UI thread: what is shown, the file open that shows it, and the header and footer facts (Scouter
//! `src/app/app.ts` `showDocument`, `src/app/chrome.ts` `viewerHeaderFor`, `viewerBadgesFor`).

use std::path::{Path, PathBuf};

use fmc::keys::Focus;
use fmc::reader::document::Document;
use fmc::reader::markdown::{View, is_markdown};
use fmc::reader::position::scroll_label;
use fmc::text::display_label;

use super::{Model, Ui};
use crate::ui::footer::{DocumentInfo, Message};
use crate::ui::header::{Badge, BadgeKind};
use crate::ui::syntax::language_for;

/// What the reader shows. `markdown` is the view of a Markdown document, `None` for any other.
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
}

impl Shown {
    pub fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the rendered Markdown view shows (Scouter `previewing`).
    pub fn previewing(&self) -> bool {
        matches!(self, Self::Text { markdown: Some(view), .. } if view.is_rendered())
    }
}

/// Shows a document the worker read, with its file revealed and selected. A file that left the tree while the read
/// ran is dropped (`rust.md`, "Workers answer through events").
pub fn show(m: &mut Model, ui: &Ui, path: &Path, document: Document) {
    if m.browser.tree.entry(path).is_none() {
        return;
    }
    let language = language_for(path);
    let markdown = language.filter(|filetype| is_markdown(filetype)).map(|_| View::default());
    ui.reader.show(&document, language);
    if markdown.is_some() {
        ui.markdown.show(&document.text);
    }
    m.browser.select(path);
    m.location = path.to_string_lossy().into_owned();
    m.message = None;
    m.notice = None;
    if std::mem::take(&mut m.focus_reader_on_open) {
        m.focus = Focus::Reader;
    }
    m.shown = Shown::Text { path: path.to_path_buf(), document, language, markdown };
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
/// then the cap.
pub fn badges(shown: &Shown) -> Vec<Badge> {
    let mut badges = vec![Badge::quiet("read-only")];
    if shown.previewing() {
        badges.push(Badge::quiet("rendered"));
    }
    if let Shown::Text { document, .. } = shown
        && document.truncated
    {
        badges.push(Badge { text: "first 1 MiB".into(), kind: BadgeKind::Pill });
    }
    badges
}

/// The rows the shown text takes: its lines, or the rendered Markdown's rows (`rendered_rows`).
pub fn content_rows(shown: &Shown, rendered_rows: usize) -> usize {
    if shown.previewing() { rendered_rows } else { lines_of(shown).max(1) }
}

fn lines_of(shown: &Shown) -> usize {
    match shown {
        Shown::Text { document, .. } => document.lines,
        Shown::None => 0,
    }
}

/// The footer's facts for a shown document (Scouter `documentInfo`), and its scroll position in a viewport of
/// `viewport` rows, where a rendered document is `rendered_rows` tall.
pub fn footer_facts(shown: &Shown, viewport: usize, rendered_rows: usize) -> Option<(DocumentInfo, String)> {
    let Shown::Text { document, language, markdown, .. } = shown else { return None };
    let info = DocumentInfo::Text {
        lines: document.lines,
        language: language.unwrap_or("plain text").into(),
        truncated: document.truncated,
        rendered: markdown.map(|view| view.is_rendered()),
    };
    Some((info, scroll_label(content_rows(shown, rendered_rows), viewport, 0)))
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
        assert_eq!(badges(&shown(Some(View::Rendered))).len(), 2);
    }
}
