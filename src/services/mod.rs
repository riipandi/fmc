//! I/O workers. Each one runs off the UI thread and returns values; the app sends them back as events
//! (`rust.md`, "Layout rules").

// Nothing calls the workers until the tree view lands (plan-00001 W2, phase 10).
#[cfg_attr(not(test), expect(dead_code, reason = "the tree view of plan-00001 phase 10 calls the listing"))]
pub mod fs;
