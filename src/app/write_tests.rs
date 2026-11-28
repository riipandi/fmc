//! The file operations through the model: each key, each menu row, the right-click, and each refusal. Every write goes to
//! a temporary folder, and the Trash is the test stand-in (`trash` in `write.rs`), so no test writes to the owner's Trash.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fmc::explorer::{Entry, Kind};
use fmc::overlay::Stop;
use opentui::testing::TestApp;
use tempfile::TempDir;

use super::*;

/// A folder that holds `names` (a name ending in `/` is a folder), browsed on a 120 by 40 screen with its listing read and
/// the first row selected. The app's worker events are wired, so a write's listing lands as it does in the app.
struct Browsing {
    app: TestApp,
    model: Rc<RefCell<Model>>,
    ui: Rc<RefCell<Ui>>,
    root: PathBuf,
    _dir: TempDir,
}

fn browsing(names: &[&str]) -> Browsing {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    for name in names {
        match name.strip_suffix('/') {
            Some(folder) => fs::create_dir(root.join(folder)).unwrap(),
            None => fs::write(root.join(name), "Hedwig\n").unwrap(),
        }
    }
    let app = TestApp::new(120, 40);
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
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
    let browsing = Browsing { app, model, ui, root, _dir: dir };
    let root = browsing.root.clone();
    let entries = crate::services::fs::list(&root, &[]).unwrap();
    {
        let mut m = browsing.model.borrow_mut();
        m.browser.listed(&root, entries);
        let first = m.browser.tree.rows().first().map(|row| row.entry.path.clone());
        if let Some(first) = first {
            m.browser.select(&first);
        }
    }
    paint(browsing.app.app(), &mut browsing.model.borrow_mut(), &mut browsing.ui.borrow_mut());
    browsing.app.render();
    browsing
}

impl Browsing {
    fn selected(&self) -> String {
        let m = self.model.borrow();
        m.browser.selected().file_name().unwrap_or_default().to_string_lossy().into_owned()
    }

    /// Selects the row named `name`, which the listing holds.
    fn select(&self, name: &str) {
        let mut m = self.model.borrow_mut();
        m.browser.select(&self.root.join(name));
    }

    /// Waits until the selection is the row named `name`: a write's listing lands on the worker.
    fn await_selected(&self, name: &str) {
        let reached = self.app.wait_until(Duration::from_secs(5), || self.selected() == name);
        assert!(reached, "{name} is selected, not {}", self.selected());
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
    }

    fn dialog(&self) -> Option<Dialog> {
        match &self.model.borrow().overlay {
            Some(Overlay::Dialog(dialog)) => Some(dialog.clone()),
            _ => None,
        }
    }

    fn menu(&self) -> Option<Vec<(String, bool)>> {
        match &self.model.borrow().overlay {
            Some(Overlay::Menu(menu)) => Some(menu.items().iter().map(|i| (i.label.clone(), i.enabled)).collect()),
            _ => None,
        }
    }

    fn field(&self) -> String {
        self.ui.borrow().overlay.dialog.field.value()
    }

    fn type_name(&self, name: &str) {
        self.app.type_text(name);
    }

    fn clear_field(&self) {
        for _ in 0..self.field().chars().count() {
            self.app.press("backspace");
        }
    }

    fn has(&self, name: &str) -> bool {
        self.root.join(name).symlink_metadata().is_ok()
    }

    /// Where the list's row `index` is drawn, on the first column of the list.
    fn row(&self, index: usize) -> (u16, u16) {
        (5, LIST_TOP + index as u16)
    }

    fn error(&self) -> Option<String> {
        self.dialog().and_then(|dialog| dialog.error)
    }

    fn footer(&self) -> Option<String> {
        self.model.borrow().message.as_ref().map(|message| message.text.clone())
    }

    /// Waits until the footer shows `text`: a paste's refusal comes back from the worker.
    fn await_footer(&self, text: &str) {
        let shown = self.app.wait_until(Duration::from_secs(5), || self.footer().as_deref() == Some(text));
        assert!(shown, "the footer shows {text:?}, not {:?}", self.footer());
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
    }

    /// Waits until the tree no longer lists `name` in the root: a move's source leaves the listing.
    fn await_unlisted(&self, name: &str) {
        let path = self.root.join(name);
        let gone =
            self.app.wait_until(Duration::from_secs(5), || self.model.borrow().browser.tree.entry(&path).is_none());
        assert!(gone, "{name} is still listed");
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
    }

    /// Whether the dialog's field holds the keyboard, which is what shows the terminal cursor in it.
    fn field_focused(&self) -> bool {
        let field = self.ui.borrow().overlay.dialog.field.clone();
        self.app.app().focused().is_some_and(|widget| widget.is(&field))
    }

    fn menu_selected(&self) -> Option<usize> {
        match &self.model.borrow().overlay {
            Some(Overlay::Menu(menu)) => Some(menu.selected()),
            _ => None,
        }
    }

    /// The label and enabled state of the open menu's row `label`.
    fn menu_row(&self, label: &str) -> Option<bool> {
        self.menu()?.iter().find(|(text, _)| text == label).map(|(_, enabled)| *enabled)
    }
}

/// A run that is on the worker and has nothing queued: it makes a second write wait, as a paste that is running does.
fn busy() -> Bulk {
    Bulk::new(crate::services::write::Kind::Copy, Vec::new(), Vec::new(), PathBuf::new())
}

