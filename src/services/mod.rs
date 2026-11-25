//! I/O workers. Each one runs off the UI thread and sends an `Event` back through the app's proxy
//! (`rust.md`, "Layout rules").

pub mod fs;
pub mod git;

use std::path::PathBuf;

use fmc::diff::Listing;
use fmc::explorer::Entry;
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
