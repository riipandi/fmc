//! The core of fmc: pure logic with no I/O and no `opentui` (`.agents/docs/rust.md`, "Layout rules").
//! The binary (`main.rs`) adds the services, the app loop, and the views on top.

pub mod diff;
pub mod explorer;
pub mod find;
pub mod keys;
pub mod reader;
pub mod text;
pub mod theme;
