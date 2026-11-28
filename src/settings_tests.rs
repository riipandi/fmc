use serde_json::json;

use super::*;
use crate::find::Matching;
use crate::reader::associations::Language;
use crate::theme::catalog::{self, DEFAULT_THEME, Theme, bundled};
use crate::theme::{Mode, ModeChoice};

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
fn the_theme_keys_are_read_and_a_theme_id_is_checked_later() {
    let parsed = parse(r#"{"theme_mode": "dark", "dark_theme": "tokyo-night", "light_theme": "hogwarts-dawn"}"#);
    assert_eq!(parsed.settings.theme_mode, ModeChoice::Dark);
    assert_eq!(parsed.settings.dark_theme, "tokyo-night");
    assert_eq!(parsed.settings.light_theme, "hogwarts-dawn");
    assert!(parsed.notices.is_empty());
    assert_eq!(parse(r#"{"theme_mode": "light"}"#).settings.theme_mode, ModeChoice::Light);
}

#[test]
fn a_bad_theme_mode_or_theme_id_keeps_its_default_and_notes_it() {
    let parsed = parse(r#"{"theme_mode": "sepia", "dark_theme": 3}"#);
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 2);
    assert!(parsed.notices[0].contains(r#"theme_mode must be "auto", "dark", or "light"; "auto" is used."#));
    assert!(parsed.notices[1].contains(r#"dark_theme must be a theme id, a string; "default" is used."#));
    assert_eq!(Settings::default().theme_mode, ModeChoice::Auto);
    assert_eq!(Settings::default().dark_theme, "default");
    assert_eq!(Settings::default().light_theme, "default");
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
fn a_skipped_entry_notice_is_one_line_with_the_user_text_shown_as_text() {
    let parsed = parse(r#"{"file_associations": {"hogwarts\nvault.rs": 3}}"#);
    assert_eq!(parsed.notices.len(), 1, "{:?}", parsed.notices);
    assert!(parsed.notices[0].contains("\"hogwarts vault.rs\" is skipped"), "{:?}", parsed.notices);
    assert!(!parsed.notices[0].contains('\n'), "{:?}", parsed.notices);
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
        "{\n  \"$schema\": \"./meta/config.schema.json\",\n  \"auto_preview\": false,\n  \"hogwarts\": {\n    \"house\": \"Gryffindor\"\n  },\n  \"reader_jump\": 8,\n  \"zeta\": 1\n}\n"
    );
}

fn user_theme(id: &str, mode: Mode) -> Theme {
    let mut theme = bundled().into_iter().find(|theme| theme.mode == mode).unwrap();
    theme.id = id.into();
    theme
}

/// Steps `row` forward until it stops, and returns each value it held.
fn walk(settings: &Settings, row: Row, themes: &[Theme]) -> Vec<String> {
    let mut values = vec![settings.shown(row)];
    let mut current = settings.clone();
    while let Some(next) = current.step(row, true, themes) {
        values.push(next.shown(row));
        current = next;
    }
    values
}

#[test]
fn every_row_names_its_key_and_its_help_fits_a_line() {
    let keys: Vec<&str> = Row::ALL.iter().map(|row| row.key()).collect();
    assert_eq!(
        keys,
        [
            "auto_preview",
            "sidebar_jump",
            "reader_jump",
            "git_gutter",
            "find_mode",
            "theme_mode",
            "dark_theme",
            "light_theme"
        ]
    );
    for row in Row::ALL {
        // The reader's pane is 48 columns wide on an 80-column screen, and a help line is cut there, not wrapped.
        assert!(row.help().chars().count() <= 46, "{} has a help line too long for the pane", row.key());
    }
}

#[test]
fn a_row_value_saves_and_reads_back_as_the_same_settings() {
    let settings = Settings {
        auto_preview: false,
        sidebar_jump: 4,
        reader_jump: 9,
        git_gutter: false,
        find_mode: Matching::Fuzzy,
        theme_mode: ModeChoice::Light,
        dark_theme: "tokyo-night".into(),
        light_theme: "gruvbox-light".into(),
        ..Settings::default()
    };
    let object: Map<String, Value> = Row::ALL.iter().map(|row| (row.key().to_string(), settings.json(*row))).collect();
    let parsed = parse(&Value::Object(object).to_string());
    assert!(parsed.notices.is_empty(), "{:?}", parsed.notices);
    assert_eq!(parsed.settings, settings);
}

#[test]
fn a_toggle_flips_both_ways() {
    let on = Settings::default();
    let off = on.step(Row::AutoPreview, true, &[]).unwrap();
    assert!(!off.auto_preview);
    assert_eq!(off.step(Row::AutoPreview, false, &[]), Some(on.clone()));
    assert!(!on.step(Row::GitGutter, true, &[]).unwrap().git_gutter);
}

#[test]
fn find_mode_and_theme_mode_stop_at_their_ends() {
    let settings = Settings::default();
    assert_eq!(settings.step(Row::FindMode, false, &[]), None, "substring is the first value");
    assert_eq!(walk(&settings, Row::FindMode, &[]), ["substring", "fuzzy"]);
    assert_eq!(walk(&settings, Row::ThemeMode, &[]), ["auto", "dark", "light"]);
    let light = Settings { theme_mode: ModeChoice::Light, ..Settings::default() };
    assert_eq!(light.step(Row::ThemeMode, true, &[]), None, "light is the last value");
    assert_eq!(light.step(Row::ThemeMode, false, &[]).unwrap().theme_mode, ModeChoice::Dark);
}

#[test]
fn a_dark_slot_walks_only_dark_themes_and_the_default() {
    let themes = [user_theme("hogwarts-dusk", Mode::Dark)];
    let walked = walk(&Settings::default(), Row::DarkTheme, &themes);
    assert_eq!(walked, catalog::slot_ids(&themes, Mode::Dark));
    assert_eq!(walked.first().map(String::as_str), Some(DEFAULT_THEME));
    assert!(walked.contains(&"hogwarts-dusk".to_string()), "a user dark theme joins the slot");
    assert!(walked.contains(&"dark".to_string()) && !walked.contains(&"light".to_string()));
}

#[test]
fn a_light_slot_walks_only_light_themes_and_the_default() {
    let walked = walk(&Settings::default(), Row::LightTheme, &[]);
    assert_eq!(walked, catalog::slot_ids(&[], Mode::Light));
    assert!(walked.contains(&"light".to_string()) && !walked.contains(&"dark".to_string()));
    let last = walked.last().unwrap().clone();
    let at_end = Settings { light_theme: last, ..Settings::default() };
    assert_eq!(at_end.step(Row::LightTheme, true, &[]), None, "the last id stops the walk");
}

#[test]
fn an_id_the_list_lacks_steps_to_its_first_entry() {
    let odd = Settings { dark_theme: "hogwarts-nowhere".into(), ..Settings::default() };
    assert_eq!(odd.step(Row::DarkTheme, true, &[]).unwrap().dark_theme, DEFAULT_THEME);
    assert_eq!(odd.step(Row::DarkTheme, false, &[]).unwrap().dark_theme, DEFAULT_THEME);
}

#[test]
fn a_number_row_is_typed_and_never_stepped() {
    let settings = Settings::default();
    assert!(Row::SidebarJump.number() && Row::ReaderJump.number() && !Row::AutoPreview.number());
    assert_eq!(settings.step(Row::SidebarJump, true, &[]), None);
    assert_eq!(settings.step(Row::ReaderJump, false, &[]), None);
    let typed = settings.with_number(Row::ReaderJump, 8);
    assert_eq!(typed.reader_jump, 8);
    assert_eq!(typed.sidebar_jump, settings.sidebar_jump, "only the typed row changes");
    assert_eq!(settings.with_number(Row::AutoPreview, 8), settings);
}

#[test]
fn a_failed_change_takes_back_its_own_row_only() {
    let before = Settings::default();
    let mut now = before.step(Row::AutoPreview, true, &[]).unwrap().step(Row::GitGutter, true, &[]).unwrap();
    now.take(Row::AutoPreview, &before);
    assert!(now.auto_preview, "auto_preview is back");
    assert!(!now.git_gutter, "git_gutter keeps its change");
}

#[test]
fn shown_gives_the_word_the_file_uses() {
    let settings = Settings { theme_mode: ModeChoice::Dark, find_mode: Matching::Fuzzy, ..Settings::default() };
    assert_eq!(settings.shown(Row::ThemeMode), "dark");
    assert_eq!(settings.shown(Row::FindMode), "fuzzy");
    assert_eq!(settings.shown(Row::AutoPreview), "true");
    assert_eq!(settings.shown(Row::DarkTheme), DEFAULT_THEME);
}

#[test]
fn each_page_group_is_one_run_of_rows_and_every_row_has_a_title() {
    let sections: Vec<&str> = Row::ALL.iter().map(|row| row.section()).collect();
    let mut seen: Vec<&str> = Vec::new();
    for section in &sections {
        if seen.last() != Some(section) {
            assert!(!seen.contains(section), "{section} is split: {sections:?}");
            seen.push(section);
        }
    }
    assert_eq!(seen, ["Browsing", "Appearance"]);
    assert!(Row::ALL.iter().all(|row| !row.title().is_empty()));
}

fn schema_value() -> Value {
    serde_json::from_str(&schema()).unwrap()
}

fn strings(value: &Value) -> Vec<String> {
    value.as_array().unwrap().iter().map(|item| item.as_str().unwrap().to_string()).collect()
}

#[test]
fn the_schema_is_draft_07_strict_on_keys_and_the_same_bytes_each_call() {
    let text = schema();
    let value = schema_value();
    assert_eq!(value["$schema"], "http://json-schema.org/draft-07/schema#");
    assert_eq!(value["type"], "object");
    assert_eq!(value["additionalProperties"], false);
    assert!(text.ends_with("}\n"));
    assert_eq!(schema(), text);
}

#[test]
fn the_schema_has_a_property_for_each_key_parse_reads_and_for_dollar_schema() {
    let schema = schema_value();
    let mut names: Vec<&str> = schema["properties"].as_object().unwrap().keys().map(String::as_str).collect();
    names.sort_unstable();
    let mut expected = KEYS.to_vec();
    expected.push("$schema");
    expected.sort_unstable();
    assert_eq!(names, expected);
    for key in KEYS {
        let parsed = parse(&format!(r#"{{"{key}": []}}"#));
        assert_eq!(parsed.notices.len(), 1, "parse reads {key}");
        assert!(parsed.notices[0].contains(key), "{key}: {:?}", parsed.notices);
    }
}

#[test]
fn each_schema_default_is_the_constant_the_parser_uses() {
    let schema = schema_value();
    let properties = &schema["properties"];
    assert_eq!(properties["auto_preview"]["default"], AUTO_PREVIEW);
    assert_eq!(properties["sidebar_jump"]["default"], SIDEBAR_JUMP);
    assert_eq!(properties["reader_jump"]["default"], READER_JUMP);
    assert_eq!(properties["git_gutter"]["default"], GIT_GUTTER);
    assert_eq!(properties["find_mode"]["default"], FIND_MODE.name());
    assert_eq!(properties["theme_mode"]["default"], THEME_MODE.name());
    assert_eq!(properties["dark_theme"]["default"], DEFAULT_THEME);
    assert_eq!(properties["light_theme"]["default"], DEFAULT_THEME);
    assert_eq!(properties["file_associations"]["default"], json!({}));
}

#[test]
fn the_find_and_theme_mode_enums_are_the_names_their_parsers_accept() {
    let schema = schema_value();
    let properties = &schema["properties"];
    let find = strings(&properties["find_mode"]["enum"]);
    let mode = strings(&properties["theme_mode"]["enum"]);
    assert_eq!(find, ["substring", "fuzzy"]);
    assert_eq!(mode, ["auto", "dark", "light"]);
    for name in &find {
        assert!(Matching::named(name).is_some(), "find_mode {name}");
    }
    for name in &mode {
        assert!(ModeChoice::named(name).is_some(), "theme_mode {name}");
    }
    assert!(Matching::ALL.iter().all(|matching| find.contains(&matching.name().to_string())));
    assert!(ModeChoice::ALL.iter().all(|choice| mode.contains(&choice.name().to_string())));
}

#[test]
fn the_file_associations_value_enum_is_the_names_the_association_parser_accepts() {
    let schema = schema_value();
    let names = strings(&schema["properties"]["file_associations"]["additionalProperties"]["enum"]);
    assert_eq!(names, Language::names());
    for name in &names {
        assert!(Language::named(name).is_some(), "file_associations {name}");
    }
    assert!(names.contains(&"plaintext".to_string()) && names.contains(&"shellscript".to_string()));
    assert!(Language::named("klingon").is_none());
}

#[test]
fn each_bundled_theme_completes_in_its_own_slot_and_the_default_in_both() {
    let schema = schema_value();
    let properties = &schema["properties"];
    let dark = strings(&properties["dark_theme"]["anyOf"][0]["enum"]);
    let light = strings(&properties["light_theme"]["anyOf"][0]["enum"]);
    assert!(dark.contains(&DEFAULT_THEME.to_string()) && light.contains(&DEFAULT_THEME.to_string()));
    for theme in bundled() {
        let slot = match theme.mode {
            Mode::Dark => &dark,
            Mode::Light => &light,
        };
        assert!(slot.contains(&theme.id), "{} completes in its slot", theme.id);
    }
}

#[test]
fn a_save_into_an_empty_file_writes_the_schema_link_first_and_keeps_the_owners_own() {
    assert_eq!(
        merged("{}", &[("reader_jump", json!(8))]).unwrap(),
        "{\n  \"$schema\": \"./meta/config.schema.json\",\n  \"reader_jump\": 8\n}\n"
    );
    let own = r#"{"$schema": "../hogwarts/settings.schema.json", "reader_jump": 3}"#;
    let kept = merged(own, &[("reader_jump", json!(8))]).unwrap();
    assert!(kept.contains(r#""$schema": "../hogwarts/settings.schema.json""#), "{kept}");
}

#[test]
fn parse_ignores_the_schema_key_without_a_notice() {
    let parsed = parse(r#"{"$schema": "./meta/config.schema.json", "reader_jump": 8}"#);
    assert_eq!(parsed.settings.reader_jump, 8);
    assert!(parsed.notices.is_empty());
}

#[test]
fn linked_puts_the_schema_first_and_keeps_every_other_byte() {
    let own = "{\n    \"theme_mode\": \"dark\",\n    \"dark_theme\": \"langdon-tweed\",\n    \"reader_jump\": 9\n}\n";
    assert_eq!(
        linked(own).unwrap(),
        "{\n    \"$schema\": \"./meta/config.schema.json\",\n    \"theme_mode\": \"dark\",\n    \"dark_theme\": \"langdon-tweed\",\n    \"reader_jump\": 9\n}\n"
    );
    assert_eq!(
        linked(r#"{"git_gutter":false}"#).unwrap(),
        r#"{"$schema": "./meta/config.schema.json","git_gutter":false}"#
    );
    let linked_text = linked(own).unwrap();
    assert_eq!(parse(&linked_text).settings, parse(own).settings);
}

#[test]
fn linked_gives_an_empty_object_one_key() {
    assert_eq!(linked("{}\n").unwrap(), "{\n  \"$schema\": \"./meta/config.schema.json\"\n}\n");
    assert_eq!(linked("  { }").unwrap(), "  {\n  \"$schema\": \"./meta/config.schema.json\"\n}");
}

#[test]
fn linked_leaves_a_file_with_its_own_schema_or_that_is_not_an_object() {
    assert_eq!(linked(r#"{"$schema": "https://hogwarts.test/fmc.json"}"#), None);
    assert_eq!(linked("{not json"), None);
    assert_eq!(linked("[true]"), None);
    assert_eq!(linked(""), None);
}

#[test]
#[should_panic(expected = "missing from settings::KEYS")]
fn a_key_read_outside_keys_fails_the_tests() {
    field(&Map::new(), "invisibility_cloak");
}
