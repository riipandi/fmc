//! The Settings page: `Ctrl+,` opens it over the content area, and each change applies and saves to
//! `settings.json` at once. The page takes every plain key while it is open, as a dialog does. The worker writes
//! one batch at a time, from the file as it is then, so a hand edit made since the start is kept and two saves never
//! race; a save that fails takes its changes back. The rows and their steps are `fmc::settings`; the drawing is
//! `ui/settings.rs`.

use std::path::PathBuf;

use fmc::keys::Key;
use fmc::overlay::{Dialog, Overlay};
use fmc::settings::{Row, Settings};
use fmc::theme::catalog::{self, Theme};
use serde_json::Value;

use super::*;
use crate::services;

/// What a number row's dialog says when its text is not a whole number of 1 or more.
const NUMBER_ERROR: &str = "Type a whole number of 1 or more.";

/// The page's state: the settings as the file has them, the user's themes, the page while it is open, and the saves.
#[derive(Default)]
pub struct Store {
    /// The settings after each applied change, whether or not its save has landed.
    pub values: Settings,
    /// The user's themes (`themes/`): the theme rows' choices and the palettes come from them.
    pub themes: Vec<Theme>,
    /// The page, while it is open.
    pub page: Option<Page>,
    /// The settings file (`services::settings::path`); `None` when there is no home folder to name it from.
    pub path: Option<PathBuf>,
    /// The batch on the worker, and the changes that wait for it.
    saving: Vec<Change>,
    queued: Vec<Change>,
}

/// The open page: its selected row, and the focus it replaced.
pub struct Page {
    pub selected: usize,
    restore: Focus,
}

/// One applied change: its row, the settings before it (a failed save takes it back), and the value the file gets.
struct Change {
    row: Row,
    before: Settings,
    json: Value,
}

/// Opens the page on its first row. It fills the content area, so the focus moves to the reader.
pub fn open(m: &mut Model) {
    if m.settings.page.is_none() {
        let restore = std::mem::replace(&mut m.focus, Focus::Reader);
        m.settings.page = Some(Page { selected: 0, restore });
    }
}

/// Closes the page and gives back the focus it replaced.
pub fn close(m: &mut Model) {
    if let Some(page) = m.settings.page.take() {
        m.focus = page.restore;
    }
}

/// A key while the page is open. Up and Down (`k`, `j`) move the selection; Enter and Space change the row, and a number
/// row opens its dialog; Left and Right (`h`, `l`) step a choice back and forward; Esc and `q` close the page.
pub fn key(app: &App, m: &mut Model, ui: &mut Ui, key: Key<'_>) {
    let Some(page) = m.settings.page.as_mut() else { return };
    if !key.plain() {
        return;
    }
    let Some(&row) = Row::ALL.get(page.selected) else { return };
    match key.name {
        "up" | "k" => page.selected = page.selected.saturating_sub(1),
        "down" | "j" => page.selected = (page.selected + 1).min(Row::ALL.len() - 1),
        "escape" | "q" => close(m),
        "return" | "space" if row.number() => open_number(m, ui, row),
        "return" | "space" | "right" | "l" => step(app, m, ui, row, true),
        "left" | "h" => step(app, m, ui, row, false),
        _ => {}
    }
}

/// Steps `row` one value, forward or back. A choice at its end does nothing, and a number row is typed, not stepped.
fn step(app: &App, m: &mut Model, ui: &mut Ui, row: Row, forward: bool) {
    let Some(next) = m.settings.values.step(row, forward, &m.settings.themes) else { return };
    if commit(m, row, next) {
        restyle(app, m, ui);
    }
}

/// Makes `next` the settings, applies `row` at once, and queues its save. Returns whether the colours changed, which
/// `restyle` repaints.
fn commit(m: &mut Model, row: Row, next: Settings) -> bool {
    let before = std::mem::replace(&mut m.settings.values, next);
    let json = m.settings.values.json(row);
    m.settings.queued.push(Change { row, before, json });
    let themed = apply(m, row);
    start(m);
    themed
}

