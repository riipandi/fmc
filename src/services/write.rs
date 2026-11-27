//! The write layer of the file-manager phase (plan-00012, D5 and D6). Each operation takes the browsed `root` and refuses
//! to write outside it. None overwrites: a taken name is an error. A link is created, renamed, copied, moved, and trashed
//! as the link, never followed. Paths in, a path or an error out; the UI calls it from a worker (copy and move) or inline.
// shortcut: nothing in the binary calls this layer until the keys land (plan-00012 phase 3); drop this allow then.
#![allow(dead_code)]

use std::fmt;
use std::fs::{self, OpenOptions};
use std::io;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

/// Why a write was refused or failed. The UI shows it as a short message (`Display`).
#[derive(Debug)]
pub enum Error {
    /// The name is not one path segment: empty, `.` or `..`, or with a `/` or a NUL.
    BadName,
    /// The name is taken; nothing was written.
    Exists,
    /// The folder the entry would be written into lies outside the browsed root, through a `..` or a link.
    Outside,
    /// A folder cannot be copied or moved into itself.
    IntoItself,
    /// Nothing is at the path.
    Missing,
    /// A pipe, a socket, or a device: copying it would block on a reader or writer that never comes.
    Unsupported,
    /// The Trash refused the entry.
    Trash(String),
    /// Any other filesystem error.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadName => write!(f, "That name is not valid."),
            Error::Exists => write!(f, "An item with that name already exists."),
            Error::Outside => write!(f, "That place is outside the folder fmc opened."),
            Error::IntoItself => write!(f, "A folder cannot go into itself."),
            Error::Missing => write!(f, "Nothing is at that path."),
            Error::Unsupported => write!(f, "Only files, folders, and links can be copied."),
            Error::Trash(message) => write!(f, "Could not move to the Trash: {message}"),
            Error::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::AlreadyExists => Error::Exists,
            io::ErrorKind::NotFound => Error::Missing,
            _ => Error::Io(error),
        }
    }
}

/// Creates an empty file `name` in `parent`. `create_new` makes the check and the create one call, so a file that
/// appears first is never truncated.
pub fn create_file(root: &Path, parent: &Path, name: &str) -> Result<PathBuf, Error> {
    segment(name.as_bytes())?;
    inside(root, parent)?;
    let path = parent.join(name);
    OpenOptions::new().write(true).create_new(true).open(&path)?;
    Ok(path)
}

/// Creates an empty folder `name` in `parent`. `mkdir` refuses a name that exists, a link included.
pub fn create_dir(root: &Path, parent: &Path, name: &str) -> Result<PathBuf, Error> {
    segment(name.as_bytes())?;
    inside(root, parent)?;
    let path = parent.join(name);
    fs::create_dir(&path)?;
    Ok(path)
}

/// Renames `path` to `name` in the same folder.
pub fn rename(root: &Path, path: &Path, name: &str) -> Result<PathBuf, Error> {
    let dest = plan(root, path, parent_of(path)?, name)?;
    // shortcut: `plan` checks the name and `rename` acts on it in two calls, and std has no no-clobber rename (that
    // needs `renamex_np` through libc), so a file created in between is replaced; upgrade to RENAME_EXCL if it matters.
    fs::rename(path, &dest)?;
    Ok(dest)
}

/// Copies `source` into `dest_dir` as `name`: a file, or a folder with everything in it. A link is copied as a link.
pub fn copy(root: &Path, source: &Path, dest_dir: &Path, name: &str) -> Result<PathBuf, Error> {
    let dest = plan(root, source, dest_dir, name)?;
    copy_entry(source, &dest)?;
    Ok(dest)
}

/// Moves `source` into `dest_dir` as `name`. Across volumes the move is a copy, then the removal of the source.
pub fn move_to(root: &Path, source: &Path, dest_dir: &Path, name: &str) -> Result<PathBuf, Error> {
    move_with(root, source, dest_dir, name, trash_path)
}

/// Moves the entry at `path` to the Trash, where Finder's Put Back restores it (plan-00012, D4).
pub fn trash(root: &Path, path: &Path) -> Result<(), Error> {
    trash_with(root, path, trash_path)
}

/// A free name for a paste into `dir`, as Finder names a duplicate: `name copy`, then `name copy 2`, before the
/// extension. A folder, a dotfile, and a name with no extension take the suffix at the end.
pub fn free_name(dir: &Path, name: &str, is_dir: bool) -> Result<String, Error> {
    segment(name.as_bytes())?;
    let (stem, ext) = split(name, is_dir);
    let mut n = 1;
    loop {
        let candidate = if n == 1 { format!("{stem} copy{ext}") } else { format!("{stem} copy {n}{ext}") };
        match fs::symlink_metadata(dir.join(&candidate)) {
            Ok(_) => n += 1,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(candidate),
            Err(error) => return Err(error.into()),
        }
    }
}

/// The stem and the extension (with its dot) of a file's name. A folder, a dotfile, and a name that ends in a dot have
/// no extension.
fn split(name: &str, is_dir: bool) -> (&str, &str) {
    match name.rfind('.') {
        Some(dot) if !is_dir && dot > 0 && dot + 1 < name.len() => name.split_at(dot),
        _ => (name, ""),
    }
}

