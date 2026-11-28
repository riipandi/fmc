use std::fs;
use std::path::Path;

use fmc::explorer::{Entry, Kind};
use opentui::testing::TestApp;

use super::mouse::Click;
use super::*;
use crate::ui::tree::LIST_TOP;

const DOWN: MouseEventKind = MouseEventKind::Down(MouseButton::Left);
const DRAG: MouseEventKind = MouseEventKind::Drag(MouseButton::Left);
const UP: MouseEventKind = MouseEventKind::Up(MouseButton::Left);

#[test]
fn a_click_completes_on_its_release_not_its_press() {
    let mut click = Click::default();
    assert!(!click.feed(DOWN, 5, 3));
    assert!(click.feed(UP, 5, 3));
}

#[test]
fn a_press_dragged_away_is_not_a_click() {
    let mut click = Click::default();
    click.feed(DOWN, 5, 3);
    click.feed(DRAG, 5, 4);
    assert!(!click.feed(UP, 5, 4));
}

#[test]
fn a_swipe_that_returns_to_its_cell_is_not_a_click() {
    let mut click = Click::default();
    click.feed(DOWN, 5, 3);
    click.feed(DRAG, 6, 3);
    click.feed(DRAG, 5, 3);
    assert!(!click.feed(UP, 5, 3));
}

#[test]
fn a_release_at_another_cell_is_not_a_click() {
    let mut click = Click::default();
    click.feed(DOWN, 5, 3);
    assert!(!click.feed(UP, 5, 4));
}

/// Two files listed under `/hogwarts` at 80x24, the first selected, with the list's mouse wired.
fn listing() -> (TestApp, Rc<RefCell<Model>>) {
    listing_at(80, 24)
}

/// Two files listed under `/hogwarts` on a `columns` by `rows` screen, the first selected, with the list's mouse wired.
/// The list's rows start at `LIST_TOP`, so the second file is on the row below the first.
fn listing_at(columns: u16, rows: u16) -> (TestApp, Rc<RefCell<Model>>) {
    let app = TestApp::new(columns, rows);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    {
        let mut m = model.borrow_mut();
        m.browser.listed(
            &root,
            vec![Entry::new(&root, "owls.txt", Kind::File), Entry::new(&root, "hedwig.txt", Kind::File)],
        );
        m.browser.select(&root.join("owls.txt"));
    }
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model)
}

fn selected(model: &Rc<RefCell<Model>>) -> PathBuf {
    model.borrow().browser.selected().to_path_buf()
}

#[test]
fn a_row_acts_when_the_release_ends_the_click() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.mouse(DOWN, 3, row);
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"), "the press alone selects nothing");
    app.mouse(UP, 3, row);
    assert_eq!(selected(&model), Path::new("/hogwarts/hedwig.txt"));
}

#[test]
fn a_drag_across_a_row_acts_on_nothing() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.drag((3, row), (3, row + 1));
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"));
}

#[test]
fn a_swipe_that_returns_to_its_row_acts_on_nothing() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.drag((3, row), (3, row));
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"), "a drag that ends on its cell is still a drag");
}

#[test]
fn a_release_off_the_row_does_not_act() {
    let (app, model) = listing();
    let row = LIST_TOP + 1;
    app.mouse(DOWN, 3, row);
    app.mouse(UP, 3, LIST_TOP - 1);
    assert_eq!(selected(&model), Path::new("/hogwarts/owls.txt"));
}

/// The bytes the phone client sent, from the probe log (`/tmp/fmc-probe/touch.log`, 2026-11-27). A tap is a press and
/// its release at one cell (43 taps, none unpaired). A swipe is a run of wheel codes, 64 up and 65 down, with no drag
/// events. These are SGR coordinates, one-based.
const TAP: &str = "\x1b[<0;31;18M\x1b[<0;31;18m";
const SWIPE_DOWN: &str =
    "\x1b[<65;13;26M\x1b[<65;12;24M\x1b[<65;11;23M\x1b[<65;10;23M\x1b[<65;9;22M\x1b[<65;8;21M\x1b[<65;7;20M";

