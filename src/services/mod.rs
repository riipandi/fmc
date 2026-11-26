//! I/O workers. Each one runs off the UI thread and sends an `Event` back through the app's proxy
//! (`rust.md`, "Layout rules").

pub mod changes;
pub mod find;
pub mod fs;
pub mod git;

use std::path::PathBuf;

use fmc::diff::{FilePatch, Listing};
use fmc::explorer::Entry;
use fmc::find::Index;
use fmc::find::text::Progress;
use fmc::reader::document::Document;
use opentui::prelude::AppProxy;

/// What a worker found, for the app to apply on the UI thread.
#[derive(Clone, Debug)]
pub enum Event {
    /// A folder listed for the tree (startup, reveal, going up).
    Listed { path: PathBuf, result: Result<Vec<Entry>, String> },
    /// A user's open of an entry: a folder to expand, or a file.
    Opened { path: PathBuf, result: Result<fs::Opened, String> },
    /// The git marks of the browsed folder; `None` outside a repository.
    Marks { root: PathBuf, result: Result<Option<Listing>, String> },
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
    /// The stacked patches of the listing of `session`; a reply for a listing that is no longer shown is dropped.
    ChangesPatched { session: u64, result: Result<Vec<FilePatch>, String> },
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