#[test]
fn a_opens_a_name_dialog_in_the_folder_of_the_selected_file() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("a");
    let dialog = b.dialog().expect("the dialog opens");
    assert_eq!(dialog.title, "New file");
    assert_eq!(dialog.focus, Stop::Field, "the name field takes the typing");
    assert!(matches!(
        b.model.borrow().overlay_task,
        Some(Task::Name { folder: false, ref dir }) if *dir == b.root
    ));
}

#[test]
fn a_typed_name_creates_the_file_and_the_tree_selects_it() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("a");
    b.type_name("sophie-neveu.md");
    b.app.press("return");
    assert!(b.dialog().is_none(), "a written name closes the dialog");
    b.await_selected("sophie-neveu.md");
    assert!(b.root.join("sophie-neveu.md").is_file());
}

#[test]
fn a_name_ending_in_a_slash_creates_a_folder_and_is_selected() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("a");
    b.type_name("langdon/");
    b.app.press("return");
    b.await_selected("langdon");
    assert!(b.root.join("langdon").is_dir());
    assert!(b.has("robert-langdon.md"), "the file that was there stays");
}

#[test]
fn capital_a_creates_a_folder_and_drops_one_trailing_slash() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("A");
    assert_eq!(b.dialog().map(|d| d.title), Some("New folder".into()));
    b.type_name("hermione-granger/");
    b.app.press("return");
    b.await_selected("hermione-granger");
    assert!(b.root.join("hermione-granger").is_dir());
}

#[test]
fn a_taken_name_keeps_the_dialog_open_with_its_error_and_writes_nothing() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("a");
    b.type_name("robert-langdon.md");
    b.app.press("return");
    assert_eq!(b.error().as_deref(), Some("An item with that name already exists."));
    assert_eq!(fs::read_to_string(b.root.join("robert-langdon.md")).unwrap(), "Hedwig\n", "nothing is overwritten");
    b.app.press("x");
    assert_eq!(b.error(), None, "typing clears the error line");
}

#[test]
fn a_name_with_a_slash_inside_and_an_empty_name_are_refused() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("a");
    b.type_name("sophie/neveu.md");
    b.app.press("return");
    assert_eq!(b.error().as_deref(), Some("That name is not valid."));
    b.clear_field();
    b.app.press("return");
    assert_eq!(b.error().as_deref(), Some("That name is not valid."), "an empty name is not a name");
    assert!(b.dialog().is_some());
}

#[test]
fn escape_closes_a_name_dialog_and_writes_nothing() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("a");
    b.type_name("sophie-neveu.md");
    b.app.press("escape");
    assert!(b.dialog().is_none());
    assert!(!b.has("sophie-neveu.md"));
}

#[test]
fn a_new_entry_on_a_folder_row_goes_inside_it_and_reveals_it_expanded() {
    let b = browsing(&["hogsmeade/", "robert-langdon.md"]);
    b.select("hogsmeade");
    b.app.press("a");
    b.type_name("owls.txt");
    b.app.press("return");
    b.await_selected("owls.txt");
    assert!(b.root.join("hogsmeade").join("owls.txt").is_file());
    assert!(b.model.borrow().browser.tree.is_expanded(&b.root.join("hogsmeade")), "the parent opens to show it");
}

#[test]
fn a_new_entry_with_no_row_goes_into_the_root() {
    let b = browsing(&[]);
    b.app.press("A");
    b.type_name("gringotts");
    b.app.press("return");
    b.await_selected("gringotts");
    assert!(b.root.join("gringotts").is_dir());
}

#[test]
fn r_prefills_the_name_and_renames_the_row_then_selects_it() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("r");
    let dialog = b.dialog().expect("the rename dialog opens");
    assert_eq!(dialog.title, "Rename");
    assert_eq!(b.field(), "robert-langdon.md");
    b.clear_field();
    b.type_name("sophie-neveu.md");
    b.app.press("return");
    b.await_selected("sophie-neveu.md");
    assert!(!b.has("robert-langdon.md"));
    assert!(b.has("sophie-neveu.md"));
}

#[test]
fn a_rename_to_a_taken_name_keeps_both_files_and_the_dialog() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md"]);
    b.select("robert-langdon.md");
    b.app.press("r");
    b.clear_field();
    b.type_name("leigh-teabing.md");
    b.app.press("return");
    assert_eq!(b.error().as_deref(), Some("An item with that name already exists."));
    assert_eq!(fs::read_to_string(b.root.join("leigh-teabing.md")).unwrap(), "Hedwig\n");
    assert!(b.has("robert-langdon.md"));
}

#[test]
fn r_on_a_name_with_a_control_character_is_refused_in_the_footer() {
    // A name with an escape byte is listed in the model alone; the field would print the byte.
    let b = browsing(&[]);
    let name = "caf\u{1b}.md";
    let path = b.root.join(name);
    {
        let mut m = b.model.borrow_mut();
        m.browser.listed(&b.root, vec![Entry { name: name.into(), path: path.clone(), kind: Kind::File }]);
        m.browser.select(&path);
    }
    b.app.press("r");
    assert!(b.dialog().is_none(), "no field shows the name");
    assert_eq!(b.footer().as_deref(), Some("This name has a control character, so fmc cannot rename it."));
}

#[test]
fn a_rename_to_the_same_name_closes_without_a_write() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("r");
    b.app.press("return");
    assert!(b.dialog().is_none());
    assert!(b.has("robert-langdon.md"));
}

