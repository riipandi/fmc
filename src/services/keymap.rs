//! Reads `keymap.json` once, before the first frame, as `services/settings.rs` reads settings.json (plan-00015 phase 4).
//! The UI thread never reads it again, so an edit applies at the next start (plan-00015, D13 (a)).

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use fmc::keys::Parsed;
use fmc::text::display_label;

use super::{MAX_CONFIG_BYTES, ReadError};

/// `keymap.json` in the config folder (`services::config_folder`).
pub fn path(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    Some(super::config_folder(config_home, home)?.join("keymap.json"))
}

/// The merged keys and the notices. A missing file has the defaults and no notice.
pub fn load(path: &Path) -> Parsed {
    match super::read_config(path) {
        Ok(text) => fmc::keys::keymap::parse(&text),
        Err(ReadError::Io(error)) if error.kind() == ErrorKind::NotFound => Parsed::default(),
        Err(ReadError::Io(error)) => refused(format!(
            "keymap.json could not be read ({}); every key keeps its default.",
            display_label(&error.to_string())
        )),
        Err(ReadError::TooLarge) => {
            refused(format!("keymap.json is larger than {} KiB; every key keeps its default.", MAX_CONFIG_BYTES / 1024))
        }
        Err(ReadError::NotText) => refused("keymap.json is not UTF-8 text; every key keeps its default.".into()),
    }
}

fn refused(notice: String) -> Parsed {
    Parsed { notices: vec![notice], ..Parsed::default() }
}

#[cfg(test)]
#[path = "keymap_tests.rs"]
mod tests;
