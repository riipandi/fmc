//! The UI state fmc remembers across runs: the sidebar width, the reader's wrap, the changes view's options, the
//! dotfiles, and the finder's matching. The binary reads and writes `meta/state.json`; this module takes its text and
//! returns its text, as `settings.rs` does. A value the file cannot use reads as unset, with no notice, so the next
//! save replaces it.

use serde_json::{Map, Value, json};

use crate::find::Matching;

/// The remembered values. `None` is a value the file does not set: the app starts from its own default for it, and the
/// finder's matching starts from `settings.json`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub sidebar_width: Option<i64>,
    pub wrap: Option<bool>,
    pub diff_split: Option<bool>,
    pub diff_wide: Option<bool>,
    pub diff_all_files: Option<bool>,
    pub dotfiles: Option<bool>,
    pub find_mode: Option<Matching>,
}

/// The keys `parse` reads and `json` writes.
pub const KEYS: [&str; 7] =
    ["sidebar_width", "wrap", "diff_split", "diff_wide", "diff_all_files", "dotfiles", "find_mode"];

/// The state the file's text sets. A text that is not a JSON object sets nothing; a value of the wrong type or an unknown
/// name sets nothing for its key, and unknown keys are ignored.
pub fn parse(text: &str) -> State {
    let Ok(Value::Object(object)) = serde_json::from_str::<Value>(text) else { return State::default() };
    State {
        sidebar_width: object.get("sidebar_width").and_then(Value::as_i64),
        wrap: flag(&object, "wrap"),
        diff_split: flag(&object, "diff_split"),
        diff_wide: flag(&object, "diff_wide"),
        diff_all_files: flag(&object, "diff_all_files"),
        dotfiles: flag(&object, "dotfiles"),
        find_mode: object.get("find_mode").and_then(Value::as_str).and_then(Matching::named),
    }
}

fn flag(object: &Map<String, Value>, key: &str) -> Option<bool> {
    object.get(key).and_then(Value::as_bool)
}

/// The file's text for `state`: each set value under its key, in pretty JSON with a final newline.
pub fn json(state: &State) -> String {
    let mut object = Map::new();
    if let Some(width) = state.sidebar_width {
        object.insert("sidebar_width".into(), json!(width));
    }
    let flags = [
        ("wrap", state.wrap),
        ("diff_split", state.diff_split),
        ("diff_wide", state.diff_wide),
        ("diff_all_files", state.diff_all_files),
        ("dotfiles", state.dotfiles),
    ];
    for (key, value) in flags {
        if let Some(value) = value {
            object.insert(key.into(), json!(value));
        }
    }
    if let Some(mode) = state.find_mode {
        object.insert("find_mode".into(), json!(mode.name()));
    }
    // INVARIANT: a map of strings, numbers, and booleans always serializes.
    let text = serde_json::to_string_pretty(&Value::Object(object)).expect("a JSON object of plain values serializes");
    text + "\n"
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
