//! The settings file's values, read from its text, and the text a save writes (`architecture.md`, "fmc writes its
//! settings.json, atomically"). The binary reads and writes the file and passes its text here; nothing here touches the disk.

use serde_json::{Map, Value};

use crate::find::Matching;
use crate::reader::associations::Associations;
use crate::text::display_label;
use crate::theme::catalog::{SCOUTER, Theme, slot_ids};
use crate::theme::{Mode, ModeChoice};

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
                    // The pattern and the reason both hold the user's text, so the whole notice is one line.
                    let notice = format!("settings.json: file_associations \"{pattern}\" is skipped: {reason}.");
                    notices.push(display_label(&notice));
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

/// One row of the Settings page: a key of the file the page edits (plan-00015, D2 and A7). The label is the key's word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    AutoPreview,
    SidebarJump,
    ReaderJump,
    GitGutter,
    FindMode,
    ThemeMode,
    DarkTheme,
    LightTheme,
}

impl Row {
    /// The page's rows, top to bottom.
    pub const ALL: [Row; 8] = [
        Row::AutoPreview,
        Row::SidebarJump,
        Row::ReaderJump,
        Row::GitGutter,
        Row::FindMode,
        Row::ThemeMode,
        Row::DarkTheme,
        Row::LightTheme,
    ];

    /// The key the row edits in `settings.json`.
    pub fn key(self) -> &'static str {
        match self {
            Row::AutoPreview => "auto_preview",
            Row::SidebarJump => "sidebar_jump",
            Row::ReaderJump => "reader_jump",
            Row::GitGutter => "git_gutter",
            Row::FindMode => "find_mode",
            Row::ThemeMode => "theme_mode",
            Row::DarkTheme => "dark_theme",
            Row::LightTheme => "light_theme",
        }
    }

    /// The row's name on the page; the help line names its `settings.json` key.
    pub fn title(self) -> &'static str {
        match self {
            Row::AutoPreview => "Auto preview",
            Row::SidebarJump => "List jump",
            Row::ReaderJump => "Reader jump",
            Row::GitGutter => "Git gutter",
            Row::FindMode => "Find mode",
            Row::ThemeMode => "Theme mode",
            Row::DarkTheme => "Dark theme",
            Row::LightTheme => "Light theme",
        }
    }

    /// The group the row sits in on the page: the rows of a group are adjacent in `ALL`.
    pub fn section(self) -> &'static str {
        match self {
            Row::ThemeMode | Row::DarkTheme | Row::LightTheme => "Appearance",
            _ => "Browsing",
        }
    }

    /// A row whose value is on or off.
    pub fn toggle(self) -> bool {
        matches!(self, Row::AutoPreview | Row::GitGutter)
    }

    /// The row's one-line help, under the list while the row is selected.
    pub fn help(self) -> &'static str {
        match self {
            Row::AutoPreview => "Reader shows the file the list selects.",
            Row::SidebarJump => "Rows the list moves for ] and [.",
            Row::ReaderJump => "Lines the reader scrolls for ] and [.",
            Row::GitGutter => "Marks changed lines in a git repository.",
            Row::FindMode => "Finder matches substring words or fuzzy.",
            Row::ThemeMode => "Follow the terminal, or force dark or light.",
            Row::DarkTheme => "The dark palette: scouter, or a dark theme.",
            Row::LightTheme => "The light palette: scouter, or a light theme.",
        }
    }

    /// A number row opens a dialog to type its value; every other row changes in place.
    pub fn number(self) -> bool {
        matches!(self, Row::SidebarJump | Row::ReaderJump)
    }
}

