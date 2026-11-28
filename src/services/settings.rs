//! Reads `settings.json` once, before the first frame, and writes it through `save_atomic` (`architecture.md`, "fmc writes its
//! settings.json, atomically"). The UI thread never reads the file again.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use fmc::settings::{self, Parsed};
use fmc::text::display_label;
use serde_json::Value;

use super::{MAX_CONFIG_BYTES, ReadError};

/// `settings.json` in the config folder (`services::config_folder`).
pub fn path(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    Some(super::config_folder(config_home, home)?.join("settings.json"))
}

/// `meta/config.schema.json` in the config folder: the JSON Schema of `settings.json`, which fmc writes (plan-00016 D1).
pub fn schema_path(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    Some(super::config_folder(config_home, home)?.join("meta").join("config.schema.json"))
}

/// Writes the schema at `path` when the file is missing or differs from `settings::schema()` (plan-00016 D5). A start
/// whose file is equal writes nothing, so its mtime holds. A failed write is the notice the footer shows, and the next
/// start tries again (O2 (a)).
pub fn ensure_schema(path: &Path) -> Option<String> {
    let schema = settings::schema();
    if super::read_config(path).is_ok_and(|current| current == schema) {
        return None;
    }
    super::save_atomic(path, &schema)
        .err()
        .map(|error| format!("meta/config.schema.json could not be written: {}.", display_label(&error.to_string())))
}

/// The settings the file sets, with a notice for each value it cannot use. A missing file has no notice.
pub fn load(path: &Path) -> Parsed {
    let notice = |text: String| Parsed { notices: vec![text], ..Parsed::default() };
    match super::read_config(path) {
        Ok(text) => settings::parse(&text),
        Err(ReadError::Io(error)) if error.kind() == ErrorKind::NotFound => Parsed::default(),
        Err(ReadError::Io(error)) => {
            notice(format!("settings.json could not be read: {}.", display_label(&error.to_string())))
        }
        Err(ReadError::TooLarge) => notice(format!(
            "settings.json is larger than {} KiB; every setting uses its default.",
            MAX_CONFIG_BYTES / 1024
        )),
        Err(ReadError::NotText) => notice("settings.json is not UTF-8 text; every setting uses its default.".into()),
    }
}

/// Sets `changes` in the settings file at `path` and writes the file (the Settings page's save, on its worker). The file
/// is read here, not when the page changed, so a hand edit made since the start is kept. A missing file starts from `{}`;
/// a file that does not parse is refused and left as it is (`settings::merged`).
pub fn update(path: &Path, changes: &[(&str, Value)]) -> Result<(), String> {
    let text = match super::read_config(path) {
        Ok(text) => text,
        Err(ReadError::Io(error)) if error.kind() == ErrorKind::NotFound => "{}".into(),
        Err(ReadError::Io(error)) => {
            return Err(format!("settings.json could not be read: {error}; nothing was saved."));
        }
        Err(ReadError::TooLarge) => {
            return Err(format!("settings.json is larger than {} KiB; nothing was saved.", MAX_CONFIG_BYTES / 1024));
        }
        Err(ReadError::NotText) => return Err("settings.json is not UTF-8 text; nothing was saved.".into()),
    };
    let text = settings::merged(&text, changes)?;
    super::save_atomic(path, &text)
        .map_err(|error| format!("settings.json could not be saved: {error}; nothing was saved."))
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
