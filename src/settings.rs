//! The settings file's values, read from its text (`architecture.md`, "a read-only settings.json").
//! The binary reads the file and passes its text here; nothing in this module touches the disk.

use serde_json::{Map, Value};

use crate::reader::associations::Associations;

pub const AUTO_PREVIEW: bool = true;
pub const SIDEBAR_JUMP: u32 = 2;
pub const READER_JUMP: u32 = 5;
pub const GIT_GUTTER: bool = true;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub auto_preview: bool,
    pub sidebar_jump: u32,
    pub reader_jump: u32,
    pub git_gutter: bool,
    pub file_associations: Associations,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_preview: AUTO_PREVIEW,
            sidebar_jump: SIDEBAR_JUMP,
            reader_jump: READER_JUMP,
            git_gutter: GIT_GUTTER,
            file_associations: Associations::default(),
        }
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
        git_gutter: boolean(&object, "git_gutter", GIT_GUTTER, notices),
        file_associations: associations(&object, notices),
    };
    parsed
}

/// The `file_associations` object: each pattern's language. A bad entry is skipped with a notice that names it, so the
/// other entries still apply.
fn associations(object: &Map<String, Value>, notices: &mut Vec<String>) -> Associations {
    let mut table = Associations::default();
    match object.get("file_associations") {
        None => {}
        Some(Value::Object(entries)) => {
            for (pattern, value) in entries {
                let added = match value.as_str() {
                    Some(language) => table.add(pattern, language),
                    None => Err("its value must be a filetype name".into()),
                };
                if let Err(reason) = added {
                    notices.push(format!("settings.json: file_associations \"{pattern}\" is skipped: {reason}."));
                }
            }
        }
        Some(_) => notices.push("settings.json: file_associations must be an object; no association is used.".into()),
    }
    table
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
