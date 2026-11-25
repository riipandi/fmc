//! Reader logic (Scouter `src/reader/`, `src/core/files.ts`): what a document is once its bytes are read, which
//! filetypes are highlighted, and the footer's scroll position. Motions, jump history, and search follow.

pub mod document;
pub mod language;
pub mod position;
