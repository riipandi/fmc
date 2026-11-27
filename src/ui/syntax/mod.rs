//! Syntax highlighting (Scouter `src/reader/grammars.ts`, `src/reader/code-view.ts`): the grammars and queries Scouter
//! uses, the filetype a path gets, and the style of each capture name from the palette (`research/scouter-inventory.md`
//! §8.3). Rust is registered by `opentui`'s `tree-sitter` feature; the others are registered here.

use std::path::Path;

use fmc::reader::associations::{Associations, Language as Association};
use fmc::reader::language::is_highlighted;
use fmc::theme::Palette;
use opentui::native::{Context, SyntaxStyle};
use opentui::prelude::{Result, Style};
use opentui::syntax::tree_sitter::Language;
use opentui::syntax::{CompiledGrammar, Grammar, SyntaxTheme, filetype_for_path, register_alias, register_lazy};

use super::styled::color;

/// Registers the grammars, compiled on first use as OpenTUI's bundled ones are. Core's queries replace the bundled
/// ones for JavaScript, TypeScript, TSX, Markdown, and Zig (`architecture.md`, "Core's highlight queries"). Call it
/// once, before the first code view is shown.
pub fn register() {
    register_lazy("javascript", || {
        grammar(tree_sitter_javascript::LANGUAGE, include_str!("queries/javascript/highlights.scm"))
    });
    register_lazy("typescript", || {
        grammar(tree_sitter_typescript::LANGUAGE_TYPESCRIPT, include_str!("queries/typescript/highlights.scm"))
    });
    register_lazy("typescriptreact", || {
        grammar(tree_sitter_typescript::LANGUAGE_TSX, include_str!("queries/typescriptreact/highlights.scm"))
    });
    register_lazy("markdown", || {
        Grammar::new(tree_sitter_md::LANGUAGE, include_str!("queries/markdown/highlights.scm"))
            .injections(include_str!("queries/markdown/injections.scm"))
            .core_captures()
            .compile()
            .ok()
    });
    register_lazy("markdown_inline", || {
        grammar(tree_sitter_md::INLINE_LANGUAGE, include_str!("queries/markdown_inline/highlights.scm"))
    });
    register_lazy("zig", || grammar(tree_sitter_zig::LANGUAGE, include_str!("queries/zig/highlights.scm")));
    register_lazy("json", || grammar(tree_sitter_json::LANGUAGE, include_str!("queries/json/highlights.scm")));
    register_lazy("css", || grammar(tree_sitter_css::LANGUAGE, include_str!("queries/css/highlights.scm")));
    register_lazy("html", || grammar(tree_sitter_html::LANGUAGE, include_str!("queries/html/highlights.scm")));
    register_lazy("go", || grammar(tree_sitter_go::LANGUAGE, include_str!("queries/go/highlights.scm")));
    register_lazy("python", || grammar(tree_sitter_python::LANGUAGE, include_str!("queries/python/highlights.scm")));
    register_lazy("bash", || grammar(tree_sitter_bash::LANGUAGE, include_str!("queries/bash/highlights.scm")));
    register_lazy("yaml", || grammar(tree_sitter_yaml::LANGUAGE, include_str!("queries/yaml/highlights.scm")));
    register_lazy("toml", || grammar(tree_sitter_toml_ng::LANGUAGE, include_str!("queries/toml/highlights.scm")));
    register_lazy("c", || grammar(tree_sitter_c::LANGUAGE, include_str!("queries/c/highlights.scm")));
    register_lazy("java", || grammar(tree_sitter_java::LANGUAGE, include_str!("queries/java/highlights.scm")));
    register_alias("javascriptreact", "javascript");
}

/// Overlapping captures cascade as Core's do (`core_captures`), which Scouter's highlighter applies to every grammar:
/// without it, a TOML key takes the style of `(bare_key) @type` where Scouter shows `(pair) @property`. A grammar that
/// does not compile leaves its files plain; the test below catches that.
fn grammar(language: impl Into<Language>, highlights: &'static str) -> Option<CompiledGrammar> {
    Grammar::new(language, highlights).core_captures().compile().ok()
}

/// The filetype a path is highlighted as. A matching `file_associations` entry decides first, and plain text turns
/// highlighting off; otherwise it is OpenTUI's filetype, kept only when Scouter highlights it (`languageFor`).
/// `root` is the folder fmc opened, which a path pattern is matched from.
pub fn language_for(associations: &Associations, root: &Path, path: &Path) -> Option<&'static str> {
    match associations.language_of(path, root) {
        Some(Association::Plain) => None,
        Some(Association::Filetype(filetype)) => Some(filetype),
        None => {
            let filetype = filetype_for_path(&path.to_string_lossy())?;
            is_highlighted(filetype).then_some(filetype)
        }
    }
}

