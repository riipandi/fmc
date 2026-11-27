//! The Settings page through the model: its keys, each change applied and saved, the refusal and the revert, and the
//! number dialog. Every save goes to a temporary config folder (A12), so no test touches the owner's `~/.config/fmc`.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::time::Duration;

use fmc::settings::Row;
use fmc::theme::ModeChoice;
use opentui::input::{KeyCode, KeyEvent, Modifiers};
use opentui::testing::TestApp;
use serde_json::Value;
use tempfile::TempDir;

use super::*;

/// The page on a 120 by 40 screen, with the settings file at `file` in a temporary config folder. The page's events are
/// wired as `run` wires them, so a save's answer reaches the model.
struct Paged {
    app: TestApp,
    model: Rc<RefCell<Model>>,
    ui: Rc<RefCell<Ui>>,
    file: PathBuf,
    _dir: TempDir,
}

fn paged(text: Option<&str>) -> Paged {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let file = root.join("config").join("fmc").join("settings.json");
    if let Some(text) = text {
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, text).unwrap();
    }
    let app = TestApp::new(120, 40);
    let model = Rc::new(RefCell::new(new_model(app.app(), &root, root.clone(), None)));
    model.borrow_mut().settings.path = Some(file.clone());
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    capture::wire(app.app(), &model, &ui);
    overlay::wire_views(&model, &ui);
    {
        let (model, ui) = (model.clone(), ui.clone());
        app.app().on_message(move |cx, event: &Event| {
            {
                let mut m = model.borrow_mut();
                let mut u = ui.borrow_mut();
                super::event(cx.app(), &mut m, &mut u, event);
            }
            paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
        });
    }
    let paged = Paged { app, model, ui, file, _dir: dir };
    paint(paged.app.app(), &mut paged.model.borrow_mut(), &mut paged.ui.borrow_mut());
    paged.app.render();
    paged
}

impl Paged {
    /// Ctrl+, from the keyboard: the kitty protocol's key, as the parser sees it.
    fn open(&self) {
        self.app.key(KeyEvent::new(KeyCode::Char(','), Modifiers::CTRL));
        assert!(self.model.borrow().settings.page.is_some(), "Ctrl+, opens the page");
    }

    fn selected(&self) -> usize {
        self.model.borrow().settings.page.as_ref().map_or(usize::MAX, |page| page.selected)
    }

    fn dialog(&self) -> Option<Dialog> {
        match &self.model.borrow().overlay {
            Some(Overlay::Dialog(dialog)) => Some(dialog.clone()),
            _ => None,
        }
    }

    fn footer(&self) -> Option<String> {
        self.model.borrow().message.as_ref().map(|message| message.text.clone())
    }

    /// The file's text, or `None` when it is missing.
    fn text(&self) -> Option<String> {
        fs::read_to_string(&self.file).ok()
    }

    /// The file's value for `key`, once it is there.
    fn saved(&self, key: &str) -> Option<Value> {
        let value: Value = serde_json::from_str(&self.text()?).ok()?;
        value.get(key).cloned()
    }

    /// Waits until the file holds `key` as `want`: a save is written on the worker.
    fn await_saved(&self, key: &str, want: Value) {
        let landed = self.app.wait_until(Duration::from_secs(5), || self.saved(key).as_ref() == Some(&want));
        assert!(landed, "{key} is {want} in the file, not {:?}", self.text());
    }

    /// Waits until the footer shows a refusal: the worker's answer has come back.
    fn await_footer(&self, text: &str) {
        let shown = self.app.wait_until(Duration::from_secs(5), || self.footer().is_some_and(|f| f.contains(text)));
        assert!(shown, "the footer shows {text:?}, not {:?}", self.footer());
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
    }

    fn value(&self, row: Row) -> String {
        self.model.borrow().settings.values.shown(row)
    }
}

