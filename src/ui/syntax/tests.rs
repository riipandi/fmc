use std::path::Path;

use fmc::reader::associations::Associations;
use fmc::theme::DARK;
use opentui::prelude::Attributes;
use opentui::syntax::{has_grammar, highlights, prewarm};

use super::*;

#[test]
fn every_vendored_grammar_compiles_with_its_query() {
    register();
    for filetype in ["json", "css", "html", "go", "python", "bash", "yaml", "toml", "c", "java"] {
        assert!(prewarm(filetype), "{filetype} does not compile");
    }
    for filetype in ["javascript", "typescript", "typescriptreact", "markdown", "markdown_inline", "zig"] {
        assert!(prewarm(filetype), "{filetype} does not compile");
    }
    assert!(has_grammar("javascriptreact") && has_grammar("rust"));
}

#[test]
fn typescript_names_a_property_as_core_does() {
    register();
    // Core's query names an interface's property `variable.member` (plain text in Scouter); the crate's says `property`.
    let source = "export interface Vault {\n  kind: string;\n}\n";
    let captures = highlights(source, "typescript").expect("typescript highlights");
    let property = source.find("kind").unwrap();
    let name = captures.iter().find(|c| c.start == property).map(|c| c.name);
    assert_eq!(name, Some("variable.member"));
}

#[test]
fn a_path_gets_the_filetype_scouter_names_or_none() {
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("hogwarts/Potter.rs")), Some("rust"));
    assert_eq!(
        language_for(&Associations::default(), Path::new(""), Path::new("Gringotts/vault.TSX")),
        Some("typescriptreact")
    );
    assert_eq!(
        language_for(&Associations::default(), Path::new(""), Path::new("Louvre/key.jsx")),
        Some("javascriptreact")
    );
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("CERN/config.toml")), Some("toml"));
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("CERN/Makefile")), None);
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("Diagon/owls.txt")), None);
}

#[test]
fn an_association_decides_before_the_filetype_lookup() {
    let mut associations = Associations::default();
    associations.add("*.rs.bak", "rust").unwrap();
    associations.add("*.env.example", "dotenv").unwrap();
    associations.add("*.txt", "plaintext").unwrap();
    associations.add("docs/**/*.txt", "markdown").unwrap();
    let root = Path::new("/Hogwarts/Gringotts");
    let at = |path: &str| language_for(&associations, root, &root.join(path));
    assert_eq!(at("vault.rs.bak"), Some("rust"));
    assert_eq!(at("louvre.env.example"), Some("bash"));
    assert_eq!(at("docs/owls.txt"), Some("markdown"));
    assert_eq!(at("owls.txt"), None);
    assert_eq!(at("CERN/config.toml"), Some("toml"));
    assert_eq!(language_for(&associations, root, Path::new("/Hogwarts/Diagon/vault.rs.bak")), Some("rust"));
}

#[test]
fn the_style_table_reads_the_palette() {
    let theme = theme(&DARK);
    let keyword = theme.get("keyword").unwrap();
    assert_eq!(keyword.fg, Some(color(DARK.syntax.keyword)));
    assert!(keyword.attributes.contains(Attributes::BOLD) && !keyword.attributes.contains(Attributes::ITALIC));
    // A capture with no style of its own takes its first part's, as Core does: `keyword.return` is a keyword.
    assert_eq!(theme.get("keyword.return"), theme.get("keyword"));
    assert_eq!(theme.get("markup.raw.block"), None);
    assert_eq!(theme.get("type.builtin"), theme.get("constant"));
}