#[test]
fn r_on_a_name_that_is_not_utf8_is_refused_in_the_footer() {
    // APFS refuses a name that is not UTF-8, so the row is listed in the model alone; nothing is read from disk.
    let b = browsing(&[]);
    let path = b.root.join(OsStr::from_bytes(b"caf\xe9.md"));
    {
        let mut m = b.model.borrow_mut();
        m.browser.listed(&b.root, vec![Entry { name: "caf\u{fffd}.md".into(), path: path.clone(), kind: Kind::File }]);
        m.browser.select(&path);
    }
    b.app.press("r");
    assert!(b.dialog().is_none(), "no field can hold the name");
    assert_eq!(b.footer().as_deref(), Some("This name is not UTF-8, so fmc cannot rename it."));
}

#[test]
fn capital_d_asks_first_and_cancel_is_the_default() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("D");
    let dialog = b.dialog().expect("the confirm dialog opens");
    assert_eq!(dialog.title, "Move to Trash?");
    assert!(dialog.message.starts_with("robert-langdon.md goes to the Trash"), "{}", dialog.message);
    assert_eq!(dialog.focus, Stop::Button(0), "Cancel has the focus");
    b.app.press("return");
    assert!(b.dialog().is_none());
    assert!(b.has("robert-langdon.md"), "Cancel keeps the file");
}

#[test]
fn delete_trashes_the_row_after_tab_and_selects_the_next_one() {
    let b = browsing(&["harry-potter.md", "hermione-granger.md", "sophie-neveu.md"]);
    b.select("hermione-granger.md");
    b.app.press("delete");
    b.app.press("tab");
    b.app.press("return");
    assert!(b.dialog().is_none());
    b.await_selected("sophie-neveu.md");
    assert!(!b.has("hermione-granger.md"));
    assert!(b.has("harry-potter.md"));
}

#[test]
fn deleting_the_last_row_selects_the_previous_one() {
    let b = browsing(&["harry-potter.md", "sophie-neveu.md"]);
    b.select("sophie-neveu.md");
    b.app.press("D");
    b.app.press("tab");
    b.app.press("return");
    b.await_selected("harry-potter.md");
    assert!(!b.has("sophie-neveu.md"));
}

#[test]
fn deleting_a_folder_selects_the_row_after_the_rows_under_it() {
    let b = browsing(&["hogsmeade/", "harry-potter.md", "sophie-neveu.md"]);
    let hogsmeade = b.root.join("hogsmeade");
    fs::write(hogsmeade.join("owls.txt"), "Hedwig\n").unwrap();
    {
        let mut m = b.model.borrow_mut();
        m.browser.listed(&hogsmeade, vec![Entry::new(&hogsmeade, "owls.txt", Kind::File)]);
        m.browser.select(&hogsmeade);
    }
    b.app.press("D");
    b.app.press("tab");
    b.app.press("return");
    b.await_selected("harry-potter.md");
    assert!(!b.has("hogsmeade"));
}

#[test]
fn the_menu_on_a_file_row_lists_the_acts_with_paste_disabled_until_a_copy() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("space");
    let items = b.menu().expect("the menu opens");
    let labels: Vec<(&str, bool)> = items.iter().map(|(label, on)| (label.as_str(), *on)).collect();
    assert_eq!(
        labels,
        [
            ("Open", true),
            ("Open With…", true),
            ("Show in Finder", true),
            ("New file…", true),
            ("New folder…", true),
            ("Rename…", true),
            ("Copy", true),
            ("Cut", true),
            ("Paste", false),
            ("Duplicate…", true),
            ("Delete…", true),
        ]
    );
}

#[test]
fn the_menu_with_no_row_lists_paste_beside_the_new_items() {
    let b = browsing(&[]);
    b.app.press("space");
    assert_eq!(
        b.menu(),
        Some(vec![
            ("Open Folder in Editor".into(), true),
            ("New file…".into(), true),
            ("New folder…".into(), true),
            ("Paste".into(), false),
        ])
    );
}

#[test]
fn the_menu_enables_paste_once_a_row_is_copied() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("c");
    b.app.press("space");
    assert_eq!(b.menu_row("Paste"), Some(true));
}

#[test]
fn a_menu_row_starts_the_act_it_names_and_a_disabled_row_is_skipped() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("space");
    // Open, Open With, Show in Finder, New file, then New folder: five moves down the menu, to Rename.
    for _ in 0..5 {
        b.app.press("down");
    }
    b.app.press("return");
    assert_eq!(b.dialog().map(|d| d.title), Some("Rename".into()));
    assert_eq!(b.field(), "robert-langdon.md");
}

#[test]
fn a_click_on_the_delete_row_asks_for_the_trash() {
    let b = browsing(&["robert-langdon.md"]);
    let (x, y) = b.row(0);
    b.app.mouse(MouseEventKind::Down(MouseButton::Right), x, y);
    b.app.render();
    assert!(b.menu().is_some(), "a right-click opens the menu");
    // Delete is the eleventh item, below three rules: the Open group, the new items, Rename, and Duplicate come before it.
    b.app.click(x + 2, y + 1 + 10 + 3);
    assert_eq!(b.dialog().map(|d| d.title), Some("Move to Trash?".into()));
}

