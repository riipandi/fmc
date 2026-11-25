//! Filesystem reads for the tree and the reader (Scouter `src/core/files.ts` `readDirectory` and `readDocument`,
//! `src/explorer/file-tree.ts` `rejectLinkLoop`). Pure in shape: a path in, values out.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use fmc::explorer::{Entry, Kind, is_noise, sort_entries};
use fmc::reader::document::{self, Document};

/// What opening an entry found: a folder's listing, or a text document.
#[derive(Clone, Debug)]
pub enum Opened {
    Folder(Vec<Entry>),
    File(Document),
}

/// Opens an entry as Scouter's `openPath` does: it follows a link and asks whether the target is a folder
/// (`src/app/app.ts:317-318`). A link to a folder lists it; a regular file is read as text.
pub fn open(path: &Path, loop_parents: &[PathBuf]) -> Result<Opened, String> {
    let metadata = fs::metadata(path).map_err(|error| describe(&error, "stat", Some(path)))?;
    if metadata.is_dir() {
        return list(path, loop_parents).map(Opened::Folder);
    }
    // Checked before the open, so a FIFO or a device is never opened: its open would wait for a writer.
    if !metadata.is_file() {
        return Err("Only regular text files can be viewed.".into());
    }
    read(path).map(Opened::File)
}

/// Reads a text document. Like Scouter, it reads one byte past the cap, so the cut can be told from a file that
/// ends at it.
pub fn read(path: &Path) -> Result<Document, String> {
    let file = File::open(path).map_err(|error| describe(&error, "open", Some(path)))?;
    let mut bytes = Vec::new();
    file.take(document::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| describe(&error, "read", None))?;
    document::decode(&bytes).map_err(|refusal| refusal.message().to_string())
}

/// Lists a folder: noise dropped, folders first, names in collation order. A symbolic link is refused when it
/// resolves to one of `loop_parents`, the folders above it in the tree.
pub fn list(path: &Path, loop_parents: &[PathBuf]) -> Result<Vec<Entry>, String> {
    if fs::symlink_metadata(path).map_err(|error| describe(&error, "lstat", Some(path)))?.file_type().is_symlink() {
        reject_loop(path, loop_parents)?;
    }
    let mut entries = Vec::new();
    for dirent in fs::read_dir(path).map_err(|error| describe(&error, "scandir", Some(path)))? {
        let dirent = dirent.map_err(|error| describe(&error, "scandir", Some(path)))?;
        let name = dirent.file_name().to_string_lossy().into_owned();
        if is_noise(&name) {
            continue;
        }
        let file_type = dirent.file_type().map_err(|error| describe(&error, "lstat", Some(&dirent.path())))?;
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
    let target = fs::canonicalize(path).map_err(|error| describe(&error, "realpath", Some(path)))?;
    for parent in loop_parents {
        if fs::canonicalize(parent).map_err(|error| describe(&error, "realpath", Some(parent)))? == target {
            return Err("Cannot expand a symbolic-link loop.".into());
        }
    }
    Ok(())
}

/// The I/O error as Node words it, which Scouter shows as is (`src/core/files.ts:describeError`):
/// `ENOENT: no such file or directory, scandir '<path>'`. A read has no path, as in Node. One place for the
/// wording, so every view that reports a file error reads the same.
pub fn describe(error: &io::Error, syscall: &str, path: Option<&Path>) -> String {
    let (code, message) = match error.kind() {
        io::ErrorKind::NotFound => ("ENOENT", "no such file or directory"),
        io::ErrorKind::PermissionDenied => ("EACCES", "permission denied"),
        io::ErrorKind::NotADirectory => ("ENOTDIR", "not a directory"),
        io::ErrorKind::IsADirectory => ("EISDIR", "illegal operation on a directory"),
        // shortcut: the other codes keep Rust's text until a tree or a reader shows one, then add its row here.
        _ => return error.to_string(),
    };
    match path {
        Some(path) => format!("{code}: {message}, {syscall} '{}'", path.display()),
        None => format!("{code}: {message}, {syscall}"),
    }
}

#[cfg(test)]
#[path = "fs_tests.rs"]
mod tests;