/// The style of each capture name, as Scouter's `syntaxStyles()` maps them. Unlisted names take their first dotted
/// part's style, as Core's `SyntaxStyle.getStyle` does.
pub fn theme(p: &Palette) -> SyntaxTheme {
    let fg = |hex: &str| Style::new().fg(color(hex));
    let s = &p.syntax;
    let (keyword, string, comment) = (fg(s.keyword), fg(s.string), fg(s.comment).italic());
    let (constant, function, punctuation) = (fg(s.constant), fg(s.function), fg(s.punctuation));
    let text = fg(p.text);
    let mut theme = SyntaxTheme::new()
        .style("search.match", Style::new().fg(color(p.match_fg)).bg(color(p.match_bg)).bold())
        .first_part_fallback()
        .style("default", text)
        .style("variable", text)
        .style("keyword", keyword.bold())
        .style("operator", keyword)
        .style("tag", keyword)
        .style("string", string)
        .style("comment", comment)
        .style("punctuation", punctuation)
        .style("function", function)
        .style("property", function)
        .style("attribute", function)
        .style("markup.heading", function.bold())
        .style("markup.bold", Style::new().bold())
        .style("markup.strong", Style::new().bold())
        .style("markup.italic", Style::new().italic())
        .style("markup.raw", string)
        .style("markup.link", function.underline());
    for name in ["number", "boolean", "constant", "type", "constructor", "label", "namespace", "escape"] {
        theme.set(name, constant);
    }
    for level in 1..=6 {
        theme.set(format!("markup.heading.{level}"), function.bold());
    }
    theme
}

#[cfg(test)]
mod tests;

/// The capture names the queries use (`src/ui/syntax/queries` and OpenTUI's bundled Rust query). The buffer's style
/// table names each one `opentui.<name>`, so a layer can paint by name; a name missing here is drawn plain.
const CAPTURE_NAMES: &[&str] = &[
    "attribute",
    "boolean",
    "character",
    "character.special",
    "charset",
    "cImport",
    "comment",
    "comment.documentation",
    "conceal",
    "constant",
    "constant.builtin",
    "constructor",
    "delimiter",
    "embedded",
    "escape",
    "function",
    "function.builtin",
    "function.call",
    "function.macro",
    "function.method",
    "function.method.call",
    "function.special",
    "import",
    "keyframes",
    "keyword",
    "keyword.conditional",
    "keyword.conditional.ternary",
    "keyword.coroutine",
    "keyword.directive",
    "keyword.exception",
    "keyword.function",
    "keyword.import",
    "keyword.modifier",
    "keyword.operator",
    "keyword.repeat",
    "keyword.return",
    "keyword.type",
    "label",
    "markup.heading",
    "markup.heading.1",
    "markup.heading.2",
    "markup.heading.3",
    "markup.heading.4",
    "markup.heading.5",
    "markup.heading.6",
    "markup.italic",
    "markup.link",
    "markup.link.bracket.close",
    "markup.link.label",
    "markup.link.url",
    "markup.list",
    "markup.list.checked",
    "markup.list.unchecked",
    "markup.quote",
    "markup.raw",
    "markup.raw.block",
    "markup.strikethrough",
    "markup.strong",
    "markup.underline",
    "media",
    "module",
    "module.builtin",
    "namespace",
    "none",
    "nospell",
    "number",
    "number.float",
    "operator",
    "property",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "spell",
    "string",
    "string.escape",
    "string.regexp",
    "string.special",
    "string.special.key",
    "string.special.url",
    "supports",
    "tag",
    "tag.attribute",
    "tag.builtin",
    "tag.delimiter",
    "tag.error",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
];

/// The style table of a code buffer: each capture name's style as the widget would register it, plus the search marks,
/// which layers name as `opentui.search.match` (Scouter `search.match`) and `opentui.search.current` (the match the view
/// is on, plan-00010 D3). A buffer keeps the table it was given, so this one stands in for the widget's own.
pub fn table(context: &Context, theme: &SyntaxTheme, search: Style, current: Style) -> Result<SyntaxStyle> {
    let table = SyntaxStyle::new(context)?;
    for name in CAPTURE_NAMES {
        if let Some(style) = theme.get(name) {
            table.register(&format!("opentui.{name}"), style)?;
        }
    }
    table.register("opentui.search.match", search)?;
    table.register("opentui.search.current", current)?;
    Ok(table)
}
