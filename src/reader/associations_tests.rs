use std::path::Path;

use super::*;

fn table(entries: &[(&str, &str)]) -> Associations {
    let mut associations = Associations::default();
    for (pattern, value) in entries {
        associations.add(pattern, value).expect("a valid entry");
    }
    associations
}

#[test]
fn a_value_names_a_filetype_an_alias_or_plain_text() {
    assert_eq!(Language::named("rust"), Some(Language::Filetype("rust")));
    assert_eq!(Language::named("typescriptreact"), Some(Language::Filetype("typescriptreact")));
    assert_eq!(Language::named("dotenv"), Some(Language::Filetype("bash")));
    assert_eq!(Language::named("env"), Some(Language::Filetype("bash")));
    assert_eq!(Language::named("shellscript"), Some(Language::Filetype("bash")));
    assert_eq!(Language::named("jsonc"), Some(Language::Filetype("json")));
    assert_eq!(Language::named("plaintext"), Some(Language::Plain));
    assert_eq!(Language::named("make"), None);
    assert_eq!(Language::named("Rust"), None);
}

#[test]
fn an_unknown_value_or_a_bad_pattern_is_refused_and_leaves_the_table() {
    let mut associations = Associations::default();
    assert_eq!(associations.add("*.log", "make").unwrap_err(), "make is not a filetype fmc shows");
    assert!(associations.add("docs/{a", "rust").unwrap_err().contains("no matching"));
    assert_eq!(associations.language(None, "gringotts.log"), None);
}

#[test]
fn a_name_pattern_sets_the_language_of_a_file_with_that_name() {
    let associations = table(&[("*.rs.bak", "rust"), (".env.*", "dotenv"), ("*.env.example", "dotenv")]);
    assert_eq!(associations.language(None, "gringotts.rs.bak"), Some(Language::Filetype("rust")));
    assert_eq!(associations.language(None, ".env.hogwarts"), Some(Language::Filetype("bash")));
    assert_eq!(associations.language(None, "louvre.env.example"), Some(Language::Filetype("bash")));
    assert_eq!(associations.language(None, "louvre.txt"), None);
}

#[test]
fn a_path_pattern_matches_the_path_under_the_opened_folder() {
    let associations = table(&[("docs/**/*.txt", "markdown")]);
    assert_eq!(associations.language(Some("docs/hogwarts.txt"), "hogwarts.txt"), Some(Language::Filetype("markdown")));
    assert_eq!(
        associations.language(Some("docs/a/b/ravenclaw.txt"), "ravenclaw.txt"),
        Some(Language::Filetype("markdown"))
    );
    assert_eq!(associations.language(Some("notes/hogwarts.txt"), "hogwarts.txt"), None);
    assert_eq!(associations.language(None, "hogwarts.txt"), None);
}

#[test]
fn a_path_pattern_beats_a_name_pattern() {
    let associations = table(&[("*.txt", "plaintext"), ("docs/*.txt", "markdown")]);
    assert_eq!(associations.language(Some("docs/hogwarts.txt"), "hogwarts.txt"), Some(Language::Filetype("markdown")));
    assert_eq!(associations.language(Some("hogwarts.txt"), "hogwarts.txt"), Some(Language::Plain));
}

#[test]
fn the_longer_pattern_wins_among_name_patterns() {
    let associations = table(&[("*.bak", "json"), ("*.rs.bak", "rust")]);
    assert_eq!(associations.language(None, "gringotts.rs.bak"), Some(Language::Filetype("rust")));
    assert_eq!(associations.language(None, "gringotts.toml.bak"), Some(Language::Filetype("json")));
}

#[test]
fn equal_patterns_resolve_by_their_order_not_by_insertion() {
    let forward = table(&[("*.a", "json"), ("?.a", "css")]);
    let backward = table(&[("?.a", "css"), ("*.a", "json")]);
    assert_eq!(forward.language(None, "x.a"), Some(Language::Filetype("json")));
    assert_eq!(backward.language(None, "x.a"), Some(Language::Filetype("json")));
}

#[test]
fn a_file_outside_the_opened_folder_matches_by_name_only() {
    let associations = table(&[("docs/*.txt", "markdown"), ("*.rs.bak", "rust")]);
    let outside = Path::new("/Hogwarts/Diagon/Alley/gringotts.rs.bak");
    assert_eq!(associations.language_of(outside, Path::new("/Hogwarts/Gringotts")), Some(Language::Filetype("rust")));
    let note = Path::new("/Hogwarts/Diagon/docs/owls.txt");
    assert_eq!(associations.language_of(note, Path::new("/Hogwarts/Diagon")), Some(Language::Filetype("markdown")));
    assert_eq!(associations.language_of(note, Path::new("/Hogwarts/Gringotts")), None);
}
