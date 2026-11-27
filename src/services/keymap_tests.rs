//! The keymap file's read at start (plan-00015 phase 4): the path rule, a missing file, a file past the cap, and text that
//! is not UTF-8. Every file is in a temporary folder (A12); no test reads the owner's `~/.config/fmc`.

use std::ffi::OsString;
use std::fs;

use fmc::keys::Action;
use tempfile::TempDir;

use super::*;

fn folder() -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("fmc").join("keymap.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    (dir, file)
}

#[test]
fn the_path_is_the_config_folder_keymap_json_and_a_relative_xdg_is_ignored() {
    let home = Some(OsString::from("/Users/hermione"));
    assert_eq!(path(Some("/tmp/hogwarts".into()), home.clone()), Some("/tmp/hogwarts/fmc/keymap.json".into()));
    assert_eq!(path(Some("relative".into()), home.clone()), Some("/Users/hermione/.config/fmc/keymap.json".into()));
    assert_eq!(path(None, home), Some("/Users/hermione/.config/fmc/keymap.json".into()));
    assert_eq!(path(None, None), None);
}

#[test]
fn a_missing_file_is_the_defaults_with_no_notice() {
    let (_dir, file) = folder();
    let parsed = load(&file);
    assert!(parsed.notices.is_empty());
    assert_eq!(parsed.keymap.bindings(), fmc::keys::Keymap::defaults().bindings());
}

#[test]
fn a_file_is_read_and_merged_over_the_defaults() {
    let (_dir, file) = folder();
    fs::write(&file, r#"{"reader.x": "toggle_wrap"}"#).unwrap();
    let parsed = load(&file);
    assert!(parsed.notices.is_empty(), "{:?}", parsed.notices);
    assert!(parsed.keymap.bindings().iter().any(|b| b.user && b.action == Action::ToggleWrap));
}

#[test]
fn a_file_past_the_cap_is_refused_unread_with_a_notice() {
    let (_dir, file) = folder();
    fs::write(&file, vec![b' '; MAX_CONFIG_BYTES as usize + 1]).unwrap();
    let parsed = load(&file);
    assert_eq!(parsed.notices.len(), 1, "{:?}", parsed.notices);
    assert!(parsed.notices[0].contains("larger than 1024 KiB"), "{:?}", parsed.notices);
    assert_eq!(parsed.keymap.bindings(), fmc::keys::Keymap::defaults().bindings());
}

#[test]
fn text_that_is_not_utf8_keeps_every_default() {
    let (_dir, file) = folder();
    fs::write(&file, [b'{', 0xff, b'}']).unwrap();
    let parsed = load(&file);
    assert_eq!(parsed.notices.len(), 1, "{:?}", parsed.notices);
    assert!(parsed.notices[0].contains("not UTF-8"));
    assert_eq!(parsed.keymap.bindings(), fmc::keys::Keymap::defaults().bindings());
}
