use super::*;

#[test]
fn an_empty_object_gives_the_defaults_without_a_notice() {
    let parsed = parse("{}");
    assert_eq!(parsed.settings, Settings { auto_preview: true, sidebar_jump: 2, reader_jump: 5, git_gutter: true });
    assert!(parsed.notices.is_empty());
}

#[test]
fn every_key_is_read() {
    let parsed = parse(r#"{"auto_preview": false, "sidebar_jump": 3, "reader_jump": 9, "git_gutter": false}"#);
    assert_eq!(parsed.settings, Settings { auto_preview: false, sidebar_jump: 3, reader_jump: 9, git_gutter: false });
    assert!(parsed.notices.is_empty());
}

#[test]
fn a_wrong_type_keeps_that_keys_default_and_notes_it() {
    let parsed = parse(r#"{"auto_preview": "yes", "sidebar_jump": 4, "reader_jump": 2.5, "git_gutter": 1}"#);
    assert_eq!(parsed.settings, Settings { auto_preview: true, sidebar_jump: 4, reader_jump: 5, git_gutter: true });
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
