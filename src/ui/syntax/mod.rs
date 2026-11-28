//! Syntax highlighting (Scouter `src/reader/grammars.ts`, `src/reader/code-view.ts`): the grammars and queries Scouter
//! uses, the filetype a path gets, and the style of each capture name from the palette.
//! Rust is registered by `opentui`'s `tree-sitter` feature; the others are registered here.

use std::path::Path;

use fmc::reader::associations::{Associations, Language as Association};
use fmc::reader::language::{fallback_filetype, is_highlighted};
use fmc::theme::{Hex, Palette};
use opentui::native::{Context, SyntaxStyle};
use opentui::prelude::{Result, Style};
use opentui::syntax::tree_sitter::Language;
use opentui::syntax::{CompiledGrammar, Grammar, SyntaxTheme, filetype_for_path, register_alias, register_lazy};

use super::styled::color;

/// Registers the grammars, compiled on first use as OpenTUI's bundled ones are. Core's queries replace the bundled
/// ones for JavaScript, TypeScript, TSX, Markdown, and Zig. Call it
/// once, before the first code view is shown.
pub fn register() {
    register_lazy("javascript", || {
        injected(
            tree_sitter_javascript::LANGUAGE,
            include_str!("queries/javascript/highlights.scm"),
            include_str!("queries/javascript/injections.scm"),
        )
    });
    register_lazy("typescript", || {
        injected(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            include_str!("queries/typescript/highlights.scm"),
            include_str!("queries/javascript/injections.scm"),
        )
    });
    register_lazy("typescriptreact", || {
        injected(
            tree_sitter_typescript::LANGUAGE_TSX,
            include_str!("queries/typescriptreact/highlights.scm"),
            include_str!("queries/javascript/injections.scm"),
        )
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
    register_lazy("html", || {
        injected(
            tree_sitter_html::LANGUAGE,
            include_str!("queries/html/highlights.scm"),
            include_str!("queries/html/injections.scm"),
        )
    });
    register_lazy("go", || grammar(tree_sitter_go::LANGUAGE, include_str!("queries/go/highlights.scm")));
    register_lazy("python", || {
        injected(
            tree_sitter_python::LANGUAGE,
            include_str!("queries/python/highlights.scm"),
            include_str!("queries/python/injections.scm"),
        )
    });
    register_lazy("bash", || grammar(tree_sitter_bash::LANGUAGE, include_str!("queries/bash/highlights.scm")));
    register_lazy("yaml", || grammar(tree_sitter_yaml::LANGUAGE, include_str!("queries/yaml/highlights.scm")));
    register_lazy("toml", || grammar(tree_sitter_toml_ng::LANGUAGE, include_str!("queries/toml/highlights.scm")));
    register_lazy("c", || grammar(tree_sitter_c::LANGUAGE, include_str!("queries/c/highlights.scm")));
    register_lazy("java", || grammar(tree_sitter_java::LANGUAGE, include_str!("queries/java/highlights.scm")));
    register_lazy("diff", || grammar(tree_sitter_diff::LANGUAGE, include_str!("queries/diff/highlights.scm")));
    register_lazy("elixir", || {
        injected(
            tree_sitter_elixir::LANGUAGE,
            include_str!("queries/elixir/highlights.scm"),
            include_str!("queries/elixir/injections.scm"),
        )
    });
    register_lazy("heex", || {
        injected(
            tree_sitter_heex::LANGUAGE,
            include_str!("queries/heex/highlights.scm"),
            include_str!("queries/heex/injections.scm"),
        )
    });
    register_lazy("hcl", || grammar(tree_sitter_hcl::LANGUAGE, include_str!("queries/hcl/highlights.scm")));
    register_lazy("jsdoc", || grammar(tree_sitter_jsdoc::LANGUAGE, include_str!("queries/jsdoc/highlights.scm")));
    register_lazy("lua", || {
        injected(
            tree_sitter_lua::LANGUAGE,
            include_str!("queries/lua/highlights.scm"),
            include_str!("queries/lua/injections.scm"),
        )
    });
    register_lazy("regex", || grammar(tree_sitter_regex::LANGUAGE, include_str!("queries/regex/highlights.scm")));
    register_lazy("sql", || grammar(tree_sitter_sequel::LANGUAGE, include_str!("queries/sql/highlights.scm")));
    register_lazy("svelte", || {
        injected(
            tree_sitter_svelte_ng::LANGUAGE,
            include_str!("queries/svelte/highlights.scm"),
            include_str!("queries/svelte/injections.scm"),
        )
    });
    register_lazy("xml", || grammar(tree_sitter_xml::LANGUAGE_XML, include_str!("queries/xml/highlights.scm")));
    register_lazy("astro", || {
        injected(
            tree_sitter_astro_next::LANGUAGE,
            include_str!("queries/astro/highlights.scm"),
            include_str!("queries/astro/injections.scm"),
        )
    });
    register_lazy("vim", || grammar(tree_sitter_vim::language(), include_str!("queries/vim/highlights.scm")));
    register_lazy("vimdoc", || grammar(vendored::vimdoc(), include_str!("queries/vimdoc/highlights.scm")));
    register_lazy("luadoc", || grammar(vendored::luadoc(), include_str!("queries/luadoc/highlights.scm")));
    register_lazy("luap", || grammar(vendored::luap(), include_str!("queries/luap/highlights.scm")));
    register_lazy("just", || grammar(vendored::just(), include_str!("queries/just/highlights.scm")));
    register_lazy("query", || grammar(tree_sitter_tsquery::LANGUAGE, include_str!("queries/query/highlights.scm")));
    register_lazy("json5", || {
        grammar(tree_sitter_json5_orchard::LANGUAGE, include_str!("queries/json5/highlights.scm"))
    });
    register_lazy("dockerfile", || {
        injected(
            tree_sitter_containerfile::LANGUAGE,
            include_str!("queries/dockerfile/highlights.scm"),
            include_str!("queries/dockerfile/injections.scm"),
        )
    });
    register_lazy("make", || grammar(tree_sitter_make::LANGUAGE, include_str!("queries/make/highlights.scm")));
    register_lazy("cmake", || grammar(tree_sitter_cmake::LANGUAGE, include_str!("queries/cmake/highlights.scm")));
    register_lazy("ini", || grammar(tree_sitter_ini::LANGUAGE, include_str!("queries/ini/highlights.scm")));
    register_lazy("properties", || {
        grammar(tree_sitter_properties::LANGUAGE, include_str!("queries/properties/highlights.scm"))
    });
    register_lazy("proto", || grammar(tree_sitter_proto::LANGUAGE, include_str!("queries/proto/highlights.scm")));
    register_lazy("graphql", || grammar(tree_sitter_graphql::LANGUAGE, include_str!("queries/graphql/highlights.scm")));
    register_lazy("nix", || grammar(tree_sitter_nix::LANGUAGE, include_str!("queries/nix/highlights.scm")));
    register_lazy("scss", || grammar(tree_sitter_scss::language(), include_str!("queries/scss/highlights.scm")));
    register_lazy("gitignore", || {
        grammar(konomanoasa_tree_sitter_gitignore::LANGUAGE, include_str!("queries/gitignore/highlights.scm"))
    });
    register_lazy("gitattributes", || {
        grammar(konomanoasa_tree_sitter_gitattributes::LANGUAGE, include_str!("queries/gitattributes/highlights.scm"))
    });
    register_lazy("gitconfig", || {
        grammar(konomanoasa_tree_sitter_gitconfig::LANGUAGE, include_str!("queries/gitconfig/highlights.scm"))
    });
    register_lazy("editorconfig", || {
        grammar(konomanoasa_tree_sitter_editorconfig::LANGUAGE, include_str!("queries/editorconfig/highlights.scm"))
    });
    register_lazy("cpp", || grammar(tree_sitter_cpp::LANGUAGE, include_str!("queries/cpp/highlights.scm")));
    register_lazy("csharp", || grammar(tree_sitter_c_sharp::LANGUAGE, include_str!("queries/csharp/highlights.scm")));
    register_lazy("ruby", || grammar(tree_sitter_ruby::LANGUAGE, include_str!("queries/ruby/highlights.scm")));
    register_lazy("php", || {
        injected(
            tree_sitter_php::LANGUAGE_PHP,
            include_str!("queries/php/highlights.scm"),
            include_str!("queries/php/injections.scm"),
        )
    });
    register_lazy("kotlin", || grammar(tree_sitter_kotlin_sg::LANGUAGE, include_str!("queries/kotlin/highlights.scm")));
    register_lazy("swift", || grammar(tree_sitter_swift::LANGUAGE, include_str!("queries/swift/highlights.scm")));
    register_lazy("dart", || grammar(tree_sitter_dart::LANGUAGE, include_str!("queries/dart/highlights.scm")));
    register_lazy("vue", || {
        injected(
            tree_sitter_vue_next::LANGUAGE,
            include_str!("queries/vue/highlights.scm"),
            include_str!("queries/vue/injections.scm"),
        )
    });
    register_alias("javascriptreact", "javascript");
}

/// Overlapping captures cascade as Core's do (`core_captures`), which Scouter's highlighter applies to every grammar:
/// without it, a TOML key takes the style of `(bare_key) @type` where Scouter shows `(pair) @property`. A grammar that
/// does not compile leaves its files plain; the test below catches that. An injection query is compiled with the
/// highlights, so a node name it does not know drops the whole grammar too.
fn grammar(language: impl Into<Language>, highlights: &'static str) -> Option<CompiledGrammar> {
    injected(language, highlights, "")
}

/// A grammar whose injection query hands the code inside a file to another grammar, by the filetype it names.
fn injected(
    language: impl Into<Language>,
    highlights: &'static str,
    injections: &'static str,
) -> Option<CompiledGrammar> {
    Grammar::new(language, highlights).injections(injections).core_captures().compile().ok()
}

/// The filetype a path is highlighted as. A matching `file_associations` entry decides first, and plain text turns
/// highlighting off. Otherwise the extensions OpenTUI does not know decide, then OpenTUI's filetype, kept only when
/// fmc highlights it (`languageFor`). `root` is the folder fmc opened, which a path pattern is matched from.
pub fn language_for(associations: &Associations, root: &Path, path: &Path) -> Option<&'static str> {
    match associations.language_of(path, root) {
        Some(Association::Plain) => None,
        Some(Association::Filetype(filetype)) => Some(filetype),
        None => {
            let filetype = fallback_filetype(path).or_else(|| filetype_for_path(&path.to_string_lossy()))?;
            is_highlighted(filetype).then_some(filetype)
        }
    }
}

/// The style of each capture name, as Scouter's `syntaxStyles()` maps them. Unlisted names take their first dotted
/// part's style, as Core's `SyntaxStyle.getStyle` does.
pub fn theme(p: &Palette) -> SyntaxTheme {
    let fg = |hex: Hex| Style::new().fg(color(hex));
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
mod vendored;

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
    "comment.doc",
    "comment.doc.__attribute__",
    "comment.documentation",
    "comment.error",
    "comment.hint",
    "comment.unused",
    "comment.warning",
    "conceal",
    "constant",
    "constant.builtin",
    "constant.builtin.boolean",
    "constant.character",
    "constant.numeric",
    "constructor",
    "delimiter",
    "embedded",
    "error",
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
    "keyword.debug",
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
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "spell",
    "string",
    "string.documentation",
    "string.escape",
    "string.regex",
    "string.regexp",
    "string.special",
    "string.special.key",
    "string.special.path",
    "string.special.symbol",
    "string.special.url",
    "supports",
    "tag",
    "tag.attribute",
    "tag.builtin",
    "tag.delimiter",
    "tag.doctype",
    "tag.error",
    "type",
    "type.builtin",
    "type.qualifier",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
];

/// The style table of a code buffer: each capture name's style as the widget would register it, plus the search marks,
/// which layers name as `opentui.search.match` (Scouter `search.match`) and `opentui.search.current` (the match the view
/// is on). A buffer keeps the table it was given, so this one stands in for the widget's own.
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
