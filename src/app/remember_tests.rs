//! The remembered UI state through the keys: a toggle is written, and an app started from the file shows it; a burst of
//! changes is one write; a failed write is one notice; dotfiles survive `-`; the state wins over `settings.json` for the
//! finder's matching. Every file lives in a temporary folder, so no test touches the owner's `~/.config/fmc`.

use std::cell::Cell;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use fmc::find::Matching;
use fmc::reader::document::decode;
use fmc::settings::Row;
use fmc::state::{self, State};
use opentui::testing::TestApp;
use tempfile::TempDir;

use super::super::*;
use super::{Loaded, start};

/// A temporary home: `config/fmc` is the config folder, and `library` is the folder the app browses.
struct Home {
    _dir: TempDir,
    root: PathBuf,
}

fn home() -> Home {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let library = root.join("library");
    fs::create_dir_all(&library).unwrap();
    fs::write(library.join(".hogwarts"), "").unwrap();
    fs::write(library.join("notes.md"), NOTES).unwrap();
    Home { _dir: dir, root }
}

const NOTES: &str = "# Hogwarts\n\nThe *Sorting Hat* waits in the Great Hall.\n";

impl Home {
    fn config(&self) -> PathBuf {
        self.root.join("config").join("fmc")
    }

    fn state_file(&self) -> PathBuf {
        self.config().join("meta").join("state.json")
    }

    fn settings_file(&self) -> PathBuf {
        self.config().join("settings.json")
    }

    fn library(&self) -> PathBuf {
        self.root.join("library")
    }

    /// The state the file holds now.
    fn saved(&self) -> State {
        state::parse(&fs::read_to_string(self.state_file()).unwrap_or_default())
    }
}

/// A running app on a 120 by 40 screen, started as `run` starts it: the settings file, then the remembered state.
/// Its events are wired as `run` wires them, and it counts the state writes that answer.
struct Session {
    app: TestApp,
    model: Rc<RefCell<Model>>,
    ui: Rc<RefCell<Ui>>,
    writes: Rc<Cell<u32>>,
}

fn launch(home: &Home) -> Session {
    let app = TestApp::new(120, 40);
    let mut model = new_model(app.app(), home.library(), None);
    let settings = crate::services::settings::load(&home.settings_file());
    model.settings.values = settings.settings.clone();
    model.settings.path = Some(home.settings_file());
    model.matching = settings.settings.find_mode;
    let file = home.state_file();
    let state = crate::services::state::load(&file);
    start(&mut model, Loaded { path: Some(file), state });
    let model = Rc::new(RefCell::new(model));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    capture::wire(app.app(), &model, &ui);
    overlay::wire_views(&model, &ui);
    let writes = Rc::new(Cell::new(0));
    {
        let (model, ui, writes) = (model.clone(), ui.clone(), writes.clone());
        app.app().on_message(move |cx, event: &Event| {
            if matches!(event, Event::StateSaved { .. }) {
                writes.set(writes.get() + 1);
            }
            {
                let mut m = model.borrow_mut();
                let mut u = ui.borrow_mut();
                settings::event(cx.app(), &mut m, &mut u, event);
            }
            paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
        });
    }
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    Session { app, model, ui, writes }
}

impl Session {
    fn press(&self, shortcut: &str) {
        self.app.press(shortcut);
    }

    /// Runs the timers past the debounce, then waits until `count` writes have answered.
    fn written(&self, count: u32) {
        self.app.advance(Duration::from_secs(2));
        let landed = self.app.wait_until(Duration::from_secs(5), || self.writes.get() >= count);
        assert!(landed, "{count} writes answered, not {}", self.writes.get());
    }

    fn footer(&self) -> Option<String> {
        self.model.borrow().message.as_ref().map(|message| message.text.clone())
    }

    /// Shows the library's `notes.md` in the reader, as an open does.
    fn open_notes(&self) {
        let root = self.model.borrow().browser.tree.root().to_path_buf();
        let path = root.join("notes.md");
        {
            // The reader shows only a file the tree lists, so the folder is listed first.
            let mut m = self.model.borrow_mut();
            m.browser.listed(&root, vec![fmc::explorer::Entry::new(&root, "notes.md", fmc::explorer::Kind::File)]);
            m.browser.select(&path);
        }
        let document = decode(NOTES.as_bytes()).expect("a text document");
        reader::show_text(&mut self.model.borrow_mut(), &self.ui.borrow(), &path, document, String::new());
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
        self.app.render();
    }
}