#[test]
fn a_right_click_selects_the_row_and_opens_the_menu_at_the_pointer() {
    let b = browsing(&["harry-potter.md", "sophie-neveu.md"]);
    let (x, y) = b.row(1);
    b.app.mouse(MouseEventKind::Down(MouseButton::Right), x, y);
    assert_eq!(b.selected(), "sophie-neveu.md");
    assert!(b.menu().is_some());
    assert_eq!(b.model.borrow().overlay_at, (x, y));
}

#[test]
fn a_right_click_on_empty_space_opens_nothing() {
    let b = browsing(&["harry-potter.md"]);
    b.app.mouse(MouseEventKind::Down(MouseButton::Right), 5, 30);
    assert!(b.menu().is_none());
}

#[test]
fn a_click_on_the_copy_row_puts_the_row_on_the_clipboard() {
    let b = browsing(&["robert-langdon.md"]);
    let (x, y) = b.row(0);
    b.app.mouse(MouseEventKind::Down(MouseButton::Right), x, y);
    b.app.render();
    // Copy is the seventh item, below the two rules that open the second and third groups.
    b.app.click(x + 2, y + 1 + 6 + 2);
    assert!(b.menu().is_none());
    let clip = b.model.borrow().clipboard.clone().expect("the copy is on the clipboard");
    assert_eq!((clip.paths, clip.cut), (vec![b.root.join("robert-langdon.md")], false));
}

#[test]
fn c_then_v_copies_a_file_into_the_selected_folder_and_reveals_the_copy() {
    let b = browsing(&["hogsmeade/", "robert-langdon.md"]);
    b.select("robert-langdon.md");
    b.app.press("c");
    b.select("hogsmeade");
    b.app.press("v");
    b.await_selected("robert-langdon.md");
    assert_eq!(fs::read_to_string(b.root.join("hogsmeade/robert-langdon.md")).unwrap(), "Hedwig\n");
    assert!(b.has("robert-langdon.md"), "a copy leaves the source");
    assert!(b.model.borrow().clipboard.is_some(), "a copy stays on the clipboard for another paste");
    assert!(b.model.borrow().browser.tree.is_expanded(&b.root.join("hogsmeade")));
}

#[test]
fn a_folder_with_a_link_inside_is_copied_and_the_link_stays_a_link() {
    let b = browsing(&["hogsmeade/", "robert-langdon.md"]);
    let hogsmeade = b.root.join("hogsmeade");
    fs::write(hogsmeade.join("owls.txt"), "Hedwig\n").unwrap();
    symlink("../robert-langdon.md", hogsmeade.join("letters")).unwrap();
    b.select("hogsmeade");
    b.app.press("c");
    b.select("robert-langdon.md");
    b.app.press("v");
    b.await_selected("hogsmeade copy");
    let copy = b.root.join("hogsmeade copy");
    assert_eq!(fs::read_to_string(copy.join("owls.txt")).unwrap(), "Hedwig\n");
    assert!(
        fs::symlink_metadata(copy.join("letters")).unwrap().file_type().is_symlink(),
        "the link is copied as a link"
    );
    assert_eq!(fs::read_link(copy.join("letters")).unwrap(), Path::new("../robert-langdon.md"));
}

#[test]
fn a_paste_into_the_same_folder_takes_a_copy_name_and_then_copy_two() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("c");
    b.app.press("v");
    b.await_selected("robert-langdon copy.md");
    b.app.press("v");
    b.await_selected("robert-langdon copy 2.md");
    assert_eq!(fs::read_to_string(b.root.join("robert-langdon.md")).unwrap(), "Hedwig\n", "the original stays");
}

#[test]
fn x_then_v_moves_the_file_and_its_row_leaves_the_root() {
    let b = browsing(&["hogsmeade/", "sophie-neveu.md"]);
    b.select("sophie-neveu.md");
    b.app.press("x");
    b.select("hogsmeade");
    b.app.press("v");
    b.await_selected("sophie-neveu.md");
    assert!(b.root.join("hogsmeade/sophie-neveu.md").is_file());
    b.await_unlisted("sophie-neveu.md");
    assert!(!b.has("sophie-neveu.md"));
    assert!(b.model.borrow().clipboard.is_none(), "the cut ends with its paste");
}

#[test]
fn a_cut_pasted_into_its_own_folder_changes_nothing_and_ends_the_cut() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("x");
    b.app.press("v");
    assert!(b.model.borrow().clipboard.is_none(), "the cut ends");
    assert!(b.model.borrow().bulk.is_none(), "nothing runs");
    assert_eq!(fs::read_dir(&b.root).unwrap().count(), 1, "nothing is copied beside it");
}

#[test]
fn a_folder_pasted_into_itself_is_refused_in_the_footer() {
    let b = browsing(&["hogsmeade/"]);
    b.select("hogsmeade");
    b.app.press("c");
    b.app.press("v");
    b.await_footer("A folder cannot go into itself.");
    assert_eq!(fs::read_dir(b.root.join("hogsmeade")).unwrap().count(), 0, "nothing was copied");
    assert!(b.model.borrow().bulk.is_none());
}

#[test]
fn a_folder_pasted_into_its_own_descendant_is_refused_in_the_footer() {
    let b = browsing(&["hogsmeade/"]);
    let hogsmeade = b.root.join("hogsmeade");
    fs::create_dir(hogsmeade.join("owls")).unwrap();
    b.select("hogsmeade");
    b.app.press("c");
    {
        let mut m = b.model.borrow_mut();
        m.browser.listed(&hogsmeade, vec![Entry::new(&hogsmeade, "owls", Kind::Directory)]);
        m.browser.select(&hogsmeade.join("owls"));
    }
    b.app.press("v");
    b.await_footer("A folder cannot go into itself.");
    assert!(!hogsmeade.join("owls/hogsmeade").exists());
}

