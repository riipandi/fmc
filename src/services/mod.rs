//! I/O workers. Each one runs off the UI thread and sends an `Event` back through the app's proxy
//! (`rust.md`, "Layout rules").

pub mod changes;
pub mod find;
pub mod fs;
pub mod git;
pub mod keymap;
pub mod poll;
pub mod settings;
pub mod theme;
pub mod write;

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};

use fmc::diff::{ChangeSet, FilePatch, Listing, Sign};
use fmc::explorer::Entry;
use fmc::find::Index;
use fmc::find::text::Progress;
use fmc::reader::document::Document;
use opentui::prelude::AppProxy;

/// `$XDG_CONFIG_HOME/fmc`, or `~/.config/fmc` when `XDG_CONFIG_HOME` is unset, empty, or relative (the XDG Base Directory
/// rule: a relative value is ignored, so a save never lands in the working directory). Settings and themes live here.
pub fn config_folder(config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = match config_home.filter(|dir| Path::new(dir).is_absolute()) {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(home?).join(".config"),
    };
    Some(base.join("fmc"))
}

/// A config file larger than this is refused unread past the cap: the reads happen before the first frame.
pub const MAX_CONFIG_BYTES: u64 = 1 << 20;

/// Why a config file could not be read whole.
pub enum ReadError {
    Io(io::Error),
    TooLarge,
    NotText,
}

/// The text of a config file, read up to `MAX_CONFIG_BYTES`. A FIFO or a device is refused before it is opened, so a
/// read at startup cannot block on it.
pub fn read_config(file: &Path) -> Result<String, ReadError> {
    if std::fs::metadata(file).is_ok_and(|meta| !meta.is_file()) {
        return Err(ReadError::Io(io::Error::new(ErrorKind::InvalidInput, "it is not a regular file")));
    }
    let mut bytes = Vec::new();
    File::open(file)
        .and_then(|handle| handle.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut bytes))
        .map_err(ReadError::Io)?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(ReadError::TooLarge);
    }
    String::from_utf8(bytes).map_err(|_| ReadError::NotText)
}

/// What a worker found, for the app to apply on the UI thread.
#[derive(Clone, Debug)]
pub enum Event {
    /// A folder listed for the tree (startup, reveal, going up).
    Listed { path: PathBuf, result: Result<Vec<Entry>, String> },
    /// A user's open of an entry: a folder to expand, or a file.
    Opened { path: PathBuf, result: Result<fs::Opened, String> },
    /// A file the list selected, read for auto-preview (`app/preview.rs`): it never moves the focus or the history.
    Selected { path: PathBuf, result: Result<fs::Opened, String> },
    /// The git marks of the browsed folder; `None` outside a repository.
    Marks { root: PathBuf, result: Result<Option<Listing>, String> },
    /// The signs of the text at `path` that `serial` showed (`app/gutter.rs`); an answer for another file or showing is
    /// dropped.
    Signed { path: PathBuf, serial: u64, result: Result<Vec<(u32, Sign)>, String> },
    /// The finder's index of the browsed folder, for the finder of that `session` (a newer one may have replaced it).
    Indexed { session: u64, index: Index },
    /// The debounce of a text search has passed: the search of `scan` starts if it is still the latest.
    ScanDue { session: u64, scan: u64 },
    /// A text search's progress, for the finder of `session` while `scan` is its latest search.
    Searched { session: u64, scan: u64, progress: Progress },
    /// A file read for the finder's preview; a `request` older than the latest one is dropped.
    Previewed { session: u64, request: u64, result: Result<Document, String> },
    /// The changes screen's listing, for the `session` that asked for it; a cancelled session is dropped.
    ChangesListed { session: u64, result: Result<changes::Listed, String> },
    /// The patches the body shows for the listing of `session`: all files, or the one selected. A `request` older than
    /// the latest one, or a listing no longer shown, is dropped.
    ChangesPatched { session: u64, request: u64, stamp: String, result: Result<Vec<FilePatch>, String> },
    /// A browse poll's answer (Scouter `refreshBrowse`): the fresh listings of the expanded folders that could be
    /// listed, the marks, and the check of the open document. `chain` is set for a timed poll, which schedules the next.
    BrowsePolled {
        root: PathBuf,
        chain: bool,
        folders: Vec<(PathBuf, Vec<Entry>)>,
        marks: Result<Option<Listing>, String>,
        document: Option<(PathBuf, fs::Reread)>,
    },
    /// The changes poll's answer for the listing of `session`: the listing and the stamp of the files the body shows.
    DiffPolled { session: u64, result: Result<(ChangeSet, String), String> },
    /// One item of a run finished (`app/write.rs`): `source` was copied, moved, or trashed as `kind` says. `result` is the
    /// new path, or the error the footer shows.
    Written { kind: write::Kind, source: PathBuf, result: Result<PathBuf, String> },
    /// The Settings page's batch of changes was written, or refused with the reason (`app/settings.rs`).
    SettingsSaved { result: Result<(), String> },
}

/// Runs `work` on its own thread, which sends as many events as it likes through `send` (a search reports its
/// progress as it goes). A send after the app has quit fails, and the event is dropped.
pub fn stream(proxy: &AppProxy, work: impl FnOnce(&dyn Fn(Event)) + Send + 'static) {
    let proxy = proxy.clone();
    std::thread::spawn(move || {
        work(&|event| {
            let _ = proxy.send(event);
        })
    });
}

/// Runs `work` on its own thread, then sends the event it returns to the UI thread. A send after the app has
/// quit fails, and the event is dropped.
pub fn spawn(proxy: &AppProxy, work: impl FnOnce() -> Event + Send + 'static) {
    let proxy = proxy.clone();
    std::thread::spawn(move || {
        let _ = proxy.send(work());
    });
}

#[cfg(test)]
mod tests;
