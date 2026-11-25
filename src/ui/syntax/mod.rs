//! Syntax highlighting (Scouter `src/reader/grammars.ts`, `src/reader/code-view.ts`): the grammars Scouter vendors,
//! the filetype a path gets, and the style of each capture name from the palette (`research/scouter-inventory.md`
//! §8.3). Rust and the JavaScript family come bundled with `opentui`.

use std::path::Path;

use fmc::reader::language::is_highlighted;
use fmc::theme::Palette;
use opentui::prelude::Style;
use opentui::syntax::tree_sitter::Language;
use opentui::syntax::{CompiledGrammar, Grammar, SyntaxTheme, filetype_for_path, register_alias, register_lazy};

use super::styled::color;

/// Registers the vendored grammars, compiled on first use as OpenTUI's bundled ones are. Call it once, before the
/// first code view is shown.
pub fn register() {
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

/// The filetype a path is highlighted as, if Scouter highlights it (`languageFor`).
pub fn language_for(path: &Path) -> Option<&'static str> {
    let filetype = filetype_for_path(&path.to_string_lossy())?;
    is_highlighted(filetype).then_some(filetype)
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
