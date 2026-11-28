//! The settings file's values, read from its text, and the text a save writes (`architecture.md`, "fmc writes its
//! settings.json, atomically"). The binary reads and writes the file and passes its text here; nothing here touches the disk.

use serde_json::{Map, Value, json};

use crate::find::Matching;
use crate::reader::associations::{Associations, Language, OpenWith};
use crate::text::display_label;
use crate::theme::catalog::{DEFAULT_THEME, Theme, slot_ids};
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
    /// An app's name or path that `e` opens the folder with (plan-00017, O3); empty opens the folder in Finder.
    pub editor: String,
    /// The apps the Open With picker offers for a file, by the pattern that wins for it (plan-00017, D8).
    pub open_with: OpenWith,
    pub find_mode: Matching,
    pub theme_mode: ModeChoice,
    /// A catalog id for the dark slot; `default` is Scouter's pair (`theme::catalog`). Checked there, not here.
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
            editor: String::new(),
            open_with: OpenWith::default(),
            find_mode: FIND_MODE,
            theme_mode: THEME_MODE,
            dark_theme: DEFAULT_THEME.into(),
            light_theme: DEFAULT_THEME.into(),
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
        editor: editor(&object, notices),
        open_with: open_with(&object, notices),
        find_mode: finding(&object, notices),
        theme_mode: theme_mode(&object, notices),
        dark_theme: theme_id(&object, "dark_theme", notices),
        light_theme: theme_id(&object, "light_theme", notices),
    };
    parsed
}

/// The keys `parse` reads: the Settings page's rows and `file_associations`. Every lookup goes through `field`, which
/// asserts its key is here, and the schema's properties are tested against this list, so a key cannot be parsed
/// without a schema property.
pub const KEYS: [&str; 11] = [
    "auto_preview",
    "sidebar_jump",
    "reader_jump",
    "git_gutter",
    "file_associations",
    "editor",
    "open_with",
    "find_mode",
    "theme_mode",
    "dark_theme",
    "light_theme",
];

fn field<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a Value> {
    debug_assert!(KEYS.contains(&key), "{key} is parsed but missing from settings::KEYS and the schema");
    object.get(key)
}

/// The `$schema` that points `settings.json` at the schema `services::settings::ensure_schema` writes beside it
/// (plan-00016 O1). Relative to `settings.json`, which editors resolve against the file's folder.
pub const SCHEMA_LINK: &str = "./meta/config.schema.json";

/// The file's text with `"$schema": SCHEMA_LINK` inserted as its first key, for the startup link (plan-00016 O1, amended
/// by the owner on 2026-11-28). The rest of the text keeps its bytes, so the values, their order, and the layout stay as
/// the owner wrote them. `None` when the file already has a `$schema` or is not a JSON object.
pub fn linked(text: &str) -> Option<String> {
    let Ok(Value::Object(mut object)) = serde_json::from_str::<Value>(text) else { return None };
    if object.contains_key("$schema") {
        return None;
    }
    // Strict JSON (plan-00015 D10): the first `{` opens the object, since only whitespace may come before it.
    let (head, rest) = text.split_at(text.find('{')? + 1);
    let body = rest.trim_start();
    let gap = &rest[..rest.len() - body.len()];
    let link = format!("\"$schema\": \"{SCHEMA_LINK}\"");
    let text =
        if object.is_empty() { format!("{head}\n  {link}\n{body}") } else { format!("{head}{gap}{link},{gap}{body}") };
    object.insert("$schema".into(), Value::from(SCHEMA_LINK));
    // INVARIANT: the insert adds one key and changes no value; a text that would parse otherwise is not written.
    (serde_json::from_str::<Value>(&text).ok() == Some(Value::Object(object))).then_some(text)
}

/// The file's text with `changes` set, for a save (`services::settings::update`). Keys the change does not name keep their
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
    // A `$schema` the owner wrote stays; only a file with none gets the link, so an editor completes the keys.
    object.entry("$schema").or_insert_with(|| Value::from(SCHEMA_LINK));
    for (key, value) in changes {
        object.insert((*key).into(), value.clone());
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(object)).map_err(|error| error.to_string())?;
    text.push('\n');
    Ok(text)
}

/// The JSON Schema of `settings.json` (draft-07), built from the same rows, defaults, and name lists the parser and the
/// Settings page use, so a name cannot drift from it (plan-00016, D2). Pretty-printed with a final newline; the same
/// bytes on every call.
pub fn schema() -> String {
    let defaults = Settings::default();
    let mut properties = Map::new();
    properties.insert(
        "$schema".into(),
        json!({ "type": "string", "description": "The schema this file is checked against. fmc writes it." }),
    );
    for row in Row::ALL {
        properties.insert(row.key().into(), row_property(row, &defaults));
    }
    properties.insert(
        "file_associations".into(),
        json!({
            "type": "object",
            "description": "Maps a file pattern, such as *.md or docs/**, to the filetype it is shown as.",
            "default": {},
            "additionalProperties": { "type": "string", "enum": Language::names() },
        }),
    );
    properties.insert(
        "editor".into(),
        json!({ "type": "string", "default": "", "description": "An app name or path, such as Zed or /Applications/Zed.app, that e opens the folder with. Empty opens the folder in Finder." }),
    );
    properties.insert(
        "open_with".into(),
        json!({
            "type": "object",
            "description": "Maps a file pattern, such as *.md, to the apps the Open With picker offers for a file that matches it: app names or .app paths.",
            "default": {},
            "additionalProperties": { "type": "array", "items": { "type": "string", "minLength": 1 } },
        }),
    );
    let root = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "fmc settings.json",
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
    });
    // INVARIANT: a Value built from literals and strings always serializes.
    let mut text = serde_json::to_string_pretty(&root).expect("a schema value always serializes");
    text.push('\n');
    text
}