/// A 49-column phone screen (the compact tier) listing sixty files, the first selected, with the mouse wired.
fn phone() -> (TestApp, Rc<RefCell<Model>>) {
    let app = TestApp::new(49, 40);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    {
        let mut m = model.borrow_mut();
        let files = (0..60).map(|i| Entry::new(&root, &format!("hogwarts-{i:02}.txt"), Kind::File)).collect();
        m.browser.listed(&root, files);
        m.browser.select(&root.join("hogwarts-00.txt"));
    }
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model)
}

#[test]
fn a_logged_tap_acts_on_the_row_it_lands_on() {
    let (app, model) = phone();
    app.feed(TAP.as_bytes());
    // The logged row 18 is SGR's 1-based row, row 17 on the screen: list line 14, as the list starts on row 3.
    assert_eq!(selected(&model), Path::new("/hogwarts/hogwarts-14.txt"));
    assert!(model.borrow().loading, "compact: a tap on a file opens it");
}

#[test]
fn a_logged_swipe_scrolls_the_list_and_opens_nothing() {
    let (app, model) = phone();
    app.feed(SWIPE_DOWN.as_bytes());
    let m = model.borrow();
    assert_eq!(m.browser.top(), 7, "seven wheel steps, one row each");
    assert_eq!(m.browser.selected(), Path::new("/hogwarts/hogwarts-00.txt"));
    assert!(!m.loading, "a swipe is no tap");
}

#[test]
fn a_tap_after_a_logged_swipe_acts_on_the_row_under_it_once_scrolled() {
    let (app, model) = phone();
    app.feed(SWIPE_DOWN.as_bytes());
    app.feed(TAP.as_bytes());
    assert_eq!(selected(&model), Path::new("/hogwarts/hogwarts-21.txt"), "list line 14 after seven rows of scroll");
}

#[test]
fn a_first_tap_on_a_row_acts_in_medium_with_the_reader_focused() {
    for columns in [60, 70, 79] {
        let (app, model) = listing_at(columns, 24);
        model.borrow_mut().focus = Focus::Reader;
        app.click(3, LIST_TOP + 1);
        assert_eq!(selected(&model), Path::new("/hogwarts/hedwig.txt"), "{columns}: the first tap selects the row");
        assert_eq!(model.borrow().focus, Focus::List, "{columns}: and the list has the focus");
    }
}

#[test]
fn a_first_tap_on_a_file_opens_it_in_compact_with_the_reader_focused() {
    let (app, model) = phone();
    model.borrow_mut().focus = Focus::Reader;
    app.feed(TAP.as_bytes());
    assert_eq!(selected(&model), Path::new("/hogwarts/hogwarts-14.txt"));
    assert!(model.borrow().loading, "compact: the first tap opens the file");
}

/// A git repository with two tracked files, both changed, on the changes screen at `columns` wide, with the mouse wired.
fn changes_at(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>, tempfile::TempDir) {
    changes_of(columns, &["hedwig.txt".into(), "owls.txt".into()])
}

/// A git repository with the named files tracked and each one changed, on the changes screen at `columns` wide.
fn changes_of(columns: u16, names: &[String]) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "dumbledore@hogwarts.test"]);
    git(root, &["config", "user.name", "Albus Dumbledore"]);
    for name in names {
        fs::write(root.join(name), "Hogwarts\n").unwrap();
    }
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "Hogwarts"]);
    for name in names {
        fs::write(root.join(name), "Hogwarts, changed\n").unwrap();
    }
    let app = TestApp::new(columns, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), root.to_path_buf(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    {
        let mut m = model.borrow_mut();
        m.changes_load = Some(1);
        changes::listed(&mut m, 1, crate::services::changes::list(root));
    }
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui, dir)
}

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

fn selected_change(model: &Rc<RefCell<Model>>) -> Option<String> {
    model.borrow().change_list.selected().map(|change| change.path.clone())
}

