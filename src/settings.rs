//! The settings file's three values, read from its text (`architecture.md`, "a read-only settings.json with three keys").
//! The binary reads the file and passes its text here; nothing in this module touches the disk.

use serde_json::{Map, Value};

pub const AUTO_PREVIEW: bool = true;
pub const SIDEBAR_JUMP: u32 = 2;
pub const READER_JUMP: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub auto_preview: bool,
    pub sidebar_jump: u32,
    pub reader_jump: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { auto_preview: AUTO_PREVIEW, sidebar_jump: SIDEBAR_JUMP, reader_jump: READER_JUMP }
    }
}

/// The values a file sets, and one notice for each value it could not use.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    pub settings: Settings,
    pub notices: Vec<String>,
}

/// Reads the file's text. A file that is not a JSON object sets nothing, so every key keeps its default; a bad value
/// keeps only its own key's default, and unknown keys are ignored.
pub fn parse(text: &str) -> Parsed {
    let mut parsed = Parsed::default();
    let object = match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(object)) => object,
        Ok(_) => {
            parsed.notices.push("settings.json must hold a JSON object; every setting uses its default.".into());
            return parsed;
        }
        Err(error) => {
            let line = error.line();
            parsed
                .notices
                .push(format!("settings.json is not valid JSON (line {line}); every setting uses its default."));
            return parsed;
        }
    };
    let notices = &mut parsed.notices;
    parsed.settings = Settings {
        auto_preview: boolean(&object, "auto_preview", AUTO_PREVIEW, notices),
        sidebar_jump: whole(&object, "sidebar_jump", SIDEBAR_JUMP, notices),
        reader_jump: whole(&object, "reader_jump", READER_JUMP, notices),
    };
    parsed
}

fn boolean(object: &Map<String, Value>, key: &str, default: bool, notices: &mut Vec<String>) -> bool {
    match object.get(key) {
        None => default,
        Some(value) => value.as_bool().unwrap_or_else(|| {
            notices.push(format!("settings.json: {key} must be true or false; {default} is used."));
            default
        }),
    }
}

fn whole(object: &Map<String, Value>, key: &str, default: u32, notices: &mut Vec<String>) -> u32 {
    match object.get(key) {
        None => default,
        Some(value) => value.as_u64().and_then(|n| u32::try_from(n).ok()).filter(|&n| n >= 1).unwrap_or_else(|| {
            notices.push(format!("settings.json: {key} must be a positive whole number; {default} is used."));
            default
        }),
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
