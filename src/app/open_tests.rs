//! `o`, `e`, `E`, and the menu rows through the model: the selected row, the shown file, a folder, a failure's footer,
//! and the binary notice's line. The launcher is the test stand-in (`launch` in `open.rs`), so no test launches an app.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use opentui::testing::TestApp;
use tempfile::TempDir;

use fmc::overlay::Overlay;
use fmc::reader::associations::OpenWith;

use crate::app::reader::Shown;
use crate::ui::binary::OPEN_HINT;

use super::*;

struct Opening {
    app: TestApp,
    model: Rc<RefCell<Model>>,
    ui: Rc<RefCell<Ui>>,
    root: PathBuf,
    _dir: TempDir,
}

/// A folder that holds `names` (a name ending in `/` is a folder, one ending in `.bin` holds a NUL byte), browsed on a
/// 120 by 40 screen with its listing read and the first row selected. The app's worker events are wired.
fn opening(names: &[&str]) -> Opening {
    opening_sized(names, 120, 40)
}

/// `opening` on a screen of `columns` by `rows`.
fn opening_sized(names: &[&str], columns: u16, rows: u16) -> Opening {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    for name in names {
        match name.strip_suffix('/') {
            Some(folder) => fs::create_dir(root.join(folder)).unwrap(),
            None if name.ends_with(".bin") => fs::write(root.join(name), b"\0Hogwarts\n").unwrap(),
            None => fs::write(root.join(name), "Hogwarts\n").unwrap(),
        }
    }
    let app = TestApp::new(columns, rows);
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    wire_mouse(&model, &ui);
    capture::wire(app.app(), &model, &ui);
    overlay::wire_views(&model, &ui);
    {
        let (model, ui) = (model.clone(), ui.clone());
        app.app().on_message(move |cx, event: &Event| {
            {
                let mut m = model.borrow_mut();
                let u = ui.borrow();
                tree::apply(&mut m, &u, event);
            }
            paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
        });
    }
    let opening = Opening { app, model, ui, root: root.clone(), _dir: dir };
    let entries = crate::services::fs::list(&root, &[]).unwrap();
    {
        let mut m = opening.model.borrow_mut();
        m.browser.listed(&root, entries);
        let first = m.browser.tree.rows().first().map(|row| row.entry.path.clone());
        if let Some(first) = first {
            m.browser.select(&first);
        }
    }
    paint(opening.app.app(), &mut opening.model.borrow_mut(), &mut opening.ui.borrow_mut());
    opening.app.render();
    opening
}

impl Opening {
    /// Selects the row named `name`, which the listing holds.
    fn select(&self, name: &str) {
        self.model.borrow_mut().browser.select(&self.root.join(name));
    }

    fn footer(&self) -> Option<Message> {
        self.model.borrow().message.clone()
    }

    /// Waits until the footer shows `text`: a launch's answer comes back from the worker.
    fn await_footer(&self, text: &str) {
        let shown =
            self.app.wait_until(Duration::from_secs(5), || self.footer().is_some_and(|message| message.text == text));
        assert!(shown, "the footer shows {text:?}, not {:?}", self.footer().map(|message| message.text));
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
    }

    /// Waits until the reader shows the file `name`, which a press of Enter on its row opens.
    fn await_shown(&self, name: &str) {
        let path = self.root.join(name);
        let shown = self
            .app
            .wait_until(Duration::from_secs(5), || self.model.borrow().shown.path().is_some_and(|shown| shown == path));
        assert!(shown, "{name} is not shown");
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
    }

    fn screen(&self) -> String {
        let capture = self.app.render();
        (0..capture.height).map(|y| capture.row(y)).collect::<Vec<_>>().join("\n")
    }
}

#[test]
fn o_opens_the_selected_file_with_the_default_app() {
    let b = opening(&["Gringotts.md"]);
    b.app.press("o");
    b.await_footer("Opened Gringotts.md.");
    assert_eq!(b.footer().map(|message| message.error), Some(false));
}

#[test]
fn o_on_a_folder_row_opens_the_folder() {
    let b = opening(&["Hogwarts/"]);
    b.app.press("o");
    b.await_footer("Opened Hogwarts.");
}

