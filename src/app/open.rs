//! Opens the selected row, the shown file, or the browsed folder in another app: `o` hands the target to
//! macOS's default app, `O` offers the apps that open it (Open With), `e` opens the folder in the editor the settings
//! name (Finder when none is set), and `E` reveals the target in Finder. The launch runs on a worker
//! (`services/launch.rs`); this file picks the target, builds the picker's rows, and words the answer.

use std::ffi::OsString;

use fmc::overlay::{Item, Menu, Overlay};
use fmc::text::display_label;

use super::write::Task;
use super::*;
use crate::services::launch::{self, How};
use crate::ui::menu::MAX_ROWS;

/// The rows of an Open With picker and the file they open. The menu's task keeps it until a row is picked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picker {
    pub target: PathBuf,
    pub choices: Vec<Choice>,
}

/// One row of the picker: an app by the value `open -a` takes, or Finder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    App { value: String, name: String, default: bool },
    Finder,
}

/// Rows a menu keeps clear of the screen's edge, so a menu does not fill it.
const MENU_CHROME: usize = 8;

/// The rows a menu shows at once on a screen of `rows`: the pool, or fewer on a short screen. A longer
/// menu scrolls.
pub fn shown_rows(rows: u16) -> usize {
    MAX_ROWS.min(usize::from(rows).saturating_sub(MENU_CHROME)).max(1)
}

/// `o`, and the menu's Open row: the target goes to its default app. A folder opens in Finder.
pub fn default_app(m: &mut Model) {
    let Some(target) = target(m) else { return };
    let done = format!("Opened {}.", write::label(&target));
    spawn_open(m, &target, How::Default, done);
}

/// `O`, and the menu's Open With row: the picker lists the apps for the target. The list is read on a worker, and
/// the picker opens when it lands.
pub fn with(m: &mut Model) {
    let Some(target) = target(m) else { return };
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let worker = target.clone();
    crate::services::spawn(&m.proxy, move || {
        let result = list(&worker, &home);
        Event::Apps { target: worker, result }
    });
}

/// The picker's list (`Event::Apps`). The picker opens when `target` is still the target, in this order: the apps the
/// `open_with` pattern names, the apps macOS lists minus those, then Finder.
pub fn listed(m: &mut Model, target: &Path, result: &Result<Vec<launch::App>, String>) {
    if self::target(m).as_deref() != Some(target) {
        return;
    }
    let apps: &[launch::App] = match result {
        Ok(apps) => apps,
        Err(text) => {
            m.message = Some(Message { text: text.clone(), error: true });
            &[]
        }
    };
    let values = m.settings.values.open_with.winner_of(target, &m.opened).cloned().unwrap_or_default();
    let configured: Vec<Choice> = values
        .iter()
        .map(|value| Choice::App { value: value.clone(), name: display_label(app_name(value)), default: false })
        .collect();
    let listed: Vec<Choice> = apps
        .iter()
        .filter(|app| app.name != "Finder" && !values.iter().any(|value| same(value, app)))
        .map(|app| Choice::App {
            value: app.path.to_string_lossy().into_owned(),
            name: app.name.clone(),
            default: app.default,
        })
        .collect();
    let groups = [configured, listed, vec![Choice::Finder]];
    let mut items = Vec::new();
    let mut choices = Vec::new();
    for rows in &groups {
        for (index, choice) in rows.iter().enumerate() {
            // A rule opens a group only when rows came before it: an empty configured group leads with no rule.
            let rule = index == 0 && !items.is_empty();
            items.push(Item { label: label_of(choice), enabled: true, rule, ..Item::default() });
            choices.push(choice.clone());
        }
    }
    let shown = shown_rows(m.rows);
    m.overlay_at = write::anchor(m);
    m.overlay_task = Some(Task::Open(Picker { target: target.to_path_buf(), choices }));
    m.overlay = Some(Overlay::Menu(Menu::scrolled(items, shown)));
}

/// Carries out the picker row `index` (`write::pick`): the target goes to the chosen app, or to Finder. Finder
/// reveals a file and opens a folder.
pub fn run(m: &Model, picker: &Picker, index: usize) {
    let Some(choice) = picker.choices.get(index) else { return };
    let target = picker.target.as_path();
    let label = write::label(target);
    match choice {
        Choice::App { value, name, .. } => {
            spawn_open(m, target, How::App(value), format!("Opened {label} in {name}."));
        }
        Choice::Finder if target.is_dir() => spawn_open(m, target, How::Default, format!("Opened {label} in Finder.")),
        Choice::Finder => spawn_open(m, target, How::Reveal, format!("Showed {label} in Finder.")),
    }
}

