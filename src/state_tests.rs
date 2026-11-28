use super::*;

#[test]
fn every_key_is_read_under_its_name() {
    let text = r#"{"sidebar_width": 34, "wrap": true, "diff_split": true, "diff_wide": true,
        "diff_all_files": true, "dotfiles": false, "find_mode": "fuzzy"}"#;
    assert_eq!(
        parse(text),
        State {
            sidebar_width: Some(34),
            wrap: Some(true),
            diff_split: Some(true),
            diff_wide: Some(true),
            diff_all_files: Some(true),
            dotfiles: Some(false),
            find_mode: Some(Matching::Fuzzy),
        }
    );
    assert_eq!(parse(r#"{"find_mode": "substring"}"#).find_mode, Some(Matching::Substring));
}

#[test]
fn a_bad_value_is_unset_for_its_key_alone() {
    let text = r#"{"sidebar_width": "34", "wrap": 1, "diff_split": "yes", "diff_wide": null,
        "diff_all_files": [], "dotfiles": "false", "find_mode": "Fuzzy"}"#;
    assert_eq!(parse(text), State::default());
    assert_eq!(parse(r#"{"sidebar_width": 30.5, "wrap": true}"#).sidebar_width, None);
    assert_eq!(parse(r#"{"sidebar_width": 30.5, "wrap": true}"#).wrap, Some(true));
}

#[test]
fn a_text_that_is_not_an_object_sets_nothing() {
    for text in ["", "not json", "[true]", "true", "34", r#""fuzzy""#] {
        assert_eq!(parse(text), State::default(), "{text:?}");
    }
}

#[test]
fn unknown_keys_are_dropped_on_the_next_write() {
    let state = parse(r#"{"theme": "tokyo-night", "wrap": true}"#);
    assert_eq!(state, State { wrap: Some(true), ..State::default() });
    assert_eq!(json(&state), "{\n  \"wrap\": true\n}\n");
}

#[test]
fn an_unset_state_writes_an_empty_object() {
    assert_eq!(json(&State::default()), "{}\n");
}

#[test]
fn the_written_text_reads_back_to_the_same_state() {
    let state = State {
        sidebar_width: Some(-3),
        wrap: Some(false),
        diff_split: Some(true),
        diff_wide: None,
        diff_all_files: Some(true),
        dotfiles: Some(true),
        find_mode: Some(Matching::Fuzzy),
    };
    assert_eq!(parse(&json(&state)), state);
    assert_eq!(parse(&json(&State::default())), State::default());
}

#[test]
fn the_keys_are_the_ones_parse_reads() {
    let state = State {
        sidebar_width: Some(1),
        wrap: Some(true),
        diff_split: Some(true),
        diff_wide: Some(true),
        diff_all_files: Some(true),
        dotfiles: Some(true),
        find_mode: Some(Matching::Fuzzy),
    };
    let written: Value = serde_json::from_str(&json(&state)).unwrap();
    let mut keys: Vec<&str> = written.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    let mut listed = KEYS.to_vec();
    listed.sort_unstable();
    assert_eq!(keys, listed);
}