fn move_with(
    root: &Path,
    source: &Path,
    dest_dir: &Path,
    name: &str,
    remove: impl FnOnce(&Path) -> Result<(), Error>,
) -> Result<PathBuf, Error> {
    let dest = plan(root, source, dest_dir, name)?;
    // shortcut: the same race as `rename`: `plan` checks the name, then `rename` acts on it, and no no-clobber rename
    // is in std.
    match fs::rename(source, &dest) {
        Ok(()) => Ok(dest),
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
            copy_then_remove(source, &dest, remove)?;
            Ok(dest)
        }
        Err(error) => Err(error.into()),
    }
}

/// The cross-volume move: copy the entry, then remove the source. A failed copy leaves the source alone; a failed
/// removal leaves the copy and the source, so nothing is lost.
fn copy_then_remove(source: &Path, dest: &Path, remove: impl FnOnce(&Path) -> Result<(), Error>) -> Result<(), Error> {
    copy_entry(source, dest)?;
    remove(source)
}

fn trash_with(root: &Path, path: &Path, delete: impl FnOnce(&Path) -> Result<(), Error>) -> Result<(), Error> {
    segment(path.file_name().ok_or(Error::BadName)?.as_encoded_bytes())?;
    inside(root, parent_of(path)?)?;
    fs::symlink_metadata(path)?;
    delete(path)
}

fn trash_path(path: &Path) -> Result<(), Error> {
    trash::delete(path).map_err(|error| Error::Trash(error.to_string()))
}

/// The checks every operation that puts `source` into `dest_dir` as `name` makes before it writes: the name, the root
/// for both folders, the source's existence, a folder into itself, and a free destination. Returns the destination.
fn plan(root: &Path, source: &Path, dest_dir: &Path, name: &str) -> Result<PathBuf, Error> {
    segment(name.as_bytes())?;
    segment(source.file_name().ok_or(Error::BadName)?.as_encoded_bytes())?;
    inside(root, parent_of(source)?)?;
    inside(root, dest_dir)?;
    let metadata = fs::symlink_metadata(source)?;
    // A link to a folder is not a folder here: it is copied and moved as the link, so it cannot loop back.
    if metadata.is_dir() && fs::canonicalize(dest_dir)?.starts_with(fs::canonicalize(source)?) {
        return Err(Error::IntoItself);
    }
    let dest = dest_dir.join(name);
    vacant(&dest)?;
    Ok(dest)
}

/// Copies one entry to `dest`, which must not exist. A link is copied as a link, and a folder recurses the same way. A
/// failure removes what this call made, never a `dest` that was already there.
fn copy_entry(source: &Path, dest: &Path) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(source)?;
    let kind = metadata.file_type();
    if kind.is_symlink() {
        symlink(fs::read_link(source)?, dest)?;
    } else if kind.is_dir() {
        fs::create_dir(dest)?;
        let copied = copy_children(source, dest)
            .and_then(|()| fs::set_permissions(dest, copied_mode(&metadata)).map_err(Error::from));
        if let Err(error) = copied {
            let _ = fs::remove_dir_all(dest);
            return Err(error);
        }
    } else if kind.is_file() {
        // shortcut: the kind check and the open are two calls, so a pipe swapped in between blocks the copy until a
        // writer opens it; `O_NONBLOCK` (a libc constant) closes that.
        let mut from = fs::File::open(source)?;
        let mut to = OpenOptions::new().write(true).create_new(true).open(dest)?;
        if let Err(error) = io::copy(&mut from, &mut to).and_then(|_| to.set_permissions(copied_mode(&metadata))) {
            drop(to);
            let _ = fs::remove_file(dest);
            return Err(error.into());
        }
    } else {
        return Err(Error::Unsupported);
    }
    Ok(())
}

fn copy_children(source: &Path, dest: &Path) -> Result<(), Error> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        copy_entry(&entry.path(), &dest.join(entry.file_name()))?;
    }
    Ok(())
}

/// A name is one path segment (plan-00012, D6): not empty, not `.` or `..`, and without a `/` or a NUL.
fn segment(name: &[u8]) -> Result<(), Error> {
    let bad = name.is_empty() || name == b"." || name == b".." || name.iter().any(|&byte| byte == b'/' || byte == 0);
    if bad { Err(Error::BadName) } else { Ok(()) }
}

/// The source's permission bits, without setuid, setgid, or the sticky bit: a copy is the user's own file, not a new
/// setuid one.
fn copied_mode(metadata: &fs::Metadata) -> fs::Permissions {
    fs::Permissions::from_mode(metadata.permissions().mode() & 0o777)
}

/// The folder `dir` lies inside the browsed `root`, both resolved. The folder is resolved, never the entry: a link
/// inside the root that points out is an entry of the root, and it is operated on as the link. The check and the write
/// are two calls, so a folder swapped for a link between them escapes; the threat is another process running as the
/// user, which can already write there.
fn inside(root: &Path, dir: &Path) -> Result<(), Error> {
    let root = fs::canonicalize(root)?;
    if fs::canonicalize(dir)?.starts_with(root) { Ok(()) } else { Err(Error::Outside) }
}

fn parent_of(path: &Path) -> Result<&Path, Error> {
    path.parent().ok_or(Error::Outside)
}

/// Fails when anything is at `path`, a dangling link included.
fn vacant(path: &Path) -> Result<(), Error> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(Error::Exists),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
#[path = "write_tests.rs"]
mod tests;