/// `E`, and the menu's Show in Finder row: the target is selected in its folder.
pub fn reveal(m: &mut Model) {
    let Some(target) = target(m) else { return };
    let done = format!("Showed {} in Finder.", write::label(&target));
    spawn_open(m, &target, How::Reveal, done);
}

/// `e`, and the menu's Open Folder in Editor row: the browsed folder goes to the editor the settings name, or to Finder
/// when none is set.
pub fn editor(m: &mut Model) {
    let root = m.browser.tree.root().to_path_buf();
    let label = write::label(&root);
    let editor = m.settings.values.editor.clone();
    if editor.is_empty() {
        spawn_open(m, &root, How::Default, format!("Opened {label}."));
    } else {
        let app = display_label(app_name(&editor));
        spawn_open(m, &root, How::App(&editor), format!("Opened {label} in {app}."));
    }
}

/// The answer of a launch (`Event::Launched`): a success shows `done`, and a failure is `open`'s first error line.
pub fn launched(m: &mut Model, done: &str, result: &Result<(), String>) {
    m.message = Some(match result {
        Ok(()) => Message { text: done.to_owned(), error: false },
        Err(text) => Message { text: text.clone(), error: true },
    });
}

/// The target of `o`, `O`, and `E`: the selected row in the tree, or the shown file in the reader. The key page is
/// a reader with no file, so it has no target.
fn target(m: &Model) -> Option<PathBuf> {
    match m.focus {
        Focus::List => m.browser.selected_entry().map(|entry| entry.path),
        Focus::Reader => m.shown.path().map(Path::to_path_buf),
    }
}

/// The app's name as the footer shows it: `Zed` for `Zed` and for `/Applications/Zed.app`.
fn app_name(app: &str) -> &str {
    let file = app.rsplit('/').next().unwrap_or(app);
    file.strip_suffix(".app").unwrap_or(file)
}

/// Whether a configured value names `app`: its `.app` path, or, for a name, its name.
fn same(value: &str, app: &launch::App) -> bool {
    if value.contains('/') { Path::new(value) == app.path } else { value == app.name }
}

/// A picker row's label: the default app is marked, as the list marks it.
fn label_of(choice: &Choice) -> String {
    match choice {
        Choice::App { name, default: true, .. } => format!("{name} (default)"),
        Choice::App { name, .. } => name.clone(),
        Choice::Finder => "Finder".into(),
    }
}

/// Hands `path` to `open` as `how` says, on a worker. `done` is what the footer says if the launch works.
fn spawn_open(m: &Model, path: &Path, how: How, done: String) {
    let args = launch::open_args(path, how);
    crate::services::spawn(&m.proxy, move || Event::Launched { done, result: launch(&args) });
}

/// Lists the apps for `target`. The tests stand in for it (`list` below).
#[cfg(not(test))]
fn list(target: &Path, home: &Path) -> Result<Vec<launch::App>, String> {
    launch::list_apps(target, home)
}

/// The test stand-in for the list: the same three apps for every target, twenty when the name holds `Crowded` (so the
/// picker scrolls), none when it holds `Silent`, and a failure when it holds `Unlisted`.
#[cfg(test)]
fn list(target: &Path, _home: &Path) -> Result<Vec<launch::App>, String> {
    let name = target.file_name().unwrap_or_default().to_string_lossy().into_owned();
    if name.contains("Unlisted") {
        return Err("The apps could not be listed.".into());
    }
    if name.contains("Silent") {
        return Ok(Vec::new());
    }
    if name.contains("Crowded") {
        return Ok((0..20)
            .map(|n| app(&format!("Marauder {n:02}"), &format!("/Applications/Marauder{n:02}.app"), false))
            .collect());
    }
    Ok(vec![
        app("Pensieve", "/Applications/Pensieve.app", true),
        app("Zed", "/Applications/Zed.app", false),
        app("Finder", "/System/Library/CoreServices/Finder.app", false),
    ])
}

#[cfg(test)]
fn app(name: &str, path: &str, default: bool) -> launch::App {
    launch::App { name: name.into(), path: path.into(), default }
}

/// Hands the arguments to `open`. The tests stand in for it, so a test never launches an app (`write.rs`, `trash`).
#[cfg(not(test))]
fn launch(args: &[OsString]) -> Result<(), String> {
    launch::run_open(args)
}

/// The test stand-in: the target must exist, as `open` requires, and no app opens.
#[cfg(test)]
fn launch(args: &[OsString]) -> Result<(), String> {
    let target = args.last().map(std::path::PathBuf::from).unwrap_or_default();
    if target.exists() {
        Ok(())
    } else {
        Err(format!("The file {} couldn't be opened because there is no such file.", target.display()))
    }
}

#[cfg(test)]
#[path = "open_tests.rs"]
mod tests;