#[test]
fn a_paste_while_one_runs_is_refused_with_a_notice() {
    let b = browsing(&["robert-langdon.md", "hogsmeade/"]);
    b.select("robert-langdon.md");
    b.app.press("c");
    // No copy is in flight in this test, so the flag stands in for one that is.
    b.model.borrow_mut().bulk = Some(busy());
    b.select("hogsmeade");
    b.app.press("v");
    assert_eq!(b.footer().as_deref(), Some("Still working. Try again when it finishes."));
    assert!(!b.root.join("hogsmeade/robert-langdon.md").exists(), "the second paste writes nothing");
}

#[test]
fn escape_in_the_list_drops_a_cut_and_the_paste_then_does_nothing() {
    let b = browsing(&["robert-langdon.md", "hogsmeade/"]);
    b.select("robert-langdon.md");
    b.app.press("x");
    b.app.press("escape");
    assert!(b.model.borrow().clipboard.is_none());
    b.select("hogsmeade");
    b.app.press("v");
    assert!(!b.root.join("hogsmeade/robert-langdon.md").exists());
    assert!(b.has("robert-langdon.md"));
}

#[test]
fn delete_refuses_a_name_that_is_not_utf8_in_the_footer() {
    // Listed in the model alone, as the rename refusal test does: APFS keeps no such name.
    let b = browsing(&[]);
    let path = b.root.join(OsStr::from_bytes(b"caf\xe9.md"));
    {
        let mut m = b.model.borrow_mut();
        m.browser.listed(&b.root, vec![Entry { name: "caf\u{fffd}.md".into(), path: path.clone(), kind: Kind::File }]);
        m.browser.select(&path);
    }
    b.app.press("D");
    assert!(b.dialog().is_none(), "no dialog names a file it cannot trash");
    assert_eq!(b.footer().as_deref(), Some("This name is not UTF-8, so fmc cannot move it to the Trash."));
}

#[test]
fn an_entry_swapped_after_the_confirm_opens_is_not_trashed() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("D");
    assert!(b.dialog().is_some());
    // The listed entry moves aside, which keeps its inode, and another file takes its name: a new inode.
    fs::rename(b.root.join("robert-langdon.md"), b.root.join("keep-langdon.md")).unwrap();
    fs::write(b.root.join("robert-langdon.md"), "Griphook\n").unwrap();
    b.app.press("tab");
    b.app.press("return");
    assert!(b.dialog().is_none(), "the confirm closes");
    assert_eq!(b.footer().as_deref(), Some("That entry changed since it was listed, so nothing went to the Trash."));
    assert_eq!(fs::read_to_string(b.root.join("robert-langdon.md")).unwrap(), "Griphook\n", "the new entry stays");
    assert!(b.has("keep-langdon.md"), "the old one is not trashed either");
    b.await_selected("robert-langdon.md");
}

#[test]
fn the_name_field_holds_the_cursor_until_tab_moves_the_focus_to_a_button() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("a");
    assert!(b.field_focused(), "the field has the keyboard, so the cursor shows in it");
    b.app.press("tab");
    assert!(!b.field_focused(), "Cancel has the focus, and the cursor leaves the field");
    assert_eq!(b.dialog().map(|d| d.focus), Some(Stop::Button(0)));
    // The stops run field, Cancel, OK: the third Tab returns the cursor to the field.
    b.app.press("tab");
    b.app.press("tab");
    assert!(b.field_focused(), "the Tab after OK returns the cursor to the field");
    b.app.press("escape");
    assert!(b.app.app().focused().is_none(), "a closed dialog leaves nothing focused");
}

#[test]
fn a_pointer_over_a_menu_row_selects_it_and_a_disabled_row_is_not_selected() {
    let b = browsing(&["robert-langdon.md"]);
    let (x, y) = b.row(0);
    b.app.mouse(MouseEventKind::Down(MouseButton::Right), x, y);
    b.app.render();
    assert_eq!(b.menu_selected(), Some(0), "the first enabled row, Open, is selected when the menu opens");
    // Rules open the second and third groups, so Copy (item 6) is drawn two rows lower.
    b.app.hover(x + 2, y + 1 + 6 + 2);
    assert_eq!(b.menu_selected(), Some(6), "Copy is selected under the pointer");
    // Paste is item 8 and disabled with nothing copied: the pointer over it leaves the selection where it was.
    b.app.hover(x + 2, y + 1 + 8 + 2);
    assert_eq!(b.menu_selected(), Some(6));
}

#[test]
fn a_pointer_over_a_dialog_button_highlights_it_and_return_still_picks_the_focused_one() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("D");
    b.app.render();
    assert_eq!(b.dialog().map(|d| d.focus), Some(Stop::Button(0)));
    let text = b.app.text();
    let (y, line) =
        text.lines().enumerate().find(|(_, line)| line.contains("Delete")).expect("the Delete button is drawn");
    let column = line.find("Delete").map(|byte| line[..byte].chars().count()).unwrap_or(0);
    b.app.hover(column as u16 + 1, y as u16);
    assert_eq!(b.dialog().map(|d| d.hovered), Some(Some(1)), "Delete is highlighted under the pointer");
    assert_eq!(b.dialog().map(|d| d.focus), Some(Stop::Button(0)), "the focus stays on Cancel");
    assert!(!b.field_focused());
    b.app.press("return");
    assert!(b.dialog().is_none(), "Return picked the focused Cancel, and the dialog closed");
    assert!(b.has("robert-langdon.md"), "nothing was trashed");
}