#[test]
fn a_first_tap_on_a_change_acts_below_80_columns_with_the_reader_focused() {
    for columns in [49, 60, 70, 79] {
        let (app, model, ui, _dir) = changes_at(columns);
        if !layout::shown(&model.borrow(), columns) {
            layout::toggle_key(&mut model.borrow_mut(), columns);
            paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
            app.render();
        }
        let before = selected_change(&model);
        model.borrow_mut().focus = Focus::Reader;
        let row = LIST_TOP + 1;
        app.click(3, row);
        assert_ne!(selected_change(&model), before, "{columns}: the first tap selects the change");
        assert_eq!(model.borrow().focus, Focus::List, "{columns}: and the list has the focus");
    }
}

#[test]
fn at_80_columns_the_first_tap_on_a_change_only_focuses_the_list() {
    let (app, model, _ui, _dir) = changes_at(80);
    model.borrow_mut().focus = Focus::Reader;
    let before = selected_change(&model);
    app.click(3, LIST_TOP + 1);
    assert_eq!(selected_change(&model), before, "the first press only focuses the list");
    assert_eq!(model.borrow().focus, Focus::List);
    app.click(3, LIST_TOP + 1);
    assert_ne!(selected_change(&model), before, "the next click selects the row");
}

/// Thirty files, so every compact list runs past its rows on a 24-row screen.
fn many() -> Vec<String> {
    (0..30).map(|i| format!("hogwarts-{i:02}.txt")).collect()
}

/// The last list row of a compact screen 24 rows tall: row 2 is the header's rule, row 22 the bottom border, row 23 the
/// bottom bar.
const LAST_ROW: u16 = 21;

/// The name in `names` that is drawn on the screen's `row`, if one is.
fn name_on(app: &TestApp, row: u16, names: &[String]) -> Option<String> {
    let line = app.render().row(u32::from(row));
    names.iter().find(|name| line.contains(name.as_str())).cloned()
}

/// The files `names` listed under `/hogwarts` on a 24-row screen `columns` wide, the first selected, with the mouse wired.
fn tree_of(columns: u16, names: &[String]) -> (TestApp, Rc<RefCell<Model>>) {
    let app = TestApp::new(columns, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&root, names.iter().map(|name| Entry::new(&root, name, Kind::File)).collect());
        m.browser.select(&root.join(&names[0]));
    }
    wire_mouse(&model, &ui);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model)
}

#[test]
fn the_first_and_last_tree_rows_open_their_file_in_compact() {
    let names = many();
    for columns in [40, 59] {
        for row in [LIST_TOP, LAST_ROW] {
            let (app, model) = tree_of(columns, &names);
            let border = app.render().row(u32::from(LAST_ROW) + 1);
            assert!(border.starts_with('╰'), "{columns}: the bottom border is below the list: {border:?}");
            let drawn = name_on(&app, row, &names).unwrap_or_else(|| panic!("{columns}: no file on row {row}"));
            app.click(4, row);
            assert!(
                selected(&model).ends_with(&drawn),
                "{columns}, row {row}: the tap opens {drawn}, which is drawn there"
            );
            assert!(model.borrow().loading, "{columns}, row {row}: a tap on a file opens it");
        }
    }
}

