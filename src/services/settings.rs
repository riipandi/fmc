//! Reads `settings.json` once, before the first frame, and writes it through `save` (`architecture.md`, "fmc writes its
//! settings.json, atomically"). The UI thread never reads the file again.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process;

use fmc::settings::{self, Parsed};

/// `settings.json` in the config folder (`services::config_folder`).
pub fn path(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    Some(super::config_folder(config_home, home)?.join("settings.json"))
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

/// Writes `text` to the settings file at `path`, or to the file a link at `path` points to (plan-00015 D14). The text goes
/// to a temp file beside the target, and the rename replaces the target only once the bytes are on disk, so a crash
/// leaves the old file or the new one. A new file gets the old file's mode. Nothing outside the folder is touched.
#[allow(dead_code)] // No caller until the Settings page (plan-00015, phase 3).
pub fn save(path: &Path, text: &str) -> io::Result<()> {
    let target = match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => fs::canonicalize(path)?,
        _ => path.to_path_buf(),
    };
    let name = target.file_name().ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "no file name"))?;
    let dir = target.parent().ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "no folder"))?;
    fs::create_dir_all(dir)?;
    let temp = dir.join(format!(".{}.{}.tmp", name.to_string_lossy(), process::id()));
    // create_new: a temp file that is already there is not ours to remove, so it is never reused.
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
    let written = write(&mut file, &target, text).and_then(|()| fs::rename(&temp, &target));
    if written.is_err() {
        let _ = fs::remove_file(&temp);
    }
    written
}

fn write(file: &mut File, target: &Path, text: &str) -> io::Result<()> {
    if let Ok(meta) = fs::metadata(target) {
        file.set_permissions(meta.permissions())?;
    }
    file.write_all(text.as_bytes())?;
    file.sync_all()
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
