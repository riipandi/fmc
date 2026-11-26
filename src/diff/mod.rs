//! Git change kinds, the change list, patch text, and the tree's marks (Scouter `src/diff/`). Pure: `services/git.rs`
//! runs git and hands the output here.

mod change;
mod list;
mod marks;
mod nav;
mod patch;
mod row;
mod stack;
#[cfg(test)]
mod tests;

pub use change::{
    Change, ChangeKind, ChangeSet, LineCounts, MAX_CHANGES, change_set, order, parse_name_status, parse_numstat,
    split_z,
};
pub use list::ChangeList;
pub use marks::{Listing, Mark, Marks, within_folder};
pub use nav::{Folds, Hunk, Jump, SPLIT_MIN_COLUMNS, can_split, context, line_at, step};
pub use patch::utf16_len;
pub use patch::{ALL_LIMIT, FilePatch, PATCH_LIMIT, Patch, binary, from_diff, from_file, not_shown, split, too_large};
pub use row::{ListRow, empty_message, header_counts, header_title, index_mark, list_row, split_path, truncated_note};
pub use stack::{Totals, active_hunk, block_at, gutter_digits, hunk_starts, totals};
