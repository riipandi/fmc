//! The fixed noise list and the tree's sort order (Scouter `src/core/files.ts:34`, `:50-56`).

use std::sync::OnceLock;

use icu_collator::options::CollatorOptions;
use icu_collator::{CollatorBorrowed, CollatorPreferences};

use super::{Entry, Kind};

/// Entries the tree never lists. Not configurable in Scouter (`docs/decision/0004-...`).
const NOISE: [&str; 7] = [".git", ".hg", ".svn", ".DS_Store", ".AppleDouble", "Thumbs.db", "desktop.ini"];

pub fn is_noise(name: &str) -> bool {
    NOISE.contains(&name)
}

/// Root collation, the order of `Intl.Collator()` under `en-US`: punctuation first, then case-insensitive letters with
/// lowercase before uppercase at the last level.
fn collator() -> &'static CollatorBorrowed<'static> {
    static COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
    // INVARIANT: the compiled collation data ships inside the icu_collator crate, so building it cannot fail.
    COLLATOR.get_or_init(|| {
        CollatorBorrowed::try_new(CollatorPreferences::default(), CollatorOptions::default())
            .expect("compiled collation data")
    })
}

/// Folders first, then names in collation order (Scouter `readDirectory`).
pub fn sort_entries(entries: &mut [Entry]) {
    entries.sort_by(|left, right| {
        let folders = (right.kind == Kind::Directory).cmp(&(left.kind == Kind::Directory));
        folders.then_with(|| collator().compare(&left.name, &right.name))
    });
}
