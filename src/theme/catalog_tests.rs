use super::*;

const DUSK: &str = r##"{
  "name": "Hogwarts Dusk",
  "mode": "dark",
  "bg": "#101418", "bg2": "#0c0f12", "bg3": "#1c232b", "bd": "#2b3440",
  "fg": "#dfe6ee", "fg2": "#9aa6b4", "fg3": "#6f7b89", "ac": "#e0b25a", "ins": "#8fd19e",
  "vis": "#7c9cd6", "cmd": "#d6a0d6", "kw": "#c792ea", "fn": "#82aaff", "str": "#a7c7a1",
  "num": "#f2a65a", "ty": "#5ec4c4", "mac": "#f07a8a", "com": "#7f8a99", "pun": "#b2b9c3",
  "prop": "#e5c07b", "warn": "#e5b880", "err": "#f2a1a1", "sel": "#2c365e", "on": "#101418"
}"##;

fn without(key: &str) -> String {
    let mut value: serde_json::Value = serde_json::from_str(DUSK).unwrap();
    value.as_object_mut().unwrap().remove(key);
    value.to_string()
}

fn with(key: &str, value: &str) -> String {
    let mut object: serde_json::Value = serde_json::from_str(DUSK).unwrap();
    object[key] = serde_json::Value::String(value.into());
    object.to_string()
}

#[test]
fn the_catalog_holds_sixteen_bundled_themes_nine_dark_and_seven_light() {
    let themes = bundled();
    let ids: Vec<&str> = themes.iter().map(|theme| theme.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "dark",
            "catppuccin",
            "tokyo-night",
            "gruvbox",
            "kanagawa",
            "rose-pine",
            "one-dark",
            "everforest",
            "nord",
            "light",
            "catppuccin-latte",
            "tokyo-night-day",
            "gruvbox-light",
            "rose-pine-dawn",
            "one-light",
            "flexoki-light",
        ]
    );
    assert_eq!(themes.iter().filter(|theme| theme.mode == Mode::Dark).count(), 9);
    assert_eq!(themes.iter().filter(|theme| theme.mode == Mode::Light).count(), 7);
    assert_eq!(themes[1].name, "Catppuccin Mocha");
}

#[test]
fn each_source_role_lands_in_its_fmc_token() {
    // The source value of each mapped role, read from the file itself and compared with the palette.
    for (id, text) in BUNDLED {
        let source: serde_json::Value = serde_json::from_str(text).unwrap();
        let role = |key: &str| source[key].as_str().unwrap().to_ascii_lowercase();
        let theme = parse(id, text).unwrap();
        let p = &theme.palette;
        let expected = [
            (&*p.fallback_background, role("bg")),
            (&*p.match_fg, role("bg")),
            (&*p.row_active_bg, role("bg3")),
            (&*p.hover, role("bg3")),
            (&*p.border, role("bd")),
            (&*p.text, role("fg")),
            (&*p.muted, role("fg2")),
            (&*p.dim, role("fg3")),
            (&*p.accent, role("ac")),
            (&*p.accent_ink, role("on")),
            (&*p.selection_bg, role("sel")),
            (&*p.error, role("err")),
            (&*p.removed, role("err")),
            (&*p.modified, role("warn")),
            (&*p.match_bg, role("warn")),
            (&*p.added, role("ins")),
            (&*p.syntax.keyword, role("kw")),
            (&*p.syntax.function, role("fn")),
            (&*p.syntax.string, role("str")),
            (&*p.syntax.constant, role("num")),
            (&*p.syntax.comment, role("com")),
            (&*p.syntax.punctuation, role("pun")),
        ];
        for (token, source_value) in expected {
            assert_eq!(token, source_value, "{id}");
        }
    }
}

#[test]
fn the_derived_values_follow_the_mix_formulas() {
    // Checked by hand for `dark`: accent_soft = mix(ac, fg, 0.35), and the others by the same rule.
    let dark = bundled().remove(0).palette;
    assert_eq!(&*dark.accent_soft, "#91b3f6");
    assert_eq!(&*dark.folder, "#b6c8ec");
    assert_eq!(&*dark.ignored, "#505663");
    assert_eq!(&*dark.added_bg, "#252d23");
    assert_eq!(&*dark.wordmark[2], "#5177bf");
}

#[test]
fn a_file_parses_with_its_mode_name_and_palette() {
    let theme = parse("hogwarts-dusk", DUSK).unwrap();
    assert_eq!(theme.id, "hogwarts-dusk");
    assert_eq!(theme.name, "Hogwarts Dusk");
    assert_eq!(theme.mode, Mode::Dark);
    assert_eq!(&*theme.palette.text, "#dfe6ee");
    assert_eq!(&*theme.palette.accent, "#e0b25a");
}

#[test]
fn the_name_defaults_to_the_id_and_unknown_keys_are_ignored() {
    let text = without("name").replace("\"mode\": \"dark\",", "\"mode\": \"dark\", \"author\": \"Sirius Black\",");
    let theme = parse("gringotts-night", &text).unwrap();
    assert_eq!(theme.name, "gringotts-night");
}

