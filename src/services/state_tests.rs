use std::fs;
use std::os::unix::fs::PermissionsExt;

use fmc::find::Matching;
use tempfile::TempDir;

use super::*;

/// A config folder in a temporary directory, with `meta/state.json` at the returned path.
fn config() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("fmc").join("meta").join("state.json");
    (dir, file)
}

#[test]
fn the_path_is_meta_state_json_in_the_config_folder() {
    let home = Some(OsString::from("/home/hermione"));
    assert_eq!(path(None, home).unwrap(), PathBuf::from("/home/hermione/.config/fmc/meta/state.json"));
    assert_eq!(path(Some(OsString::from("/srv/xdg")), None).unwrap(), PathBuf::from("/srv/xdg/fmc/meta/state.json"));
}

#[test]
fn a_missing_file_loads_as_unset() {
    let (_dir, file) = config();
    assert_eq!(load(&file), State::default());
}

#[test]
fn a_file_that_does_not_parse_loads_as_unset() {
    let (_dir, file) = config();
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "{ wrap: true").unwrap();
    assert_eq!(load(&file), State::default());
}

#[test]
fn a_saved_state_loads_back() {
    let (_dir, file) = config();
    let state =
        State { wrap: Some(true), sidebar_width: Some(36), find_mode: Some(Matching::Fuzzy), ..State::default() };
    save(&file, &state).unwrap();
    assert_eq!(load(&file), state);
    let next = State { wrap: Some(false), ..state };
    save(&file, &next).unwrap();
    assert_eq!(load(&file), next);
}

#[test]
fn a_folder_that_cannot_be_written_is_an_error_for_the_footer() {
    let (dir, file) = config();
    let folder = dir.path().join("fmc");
    fs::create_dir_all(&folder).unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o555)).unwrap();
    let error = save(&file, &State { wrap: Some(true), ..State::default() }).unwrap_err();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(error.starts_with("meta/state.json could not be written: "), "{error}");
    assert!(!file.exists());
}
