//! The finder's index walk (Scouter `src/find/search.ts`, `indexTree`): a breadth-first walk of the browsed folder, one
//! level at a time, that skips what the caller says and stops at the limit. It runs on a worker, never on the UI thread.

use std::path::Path;

use fmc::explorer::{Kind, relative};
use fmc::find::{Index, Indexed, sort_listing};

use super::fs;

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

#[cfg(test)]
#[path = "find_tests.rs"]
mod tests;