#[test]
fn capital_r_asks_for_a_copy_name_and_the_copy_is_revealed() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("R");
    let dialog = b.dialog().expect("the duplicate dialog opens");
    assert_eq!(dialog.title, "Duplicate");
    assert_eq!(b.field(), "robert-langdon copy.md", "the name a paste would give");
    b.app.press("return");
    b.await_selected("robert-langdon copy.md");
    assert_eq!(fs::read_to_string(b.root.join("robert-langdon copy.md")).unwrap(), "Hedwig\n");
    assert!(b.has("robert-langdon.md"), "the original stays");
    assert!(b.dialog().is_none());
}

#[test]
fn a_duplicate_of_a_folder_keeps_its_links_as_links() {
    let b = browsing(&["hogsmeade/", "robert-langdon.md"]);
    let hogsmeade = b.root.join("hogsmeade");
    fs::write(hogsmeade.join("owls.txt"), "Hedwig\n").unwrap();
    symlink("../robert-langdon.md", hogsmeade.join("letters")).unwrap();
    b.select("hogsmeade");
    b.app.press("R");
    assert_eq!(b.field(), "hogsmeade copy");
    b.app.press("return");
    b.await_selected("hogsmeade copy");
    let copy = b.root.join("hogsmeade copy");
    assert_eq!(fs::read_to_string(copy.join("owls.txt")).unwrap(), "Hedwig\n");
    assert!(fs::symlink_metadata(copy.join("letters")).unwrap().file_type().is_symlink());
}

#[test]
fn a_duplicate_to_a_taken_name_keeps_the_dialog_with_its_error_and_writes_nothing() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md"]);
    b.select("robert-langdon.md");
    b.app.press("R");
    b.clear_field();
    b.type_name("leigh-teabing.md");
    b.app.press("return");
    assert_eq!(b.error().as_deref(), Some("An item with that name already exists."));
    assert!(b.dialog().is_some(), "the dialog stays open for another name");
    let mut names: Vec<String> =
        fs::read_dir(&b.root).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    names.sort();
    assert_eq!(names, ["leigh-teabing.md", "robert-langdon.md"]);
}

#[test]
fn a_duplicate_while_a_paste_runs_is_refused_in_the_dialog() {
    let b = browsing(&["robert-langdon.md"]);
    b.app.press("R");
    // No copy is in flight in this test, so the flag stands in for one that is.
    b.model.borrow_mut().bulk = Some(busy());
    b.app.press("return");
    assert_eq!(b.error().as_deref(), Some("Still working. Try again when it finishes."));
    assert!(b.dialog().is_some(), "the dialog stays open until the paste finishes");
    assert!(!b.has("robert-langdon copy.md"), "nothing is copied");
}

#[test]
fn capital_r_with_no_row_opens_nothing() {
    let b = browsing(&[]);
    b.app.press("R");
    assert!(b.dialog().is_none());
    assert!(b.menu().is_none());
}

#[test]
fn s_marks_the_row_and_moves_down_and_capital_s_clears_every_mark() {
    let b = browsing(&["robert-langdon.md", "sophie-neveu.md", "leigh-teabing.md"]);
    b.select("leigh-teabing.md");
    b.app.press("s");
    assert_eq!(b.selected(), "robert-langdon.md", "the selection moves down one row");
    b.app.press("s");
    b.app.press("s");
    assert_eq!(b.selected(), "sophie-neveu.md", "the last row keeps the selection");
    assert_eq!(b.model.borrow().browser.marked().len(), 3);
    assert!(b.app.text().contains("3 marked"), "the footer counts the marks:\n{}", b.app.text());
    assert!(b.app.text().contains('✓'), "a marked row takes a check:\n{}", b.app.text());
    b.app.press("S");
    assert!(b.model.borrow().browser.marked().is_empty());
    assert!(!b.app.text().contains("marked"), "no count without marks:\n{}", b.app.text());
}

#[test]
fn a_second_s_on_a_marked_row_unmarks_it() {
    let b = browsing(&["robert-langdon.md", "sophie-neveu.md"]);
    b.select("robert-langdon.md");
    b.app.press("s");
    b.select("robert-langdon.md");
    b.app.press("s");
    assert!(b.model.borrow().browser.marked().is_empty());
}

#[test]
fn escape_drops_the_clipboard_first_and_then_the_marks() {
    let b = browsing(&["robert-langdon.md", "sophie-neveu.md"]);
    b.select("robert-langdon.md");
    b.app.press("c");
    b.app.press("s");
    b.app.press("escape");
    assert!(b.model.borrow().clipboard.is_none(), "the first Esc drops the copy");
    assert_eq!(b.model.borrow().browser.marked().len(), 1, "and keeps the marks");
    b.app.press("escape");
    assert!(b.model.borrow().browser.marked().is_empty(), "the second Esc drops the marks");
}

/// Marks `count` rows, from `leigh-teabing.md` down: each `s` moves the selection to the next row.
fn mark_from_leigh(b: &Browsing, count: usize) {
    b.select("leigh-teabing.md");
    for _ in 0..count {
        b.app.press("s");
    }
}