/// Carries `row`'s value from the settings into the model the app reads. Returns whether the palettes changed.
fn apply(m: &mut Model, row: Row) -> bool {
    match row {
        Row::AutoPreview => m.auto_preview = m.settings.values.auto_preview,
        Row::SidebarJump => m.sidebar_jump = m.settings.values.sidebar_jump,
        Row::ReaderJump => m.reader_jump = m.settings.values.reader_jump,
        Row::GitGutter => m.git_gutter = m.settings.values.git_gutter,
        Row::FindMode => m.matching = m.settings.values.find_mode,
        Row::ThemeMode | Row::DarkTheme | Row::LightTheme => {
            let values = m.settings.values.clone();
            m.choice = values.theme_mode;
            // The page offers only ids the catalog holds for their slot, so `choose` adds no notice here.
            let chosen = catalog::choose(m.settings.themes.clone(), &values.dark_theme, &values.light_theme);
            (m.dark, m.light) = (chosen.dark, chosen.light);
            return true;
        }
    }
    false
}

/// Starts the worker on the queued changes, unless a batch is on it. The worker reads the file, sets the changes, and
/// writes it (`services::settings::update`); its answer is `Event::SettingsSaved`.
fn start(m: &mut Model) {
    if !m.settings.saving.is_empty() || m.settings.queued.is_empty() {
        return;
    }
    m.settings.saving = std::mem::take(&mut m.settings.queued);
    let changes: Vec<(&'static str, Value)> =
        m.settings.saving.iter().map(|change| (change.row.key(), change.json.clone())).collect();
    let path = m.settings.path.clone();
    services::spawn(&m.proxy, move || Event::SettingsSaved {
        result: match path {
            Some(path) => services::settings::update(&path, &changes),
            None => Err("settings.json has no folder to be saved in; nothing was saved.".into()),
        },
    });
}

/// The worker's answer. A written batch lets the queue go next. A refused or failed batch takes back its changes and
/// the queued ones, last first, so each row returns to the value it had before the first of them, and the footer says
/// why. The file is untouched by a refusal.
pub fn saved(app: &App, m: &mut Model, ui: &mut Ui, result: Result<(), String>) {
    match result {
        Ok(()) => {
            m.settings.saving.clear();
            start(m);
        }
        Err(reason) => {
            let batch = std::mem::take(&mut m.settings.saving);
            let queued = std::mem::take(&mut m.settings.queued);
            let mut themed = false;
            for change in batch.iter().chain(&queued).rev() {
                m.settings.values.take(change.row, &change.before);
                themed |= apply(m, change.row);
            }
            if themed {
                restyle(app, m, ui);
            }
            m.message = Some(Message { text: format!("Not saved: {reason} The value went back."), error: true });
        }
    }
}

/// Opens the dialog that types a number row's value, with the value in its field.
fn open_number(m: &mut Model, ui: &mut Ui, row: Row) {
    let message = format!("{} Type a whole number of 1 or more.", row.help());
    let dialog = Dialog::new(row.key(), &message, true, &["Cancel", "OK"], 1);
    ui.overlay.dialog.field.set_value(m.settings.values.shown(row));
    m.overlay = Some(Overlay::Dialog(dialog));
    m.overlay_task = Some(write::Task::Setting(row));
}

/// The number dialog's OK (`write::submit`). A whole number of 1 or more is saved, or nothing when it is the value the row
/// has; other text keeps the dialog open with its error.
pub fn submit(app: &App, m: &mut Model, row: Row, typed: &str) {
    match typed.trim().parse::<u32>() {
        Ok(value) if value >= 1 => {
            overlay::close(app, m);
            let next = m.settings.values.with_number(row, value);
            if next != m.settings.values {
                // A number row changes no colour, so the return value does not matter here.
                commit(m, row, next);
            }
        }
        _ => {
            if let Some(Overlay::Dialog(dialog)) = m.overlay.as_mut() {
                dialog.error = Some(NUMBER_ERROR.into());
            }
        }
    }
}

/// An event from a worker: the save's answer here, every other one to the tree (`tree::apply`).
pub fn event(app: &App, m: &mut Model, ui: &mut Ui, event: &Event) {
    match event {
        Event::SettingsSaved { result } => saved(app, m, ui, result.clone()),
        _ => tree::apply(m, ui, event),
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
