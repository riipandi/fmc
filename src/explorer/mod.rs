//! The file tree model (Scouter `src/explorer/`, `src/core/files.ts`): the rows, expansion, and reveal, with the
//! sort order. Listing a folder is I/O, so the model takes the listing as values (`services/fs.rs`).

mod sort;
mod tree;

use std::path::{Path, PathBuf};

pub use sort::{is_noise, sort_entries};
pub use tree::{Expand, Reveal, Row, Tree};

/// What a directory entry is, as `readdir` reports it without following links (Scouter `DirectoryEntry.kind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Directory,
    File,
    Link,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub kind: Kind,
}

impl Entry {
    pub fn new(folder: &Path, name: &str, kind: Kind) -> Self {
        Self { name: name.to_string(), path: folder.join(name), kind }
    }
}

#[cfg(test)]
mod tests;
