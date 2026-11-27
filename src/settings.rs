//! The settings file's values, read from its text, and the text a save writes (`architecture.md`, "fmc writes its
//! settings.json, atomically"). The binary reads and writes the file and passes its text here; nothing here touches the disk.

use serde_json::{Map, Value};

use crate::find::Matching;
use crate::reader::associations::Associations;
use crate::theme::ModeChoice;
use crate::theme::catalog::SCOUTER;

pub const AUTO_PREVIEW: bool = true;
pub const SIDEBAR_JUMP: u32 = 2;
pub const READER_JUMP: u32 = 5;
pub const GIT_GUTTER: bool = true;
pub const FIND_MODE: Matching = Matching::Substring;
pub const THEME_MODE: ModeChoice = ModeChoice::Auto;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub auto_preview: bool,
    pub sidebar_jump: u32,
    pub reader_jump: u32,
    pub git_gutter: bool,
    pub file_associations: Associations,
    pub find_mode: Matching,
    pub theme_mode: ModeChoice,
    /// A catalog id for the dark slot; `scouter` is Scouter's pair (`theme::catalog`). Checked there, not here.
    pub dark_theme: String,
    pub light_theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_preview: AUTO_PREVIEW,
            sidebar_jump: SIDEBAR_JUMP,
            reader_jump: READER_JUMP,
            git_gutter: GIT_GUTTER,
            file_associations: Associations::default(),
            find_mode: FIND_MODE,
            theme_mode: THEME_MODE,
            dark_theme: SCOUTER.into(),
            light_theme: SCOUTER.into(),
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
        find_mode: finding(&object, notices),
        theme_mode: theme_mode(&object, notices),
        dark_theme: theme_id(&object, "dark_theme", notices),
        light_theme: theme_id(&object, "light_theme", notices),
    };
    parsed
}

/// The file's text with `changes` set, for a save (`services::settings::save`). Keys the change does not name keep their
/// values, but serde_json keeps no file order, so a save rewrites the keys sorted. A text that is not a JSON object is
/// refused, so the caller leaves the file as it is.
pub fn merged(text: &str, changes: &[(&str, Value)]) -> Result<String, String> {
    let mut object = match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(object)) => object,
        Ok(_) => return Err("settings.json must hold a JSON object; nothing was saved.".into()),
        Err(error) => {
            return Err(format!("settings.json is not valid JSON (line {}); nothing was saved.", error.line()));
        }
    };
    for (key, value) in changes {
        object.insert((*key).into(), value.clone());
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(object)).map_err(|error| error.to_string())?;
    text.push('\n');
    Ok(text)
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

/// `find_mode`: the name matching the finder starts with (plan-00006 D6). Any other value keeps Scouter's substring match
/// and says so, as the other keys do.
fn finding(object: &Map<String, Value>, notices: &mut Vec<String>) -> Matching {
    match object.get("find_mode") {
        None => FIND_MODE,
        Some(value) => value.as_str().and_then(Matching::named).unwrap_or_else(|| {
            notices.push("settings.json: find_mode must be \"substring\" or \"fuzzy\"; \"substring\" is used.".into());
            FIND_MODE
        }),
    }
}

/// `theme_mode`: `auto` follows the terminal, `dark` and `light` force a mode (plan-00015, A7).
fn theme_mode(object: &Map<String, Value>, notices: &mut Vec<String>) -> ModeChoice {
    match object.get("theme_mode") {
        None => THEME_MODE,
        Some(value) => value.as_str().and_then(ModeChoice::named).unwrap_or_else(|| {
            notices
                .push("settings.json: theme_mode must be \"auto\", \"dark\", or \"light\"; \"auto\" is used.".into());
            THEME_MODE
        }),
    }
}

/// A theme id is any string here; `theme::catalog::choose` checks it against the catalog and its slot.
fn theme_id(object: &Map<String, Value>, key: &str, notices: &mut Vec<String>) -> String {
    match object.get(key) {
        None => SCOUTER.into(),
        Some(Value::String(id)) => id.clone(),
        Some(_) => {
            notices.push(format!("settings.json: {key} must be a theme id, a string; \"{SCOUTER}\" is used."));
            SCOUTER.into()
        }
    }
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
