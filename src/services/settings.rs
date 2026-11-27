//! Reads `settings.json` once, before the first frame (`architecture.md`, "a read-only settings.json with its keys").
//! The UI thread never reads the file again.

use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use fmc::settings::{self, Parsed};

/// `$XDG_CONFIG_HOME/fmc/settings.json`, or `~/.config/fmc/settings.json` when `XDG_CONFIG_HOME` is unset or empty.
pub fn path(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = match config_home.filter(|dir| !dir.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(home?).join(".config"),
    };
    Some(base.join("fmc").join("settings.json"))
}

/// The settings the file sets, with a notice for each value it cannot use. A missing file has no notice.
pub fn load(path: &Path) -> Parsed {
    match fs::read_to_string(path) {
        Ok(text) => settings::parse(&text),
        Err(error) if error.kind() == ErrorKind::NotFound => Parsed::default(),
        Err(error) => {
            Parsed { notices: vec![format!("settings.json could not be read: {error}.")], ..Parsed::default() }
        }
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
