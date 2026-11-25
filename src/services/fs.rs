//! Filesystem reads for the tree (Scouter `src/core/files.ts` `readDirectory`, `src/explorer/file-tree.ts`
//! `rejectLinkLoop`). Pure in shape: a folder in, its entries out.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fmc::explorer::{Entry, Kind, is_noise, sort_entries};

/// What opening an entry found: a folder's listing, or a file.
#[derive(Clone, Debug)]
pub enum Opened {
    Folder(Vec<Entry>),
    File,
}

/// Opens an entry as Scouter's `openPath` does: it follows a link and asks whether the target is a folder
/// (`src/app/app.ts:317-318`). A link to a folder lists it; a link to a file is a file.
pub fn open(path: &Path, loop_parents: &[PathBuf]) -> Result<Opened, String> {
    if fs::metadata(path).map_err(text)?.is_dir() {
        return list(path, loop_parents).map(Opened::Folder);
    }
    Ok(Opened::File)
}

/// Lists a folder: noise dropped, folders first, names in collation order. A symbolic link is refused when it
/// resolves to one of `loop_parents`, the folders above it in the tree.
pub fn list(path: &Path, loop_parents: &[PathBuf]) -> Result<Vec<Entry>, String> {
    if fs::symlink_metadata(path).map_err(text)?.file_type().is_symlink() {
        reject_loop(path, loop_parents)?;
    }
    let mut entries = Vec::new();
    for dirent in fs::read_dir(path).map_err(text)? {
        let dirent = dirent.map_err(text)?;
        let name = dirent.file_name().to_string_lossy().into_owned();
        if is_noise(&name) {
            continue;
        }
        let file_type = dirent.file_type().map_err(text)?;
        let kind = if file_type.is_dir() {
            Kind::Directory
        } else if file_type.is_file() {
            Kind::File
        } else if file_type.is_symlink() {
            Kind::Link
        } else {
            Kind::Other
        };
        entries.push(Entry { name, path: dirent.path(), kind });
    }
    sort_entries(&mut entries);
    Ok(entries)
}

fn reject_loop(path: &Path, loop_parents: &[PathBuf]) -> Result<(), String> {
    let target = fs::canonicalize(path).map_err(text)?;
    for parent in loop_parents {
        if fs::canonicalize(parent).map_err(text)? == target {
            return Err("Cannot expand a symbolic-link loop.".into());
        }
    }
    Ok(())
}

fn text(error: io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
#[path = "fs_tests.rs"]
mod tests;
