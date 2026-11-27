use serde_json::json;

use super::*;
use crate::find::Matching;
use crate::reader::associations::Language;

#[test]
fn find_mode_names_the_start_mode_and_a_bad_value_keeps_substring() {
    assert_eq!(parse("{}").settings.find_mode, Matching::Substring);
    assert_eq!(parse(r#"{"find_mode": "fuzzy"}"#).settings.find_mode, Matching::Fuzzy);
    assert_eq!(parse(r#"{"find_mode": "substring"}"#).settings.find_mode, Matching::Substring);
    let parsed = parse(r#"{"find_mode": "Fuzzy"}"#);
    assert_eq!(parsed.settings.find_mode, Matching::Substring);
    assert_eq!(parsed.notices.len(), 1);
    assert!(parsed.notices[0].contains(r#"find_mode must be "substring" or "fuzzy"; "substring" is used."#));
    assert_eq!(parse(r#"{"find_mode": 1}"#).notices.len(), 1);
}

#[test]
fn an_empty_object_gives_the_defaults_without_a_notice() {
    let parsed = parse("{}");
    assert_eq!(
        parsed.settings,
        Settings { auto_preview: true, sidebar_jump: 2, reader_jump: 5, git_gutter: true, ..Settings::default() }
    );
    assert!(parsed.notices.is_empty());
}

#[test]
fn every_key_is_read() {
    let parsed = parse(r#"{"auto_preview": false, "sidebar_jump": 3, "reader_jump": 9, "git_gutter": false}"#);
    assert_eq!(
        parsed.settings,
        Settings { auto_preview: false, sidebar_jump: 3, reader_jump: 9, git_gutter: false, ..Settings::default() }
    );
    assert!(parsed.notices.is_empty());
}

#[test]
fn a_wrong_type_keeps_that_keys_default_and_notes_it() {
    let parsed = parse(r#"{"auto_preview": "yes", "sidebar_jump": 4, "reader_jump": 2.5, "git_gutter": 1}"#);
    assert_eq!(
        parsed.settings,
        Settings { auto_preview: true, sidebar_jump: 4, reader_jump: 5, git_gutter: true, ..Settings::default() }
    );
    assert_eq!(parsed.notices.len(), 3);
    assert!(parsed.notices[0].contains("auto_preview must be true or false"));
    assert!(parsed.notices[1].contains("reader_jump must be a positive whole number"));
    assert!(parsed.notices[2].contains("git_gutter must be true or false"));
}

#[test]
fn zero_and_a_negative_number_are_not_a_jump() {
    let parsed = parse(r#"{"sidebar_jump": 0, "reader_jump": -3}"#);
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 2);
}

#[test]
fn a_number_past_u32_is_not_a_jump() {
    let parsed = parse(r#"{"reader_jump": 5000000000}"#);
    assert_eq!(parsed.settings.reader_jump, 5);
    assert_eq!(parsed.notices.len(), 1);
}

#[test]
fn invalid_json_sets_nothing_and_says_where_it_fails() {
    let parsed = parse("{\n  \"sidebar_jump\": 3,\n");
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 1);
    assert!(parsed.notices[0].starts_with("settings.json is not valid JSON (line "));
}

#[test]
fn a_top_level_array_sets_nothing() {
    let parsed = parse(r#"[{"sidebar_jump": 3}]"#);
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 1);
}

#[test]
fn unknown_keys_are_ignored() {
    let parsed = parse(r#"{"theme": "dark", "reader_jump": 7}"#);
    assert_eq!(parsed.settings.reader_jump, 7);
    assert!(parsed.notices.is_empty());
}

#[test]
fn every_bad_value_is_noted_not_only_the_first() {
    let parsed = parse(r#"{"auto_preview": 1, "sidebar_jump": "3", "reader_jump": null}"#);
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 3);
}

#[test]
fn the_git_gutter_is_on_unless_the_file_turns_it_off() {
    assert!(parse("{}").settings.git_gutter);
    assert!(!parse(r#"{"git_gutter": false}"#).settings.git_gutter);
}

#[test]
fn file_associations_map_patterns_to_languages() {
    let parsed =
        parse(r#"{"file_associations": {"*.rs.bak": "rust", "*.env.example": "dotenv", "docs/**/*.txt": "markdown"}}"#);
    assert!(parsed.notices.is_empty());
    let associations = parsed.settings.file_associations;
    assert_eq!(associations.language(None, "gringotts.rs.bak"), Some(Language::Filetype("rust")));
    assert_eq!(associations.language(None, "louvre.env.example"), Some(Language::Filetype("bash")));
    assert_eq!(associations.language(Some("docs/owls.txt"), "owls.txt"), Some(Language::Filetype("markdown")));
}

#[test]
fn a_bad_entry_is_skipped_with_a_notice_that_names_it() {
    let parsed = parse(r#"{"file_associations": {"*.log": "make", "docs/{a": "rust", "*.rs": 3, "*.bak": "json"}}"#);
    assert_eq!(parsed.notices.len(), 3);
    assert!(parsed.notices.iter().any(|n| n.contains("\"*.log\" is skipped: make is not a filetype fmc shows")));
    assert!(parsed.notices.iter().any(|n| n.contains("\"docs/{a\" is skipped: a `{` has no matching `}`")));
    assert!(parsed.notices.iter().any(|n| n.contains("\"*.rs\" is skipped: its value must be a filetype name")));
    assert_eq!(parsed.settings.file_associations.language(None, "vault.bak"), Some(Language::Filetype("json")));
}

#[test]
fn a_file_associations_value_that_is_not_an_object_sets_nothing() {
    let parsed = parse(r#"{"file_associations": ["*.rs"]}"#);
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 1);
    assert!(parsed.notices[0].contains("file_associations must be an object"));
}

#[test]
fn a_save_refuses_text_that_is_not_an_object() {
    let change = [("auto_preview", json!(false))];
    assert!(merged("{not json", &change).unwrap_err().starts_with("settings.json is not valid JSON (line 1)"));
    assert!(merged("", &change).unwrap_err().contains("not valid JSON"));
    assert!(merged("[true]", &change).unwrap_err().starts_with("settings.json must hold a JSON object"));
}

#[test]
fn a_save_sets_its_keys_and_keeps_the_others_sorted_with_a_final_newline() {
    let text = r#"{"zeta": 1, "reader_jump": 3, "hogwarts": {"house": "Gryffindor"}}"#;
    let changes = [("auto_preview", json!(false)), ("reader_jump", json!(8))];
    assert_eq!(
        merged(text, &changes).unwrap(),
        "{\n  \"auto_preview\": false,\n  \"hogwarts\": {\n    \"house\": \"Gryffindor\"\n  },\n  \"reader_jump\": 8,\n  \"zeta\": 1\n}\n"
    );
}
