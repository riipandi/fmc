//! Reads the user's themes from `themes/` in the config folder once, before the first frame (plan-00015, D7), as
//! `services/settings.rs` reads the settings. The UI thread never reads the folder again.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

use fmc::text::display_label;
use fmc::theme::catalog::{self, Theme};

/// A theme file larger than this is skipped unread past the cap: a palette is a few hundred bytes, and the read happens
/// before the first frame.
const MAX_BYTES: u64 = 1 << 20;

/// The themes the folder holds, and a notice for each file that was skipped.
#[derive(Debug, Default)]
pub struct Loaded {
    pub themes: Vec<Theme>,
    pub notices: Vec<String>,
}

/// `themes/` in the config folder, or `None` when there is no home folder to name it from.
pub fn folder(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    Some(super::config_folder(config_home, home)?.join("themes"))
}

/// Every `*.json` file in `dir`, with its file stem as the theme id. A missing folder has no notice. A file that
/// cannot be read or parsed is skipped with a notice that names it; other files are ignored.
pub fn load(dir: &Path) -> Loaded {
    let mut loaded = Loaded::default();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return loaded,
        Err(error) => {
            loaded.notices.push(format!("themes could not be read: {}.", display_label(&error.to_string())));
            return loaded;
        }
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "json") && path.is_file())
        .collect();
    // read_dir has no order; a fixed one keeps the notices in the same order each start.
    files.sort();
    for file in files {
        let (Some(id), Some(name)) = (file.file_stem(), file.file_name()) else { continue };
        let id = id.to_string_lossy();
        let name = display_label(&name.to_string_lossy());
        let outcome = read(&file).and_then(|text| catalog::parse(&id, &text));
        match outcome {
            Ok(theme) => loaded.themes.push(theme),
            Err(reason) => loaded.notices.push(format!("themes/{name} is skipped: {}.", display_label(&reason))),
        }
    }
    loaded
}

/// The file's text, or why it cannot be read: a read error, a size past `MAX_BYTES`, or text that is not UTF-8.
fn read(file: &Path) -> Result<String, String> {
    let mut bytes = Vec::new();
    File::open(file)
        .and_then(|handle| handle.take(MAX_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|error| format!("it could not be read ({})", display_label(&error.to_string())))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(format!("it is larger than {} KiB", MAX_BYTES / 1024));
    }
    String::from_utf8(bytes).map_err(|_| "it is not UTF-8 text".into())
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