#[test]
fn ctrl_comma_opens_the_page_with_the_reader_focus_and_escape_gives_the_focus_back() {
    let p = paged(None);
    p.open();
    assert_eq!(p.model.borrow().focus, Focus::Reader);
    assert_eq!(p.selected(), 0);
    p.app.press("escape");
    assert!(p.model.borrow().settings.page.is_none(), "Esc closes the page");
    assert_eq!(p.model.borrow().focus, Focus::List, "the focus goes back to the list");
}

#[test]
fn the_page_draws_each_row_with_its_value_and_the_help_of_the_selected_row() {
    let p = paged(None);
    p.open();
    p.app.render();
    let screen = p.app.text();
    for row in Row::ALL {
        assert!(screen.contains(row.title()), "the page lacks {}", row.title());
    }
    for heading in ["BROWSING", "APPEARANCE"] {
        assert!(screen.contains(heading), "the page lacks the {heading} heading");
    }
    assert!(screen.contains("● on"), "an on-or-off row shows its dot");
    assert!(screen.contains("‹ auto ›"), "a choice shows between its arrows");
    assert!(screen.contains("settings.json  auto_preview"), "the selected row's key is named");
    assert!(screen.contains(Row::AutoPreview.help()), "the help of the selected row is drawn");
}

#[test]
fn the_page_takes_every_key_so_the_page_under_it_does_not_act() {
    let p = paged(None);
    p.open();
    p.app.press("d");
    assert!(p.model.borrow().changes.is_none(), "d does not open the changes screen under the page");
    assert!(p.model.borrow().settings.page.is_some());
}

#[test]
fn up_and_down_move_the_selection_within_the_rows_and_q_closes() {
    let p = paged(None);
    p.open();
    p.app.press("down");
    p.app.press("j");
    assert_eq!(p.selected(), 2);
    p.app.press("k");
    p.app.press("up");
    p.app.press("up");
    p.app.press("up");
    assert_eq!(p.selected(), 0, "the selection stops at the first row");
    for _ in 0..10 {
        p.app.press("down");
    }
    assert_eq!(p.selected(), Row::ALL.len() - 1, "the selection stops at the last row");
    p.app.press("q");
    assert!(p.model.borrow().settings.page.is_none());
}

