use std::ffi::OsString;
use std::fs;

use fmc::reader::associations::Language;
use fmc::settings::Settings;
use tempfile::TempDir;

use serde_json::json;

use super::{ensure_schema, link_schema, load, path, schema_path, update};
use crate::services::save_atomic;

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
fn a_file_past_the_cap_is_refused_unread_with_the_defaults() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, vec![b' '; crate::services::MAX_CONFIG_BYTES as usize + 1]).unwrap();
    let parsed = load(&file);
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.notices.len(), 1, "{:?}", parsed.notices);
    assert!(parsed.notices[0].contains("larger than 1024 KiB"), "{:?}", parsed.notices);
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
    save_atomic(&file, "{\n  \"reader_jump\": 8\n}\n").unwrap();
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
    save_atomic(&link, "{\"auto_preview\": false}\n").unwrap();
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
    let result = save_atomic(&file, "{\"reader_jump\": 8}\n");
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
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "{\n  \"$schema\": \"./meta/config.schema.json\",\n  \"auto_preview\": false\n}\n"
    );
}

#[test]
fn an_update_keeps_a_hand_edit_made_since_the_start() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, r#"{"hogwarts": "Gryffindor"}"#).unwrap();
    update(&file, &[("reader_jump", json!(8))]).unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "{\n  \"$schema\": \"./meta/config.schema.json\",\n  \"hogwarts\": \"Gryffindor\",\n  \"reader_jump\": 8\n}\n"
    );
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
    assert_eq!(reason, "settings.json could not be read: it is not a regular file; nothing was saved.");
}

fn schema_file(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("fmc").join("meta").join("config.schema.json")
}

#[test]
fn the_schema_path_is_under_the_config_folder_and_a_relative_xdg_is_ignored() {
    let path = schema_path(Some(OsString::from("/tmp/hogwarts")), Some(OsString::from("/home/hermione")));
    assert_eq!(path.unwrap(), std::path::Path::new("/tmp/hogwarts/fmc/meta/config.schema.json"));
    let relative = schema_path(Some(OsString::from("relative/dir")), Some(OsString::from("/home/hermione")));
    assert_eq!(relative.unwrap(), std::path::Path::new("/home/hermione/.config/fmc/meta/config.schema.json"));
    assert!(schema_path(None, None).is_none());
}

#[test]
fn a_missing_schema_is_created_with_its_folders_and_nothing_else() {
    let dir = TempDir::new().unwrap();
    let file = schema_file(&dir);
    assert_eq!(ensure_schema(&file), None);
    assert_eq!(fs::read_to_string(&file).unwrap(), fmc::settings::schema());
    assert_eq!(fs::read_dir(file.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn a_stale_schema_is_rewritten_to_the_running_build() {
    let dir = TempDir::new().unwrap();
    let file = schema_file(&dir);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "{\"title\": \"an older build\"}\n").unwrap();
    assert_eq!(ensure_schema(&file), None);
    assert_eq!(fs::read_to_string(&file).unwrap(), fmc::settings::schema());
}

#[test]
fn an_equal_schema_is_not_rewritten() {
    use std::time::{Duration, SystemTime};
    let dir = TempDir::new().unwrap();
    let file = schema_file(&dir);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, fmc::settings::schema()).unwrap();
    let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    fs::File::options().write(true).open(&file).unwrap().set_modified(old).unwrap();
    assert_eq!(ensure_schema(&file), None);
    assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), old);
}

#[test]
fn a_read_only_folder_gives_a_notice_and_no_panic() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let file = schema_file(&dir);
    let meta = file.parent().unwrap().to_path_buf();
    fs::create_dir_all(dir.path().join("fmc")).unwrap();
    fs::create_dir(&meta).unwrap();
    fs::set_permissions(&meta, fs::Permissions::from_mode(0o555)).unwrap();
    let notice = ensure_schema(&file);
    fs::set_permissions(&meta, fs::Permissions::from_mode(0o755)).unwrap();
    let notice = notice.expect("a read-only folder is a notice");
    assert!(notice.starts_with("meta/config.schema.json could not be written: "), "{notice}");
    assert!(!file.exists());
}

#[test]
fn a_schema_link_is_written_through_and_the_link_stays() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("dotfiles").join("hogwarts-schema.json");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "{}\n").unwrap();
    let file = schema_file(&dir);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, &file).unwrap();
    assert_eq!(ensure_schema(&file), None);
    assert!(fs::symlink_metadata(&file).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_to_string(&target).unwrap(), fmc::settings::schema());
}

#[test]
fn link_schema_adds_the_key_to_an_existing_file_and_keeps_its_values() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("settings.json");
    let own = "{\n  \"sidebar_jump\": 7,\n  \"auto_preview\": false\n}\n";
    std::fs::write(&file, own).unwrap();
    let before = load(&file).settings;
    assert_eq!(link_schema(&file), None);
    let text = std::fs::read_to_string(&file).unwrap();
    assert_eq!(
        text,
        "{\n  \"$schema\": \"./meta/config.schema.json\",\n  \"sidebar_jump\": 7,\n  \"auto_preview\": false\n}\n"
    );
    assert_eq!(load(&file).settings, before);
}

#[test]
fn link_schema_leaves_a_missing_file_a_linked_file_and_a_broken_file_alone() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("settings.json");
    assert_eq!(link_schema(&file), None);
    assert!(!file.exists());
    for text in ["{\"$schema\": \"https://gringotts.test/vault.json\"}", "{\"reader_jump\": "] {
        std::fs::write(&file, text).unwrap();
        assert_eq!(link_schema(&file), None);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), text);
    }
}

#[cfg(unix)]
#[test]
fn link_schema_in_a_read_only_folder_is_a_notice_and_the_file_keeps_its_bytes() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let folder = dir.path().join("fmc");
    std::fs::create_dir(&folder).unwrap();
    let file = folder.join("settings.json");
    std::fs::write(&file, "{\"git_gutter\": false}").unwrap();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o555)).unwrap();
    let notice = link_schema(&file);
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(notice.unwrap().starts_with("settings.json could not be linked to its schema"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "{\"git_gutter\": false}");
}