impl Settings {
    /// The value as the page shows it: the file's own word for it.
    pub fn shown(&self, row: Row) -> String {
        match row {
            Row::AutoPreview => self.auto_preview.to_string(),
            Row::SidebarJump => self.sidebar_jump.to_string(),
            Row::ReaderJump => self.reader_jump.to_string(),
            Row::GitGutter => self.git_gutter.to_string(),
            Row::FindMode => self.find_mode.name().into(),
            Row::ThemeMode => self.theme_mode.name().into(),
            Row::DarkTheme => self.dark_theme.clone(),
            Row::LightTheme => self.light_theme.clone(),
        }
    }

    /// The value the file stores for the row.
    pub fn json(&self, row: Row) -> Value {
        match row {
            Row::AutoPreview => Value::Bool(self.auto_preview),
            Row::SidebarJump => Value::from(self.sidebar_jump),
            Row::ReaderJump => Value::from(self.reader_jump),
            Row::GitGutter => Value::Bool(self.git_gutter),
            Row::FindMode => Value::from(self.find_mode.name()),
            Row::ThemeMode => Value::from(self.theme_mode.name()),
            Row::DarkTheme => Value::from(self.dark_theme.as_str()),
            Row::LightTheme => Value::from(self.light_theme.as_str()),
        }
    }

    /// Copies one row's value from `from`. A save that fails takes its change back this way, and leaves the other rows.
    pub fn take(&mut self, row: Row, from: &Settings) {
        match row {
            Row::AutoPreview => self.auto_preview = from.auto_preview,
            Row::SidebarJump => self.sidebar_jump = from.sidebar_jump,
            Row::ReaderJump => self.reader_jump = from.reader_jump,
            Row::GitGutter => self.git_gutter = from.git_gutter,
            Row::FindMode => self.find_mode = from.find_mode,
            Row::ThemeMode => self.theme_mode = from.theme_mode,
            Row::DarkTheme => self.dark_theme.clone_from(&from.dark_theme),
            Row::LightTheme => self.light_theme.clone_from(&from.light_theme),
        }
    }

    /// The settings with a number row set to `value`; any other row is unchanged.
    pub fn with_number(&self, row: Row, value: u32) -> Self {
        let mut next = self.clone();
        match row {
            Row::SidebarJump => next.sidebar_jump = value,
            Row::ReaderJump => next.reader_jump = value,
            _ => {}
        }
        next
    }

    /// The settings after one step of `row`: a toggle flips, and a choice moves to the next value, or the previous one
    /// when `forward` is false. A choice does not wrap: `None` at its end, and for a number row, which is typed.
    /// `themes` are the user's themes, which join the theme rows' choices.
    pub fn step(&self, row: Row, forward: bool, themes: &[Theme]) -> Option<Self> {
        let mut next = self.clone();
        match row {
            Row::AutoPreview => next.auto_preview = !self.auto_preview,
            Row::GitGutter => next.git_gutter = !self.git_gutter,
            Row::SidebarJump | Row::ReaderJump => return None,
            Row::FindMode => {
                next.find_mode = along(&[Matching::Substring, Matching::Fuzzy], &self.find_mode, forward)?;
            }
            Row::ThemeMode => {
                next.theme_mode =
                    along(&[ModeChoice::Auto, ModeChoice::Dark, ModeChoice::Light], &self.theme_mode, forward)?;
            }
            Row::DarkTheme => {
                next.dark_theme = along(&slot_ids(themes, Mode::Dark), &self.dark_theme, forward)?;
            }
            Row::LightTheme => {
                next.light_theme = along(&slot_ids(themes, Mode::Light), &self.light_theme, forward)?;
            }
        }
        Some(next)
    }
}

/// The entry after `current` in `list`, or before it; `None` past either end. A current value the list does not hold
/// steps to the first entry, so a value the file set by hand still has a way out.
fn along<T: PartialEq + Clone>(list: &[T], current: &T, forward: bool) -> Option<T> {
    let next = match (list.iter().position(|item| item == current), forward) {
        (Some(at), true) => at + 1,
        (Some(at), false) => at.checked_sub(1)?,
        (None, _) => 0,
    };
    list.get(next).cloned()
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
