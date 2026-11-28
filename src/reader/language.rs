//! The filetypes fmc highlights: Scouter's (`src/reader/code-view.ts:20-29`, `languageFor`) and the owner's extra
//! grammars. Other files show as plain text.

use std::path::Path;

/// OpenTUI's bundled grammars (JavaScript, TypeScript, TSX, Markdown, Zig, and Rust), the ten vendored ones Scouter
/// uses, and the twelve the owner added. `javascriptreact` is an alias of the
/// JavaScript grammar.
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
    "diff",
    "elixir",
    "heex",
    "hcl",
    "jsdoc",
    "lua",
    "regex",
    "sql",
    "svelte",
    "xml",
    "astro",
    "vim",
    "query",
    "json5",
    "just",
    "vimdoc",
];

/// Extensions OpenTUI's table does not know, with the filetype fmc highlights them as. `.tf` and `.tfvars` are
/// Terraform, which HCL covers; `.scm` is a tree-sitter query, because fmc has no Scheme grammar.
const EXTENSIONS: &[(&str, &str)] = &[
    ("heex", "heex"),
    ("hcl", "hcl"),
    ("tf", "hcl"),
    ("tfvars", "hcl"),
    ("json5", "json5"),
    ("just", "just"),
    ("scm", "query"),
];

/// File names OpenTUI's table does not know. A `justfile` has no extension, so the name is the only key (`Justfile` is
/// matched through the lowercased name).
const NAMES: &[(&str, &str)] = &[("justfile", "just"), (".justfile", "just")];

/// Whether a filetype, as OpenTUI names it from a path, is one fmc highlights.
pub fn is_highlighted(filetype: &str) -> bool {
    HIGHLIGHTED.contains(&filetype)
}

/// The filetype of a file name or extension that OpenTUI's table lacks, from `NAMES` first and then `EXTENSIONS`; `None`
/// leaves the path to OpenTUI's lookup.
pub fn fallback_filetype(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    if let Some((_, filetype)) = NAMES.iter().find(|(known, _)| *known == name) {
        return Some(filetype);
    }
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    EXTENSIONS.iter().find(|(known, _)| *known == extension).map(|(_, filetype)| *filetype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_scouters_languages_and_the_owners_extra_ones_and_no_others() {
        assert_eq!(HIGHLIGHTED.len(), 34);
        assert!(is_highlighted("rust") && is_highlighted("javascriptreact") && is_highlighted("markdown_inline"));
        assert!(is_highlighted("elixir") && is_highlighted("hcl") && is_highlighted("vim"));
        assert!(!is_highlighted("make") && !is_highlighted("cpp") && !is_highlighted("plain text"));
        assert!(is_highlighted("query") && is_highlighted("json5"));
        assert!(is_highlighted("just") && is_highlighted("vimdoc"));
        // luadoc and luap are injected into Lua only, so they have no file of their own.
        assert!(!is_highlighted("eex") && !is_highlighted("luadoc") && !is_highlighted("luap"));
    }

    #[test]
    fn the_extensions_opentui_lacks_name_their_filetype() {
        let at = |path: &str| fallback_filetype(Path::new(path));
        assert_eq!(at("Gringotts/vault.tf"), Some("hcl"));
        assert_eq!(at("Hogwarts/owls.TFVARS"), Some("hcl"));
        assert_eq!(at("Louvre/keys.hcl"), Some("hcl"));
        assert_eq!(at("Hogwarts/live.heex"), Some("heex"));
        assert_eq!(at("Diagon/owls.json5"), Some("json5"));
        assert_eq!(at("Diagon/query.SCM"), Some("query"));
        // The name of a file with no extension is its key, in any case.
        assert_eq!(at("Hogwarts/justfile"), Some("just"));
        assert_eq!(at("Hogwarts/Justfile"), Some("just"));
        // OpenTUI knows these, so they are left to its lookup.
        assert_eq!(at("Hogwarts/owls.ex"), None);
        assert_eq!(at("CERN/config.toml"), None);
    }
}
