//! Git change kinds, the change list, patch text, and the tree's marks (Scouter `src/diff/`). Pure: `services/git.rs`
//! runs git and hands the output here.

mod change;
mod marks;
mod patch;
#[cfg(test)]
mod tests;

pub use change::{
    Change, ChangeKind, ChangeSet, LineCounts, MAX_CHANGES, change_set, order, parse_name_status, parse_numstat,
    split_z,
};
pub use marks::{Listing, Mark, Marks};
pub use patch::utf16_len;
pub use patch::{ALL_LIMIT, FilePatch, PATCH_LIMIT, Patch, binary, from_diff, from_file, not_shown, split, too_large};