#[test]
fn c_then_v_copies_every_marked_row_into_the_folder_and_clears_the_marks() {
    let b = browsing(&["hogwarts/", "leigh-teabing.md", "robert-langdon.md", "sophie-neveu.md"]);
    mark_from_leigh(&b, 3);
    b.app.press("c");
    assert_eq!(b.footer().as_deref(), Some("Copied 3 items. Paste them with v."));
    b.select("hogwarts");
    b.app.press("v");
    b.await_selected("sophie-neveu.md");
    assert!(b.has("hogwarts/leigh-teabing.md") && b.has("hogwarts/robert-langdon.md"));
    assert!(b.has("hogwarts/sophie-neveu.md") && b.has("sophie-neveu.md"), "a copy leaves the originals");
    assert!(b.model.borrow().browser.marked().is_empty(), "a run that lands every item clears the marks");
    assert!(b.model.borrow().clipboard.is_some(), "a copy stays on the clipboard for another paste");
}

#[test]
fn x_then_v_moves_every_marked_row_and_the_rows_leave_the_root() {
    let b = browsing(&["hogwarts/", "leigh-teabing.md", "robert-langdon.md", "sophie-neveu.md"]);
    mark_from_leigh(&b, 2);
    b.app.press("x");
    b.select("hogwarts");
    b.app.press("v");
    b.await_selected("robert-langdon.md");
    assert!(b.has("hogwarts/leigh-teabing.md") && b.has("hogwarts/robert-langdon.md"));
    assert!(!b.has("leigh-teabing.md") && !b.has("robert-langdon.md"), "a move leaves nothing behind");
    assert!(b.has("sophie-neveu.md"), "an unmarked row stays");
    assert!(b.model.borrow().clipboard.is_none(), "the cut ends with its paste");
}

#[test]
fn a_paste_that_meets_a_missing_entry_stops_and_says_how_many_landed() {
    let b = browsing(&["hogwarts/", "leigh-teabing.md", "robert-langdon.md", "sophie-neveu.md"]);
    mark_from_leigh(&b, 3);
    b.app.press("c");
    // The second item goes missing after the copy was made: the run copies the first and stops there.
    fs::remove_file(b.root.join("robert-langdon.md")).unwrap();
    b.select("hogwarts");
    b.app.press("v");
    b.await_footer("Copied 1 of 3. Nothing is at that path.");
    assert!(b.has("hogwarts/leigh-teabing.md"), "the first item landed");
    assert!(!b.has("hogwarts/sophie-neveu.md"), "the run did not go on past its error");
    assert_eq!(b.model.borrow().browser.marked().len(), 3, "a run that stops keeps its marks");
    assert!(b.model.borrow().clipboard.is_some(), "and the clipboard");
}

#[test]
fn a_cut_of_marks_pasted_where_they_lie_changes_nothing_and_ends_the_cut() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md"]);
    mark_from_leigh(&b, 2);
    b.app.press("x");
    b.app.press("v");
    assert!(b.model.borrow().clipboard.is_none(), "every item is already here, so the cut ends");
    assert!(b.has("leigh-teabing.md") && b.has("robert-langdon.md"));
    assert_eq!(b.model.borrow().browser.marked().len(), 2, "nothing ran, so the marks stay");
}

#[test]
fn the_menu_counts_the_marks_and_the_clipboard() {
    let b = browsing(&["hogwarts/", "leigh-teabing.md", "robert-langdon.md"]);
    mark_from_leigh(&b, 2);
    b.app.press("space");
    assert_eq!(b.menu_row("Copy 2 items"), Some(true));
    assert_eq!(b.menu_row("Cut 2 items"), Some(true));
    assert_eq!(b.menu_row("Paste"), Some(false), "nothing is on the clipboard yet");
    b.app.press("escape");
    b.app.press("c");
    b.app.press("space");
    assert_eq!(b.menu_row("Paste 2 items"), Some(true));
}

#[test]
fn r_renames_the_selected_row_while_other_rows_are_marked() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md"]);
    mark_from_leigh(&b, 1);
    assert_eq!(b.selected(), "robert-langdon.md");
    b.app.press("r");
    assert_eq!(b.field(), "robert-langdon.md", "a rename is single-row");
}

#[test]
fn an_entry_replaced_after_it_was_copied_is_skipped_and_the_run_stops() {
    let b = browsing(&["hogwarts/", "leigh-teabing.md", "robert-langdon.md", "sophie-neveu.md"]);
    mark_from_leigh(&b, 3);
    b.app.press("c");
    // A new file takes robert's name: written beside the old one and renamed over it, so it has a new inode, and the
    // old one is gone before the copy runs.
    fs::write(b.root.join("swap.tmp"), "Hedwig\n").unwrap();
    fs::rename(b.root.join("swap.tmp"), b.root.join("robert-langdon.md")).unwrap();
    b.select("hogwarts");
    b.app.press("v");
    b.await_footer("Copied 1 of 3. An item changed after it was copied or cut, so it was skipped.");
    assert!(b.has("hogwarts/leigh-teabing.md"), "the item before the swap landed");
    assert!(!b.has("hogwarts/robert-langdon.md"), "the replaced entry was not copied");
    assert!(!b.has("hogwarts/sophie-neveu.md"), "the run stopped there");
    assert!(b.model.borrow().clipboard.is_some(), "a copy stays on the clipboard");
}