fn row_property(row: Row, defaults: &Settings) -> Value {
    let (default, help) = (defaults.json(row), row.help());
    match row {
        Row::AutoPreview | Row::GitGutter => {
            json!({ "type": "boolean", "default": default, "description": help })
        }
        Row::SidebarJump | Row::ReaderJump => {
            json!({ "type": "integer", "minimum": 1, "maximum": u32::MAX, "default": default, "description": help })
        }
        Row::FindMode => {
            json!({ "type": "string", "enum": Matching::ALL.map(Matching::name), "default": default, "description": help })
        }
        Row::ThemeMode => {
            json!({ "type": "string", "enum": ModeChoice::ALL.map(ModeChoice::name), "default": default, "description": help })
        }
        Row::DarkTheme => theme_property(slot_ids(&[], Mode::Dark), default, help),
        Row::LightTheme => theme_property(slot_ids(&[], Mode::Light), default, help),
    }
}

/// A theme id completes from the slot's bundled ids and the default, but any string is accepted: a user's theme in
/// `themes/` is a valid id that the schema cannot list.
fn theme_property(ids: Vec<String>, default: Value, help: &str) -> Value {
    json!({
        "anyOf": [{ "type": "string", "enum": ids }, { "type": "string" }],
        "default": default,
        "description": help,
    })
}

/// The `file_associations` object: each pattern's language. A bad entry is skipped with a notice that names it, so the
/// other entries still apply.
fn associations(object: &Map<String, Value>, notices: &mut Vec<String>) -> Associations {
    let mut table = Associations::default();
    match field(object, "file_associations") {
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

/// `editor`: the app `e` opens the folder with, as `open -a` takes it. Empty, the default, opens the folder in Finder. Any
/// other value is kept as written; `open` reports an app it cannot find.
fn editor(object: &Map<String, Value>, notices: &mut Vec<String>) -> String {
    match field(object, "editor") {
        None => String::new(),
        Some(Value::String(app)) => app.clone(),
        Some(_) => {
            notices.push(
                "settings.json: editor must be an app name or path, a string; the folder opens in Finder.".into(),
            );
            String::new()
        }
    }
}

/// `open_with` (plan-00017, D8): the apps a pattern offers in the Open With picker. A bad entry is skipped with a notice that
/// names its pattern: a pattern that does not parse, a value that is not an array, or an item that is not a non-empty string.
fn open_with(object: &Map<String, Value>, notices: &mut Vec<String>) -> OpenWith {
    let mut table = OpenWith::default();
    match field(object, "open_with") {
        None => {}
        Some(Value::Object(entries)) => {
            for (pattern, value) in entries {
                let Some(items) = value.as_array() else {
                    notices.push(display_label(&format!(
                        "settings.json: open_with \"{pattern}\" is skipped: its value must be an array of app names."
                    )));
                    continue;
                };
                let mut apps = Vec::new();
                for item in items {
                    match item.as_str() {
                        Some(app) if !app.is_empty() => apps.push(app.to_owned()),
                        _ => notices.push(display_label(&format!(
                            "settings.json: open_with \"{pattern}\" has an item that is not a non-empty string; it is skipped."
                        ))),
                    }
                }
                if let Err(reason) = table.add_value(pattern, apps) {
                    notices
                        .push(display_label(&format!("settings.json: open_with \"{pattern}\" is skipped: {reason}.")));
                }
            }
        }
        Some(_) => notices.push("settings.json: open_with must be an object; no app is offered.".into()),
    }
    table
}

/// `find_mode`: the name matching the finder starts with (plan-00006 D6). Any other value keeps Scouter's substring match
/// and says so, as the other keys do.
fn finding(object: &Map<String, Value>, notices: &mut Vec<String>) -> Matching {
    match field(object, "find_mode") {
        None => FIND_MODE,
        Some(value) => value.as_str().and_then(Matching::named).unwrap_or_else(|| {
            notices.push("settings.json: find_mode must be \"substring\" or \"fuzzy\"; \"substring\" is used.".into());
            FIND_MODE
        }),
    }
}

/// `theme_mode`: `auto` follows the terminal, `dark` and `light` force a mode (plan-00015, A7).
fn theme_mode(object: &Map<String, Value>, notices: &mut Vec<String>) -> ModeChoice {
    match field(object, "theme_mode") {
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
    match field(object, key) {
        None => DEFAULT_THEME.into(),
        Some(Value::String(id)) => id.clone(),
        Some(_) => {
            notices.push(format!("settings.json: {key} must be a theme id, a string; \"{DEFAULT_THEME}\" is used."));
            DEFAULT_THEME.into()
        }
    }
}

fn boolean(object: &Map<String, Value>, key: &str, default: bool, notices: &mut Vec<String>) -> bool {
    match field(object, key) {
        None => default,
        Some(value) => value.as_bool().unwrap_or_else(|| {
            notices.push(format!("settings.json: {key} must be true or false; {default} is used."));
            default
        }),
    }
}

fn whole(object: &Map<String, Value>, key: &str, default: u32, notices: &mut Vec<String>) -> u32 {
    match field(object, key) {
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
            Row::DarkTheme => "The dark palette: default, or a dark theme.",
            Row::LightTheme => "The light palette: default, or a light theme.",
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
            Row::FindMode => next.find_mode = along(&Matching::ALL, &self.find_mode, forward)?,
            Row::ThemeMode => next.theme_mode = along(&ModeChoice::ALL, &self.theme_mode, forward)?,
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
