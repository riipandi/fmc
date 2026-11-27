use std::ffi::OsString;
use std::fs;

use fmc::reader::associations::Language;
use fmc::settings::Settings;
use tempfile::TempDir;

use serde_json::json;

use super::{load, path, save, update};

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
    assert_eq!(
        parsed.settings,
        Settings { auto_preview: false, sidebar_jump: 2, reader_jump: 8, git_gutter: true, ..Settings::default() }
    );
    assert!(parsed.notices.is_empty());
}

#[test]
fn git_gutter_is_read_from_the_config_home_the_path_names() {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_path_buf();
    let file = path(Some(home.into_os_string()), None).unwrap();
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, r#"{"git_gutter": false}"#).unwrap();
    let parsed = load(&file);
    assert!(!parsed.settings.git_gutter);
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

#[test]
fn file_associations_are_read_from_the_config_home() {
    let dir = TempDir::new().unwrap();
    let file = path(Some(dir.path().as_os_str().to_owned()), None).unwrap();
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, r#"{"file_associations": {"*.rs.bak": "rust"}}"#).unwrap();
    let parsed = load(&file);
    assert!(parsed.notices.is_empty());
    assert_eq!(parsed.settings.file_associations.language(None, "gringotts.rs.bak"), Some(Language::Filetype("rust")));
}

#[test]
fn save_creates_the_file_and_its_folder_and_leaves_no_temp_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fmc").join("settings.json");
    save(&file, "{\n  \"reader_jump\": 8\n}\n").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "{\n  \"reader_jump\": 8\n}\n");
    assert_eq!(fs::read_dir(file.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn save_writes_the_link_target_and_the_link_stays() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("dotfiles").join("hogwarts-settings.json");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "{}\n").unwrap();
    let link = dir.path().join("fmc").join("settings.json");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    save(&link, "{\"auto_preview\": false}\n").unwrap();
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_to_string(&target).unwrap(), "{\"auto_preview\": false}\n");
    assert_eq!(fs::read_dir(target.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn a_read_only_folder_keeps_the_original_and_writes_no_temp_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let folder = dir.path().join("fmc");
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join("settings.json");
    fs::write(&file, "{\"reader_jump\": 3}\n").unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o555)).unwrap();
    let result = save(&file, "{\"reader_jump\": 8}\n");
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), "{\"reader_jump\": 3}\n");
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
}

#[test]
fn an_update_creates_a_missing_file_with_only_its_changes() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fmc").join("settings.json");
    update(&file, &[("auto_preview", json!(false))]).unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "{\n  \"auto_preview\": false\n}\n");
}

#[test]
fn an_update_keeps_a_hand_edit_made_since_the_start() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, r#"{"hogwarts": "Gryffindor"}"#).unwrap();
    update(&file, &[("reader_jump", json!(8))]).unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "{\n  \"hogwarts\": \"Gryffindor\",\n  \"reader_jump\": 8\n}\n");
}

#[test]
fn an_update_refuses_a_file_that_does_not_parse_and_keeps_its_bytes() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, "{ \"auto_preview\": true,, }").unwrap();
    let reason = update(&file, &[("auto_preview", json!(false))]).unwrap_err();
    assert_eq!(reason, "settings.json is not valid JSON (line 1); nothing was saved.");
    assert_eq!(fs::read_to_string(&file).unwrap(), "{ \"auto_preview\": true,, }");
}

#[test]
fn a_relative_xdg_config_home_is_ignored_so_nothing_lands_in_the_working_directory() {
    let path = path(Some(OsString::from("hogwarts")), Some(OsString::from("/home/hermione")));
    assert_eq!(path.unwrap(), std::path::Path::new("/home/hermione/.config/fmc/settings.json"));
}

#[test]
fn an_update_refuses_a_fifo_without_blocking_on_it() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("settings.json");
    let made = std::process::Command::new("mkfifo").arg(&file).status().unwrap();
    assert!(made.success());
    let reason = update(&file, &[("auto_preview", json!(false))]).unwrap_err();
    assert_eq!(reason, "settings.json is not a regular file; nothing was saved.");
}