#[test]
fn a_mark_on_a_file_that_is_gone_is_left_out_of_a_copy() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md"]);
    mark_from_leigh(&b, 1);
    fs::remove_file(b.root.join("leigh-teabing.md")).unwrap();
    b.select("robert-langdon.md");
    b.app.press("c");
    assert!(b.model.borrow().clipboard.is_none(), "nothing is left to copy, so the clipboard stays empty");
}

#[test]
fn capital_d_on_marks_names_every_entry_and_trashes_them_together() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md", "sophie-neveu.md"]);
    mark_from_leigh(&b, 3);
    b.app.press("D");
    let dialog = b.dialog().expect("the confirm opens");
    assert_eq!(dialog.title, "Move 3 items to the Trash?");
    assert!(dialog.message.ends_with("leigh-teabing.md\nrobert-langdon.md\nsophie-neveu.md"), "{}", dialog.message);
    b.app.press("tab");
    b.app.press("return");
    assert!(b.dialog().is_none());
    // The files leave on the worker; the run's answer then clears the marks, so both are waited for.
    let landed = b.app.wait_until(Duration::from_secs(5), || {
        !b.has("leigh-teabing.md")
            && !b.has("robert-langdon.md")
            && !b.has("sophie-neveu.md")
            && b.model.borrow().browser.marked().is_empty()
    });
    assert!(landed, "every marked entry went to the Trash, and a run that landed them clears the marks");
}

#[test]
fn the_confirm_names_five_entries_and_counts_the_rest_and_cancel_keeps_them() {
    let b = browsing(&[
        "albus-dumbledore.md",
        "diagon-alley.md",
        "gringotts.md",
        "harry-potter.md",
        "hermione-granger.md",
        "hogwarts-owls.md",
        "minerva-mcgonagall.md",
    ]);
    b.select("albus-dumbledore.md");
    for _ in 0..7 {
        b.app.press("s");
    }
    b.app.press("D");
    let dialog = b.dialog().expect("the confirm opens");
    assert_eq!(dialog.title, "Move 7 items to the Trash?");
    assert_eq!(
        dialog.message,
        "These go to the Trash. Finder's Put Back restores them:\n\
         albus-dumbledore.md\ndiagon-alley.md\ngringotts.md\nharry-potter.md\nhermione-granger.md\nand 2 more"
    );
    b.app.press("return");
    assert!(b.dialog().is_none());
    assert!(b.has("albus-dumbledore.md") && b.has("minerva-mcgonagall.md"), "Cancel keeps every entry");
    assert_eq!(b.model.borrow().browser.marked().len(), 7, "and its marks");
}

#[test]
fn an_entry_replaced_after_the_confirm_opens_refuses_the_whole_run() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md", "sophie-neveu.md"]);
    mark_from_leigh(&b, 2);
    b.app.press("D");
    assert!(b.dialog().is_some());
    fs::write(b.root.join("swap.tmp"), "Griphook\n").unwrap();
    fs::rename(b.root.join("swap.tmp"), b.root.join("robert-langdon.md")).unwrap();
    b.app.press("tab");
    b.app.press("return");
    assert_eq!(b.footer().as_deref(), Some("That entry changed since it was listed, so nothing went to the Trash."));
    assert!(b.has("leigh-teabing.md"), "the entry before the swap is not trashed either");
    assert_eq!(fs::read_to_string(b.root.join("robert-langdon.md")).unwrap(), "Griphook\n", "the new entry stays");
}

#[test]
fn a_mark_whose_entry_is_gone_before_the_confirm_is_not_named() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md", "sophie-neveu.md"]);
    mark_from_leigh(&b, 3);
    fs::remove_file(b.root.join("leigh-teabing.md")).unwrap();
    b.app.press("D");
    let dialog = b.dialog().expect("the confirm opens");
    assert_eq!(dialog.title, "Move 2 items to the Trash?");
    assert!(!dialog.message.contains("leigh-teabing"), "{}", dialog.message);
}

#[test]
fn the_menu_names_the_delete_count() {
    let b = browsing(&["leigh-teabing.md", "robert-langdon.md"]);
    mark_from_leigh(&b, 2);
    b.app.press("space");
    assert_eq!(b.menu_row("Delete 2 items…"), Some(true));
}

#[test]
fn a_link_is_refused_before_the_confirm_because_finder_cannot_trash_it() {
    let b = browsing(&["vittoria-vetra.md"]);
    symlink(b.root.join("vittoria-vetra.md"), b.root.join("robert-langdon-link.md")).unwrap();
    let entries = crate::services::fs::list(&b.root, &[]).unwrap();
    b.model.borrow_mut().browser.listed(&b.root, entries);
    b.select("robert-langdon-link.md");
    b.app.press("D");
    assert!(b.dialog().is_none(), "no confirm opens for a link");
    assert_eq!(b.footer().as_deref(), Some("A link cannot go to the Trash through Finder, so nothing was moved."));
    assert!(b.root.join("robert-langdon-link.md").symlink_metadata().is_ok(), "the link stays");
}

#[test]
fn a_trash_run_reveals_the_row_after_the_selection_when_that_row_is_kept() {
    let b = browsing(&["harry-potter.md", "hermione-granger.md", "robert-langdon.md", "sophie-neveu.md"]);
    b.select("harry-potter.md");
    b.app.press("s");
    assert_eq!(b.selected(), "hermione-granger.md");
    b.app.press("D");
    b.app.press("tab");
    b.app.press("return");
    b.await_selected("robert-langdon.md");
    assert!(!b.has("harry-potter.md") && b.has("hermione-granger.md"));
}
