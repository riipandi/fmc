//! `meta/state.json` in the config folder: the UI state fmc remembers across runs (`state.rs`). The app reads it once
//! before the first frame and writes it through `save_atomic`; a write runs on a worker, never on the UI thread.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use fmc::state::{self, State};
use fmc::text::display_label;

/// `meta/state.json` in the config folder (`services::config_folder`).
pub fn path(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    Some(super::config_folder(config_home, home)?.join("meta").join("state.json"))
}

/// The remembered state at the start: the file's path, and the values it sets.
#[derive(Default)]
pub struct Loaded {
    pub path: Option<PathBuf>,
    pub state: State,
}

/// Reads the file in the config folder once, before the first frame.
pub fn read(config_home: Option<OsString>, home: Option<OsString>) -> Loaded {
    let path = path(config_home, home);
    let state = path.as_deref().map(load).unwrap_or_default();
    Loaded { path, state }
}

/// The state the file sets. A missing file, a file that cannot be read, and a file that does not parse all read as
/// unset, with no notice: the next save replaces the file.
pub fn load(path: &Path) -> State {
    super::read_config(path).map(|text| state::parse(&text)).unwrap_or_default()
}

/// Writes `state` to `path` through `save_atomic`. The error is the footer's notice.
pub fn save(path: &Path, state: &State) -> Result<(), String> {
    super::save_atomic(path, &state::json(state))
        .map_err(|error| format!("meta/state.json could not be written: {}.", display_label(&error.to_string())))
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
