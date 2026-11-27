use std::ffi::OsString;
use std::fs;
use std::path::Path;

use fmc::theme::catalog::{self, choose};
use tempfile::TempDir;

use super::{folder, load};

/// A valid theme file's text: a bundled theme's, which has the 24 roles, under a name nobody bundles.
const HOGWARTS_DUSK: &str = include_str!("../theme/catalog/nord.json");

fn write(dir: &Path, name: &str, text: &str) {
    fs::write(dir.join(name), text).unwrap();
}

#[test]
fn the_themes_folder_is_under_the_config_folder() {
    let xdg = folder(Some(OsString::from("/tmp/hogwarts")), Some(OsString::from("/home/hermione")));
    assert_eq!(xdg.unwrap(), Path::new("/tmp/hogwarts/fmc/themes"));
    let home = folder(Some(OsString::new()), Some(OsString::from("/home/hermione")));
    assert_eq!(home.unwrap(), Path::new("/home/hermione/.config/fmc/themes"));
    assert!(folder(None, None).is_none());
}

#[test]
fn a_missing_folder_has_no_themes_and_no_notice() {
    let dir = TempDir::new().unwrap();
    let loaded = load(&dir.path().join("themes"));
    assert!(loaded.themes.is_empty());
    assert!(loaded.notices.is_empty());
}

#[test]
fn a_json_file_is_a_theme_named_by_its_stem_and_other_files_are_ignored() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "hogwarts-dusk.json", HOGWARTS_DUSK);
    write(dir.path(), "README.md", "Dan Brown's Langdon reads here.");
    write(dir.path(), "gringotts-night", HOGWARTS_DUSK);
    fs::create_dir(dir.path().join("diagon-alley.json")).unwrap();
    let loaded = load(dir.path());
    assert_eq!(loaded.themes.len(), 1);
    assert_eq!(loaded.themes[0].id, "hogwarts-dusk");
    assert!(loaded.notices.is_empty());
}

#[test]
fn a_file_that_cannot_be_parsed_is_skipped_with_a_notice_that_names_it() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "gringotts-night.json", "{ \"mode\": ");
    write(dir.path(), "hogwarts-dusk.json", HOGWARTS_DUSK);
    let loaded = load(dir.path());
    assert_eq!(loaded.themes.len(), 1);
    assert_eq!(loaded.notices.len(), 1);
    assert!(loaded.notices[0].starts_with("themes/gringotts-night.json is skipped: it is not valid JSON"));
}

#[test]
fn a_bad_role_skips_its_file_and_the_notice_names_the_role() {
    let dir = TempDir::new().unwrap();
    let mut value: serde_json::Value = serde_json::from_str(HOGWARTS_DUSK).unwrap();
    value["fg"] = "#zz".into();
    write(dir.path(), "hogwarts-dusk.json", &value.to_string());
    let loaded = load(dir.path());
    assert!(loaded.themes.is_empty());
    assert_eq!(loaded.notices, ["themes/hogwarts-dusk.json is skipped: role \"fg\" must be #rrggbb, not \"#zz\"."]);
}

#[test]
fn a_user_file_with_a_bundled_id_replaces_that_theme() {
    let dir = TempDir::new().unwrap();
    let mut value: serde_json::Value = serde_json::from_str(HOGWARTS_DUSK).unwrap();
    value["bg"] = "#000000".into();
    let text = value.to_string();
    write(dir.path(), "nord.json", &text);
    let loaded = load(dir.path());
    let chosen = choose(loaded.themes, "nord", "default");
    assert!(chosen.notices.is_empty());
    assert_eq!(chosen.dark, catalog::parse("nord", &text).unwrap().palette);
    let bundled = catalog::bundled().into_iter().find(|theme| theme.id == "nord").unwrap();
    assert_ne!(chosen.dark, bundled.palette);
}

#[test]
fn a_file_past_the_size_cap_or_not_utf8_is_skipped_with_its_reason() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "gringotts-night.json", &" ".repeat(2 << 20));
    fs::write(dir.path().join("diagon-alley.json"), [0xff, 0xfe, 0x00]).unwrap();
    let loaded = load(dir.path());
    assert!(loaded.themes.is_empty());
    assert_eq!(
        loaded.notices,
        [
            "themes/diagon-alley.json is skipped: it is not UTF-8 text.",
            "themes/gringotts-night.json is skipped: it is larger than 1024 KiB.",
        ]
    );
}