#[test]
fn each_failure_names_its_reason() {
    let reason = |text: &str| parse("hogwarts-dusk", text).unwrap_err();
    assert_eq!(reason("[]"), "it must hold a JSON object");
    assert!(reason("{").starts_with("it is not valid JSON (line 1)"));
    assert_eq!(reason(&without("mode")), "mode must be \"dark\" or \"light\"");
    assert_eq!(reason(&with("mode", "sepia")), "mode must be \"dark\" or \"light\"");
    assert_eq!(reason(&without("fg")), "role \"fg\" is missing");
    assert_eq!(reason(&without("kw")), "role \"kw\" is missing");
    assert_eq!(reason(&without("prop")), "role \"prop\" is missing");
    assert_eq!(reason(&with("fg", "#dfe6e")), "role \"fg\" must be #rrggbb, not \"#dfe6e\"");
    assert_eq!(reason(&with("ac", "gold")), "role \"ac\" must be #rrggbb, not \"gold\"");
    assert_eq!(reason(&with("ins", "#gggggg")), "role \"ins\" must be #rrggbb, not \"#gggggg\"");
    assert_eq!(reason(&with("err", "d00")), "role \"err\" must be #rrggbb, not \"d00\"");
    let number = DUSK.replace("\"err\": \"#f2a1a1\"", "\"err\": 16711680");
    assert_eq!(reason(&number), "role \"err\" must be #rrggbb");
    let name = DUSK.replace("\"name\": \"Hogwarts Dusk\"", "\"name\": 7");
    assert_eq!(reason(&name), "name must be a string");
}

#[test]
fn a_role_may_be_upper_case_and_is_kept_lower_case() {
    let theme = parse("hogwarts-dusk", &with("fg", "#DFE6EE")).unwrap();
    assert_eq!(&*theme.palette.text, "#dfe6ee");
}

#[test]
fn the_default_pair_is_not_a_file_id() {
    assert!(parse(DEFAULT_THEME, DUSK).unwrap_err().contains("built-in pair"));
}

#[test]
fn a_dark_id_in_the_dark_slot_is_used_and_the_default_is_the_default_theme() {
    let chosen = choose(Vec::new(), DEFAULT_THEME, DEFAULT_THEME);
    assert_eq!(chosen.dark, DARK);
    assert_eq!(chosen.light, LIGHT);
    assert!(chosen.notices.is_empty());
    let chosen = choose(Vec::new(), "tokyo-night", "gruvbox-light");
    assert_eq!(&*chosen.dark.accent, "#7aa2f7");
    assert_eq!(chosen.light, bundled()[12].palette, "gruvbox-light");
    assert!(chosen.notices.is_empty());
}

#[test]
fn a_wrong_slot_or_an_unknown_id_keeps_the_default_and_names_the_key() {
    let chosen = choose(Vec::new(), "gruvbox-light", "tokyo-night");
    assert_eq!(chosen.dark, DARK);
    assert_eq!(chosen.light, LIGHT);
    assert_eq!(
        chosen.notices,
        [
            "settings.json: dark_theme \"gruvbox-light\" is a light theme; the default theme is used.",
            "settings.json: light_theme \"tokyo-night\" is a dark theme; the default theme is used.",
        ]
    );
    let chosen = choose(Vec::new(), "hogwarts-dusk", DEFAULT_THEME);
    assert_eq!(chosen.dark, DARK);
    assert_eq!(
        chosen.notices,
        ["settings.json: dark_theme \"hogwarts-dusk\" is not a theme; the default theme is used."]
    );
}

#[test]
fn a_user_file_is_a_theme_and_replaces_a_bundled_id() {
    let user = parse("hogwarts-dusk", DUSK).unwrap();
    let chosen = choose(vec![user], "hogwarts-dusk", DEFAULT_THEME);
    assert_eq!(&*chosen.dark.accent, "#e0b25a");
    assert!(chosen.notices.is_empty());
    let replacement = parse("tokyo-night", DUSK).unwrap();
    let chosen = choose(vec![replacement], "tokyo-night", DEFAULT_THEME);
    assert_eq!(&*chosen.dark.accent, "#e0b25a", "the user file wins over the bundled tokyo-night");
}

#[test]
fn a_user_file_in_the_wrong_slot_is_refused_with_a_notice() {
    let light = parse("gringotts-day", &with("mode", "light")).unwrap();
    let chosen = choose(vec![light.clone()], "gringotts-day", DEFAULT_THEME);
    assert_eq!(chosen.dark, DARK);
    assert_eq!(chosen.light, LIGHT);
    assert_eq!(
        chosen.notices,
        ["settings.json: dark_theme \"gringotts-day\" is a light theme; the default theme is used."]
    );
    let chosen = choose(vec![light.clone()], DEFAULT_THEME, "gringotts-day");
    assert_eq!(chosen.light, light.palette);
    assert!(chosen.notices.is_empty());
}

#[test]
fn an_id_from_settings_is_shown_without_its_control_characters() {
    let chosen = choose(Vec::new(), "tokyo\u{1b}[2J", DEFAULT_THEME);
    assert_eq!(
        chosen.notices,
        ["settings.json: dark_theme \"tokyo\u{fffd}[2J\" is not a theme; the default theme is used."]
    );
}
