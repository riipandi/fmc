//! Markdown's two views (Scouter `src/reader/code-view.ts`, `previewing` and `toggleSource`): the rendered view is the
//! default, `m` switches to the source, and the search and paragraph motions refuse while the rendered view shows.

/// How a Markdown document is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    Rendered,
    Source,
}

impl View {
    pub fn toggled(self) -> Self {
        match self {
            Self::Rendered => Self::Source,
            Self::Source => Self::Rendered,
        }
    }

    pub fn is_rendered(self) -> bool {
        self == Self::Rendered
    }
}

/// The footer message for `/` in the rendered view (Scouter `startSearch`, `src/app/app.ts:999-1005`).
pub const SEARCH_REFUSED: &str = "Press m to search this Markdown file's source.";

/// The footer message for `{` or `}` in the rendered view (Scouter `src/reader/reader-motion.ts:103-106`).
pub const PARAGRAPH_REFUSED: &str = "Press m to move by paragraph in this Markdown file's source.";

/// Whether a filetype is Markdown: `.md` and `.markdown` (Scouter `languageFor`, `src/reader/code-view.ts:230-232`).
pub fn is_markdown(filetype: &str) -> bool {
    filetype == "markdown"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_opens_rendered_and_m_toggles_it() {
        assert_eq!(View::default(), View::Rendered);
        assert_eq!(View::default().toggled(), View::Source);
        assert_eq!(View::default().toggled().toggled(), View::Rendered);
        assert!(!View::Source.is_rendered());
    }

    #[test]
    fn only_markdown_is_markdown() {
        assert!(is_markdown("markdown"));
        assert!(!is_markdown("markdown_inline") && !is_markdown("rust") && !is_markdown("mermaid"));
    }

    #[test]
    fn the_refusals_name_the_key_that_works() {
        assert!(SEARCH_REFUSED.starts_with("Press m to search this Markdown file's source"));
        assert!(PARAGRAPH_REFUSED.starts_with("Press m to move by paragraph"));
    }
}
