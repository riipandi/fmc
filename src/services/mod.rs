//! I/O workers. Each one runs off the UI thread and sends an `Event` back through the app's proxy.

pub mod changes;
pub mod find;
pub mod fs;
pub mod git;
pub mod keymap;
pub mod launch;
pub mod poll;
pub mod settings;
pub mod state;
pub mod theme;
pub mod write;

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use fmc::diff::{ChangeSet, FilePatch, Listing, Sign};
use fmc::explorer::Entry;
use fmc::find::Index;
use fmc::find::text::Progress;
use fmc::reader::document::Document;
use fmc::text::display_text;
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

/// Writes `text` to `path`, or to the file a link at `path` points to. The text goes to a temp file
/// beside the target, and the rename replaces the target only once the bytes are on disk, so a crash leaves the old file
/// or the new one. A new file gets the old file's mode. Nothing outside the target's folder is touched.
pub fn save_atomic(path: &Path, text: &str) -> io::Result<()> {
    let target = match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => std::fs::canonicalize(path)?,
        _ => path.to_path_buf(),
    };
    let name = target.file_name().ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "no file name"))?;
    let dir = target.parent().ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "no folder"))?;
    std::fs::create_dir_all(dir)?;
    let temp = dir.join(format!(".{}.{}.tmp", name.to_string_lossy(), std::process::id()));
    // create_new: a temp file that is already there is not ours to remove, so it is never reused.
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
    let written = write_bytes(&mut file, &target, text).and_then(|()| std::fs::rename(&temp, &target));
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

fn write_bytes(file: &mut File, target: &Path, text: &str) -> io::Result<()> {
    if let Ok(meta) = std::fs::metadata(target) {
        file.set_permissions(meta.permissions())?;
    }
    file.write_all(text.as_bytes())?;
    file.sync_all()
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
    /// listed, the marks, the branch (`None` outside a repository), and the check of the open document. `chain` is set
    /// for a timed poll, which schedules the next.
    BrowsePolled {
        root: PathBuf,
        chain: bool,
        folders: Vec<(PathBuf, Vec<Entry>)>,
        marks: Result<Option<Listing>, String>,
        branch: Option<String>,
        document: Option<(PathBuf, fs::Reread)>,
    },
    /// The changes poll's answer for the listing of `session`: the listing and the stamp of the files the body shows.
    DiffPolled { session: u64, result: Result<(ChangeSet, String), String> },
    /// One item of a run finished (`app/write.rs`): `source` was copied, moved, or trashed as `kind` says. `result` is the
    /// new path, or the error the footer shows.
    Written { kind: write::Kind, source: PathBuf, result: Result<PathBuf, String> },
    /// The Settings page's batch of changes was written, or refused with the reason (`app/settings.rs`).
    SettingsSaved { result: Result<(), String> },
    /// The remembered UI state was written, or refused with the reason (`app/remember.rs`).
    StateSaved { result: Result<(), String> },
    /// A launch finished (`app/open.rs`): `done` is the footer's text on success, `result` is `open`'s first error.
    Launched { done: String, result: Result<(), String> },
    /// The apps the Open With picker lists for `target` (`app/open.rs`); the picker opens only if `target` is still the one shown.
    Apps { target: PathBuf, result: Result<Vec<launch::App>, String> },
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

/// How often a running command is checked for its exit, for an overflow, and for the timeout.
const POLL: Duration = Duration::from_millis(5);
/// The status Bun reports for a process ended by SIGTERM, which is how Scouter's timeout ends git (`src/diff/git.ts:59`).
const TERMINATED: i32 = 143;
const NO_OUTPUT: &str = "Command output could not be read.";
/// What one git call printed: its text, and whether the cap cut it (`src/diff/git.ts:76`).
struct Output {
    text: String,
    truncated: bool,
}

/// Runs `command` and collects its stdout up to `limit` bytes. The command is killed when it writes past the limit or
/// runs past `timeout`. Past the limit, a `partial` run keeps the bytes it read; any other run is cut and has no text
/// (`src/diff/git.ts:47-107`). An error names the program as it was spawned (`git`, `/usr/bin/open`).
fn capture(mut command: Command, limit: usize, partial: bool, timeout: Duration) -> Result<Output, String> {
    let program = command.get_program().to_string_lossy().into_owned();
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|_| format!("{program} is not installed or could not be started."))?;
    let stdout = child.stdout.take().ok_or(NO_OUTPUT)?;
    let mut stderr = child.stderr.take().ok_or(NO_OUTPUT)?;
    let overflow = Arc::new(AtomicBool::new(false));
    let reader = {
        let overflow = Arc::clone(&overflow);
        thread::spawn(move || read_capped(stdout, limit, &overflow))
    };
    // stderr is drained beside stdout, so a command that fills either pipe cannot wait on this loop.
    let errors = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        bytes
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            break status;
        }
        if overflow.load(Ordering::SeqCst) || Instant::now() >= deadline {
            let _ = child.kill();
            break child.wait().map_err(|error| error.to_string())?;
        }
        thread::sleep(POLL);
    };
    let mut bytes = reader.join().map_err(|_| NO_OUTPUT.to_string())?.map_err(|error| error.to_string())?;
    let stderr = errors.join().unwrap_or_default();
    let truncated = overflow.load(Ordering::SeqCst);
    if !truncated && !status.success() {
        let code = status.code().unwrap_or(TERMINATED);
        let stderr = String::from_utf8_lossy(&stderr);
        let reason = stderr.trim().split('\n').next().unwrap_or_default();
        return Err(if reason.is_empty() {
            format!("{program} exited with status {code}.")
        } else {
            display_text(reason)
        });
    }
    if truncated {
        bytes.truncate(limit);
    }
    let text = if truncated && !partial { String::new() } else { String::from_utf8_lossy(&bytes).into_owned() };
    Ok(Output { text, truncated })
}

/// Reads stdout up to `limit` bytes, and one byte past it, so a cut is told from an output that ends exactly there.
/// A read past the limit sets `overflow`, and the waiting loop kills the command.
fn read_capped(stdout: ChildStdout, limit: usize, overflow: &AtomicBool) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    stdout.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        overflow.store(true, Ordering::SeqCst);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests;
