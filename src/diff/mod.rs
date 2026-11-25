//! Git change kinds and the tree's marks (Scouter `src/diff/`). Pure: `services/git.rs` runs git and hands the
//! output here.

mod marks;
#[cfg(test)]
mod tests;

pub use marks::{ChangeKind, Listing, MAX_CHANGES, Mark, Marks, parse_name_status, split_z};