#[test]
fn o_in_the_reader_opens_the_shown_file_not_the_selected_row() {
    let b = opening(&["Gringotts.md", "Pensieve.md"]);
    b.app.press("return");
    b.await_shown("Gringotts.md");
    b.select("Pensieve.md");
    // The focus is the reader's, as Tab gives it for a shown file; `o` then takes the shown file.
    b.model.borrow_mut().focus = Focus::Reader;
    b.app.press("o");
    b.await_footer("Opened Gringotts.md.");
}

#[test]
fn o_on_the_key_page_has_no_target_and_says_nothing() {
    let b = opening(&["Gringotts.md"]);
    b.model.borrow_mut().focus = Focus::Reader;
    b.app.press("o");
    assert!(b.footer().is_none(), "the key page opened something");
}

#[test]
fn a_failed_launch_keeps_the_reason_in_the_footer_as_an_error() {
    let b = opening(&["Gringotts.md"]);
    fs::remove_file(b.root.join("Gringotts.md")).unwrap();
    b.app.press("o");
    let failed = b.app.wait_until(Duration::from_secs(5), || b.footer().is_some_and(|message| message.error));
    assert!(failed, "no error reached the footer");
    let text = b.footer().map(|message| message.text).unwrap_or_default();
    assert!(text.contains("no such file"), "{text}");
}

#[test]
fn the_menu_opens_with_an_open_row_that_launches_the_row() {
    let b = opening(&["Gringotts.md"]);
    b.app.press("space");
    assert!(matches!(b.model.borrow().overlay, Some(Overlay::Menu(_))), "the menu opens");
    b.app.press("return");
    b.await_footer("Opened Gringotts.md.");
}

#[test]
fn the_binary_notice_names_the_key_and_o_opens_the_file() {
    let b = opening(&["Pensieve.bin"]);
    b.app.press("return");
    let shown = b.app.wait_until(Duration::from_secs(5), || matches!(b.model.borrow().shown, Shown::Binary { .. }));
    assert!(shown, "the binary notice is not shown");
    assert!(b.screen().contains(OPEN_HINT), "the notice lacks the key line");
    b.app.press("o");
    b.await_footer("Opened Pensieve.bin.");
}

/// The browsed folder's name, as the footer shows it.
fn folder_name(b: &Opening) -> String {
    b.root.file_name().unwrap().to_string_lossy().into_owned()
}

#[test]
fn e_opens_the_browsed_folder_in_finder_when_no_editor_is_set() {
    let b = opening(&["Gringotts.md"]);
    b.app.press("e");
    b.await_footer(&format!("Opened {}.", folder_name(&b)));
}

#[test]
fn e_opens_the_browsed_folder_in_the_editor_the_settings_name() {
    let b = opening(&["Gringotts.md"]);
    b.model.borrow_mut().settings.values.editor = "/Applications/Zed.app".into();
    b.app.press("e");
    b.await_footer(&format!("Opened {} in Zed.", folder_name(&b)));
}

#[test]
fn capital_e_shows_the_selected_file_in_finder() {
    let b = opening(&["Gringotts.md"]);
    b.app.press("E");
    b.await_footer("Showed Gringotts.md in Finder.");
}

#[test]
fn the_menu_show_in_finder_row_reveals_the_row() {
    let b = opening(&["Gringotts.md"]);
    b.app.press("space");
    // Open, then Open With, then Show in Finder.
    b.app.press("down");
    b.app.press("down");
    b.app.press("return");
    b.await_footer("Showed Gringotts.md in Finder.");
}

/// The labels of the open menu's rows, in item order; `None` when no menu is open.
fn menu_labels(b: &Opening) -> Option<Vec<String>> {
    match &b.model.borrow().overlay {
        Some(Overlay::Menu(menu)) => Some(menu.items().iter().map(|item| item.label.clone()).collect()),
        _ => None,
    }
}

/// Waits until the picker is open: the list of apps lands on the worker.
fn await_picker(b: &Opening) {
    let open = b.app.wait_until(Duration::from_secs(5), || matches!(b.model.borrow().overlay, Some(Overlay::Menu(_))));
    assert!(open, "the picker does not open");
    paint(b.app.app(), &mut b.model.borrow_mut(), &mut b.ui.borrow_mut());
}

/// Sets the `open_with` pattern `pattern` to the apps `apps`, as `settings.json` would.
fn open_with(b: &Opening, pattern: &str, apps: &[&str]) {
    let mut table = OpenWith::default();
    table.add_value(pattern, apps.iter().map(|app| app.to_string()).collect()).unwrap();
    b.model.borrow_mut().settings.values.open_with = table;
}