#[test]
fn the_first_and_last_finder_rows_open_their_file_in_compact() {
    let names = many();
    for columns in [40, 59] {
        for row in [LIST_TOP, LAST_ROW] {
            let dir = tempfile::tempdir().unwrap();
            for name in &names {
                fs::write(dir.path().join(name), "Hogwarts\n").unwrap();
            }
            let app = TestApp::new(columns, 24);
            let model = Rc::new(RefCell::new(new_model(app.app(), dir.path().to_path_buf(), None)));
            let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
            crate::app::capture::wire(app.app(), &model, &ui);
            wire_mouse(&model, &ui);
            app.press("/");
            {
                // The index is walked on a worker in the app; here the walk runs at once and its answer is applied.
                let mut m = model.borrow_mut();
                let index = crate::services::find::index(dir.path(), |_| false, fmc::find::MAX_INDEXED);
                let session = m.find_session;
                find::indexed(&mut m, session, index);
            }
            app.type_text("txt");
            paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
            app.render();
            let drawn = name_on(&app, row, &names).unwrap_or_else(|| panic!("{columns}: no match on row {row}"));
            app.click(4, row);
            // The folder is not listed here, so the pick waits on its reveal: the pending open names the row's file.
            let pending = model.borrow().pending.as_ref().map(|pending| (pending.target.clone(), pending.open));
            assert_eq!(
                pending.map(|(target, open)| (target.ends_with(&drawn), open)),
                Some((true, true)),
                "{columns}, row {row}: the tap opens {drawn}, drawn there"
            );
            assert!(model.borrow().finder.is_none(), "{columns}, row {row}: picking closes the finder");
        }
    }
}

#[test]
fn the_first_and_last_change_rows_open_their_change_in_compact() {
    let names = many();
    for columns in [40, 59] {
        for row in [LIST_TOP, LAST_ROW] {
            let (app, model, _ui, _dir) = changes_of(columns, &names);
            let drawn = name_on(&app, row, &names).unwrap_or_else(|| panic!("{columns}: no change on row {row}"));
            app.click(4, row);
            assert!(
                selected_change(&model).is_some_and(|path| path.ends_with(drawn.as_str())),
                "{columns}, row {row}: the tap selects {drawn}, which is drawn there"
            );
        }
    }
}

#[test]
fn a_tap_on_the_find_segment_of_the_bottom_bar_opens_the_finder_at_59_columns() {
    let (app, model) = tree_of(59, &many());
    app.click(crate::ui::bars::cells(59, 3)[0].start as u16, 23);
    assert!(model.borrow().finder.is_some(), "the Find segment presses /");
}

/// After the terminal gets shorter, the tree's thumb still ends on the list's last row.
#[test]
fn the_tree_thumb_reaches_the_bottom_after_the_terminal_shrinks() {
    let app = TestApp::new(80, 40);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui: Rc<RefCell<Ui>> = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    let names: Vec<String> = (0..60).map(|n| format!("owl-{n:02}.txt")).collect();
    model.borrow_mut().browser.listed(&root, names.iter().map(|name| Entry::new(&root, name, Kind::File)).collect());
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    app.resize(80, 20);
    model.borrow_mut().browser.select(&root.join("owl-59.txt"));
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    let text = app.text();
    let rows: Vec<&str> = text.lines().collect();
    // 20 rows: the list ends above the bottom border (row 17) and the two footer rows.
    let last = rows[16];
    assert!(last.contains("owl-59.txt"), "the list is at its end: {rows:#?}");
    assert!(last.contains('▐') || last.contains('▗'), "the thumb reaches the last row: {rows:#?}");
}

/// A click on a change row in the split view selects it and keeps the focus on the list, as a tree click does. Below 80
/// columns the open diff hides the list, so the widths here are wide.
#[test]
fn a_change_row_click_keeps_the_focus_on_the_list_in_the_split_view() {
    // Short names, so the counts beside them leave the whole name drawn in a medium-width sidebar.
    let names: Vec<String> = (0..10).map(|i| format!("owl-{i}.md")).collect();
    for columns in [80, 120] {
        let (app, model, _ui, _dir) = changes_of(columns, &names);
        let row = LIST_TOP + 1;
        let drawn = name_on(&app, row, &names).unwrap_or_else(|| panic!("{columns}: no change on row {row}"));
        app.click(4, row);
        assert!(
            selected_change(&model).is_some_and(|path| path.ends_with(drawn.as_str())),
            "{columns}: selects {drawn}"
        );
        assert_eq!(model.borrow().focus, Focus::List, "{columns}: the focus stays on the list");
        // The patches arrive on a worker; the body would take the focus then if the click asked for it.
        assert!(!model.borrow().changes.as_ref().unwrap().focus_when_ready, "{columns}: no focus move is queued");
    }
}
