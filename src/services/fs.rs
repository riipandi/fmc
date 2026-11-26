//! Filesystem reads for the tree and the reader (Scouter `src/core/files.ts` `readDirectory` and `readDocument`,
//! `src/explorer/file-tree.ts` `rejectLinkLoop`). Pure in shape: a path in, values out.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use fmc::explorer::{Entry, Kind, is_noise, sort_entries};
use fmc::reader::document::{self, Document};
use fmc::reader::image;
use imagesize::ImageType;

/// What opening an entry found: a folder's listing, a text document, an image's header facts and bytes, or the size
/// of a binary file.
#[derive(Clone, Debug)]
pub enum Opened {
    Folder(Vec<Entry>),
    /// A text document and its stamp (`stamp`), which a poll compares with the file's stamp now.
    Text {
        document: Document,
        stamp: String,
    },
    Image {
        info: image::Info,
        bytes: Vec<u8>,
    },
    Binary {
        size: u64,
    },
}

/// Opens an entry as Scouter's `openPath` does: it follows a link and asks whether the target is a folder
/// (`src/app/app.ts:317-318`). A link to a folder lists it; an image extension reads an image; any other regular
/// file is read as text, and a binary one is reported by its size.
pub fn open(path: &Path, loop_parents: &[PathBuf]) -> Result<Opened, String> {
    let metadata = fs::metadata(path).map_err(|error| describe(&error, "stat", Some(path)))?;
    if metadata.is_dir() {
        return list(path, loop_parents).map(Opened::Folder);
    }
    if image::is_image_path(path) {
        return read_image(path, &metadata);
    }
    // Checked before the open, so a FIFO or a device is never opened: its open would wait for a writer.
    if !metadata.is_file() {
        return Err("Only regular text files can be viewed.".into());
    }
    read(path, &metadata)
}

/// A file's stamp: its modification time and size, which change when its text does (Scouter `formatFileStamp`).
pub fn stamp(metadata: &fs::Metadata) -> String {
    let modified = metadata.modified().ok().and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok());
    format!("{}:{}", modified.map_or(0, |time| time.as_nanos()), metadata.len())
}

/// What a poll found of an open text document (Scouter `refreshDocument`).
#[derive(Clone, Debug)]
pub enum Reread {
    /// The file is gone (or cannot be stat'ed): the reader keeps its content and says so.
    Missing,
    /// The file changed and reads again: its new document and stamp.
    Changed { document: Document, stamp: String },
    /// The stamp is the one shown, or the file no longer reads as text; the reader keeps what it shows.
    Unchanged,
}

/// Checks the open text document shown with `shown` as its stamp: a changed file is read again (Scouter
/// `refreshDocument`, `src/app/reader-pane.ts:173-201`).
pub fn reread(path: &Path, shown: &str) -> Reread {
    let Ok(metadata) = fs::metadata(path) else { return Reread::Missing };
    if stamp(&metadata) == shown {
        return Reread::Unchanged;
    }
    match open(path, &[]) {
        Ok(Opened::Text { document, stamp }) => Reread::Changed { document, stamp },
        _ => Reread::Unchanged,
    }
}

/// Reads a text document. Like Scouter, it reads one byte past the cap, so the cut can be told from a file that
/// ends at it.
fn read(path: &Path, metadata: &fs::Metadata) -> Result<Opened, String> {
    let file = File::open(path).map_err(|error| describe(&error, "open", Some(path)))?;
    let mut bytes = Vec::new();
    file.take(document::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| describe(&error, "read", None))?;
    match document::decode(&bytes) {
        Ok(document) => Ok(Opened::Text { document, stamp: stamp(metadata) }),
        Err(document::Refusal::Binary) => Ok(Opened::Binary { size: metadata.len() }),
        Err(refusal) => Err(refusal.message().to_string()),
    }
}

/// Reads an image up to the cap and reads its header. A format the decoder does not report, or a header it cannot
/// parse, is unreadable (`src/reader/image-view.ts:5-12`).
fn read_image(path: &Path, metadata: &fs::Metadata) -> Result<Opened, String> {
    if !metadata.is_file() {
        return Err("Only regular files can be viewed.".into());
    }
    if metadata.len() > image::MAX_BYTES {
        return Err(image::TOO_LARGE.into());
    }
    let file = File::open(path).map_err(|error| describe(&error, "open", Some(path)))?;
    let mut bytes = Vec::new();
    file.take(image::MAX_BYTES + 1).read_to_end(&mut bytes).map_err(|error| describe(&error, "read", None))?;
    if bytes.len() as u64 > image::MAX_BYTES {
        return Err(image::TOO_LARGE.into());
    }
    let format = match imagesize::image_type(&bytes) {
        Ok(ImageType::Png) => "png",
        Ok(ImageType::Jpeg) => "jpeg",
        Ok(ImageType::Webp) => "webp",
        Ok(ImageType::Gif) => "gif",
        _ => return Err(image::UNREADABLE.into()),
    };
    let size = imagesize::blob_size(&bytes).map_err(|_| image::UNREADABLE.to_string())?;
    let info = image::Info { width: size.width as u32, height: size.height as u32, format };
    Ok(Opened::Image { info, bytes })
}

/// Lists a folder: noise dropped, folders first, names in collation order. A symbolic link is refused when it
/// resolves to one of `loop_parents`, the folders above it in the tree.
pub fn list(path: &Path, loop_parents: &[PathBuf]) -> Result<Vec<Entry>, String> {
    if fs::symlink_metadata(path).map_err(|error| describe(&error, "lstat", Some(path)))?.file_type().is_symlink() {
        reject_loop(path, loop_parents)?;
    }
    let mut entries = read_entries(path)?;
    sort_entries(&mut entries);
    Ok(entries)
}

/// A folder's entries with the noise dropped, in directory order. A link stays a link: a folder is not followed.
pub fn read_entries(path: &Path) -> Result<Vec<Entry>, String> {
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