#[test]
fn the_picker_lists_the_configured_apps_then_the_listed_ones_then_finder_once() {
    let b = opening(&["Gringotts.md"]);
    open_with(&b, "*.md", &["Zed", "Typora"]);
    b.app.press("O");
    await_picker(&b);
    // Zed is configured, so the list's Zed is dropped; the default comes first in the list; Finder is last.
    assert_eq!(
        menu_labels(&b),
        Some(vec!["Zed".into(), "Typora".into(), "Pensieve (default)".into(), "Finder".into()])
    );
}

#[test]
fn the_picker_keeps_finder_when_the_listing_is_empty_or_fails() {
    let silent = opening(&["Sirius-Silent.md"]);
    silent.app.press("O");
    await_picker(&silent);
    assert_eq!(menu_labels(&silent), Some(vec!["Finder".into()]));

    let unlisted = opening(&["Dobby-Unlisted.md"]);
    unlisted.app.press("O");
    let failed = unlisted.app.wait_until(Duration::from_secs(5), || unlisted.footer().is_some_and(|m| m.error));
    assert!(failed, "the listing failure does not reach the footer");
    await_picker(&unlisted);
    assert_eq!(menu_labels(&unlisted), Some(vec!["Finder".into()]));
}

#[test]
fn a_folder_opens_the_same_picker_and_finder_opens_the_folder_in_finder() {
    let b = opening(&["Hogwarts/"]);
    b.app.press("O");
    await_picker(&b);
    assert_eq!(menu_labels(&b), Some(vec!["Pensieve (default)".into(), "Zed".into(), "Finder".into()]));
    b.app.press("down");
    b.app.press("down");
    b.app.press("return");
    b.await_footer("Opened Hogwarts in Finder.");
}

#[test]
fn picking_a_configured_app_opens_the_file_with_it() {
    let b = opening(&["Gringotts.md"]);
    open_with(&b, "*.md", &["Zed"]);
    b.app.press("O");
    await_picker(&b);
    b.app.press("return");
    b.await_footer("Opened Gringotts.md in Zed.");
}

#[test]
fn escape_closes_the_picker_and_launches_nothing() {
    let b = opening(&["Gringotts.md"]);
    b.app.press("O");
    await_picker(&b);
    b.app.press("escape");
    assert!(menu_labels(&b).is_none(), "the picker stays open");
    assert!(b.footer().is_none(), "something launched");
}

#[test]
fn a_long_picker_scrolls_so_the_selection_stays_in_view_at_120_columns() {
    let b = opening(&["Crowded.md"]);
    b.app.press("O");
    await_picker(&b);
    for _ in 0..17 {
        b.app.press("down");
    }
    let shown = match &b.model.borrow().overlay {
        Some(Overlay::Menu(menu)) => menu.window().iter().map(|item| item.label.clone()).collect::<Vec<_>>(),
        _ => panic!("the picker closed"),
    };
    assert!(shown.contains(&"Marauder 17".to_string()), "the selection scrolled out of view: {shown:?}");
    assert_eq!(shown.len(), 16, "the window shows sixteen rows");
    b.app.press("return");
    b.await_footer("Opened Crowded.md in Marauder 17.");
}

#[test]
fn a_long_picker_scrolls_at_40_columns_too() {
    let b = opening_sized(&["Crowded.md"], 40, 24);
    b.app.press("O");
    await_picker(&b);
    for _ in 0..17 {
        b.app.press("down");
    }
    let shown = match &b.model.borrow().overlay {
        Some(Overlay::Menu(menu)) => menu.window().iter().map(|item| item.label.clone()).collect::<Vec<_>>(),
        _ => panic!("the picker closed"),
    };
    assert!(shown.contains(&"Marauder 17".to_string()), "the selection scrolled out of view: {shown:?}");
}

#[test]
fn the_picker_draws_a_rule_only_between_groups_that_have_rows() {
    let b = opening(&["Gringotts.md"]);
    b.app.press("O");
    await_picker(&b);
    let rules: Vec<bool> = match &b.model.borrow().overlay {
        Some(Overlay::Menu(menu)) => menu.items().iter().map(|item| item.rule).collect(),
        _ => panic!("the picker does not open"),
    };
    // No configured app leads the list with no rule; Finder starts its own group after the listed apps.
    assert_eq!(rules, vec![false, false, true]);
}
