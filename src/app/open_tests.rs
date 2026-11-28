//! `o`, `e`, `E`, and the menu rows through the model: the selected row, the shown file, a folder, a failure's footer,
//! and the binary notice's line. The launcher is the test stand-in (`launch` in `open.rs`), so no test launches an app.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use opentui::testing::TestApp;
use tempfile::TempDir;

use fmc::overlay::Overlay;

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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    for name in names {
        match name.strip_suffix('/') {
            Some(folder) => fs::create_dir(root.join(folder)).unwrap(),
            None if name.ends_with(".bin") => fs::write(root.join(name), b"\0Hogwarts\n").unwrap(),
            None => fs::write(root.join(name), "Hogwarts\n").unwrap(),
        }
    }
    let app = TestApp::new(120, 40);
    let model = Rc::new(RefCell::new(new_model(app.app(), &root, root.clone(), None)));
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
    b.app.press("down");
    b.app.press("return");
    b.await_footer("Showed Gringotts.md in Finder.");
}
