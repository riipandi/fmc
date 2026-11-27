//! The filetypes Scouter highlights (`src/reader/code-view.ts:12-20`, `languageFor`). Other files show as plain text.

/// OpenTUI's bundled grammars (JavaScript, TypeScript, TSX, Markdown, Zig, and Rust) and the ten vendored ones.
/// `javascriptreact` is an alias of the JavaScript grammar.
pub const HIGHLIGHTED: &[&str] = &[
    "javascript",
    "javascriptreact",
    "typescript",
    "typescriptreact",
    "markdown",
    "markdown_inline",
    "zig",
    "rust",
    "json",
    "css",
    "html",
    "go",
    "python",
    "bash",
    "yaml",
    "toml",
    "c",
    "java",
];

/// Whether a filetype, as OpenTUI names it from a path, is one Scouter highlights.
pub fn is_highlighted(filetype: &str) -> bool {
    HIGHLIGHTED.contains(&filetype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_scouters_sixteen_languages_and_no_others() {
        assert_eq!(HIGHLIGHTED.len(), 18);
        assert!(is_highlighted("rust") && is_highlighted("javascriptreact") && is_highlighted("markdown_inline"));
        assert!(!is_highlighted("make") && !is_highlighted("cpp") && !is_highlighted("plain text"));
    }
}