#[test]
fn nothing_is_written_at_the_start() {
    let home = home();
    let s = launch(&home);
    s.app.advance(Duration::from_secs(2));
    assert_eq!(s.writes.get(), 0);
    assert!(!home.state_file().exists(), "a start that changes nothing writes no file");
}

#[test]
fn wrap_survives_a_restart() {
    let home = home();
    let s = launch(&home);
    s.open_notes();
    s.press("w");
    assert!(s.model.borrow().wrap);
    s.written(1);
    drop(s);
    let s = launch(&home);
    assert!(s.model.borrow().wrap, "wrap is on again");
    assert_eq!(home.saved().wrap, Some(true));
}

#[test]
fn the_sidebar_width_survives_a_restart() {
    let home = home();
    let s = launch(&home);
    s.press("ctrl+right");
    s.press("ctrl+right");
    let width = s.model.borrow().sidebar;
    assert!(width.is_some(), "Ctrl+→ sets a width");
    s.written(1);
    drop(s);
    let s = launch(&home);
    assert_eq!(s.model.borrow().sidebar, width);
}

#[test]
fn dotfiles_survive_a_restart() {
    let home = home();
    let s = launch(&home);
    assert!(s.model.borrow().browser.dotfiles());
    s.press(".");
    assert!(!s.model.borrow().browser.dotfiles());
    s.written(1);
    drop(s);
    let s = launch(&home);
    assert!(!s.model.borrow().browser.dotfiles(), "dotfiles stay hidden");
}

#[test]
fn dotfiles_survive_going_up_a_folder() {
    let home = home();
    let s = launch(&home);
    s.press(".");
    s.press("-");
    assert_eq!(s.model.borrow().browser.tree.root(), home.root, "- went up to the folder above");
    assert!(!s.model.borrow().browser.dotfiles(), "a new tree keeps the choice");
}

#[test]
fn the_finder_matching_survives_a_restart() {
    let home = home();
    let s = launch(&home);
    s.press("/");
    s.press("ctrl+f");
    assert_eq!(s.model.borrow().matching, Matching::Fuzzy);
    s.written(1);
    drop(s);
    let s = launch(&home);
    assert_eq!(s.model.borrow().matching, Matching::Fuzzy);
}

/// A git repository in the library: `notes.md` is committed, then edited, so the changes screen lists it.
fn repository(home: &Home) {
    let root = home.library();
    let git = |args: &[&str]| {
        let status = Command::new("git").arg("-C").arg(&root).args(args).status().unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "dumbledore@hogwarts.test"]);
    git(&["config", "user.name", "Albus Dumbledore"]);
    git(&["add", "notes.md"]);
    git(&["commit", "-q", "-m", "The Sorting Hat"]);
    fs::write(root.join("notes.md"), format!("{NOTES}Gryffindor waits too.\n")).unwrap();
}

#[test]
fn the_changes_view_options_survive_a_restart() {
    let home = home();
    repository(&home);
    let s = launch(&home);
    s.press("d");
    assert!(s.app.wait_until(Duration::from_secs(5), || s.model.borrow().changes.is_some()), "the changes list");
    s.press("s");
    s.press("x");
    s.press("a");
    let before = {
        let m = s.model.borrow();
        (m.diff_options.split, m.diff_options.wide, m.diff_options.one_file)
    };
    assert_eq!(before, (true, true, false), "split, wide, and all files");
    s.written(1);
    drop(s);
    let s = launch(&home);
    let after = {
        let m = s.model.borrow();
        (m.diff_options.split, m.diff_options.wide, m.diff_options.one_file)
    };
    assert_eq!(after, before);
}

#[test]
fn a_burst_of_changes_is_one_write() {
    let home = home();
    let s = launch(&home);
    s.open_notes();
    s.press("w");
    s.app.advance(Duration::from_millis(300));
    s.press(".");
    s.app.advance(Duration::from_millis(300));
    s.press("ctrl+right");
    s.written(1);
    assert_eq!(s.writes.get(), 1, "three changes, one write");
    let saved = home.saved();
    assert_eq!((saved.wrap, saved.dotfiles), (Some(true), Some(false)));
    assert!(saved.sidebar_width.is_some());
}