#[test]
fn space_flips_a_toggle_applies_it_at_once_and_saves_it_keeping_the_other_keys() {
    let p = paged(Some(r#"{"reader_jump": 8}"#));
    p.open();
    p.app.press("space");
    assert!(!p.model.borrow().auto_preview, "the app reads the new value at once");
    assert_eq!(p.value(Row::AutoPreview), "false");
    p.await_saved("auto_preview", Value::Bool(false));
    assert_eq!(p.text().unwrap(), "{\n  \"auto_preview\": false,\n  \"reader_jump\": 8\n}\n");
}

#[test]
fn a_change_creates_a_missing_file_with_only_its_key() {
    let p = paged(None);
    p.open();
    p.app.press("space");
    p.await_saved("auto_preview", Value::Bool(false));
    assert_eq!(p.text().unwrap(), "{\n  \"auto_preview\": false\n}\n");
}

#[test]
fn a_file_that_does_not_parse_is_kept_and_the_value_goes_back_with_a_notice() {
    let bytes = "{ \"auto_preview\": true,, }";
    let p = paged(Some(bytes));
    p.open();
    p.app.press("space");
    p.await_footer("Not saved: settings.json is not valid JSON (line 1); nothing was saved. The value went back.");
    assert_eq!(p.text().unwrap(), bytes, "the bytes are untouched");
    assert!(p.model.borrow().auto_preview, "the app reads the old value again");
    assert_eq!(p.value(Row::AutoPreview), "true");
}

#[test]
fn a_symlinked_settings_file_is_written_through_and_the_link_stays() {
    let p = paged(None);
    let target = p.file.parent().unwrap().parent().unwrap().join("dotfiles").join("settings.json");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "{}\n").unwrap();
    fs::create_dir_all(p.file.parent().unwrap()).unwrap();
    symlink(&target, &p.file).unwrap();
    p.open();
    p.app.press("space");
    p.await_saved("auto_preview", Value::Bool(false));
    assert!(fs::symlink_metadata(&p.file).unwrap().file_type().is_symlink(), "the link stays");
    assert_eq!(fs::read_to_string(&target).unwrap(), "{\n  \"auto_preview\": false\n}\n");
}

#[test]
fn a_missing_config_folder_refuses_the_save_and_the_value_goes_back() {
    let p = paged(None);
    p.model.borrow_mut().settings.path = None;
    p.open();
    p.app.press("space");
    p.await_footer("has no folder to be saved in");
    assert!(p.model.borrow().auto_preview, "the value went back");
}

#[test]
fn a_theme_row_chooses_the_mode_at_once_and_saves_it() {
    let p = paged(None);
    p.open();
    for _ in 0..5 {
        p.app.press("down");
    }
    assert_eq!(p.selected(), 5, "theme_mode is the sixth row");
    p.app.press("space");
    assert_eq!(p.model.borrow().choice, ModeChoice::Dark, "the forced mode applies at once");
    p.await_saved("theme_mode", Value::from("dark"));
    p.app.press("right");
    p.app.press("right");
    assert_eq!(p.value(Row::ThemeMode), "light", "Right steps to the next choice");
    p.app.press("right");
    assert_eq!(p.value(Row::ThemeMode), "light", "the last choice stops");
}

#[test]
fn two_changes_in_a_row_both_land_in_the_file() {
    let p = paged(None);
    p.open();
    p.app.press("space");
    for _ in 0..3 {
        p.app.press("down");
    }
    p.app.press("space");
    p.await_saved("auto_preview", Value::Bool(false));
    p.await_saved("git_gutter", Value::Bool(false));
    assert_eq!(p.text().unwrap(), "{\n  \"auto_preview\": false,\n  \"git_gutter\": false\n}\n", "one write at a time");
}

#[test]
fn enter_on_a_number_opens_its_dialog_with_the_value() {
    let p = paged(None);
    p.open();
    p.app.press("down");
    p.app.press("return");
    let dialog = p.dialog().expect("the number's dialog opens");
    assert_eq!(dialog.title, "sidebar_jump");
    assert!(dialog.field, "the value is typed in the dialog's field");
    assert_eq!(p.ui.borrow().overlay.dialog.field.value(), "2", "the field holds the value");
    assert!(matches!(p.model.borrow().overlay_task, Some(write::Task::Setting(Row::SidebarJump))));
    assert_eq!(p.model.borrow().sidebar_jump, 2, "nothing changes until the dialog is answered");
}

#[test]
fn a_number_that_is_not_a_whole_number_of_one_or_more_keeps_the_value_and_the_dialog() {
    let p = paged(None);
    p.open();
    p.app.press("down");
    p.app.press("return");
    p.ui.borrow().overlay.dialog.field.set_value("0");
    p.app.press("return");
    let dialog = p.dialog().expect("the dialog stays open");
    assert_eq!(dialog.error.as_deref(), Some("Type a whole number of 1 or more."));
    assert_eq!(p.model.borrow().sidebar_jump, 2, "the value is kept");
    assert!(p.text().is_none(), "nothing is written");
}

#[test]
fn a_typed_number_saves_and_applies_to_the_list() {
    let p = paged(None);
    p.open();
    p.app.press("down");
    p.app.press("return");
    p.ui.borrow().overlay.dialog.field.set_value(" 7 ");
    p.app.press("return");
    assert!(p.dialog().is_none(), "a valid number closes the dialog");
    assert_eq!(p.model.borrow().sidebar_jump, 7, "the list moves by the new count");
    p.await_saved("sidebar_jump", Value::from(7));
}

#[test]
fn a_click_on_the_list_behind_the_page_does_nothing() {
    let p = paged(None);
    p.open();
    p.app.click(5, LIST_TOP);
    assert!(p.model.borrow().settings.page.is_some(), "the page stays open");
    assert_eq!(p.model.borrow().focus, Focus::Reader, "the list behind the page does not take the focus");
}
