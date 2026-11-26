use std::ffi::OsString;
use std::fs;

use fmc::settings::Settings;
use tempfile::TempDir;

use super::{load, path};

#[test]
fn the_path_is_under_xdg_config_home_when_it_is_set() {
    let path = path(Some(OsString::from("/tmp/hogwarts")), Some(OsString::from("/home/hermione")));
    assert_eq!(path.unwrap(), std::path::Path::new("/tmp/hogwarts/fmc/settings.json"));
}

#[test]
fn an_empty_xdg_config_home_falls_back_to_the_home_directory() {
    let path = path(Some(OsString::new()), Some(OsString::from("/home/hermione")));
    assert_eq!(path.unwrap(), std::path::Path::new("/home/hermione/.config/fmc/settings.json"));
}

#[test]
fn no_home_and_no_xdg_config_home_has_no_path() {
    assert!(path(None, None).is_none());
}

#[test]
fn a_missing_file_is_the_defaults_without_a_notice() {
    let dir = TempDir::new().unwrap();
    let parsed = load(&dir.path().join("fmc").join("settings.json"));
    assert_eq!(parsed.settings, Settings::default());
    assert!(parsed.notices.is_empty());
}

#[test]
fn a_file_under_the_config_home_is_read() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fmc").join("settings.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, r#"{"reader_jump": 8, "auto_preview": false}"#).unwrap();
    let parsed = load(&file);
    assert_eq!(parsed.settings, Settings { auto_preview: false, sidebar_jump: 2, reader_jump: 8 });
    assert!(parsed.notices.is_empty());
}

#[test]
fn a_path_that_cannot_be_read_is_the_defaults_with_a_notice() {
    let dir = TempDir::new().unwrap();
    let parsed = load(dir.path());
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 1);
    assert!(parsed.notices[0].starts_with("settings.json could not be read: "));
}