#[test]
fn the_sidebar_visibility_is_not_saved() {
    let home = home();
    let s = launch(&home);
    s.open_notes();
    s.press("ctrl+/");
    assert!(!layout::shown(&s.model.borrow(), 120), "the sidebar is hidden by the choice");
    s.press("w");
    s.written(1);
    let text = fs::read_to_string(home.state_file()).unwrap();
    let keys: Vec<String> =
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text).unwrap().keys().cloned().collect();
    assert!(keys.iter().all(|key| state::KEYS.contains(&key.as_str())), "only remembered keys: {keys:?}");
    drop(s);
    let s = launch(&home);
    assert!(layout::shown(&s.model.borrow(), 120), "the choice is not kept: the sidebar shows again");
}

#[test]
fn a_failed_write_is_one_notice_for_the_run() {
    let home = home();
    let meta = home.config().join("meta");
    fs::create_dir_all(&meta).unwrap();
    fs::set_permissions(&meta, fs::Permissions::from_mode(0o555)).unwrap();
    let s = launch(&home);
    s.open_notes();
    s.press("w");
    s.written(1);
    let first = s.footer().expect("the first failure is shown");
    assert!(first.starts_with("meta/state.json could not be written: "), "{first}");
    s.model.borrow_mut().message = None;
    s.press(".");
    s.written(2);
    assert_eq!(s.writes.get(), 2, "the second write was answered");
    assert_eq!(s.footer(), None, "a later failure is not shown again");
    drop(s);
    fs::set_permissions(&meta, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn the_settings_file_seeds_a_fresh_state_and_the_state_wins_after() {
    let home = home();
    fs::create_dir_all(home.config()).unwrap();
    fs::write(home.settings_file(), r#"{"find_mode": "fuzzy"}"#).unwrap();
    let s = launch(&home);
    assert_eq!(s.model.borrow().matching, Matching::Fuzzy, "the setting starts a fresh state");
    s.open_notes();
    s.press("w");
    s.written(1);
    assert_eq!(home.saved().find_mode, Some(Matching::Fuzzy));
    fs::write(home.settings_file(), r#"{"find_mode": "substring"}"#).unwrap();
    drop(s);
    let s = launch(&home);
    assert_eq!(s.model.borrow().matching, Matching::Fuzzy, "the state wins over settings.json");
}

#[test]
fn the_settings_page_writes_the_finder_mode_to_the_state() {
    let home = home();
    let s = launch(&home);
    s.press("ctrl+,");
    let row = Row::ALL.iter().position(|row| *row == Row::FindMode).unwrap();
    for _ in 0..row {
        s.press("down");
    }
    s.press("right");
    assert_eq!(s.model.borrow().matching, Matching::Fuzzy);
    s.written(1);
    assert_eq!(home.saved().find_mode, Some(Matching::Fuzzy));
    drop(s);
    let s = launch(&home);
    assert_eq!(s.model.borrow().matching, Matching::Fuzzy, "the next run starts with the page's value");
}

#[test]
fn bad_values_read_as_their_defaults_with_no_notice() {
    let home = home();
    fs::create_dir_all(home.state_file().parent().unwrap()).unwrap();
    fs::write(home.state_file(), r#"{"wrap": "yes", "sidebar_width": "wide", "dotfiles": 0, "find_mode": "Fuzzy"}"#)
        .unwrap();
    let s = launch(&home);
    let m = s.model.borrow();
    assert!(!m.wrap);
    assert_eq!(m.sidebar, None);
    assert!(m.browser.dotfiles());
    assert_eq!(m.matching, Matching::Substring);
    assert_eq!(s.footer(), None);
}

#[test]
fn a_write_still_waiting_when_the_app_quits_is_made() {
    let home = home();
    let s = launch(&home);
    s.open_notes();
    s.press("w");
    super::flush(&s.model.borrow());
    assert_eq!(home.saved().wrap, Some(true), "the change is in the file before the timer");
    assert_eq!(s.writes.get(), 0, "the quit's write has no answer to count");
}
