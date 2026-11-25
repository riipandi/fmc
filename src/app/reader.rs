//! The reader on the UI thread: what is shown, the file open that shows it, and the header and footer facts (Scouter
//! `src/app/app.ts` `showDocument`, `src/app/chrome.ts` `viewerHeaderFor`, `viewerBadgesFor`).

use std::path::{Path, PathBuf};

use fmc::keys::Focus;
use fmc::reader::document::Document;
use fmc::reader::position::scroll_label;
use fmc::text::display_label;

use super::{Model, Ui};
use crate::ui::footer::DocumentInfo;
use crate::ui::header::{Badge, BadgeKind};
use crate::ui::syntax::language_for;

/// What the reader shows.
#[derive(Debug, Default)]
pub enum Shown {
    #[default]
    None,
    Text {
        path: PathBuf,
        document: Document,
        language: Option<&'static str>,
    },
}

impl Shown {
    pub fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Shows a document the worker read, with its file revealed and selected. A file that left the tree while the read
/// ran is dropped (`rust.md`, "Workers answer through events").
pub fn show(m: &mut Model, ui: &Ui, path: &Path, document: Document) {
    if m.browser.tree.entry(path).is_none() {
        return;
    }
    let language = language_for(path);
    ui.reader.show(&document, language);
    m.browser.select(path);
    m.location = path.to_string_lossy().into_owned();
    m.message = None;
    m.notice = None;
    if std::mem::take(&mut m.focus_reader_on_open) {
        m.focus = Focus::Reader;
    }
    m.shown = Shown::Text { path: path.to_path_buf(), document, language };
}

/// The title and the dim parent folder of a path under `root` (Scouter `titleParts`).
pub fn title_parts(root: &Path, path: &Path) -> (String, Option<String>) {
    let title = display_label(&path.file_name().unwrap_or_default().to_string_lossy());
    let Ok(rest) = path.strip_prefix(root) else { return (title, None) };
    let folder = rest.parent().filter(|folder| !folder.as_os_str().is_empty());
    (title, folder.map(|folder| format!("{}/", display_label(&folder.to_string_lossy()))))
}

/// The viewer header's badges for what is shown (Scouter `viewerBadgesFor`): the read-only mark, then the cap.
pub fn badges(shown: &Shown) -> Vec<Badge> {
    let mut badges = vec![Badge::quiet("read-only")];
    if let Shown::Text { document, .. } = shown
        && document.truncated
    {
        badges.push(Badge { text: "first 1 MiB".into(), kind: BadgeKind::Pill });
    }
    badges
}

/// The footer's facts for a shown document (Scouter `documentInfo`), and its scroll position in a viewport of
/// `viewport` rows.
pub fn footer_facts(shown: &Shown, viewport: usize) -> Option<(DocumentInfo, String)> {
    let Shown::Text { document, language, .. } = shown else { return None };
    let info = DocumentInfo::Text {
        lines: document.lines,
        language: language.unwrap_or("plain text").into(),
        truncated: document.truncated,
        rendered: None,
    };
    Some((info, scroll_label(document.lines.max(1), viewport, 0)))
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
}
