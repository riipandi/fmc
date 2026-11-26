//! The finder's index walk (Scouter `src/find/search.ts`, `indexTree`): a breadth-first walk of the browsed folder, one
//! level at a time, that skips what the caller says and stops at the limit. It runs on a worker, never on the UI thread.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use fmc::explorer::{Kind, relative};
use fmc::find::text::{Progress, Scan};
use fmc::find::{Index, Indexed, sort_listing};
use fmc::reader::document::Document;
use fmc::reader::image;

use super::fs;

/// How often a text search reports its progress (Scouter `REPORT_INTERVAL_MS`).
const REPORT_EVERY: Duration = Duration::from_millis(120);

/// Walks `root` breadth first. Each folder lists folders first, then names by code unit; a path that `skip` names is
/// left out with everything under it, and a link is listed but not followed (Scouter `indexTree`).
pub fn index(root: &Path, skip: impl Fn(&Path) -> bool, limit: usize) -> Index {
    let mut entries = Vec::new();
    let mut level = vec![root.to_path_buf()];
    while !level.is_empty() {
        let mut next = Vec::new();
        for folder in &level {
            // An unreadable folder contributes nothing, as Scouter's `catch(() => [])` does.
            let mut children = fs::read_entries(folder).unwrap_or_default();
            sort_listing(&mut children);
            for child in children {
                if skip(&child.path) {
                    continue;
                }
                if entries.len() >= limit {
                    return Index { entries, truncated: true };
                }
                if child.kind == Kind::Directory {
                    next.push(child.path.clone());
                }
                entries.push(Indexed::new(fmc::explorer::Entry {
                    name: relative(root, &child.path),
                    path: child.path,
                    kind: child.kind,
                }));
            }
        }
        level = next;
    }
    Index { entries, truncated: false }
}

/// Searches the index's files for `query` (Scouter `searchText`), reporting progress every 120 ms and once at the
/// end. A search whose `cancelled` flag is set stops between files and reports nothing more.
pub fn search(index: &Index, query: &str, cancelled: &AtomicBool, report: &dyn Fn(Progress)) {
    let files: Vec<&Indexed> = index.entries.iter().filter(|entry| entry.entry.kind == Kind::File).collect();
    let mut scan = Scan::new(query, files.len());
    let mut reported = Instant::now();
    for entry in files {
        if cancelled.load(Ordering::Relaxed) {
            return;
        }
        if scan.stopped() {
            break;
        }
        let path = &entry.entry.path;
        // An image or a file that does not read as text counts as scanned, as Scouter's `readDocument` failure does.
        let text = match image::is_image_path(path) {
            true => None,
            false => match fs::open(path, &[]) {
                Ok(fs::Opened::Text { document, .. }) => Some(document),
                _ => None,
            },
        };
        match text {
            Some(document) => scan.file(path, &entry.entry.name, &document.text),
            None => scan.skipped(),
        }
        if reported.elapsed() >= REPORT_EVERY {
            report(scan.progress(false));
            reported = Instant::now();
        }
    }
    if !cancelled.load(Ordering::Relaxed) {
        report(scan.progress(true));
    }
}

/// The text of a file for the finder's preview; a file that is not text is an error, which the preview ignores.
pub fn preview(path: &Path) -> Result<Document, String> {
    match fs::open(path, &[])? {
        fs::Opened::Text { document, .. } => Ok(document),
        _ => Err("Only UTF-8 text files are previewed.".into()),
    }
}

#[cfg(test)]
#[path = "find_tests.rs"]
mod tests;
