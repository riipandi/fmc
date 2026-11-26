use std::path::Path;

use opentui::testing::TestApp;

use super::*;

/// The panes, wired and painted on a headless screen `columns` wide, with nothing open.
fn panes(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root, None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    wire_mouse(&model, &ui);
    app.render();
    (app, model, ui)
}

/// Flips the sidebar as `Ctrl+/` does, and paints the frame that the mark and the boxes follow.
fn toggle(app: &TestApp, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, columns: u16) {
    let mut m = model.borrow_mut();
    layout::toggle_key(&mut m, columns);
    paint(app.app(), &mut m, &mut ui.borrow_mut());
    app.render();
}

/// Opens the key page, which is a document for the 80-column rule.
fn open_document(app: &TestApp, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let mut m = model.borrow_mut();
    reader::toggle_keys(&mut m, &ui.borrow());
    paint(app.app(), &mut m, &mut ui.borrow_mut());
    app.render();
}

#[test]
fn the_welcome_screen_keeps_the_sidebar_at_any_width() {
    let (app, model, _) = panes(60);
    assert!(layout::shown(&model.borrow(), 60));
    assert_eq!(app.render().row(0).chars().nth(26), Some('─'), "no mark on the welcome screen");
}

#[test]
fn a_document_under_80_columns_hides_the_sidebar_and_draws_no_mark() {
    let (app, model, ui) = panes(79);
    open_document(&app, &model, &ui);
    assert!(!layout::shown(&model.borrow(), 79));
    assert_ne!(app.render().row(0).chars().next(), Some('»'), "the » mark is drawn at 80 columns and wider only");
    assert!(layout::menu_box(&model.borrow(), 79));
}

#[test]
fn the_mark_that_shows_the_sidebar_focuses_it() {
    let (app, model, ui) = panes(80);
    open_document(&app, &model, &ui);
    toggle(&app, &model, &ui, 80);
    assert!(!layout::shown(&model.borrow(), 80));
    model.borrow_mut().focus = Focus::Reader;
    app.click(0, 0);
    assert!(layout::shown(&model.borrow(), 80));
    assert_eq!(model.borrow().focus, Focus::List);
}

#[test]
fn a_hidden_sidebar_does_not_keep_the_focus() {
    let (app, model, ui) = panes(79);
    open_document(&app, &model, &ui);
    model.borrow_mut().focus = Focus::List;
    layout::sync(&mut model.borrow_mut(), 79);
    assert_eq!(model.borrow().focus, Focus::Reader);
}

#[test]
fn ctrl_arrows_do_not_resize_a_hidden_sidebar() {
    let (app, model, ui) = panes(79);
    open_document(&app, &model, &ui);
    let before = model.borrow().sidebar;
    layout::resize(&mut model.borrow_mut(), 79, 1);
    assert_eq!(model.borrow().sidebar, before);
}

#[test]
fn a_tap_on_the_reader_header_shows_a_hidden_sidebar_at_every_phone_width() {
    for columns in [40, 59, 60, 79] {
        let (app, model, ui) = panes(columns);
        open_document(&app, &model, &ui);
        assert!(!layout::shown(&model.borrow(), columns), "{columns}: a document hides the sidebar");
        app.click(columns / 2, 1);
        let m = model.borrow();
        assert!(layout::shown(&m, columns), "{columns}: the header tap shows the sidebar");
        assert_eq!(m.focus, Focus::List, "{columns}: and focuses it");
        assert_eq!(layout::drawer(&m, columns), columns < 60, "{columns}: a drawer below 60 columns");
        let width = if columns < 60 { columns } else { 28 };
        assert_eq!(layout::width(&m, columns), width, "{columns}: the drawer is full width, a sidebar 28 wide");
    }
}

#[test]
fn a_tap_on_the_reader_header_does_nothing_at_80_columns() {
    let (app, model, ui) = panes(80);
    open_document(&app, &model, &ui);
    app.click(40, 1);
    let m = model.borrow();
    assert!(layout::shown(&m, 80));
    assert_eq!(m.focus, Focus::Reader, "the press focuses the reader, as before");
}

#[test]
fn a_tap_on_the_list_header_closes_a_drawer_below_60_columns() {
    let (app, model, ui) = panes(40);
    open_document(&app, &model, &ui);
    app.click(20, 1);
    assert!(layout::shown(&model.borrow(), 40));
    app.click(5, 1);
    assert!(!layout::shown(&model.borrow(), 40));
    assert_eq!(model.borrow().focus, Focus::Reader);
}

#[test]
fn a_drawer_covers_the_reader_until_the_focus_reaches_the_reader() {
    let (app, model, ui) = panes(40);
    open_document(&app, &model, &ui);
    app.click(20, 1);
    assert!(!app.render().row(1).contains("Reader"), "the list takes the full width");
    model.borrow_mut().focus = Focus::Reader;
    layout::sync(&mut model.borrow_mut(), 40);
    assert!(!layout::shown(&model.borrow(), 40));
}

#[test]
fn the_welcome_shows_its_text_full_width_and_its_list_as_a_drawer_below_60() {
    let (app, model, _) = panes(40);
    assert!(!layout::drawer(&model.borrow(), 40), "the list starts hidden");
    assert_eq!(layout::width(&model.borrow(), 40), 0);
    app.click(20, 1);
    assert!(layout::drawer(&model.borrow(), 40), "a header tap opens the list as the drawer");
    app.click(5, 1);
    assert!(!layout::drawer(&model.borrow(), 40), "a second tap hides it");
}

#[test]
fn closing_a_drawer_leaves_a_sidebar_beside_the_reader_alone() {
    let (app, model, ui) = panes(60);
    open_document(&app, &model, &ui);
    toggle(&app, &model, &ui, 60);
    layout::close_drawer(&mut model.borrow_mut(), 60);
    assert!(layout::shown(&model.borrow(), 60));
}

/// The screen's rows as text, for a render test.
fn screen(app: &TestApp) -> String {
    let frame = app.render();
    (0..24).map(|y| format!("{}\n", frame.row(y))).collect()
}

#[test]
fn the_welcome_drops_its_card_below_50_columns_and_wraps_its_lines() {
    for (columns, card) in [(40, false), (49, false), (50, true)] {
        let (app, _, _) = panes(columns);
        assert_eq!(screen(&app).contains("open file or folder"), card, "{columns}: the key card");
    }
    let (app, _, _) = panes(40);
    assert!(screen(&app).contains("Just reading."), "the closing line wraps instead of cutting its last words");
}

/// The column of `glyph` in row `y`, which the phone boxes draw one cell wide.
fn column_of(app: &TestApp, y: u16, glyph: char) -> Option<u16> {
    app.render().row(u32::from(y)).chars().position(|c| c == glyph).map(|x| x as u16)
}

/// The screen's rows as text, for a render test.
fn rows(app: &TestApp) -> Vec<String> {
    let frame = app.render();
    (0..24).map(|y| frame.row(y)).collect()
}

#[test]
fn the_box_glyphs_are_one_cell_wide_and_the_menu_takes_the_ascii_safe_one() {
    let app = TestApp::new(40, 24);
    let context = app.app().session().context();
    let width = |text: &str| context.text_width(text, app.app().width_method()).unwrap();
    assert_eq!(width("≡"), 1, "the menu glyph");
    assert_eq!(width("✕"), 1, "the close glyph");
    assert_eq!(width("☰"), 2, "why the menu glyph is ≡: opentui measures ☰ two cells wide");
}

#[test]
fn the_menu_box_shows_the_list_as_a_drawer_below_60_and_beside_the_reader_at_60_and_79() {
    for columns in [40, 59, 60, 79] {
        // A welcome screen below 60 starts with its list hidden; a document hides it from 80 down.
        for cell in 0..3 {
            let (app, model, ui) = panes(columns);
            if columns >= 60 {
                open_document(&app, &model, &ui);
            }
            let x = column_of(&app, 1, '≡').unwrap_or_else(|| panic!("{columns}: no menu box"));
            app.click(x - 1 + cell, 1);
            let m = model.borrow();
            assert!(layout::shown(&m, columns), "{columns}, cell {cell}: the box shows the list");
            assert_eq!(m.focus, Focus::List, "{columns}: and focuses it");
            assert_eq!(layout::drawer(&m, columns), columns < 60, "{columns}: a drawer below 60");
        }
    }
}

#[test]
fn the_close_box_hides_the_list_at_40_59_60_and_79() {
    for columns in [40, 59, 60, 79] {
        for cell in 0..3 {
            let (app, model, ui) = panes(columns);
            if columns >= 60 {
                open_document(&app, &model, &ui);
            }
            app.click(columns / 2, 1);
            assert!(layout::shown(&model.borrow(), columns), "{columns}: the list shows");
            let x = column_of(&app, 1, '✕').unwrap_or_else(|| panic!("{columns}: no close box"));
            app.click(x - 1 + cell, 1);
            assert!(!layout::shown(&model.borrow(), columns), "{columns}, cell {cell}: the close box hides the list");
            assert!(column_of(&app, 1, '≡').is_some(), "{columns}: the menu box is back");
        }
    }
}

#[test]
fn the_boxes_are_absent_at_80_columns_and_the_header_tap_does_nothing() {
    let (app, model, ui) = panes(80);
    open_document(&app, &model, &ui);
    toggle(&app, &model, &ui, 80);
    assert!(column_of(&app, 1, '≡').is_none(), "no menu box at 80 columns");
    app.click(40, 1);
    assert!(!layout::shown(&model.borrow(), 80), "the reader header does not show the list at 80");
    assert!(column_of(&app, 1, '✕').is_none());
    toggle(&app, &model, &ui, 80);
    app.click(20, 1);
    assert!(layout::shown(&model.borrow(), 80));
    assert!(column_of(&app, 1, '✕').is_none(), "no close box at 80 columns");
}

#[test]
fn the_close_box_is_absent_on_the_welcome_screen_between_60_and_79() {
    for columns in [60, 79] {
        let (app, model, _) = panes(columns);
        assert!(layout::shown(&model.borrow(), columns), "the welcome keeps its list");
        assert!(column_of(&app, 1, '✕').is_none(), "{columns}: the list cannot hide here");
        app.click(5, 1);
        assert!(layout::shown(&model.borrow(), columns), "{columns}: and a tap does not hide it");
    }
}

#[test]
fn the_welcome_names_the_menu_while_the_list_is_hidden_below_80() {
    for columns in [40, 59] {
        let (app, _, _) = panes(columns);
        assert!(rows(&app).iter().any(|row| row.contains("Tap ≡ to browse files.")), "{columns}: the menu prompt");
        assert!(!rows(&app).iter().any(|row| row.contains("Select a file")), "{columns}: no select prompt");
    }
    for columns in [60, 79, 80] {
        let (app, _, _) = panes(columns);
        assert!(
            rows(&app).iter().any(|row| row.contains("Select a file and press Enter.")),
            "{columns}: keeps its text"
        );
        assert!(!rows(&app).iter().any(|row| row.contains("Tap ≡")), "{columns}: no menu prompt");
    }
}

#[test]
fn the_welcome_returns_to_the_menu_prompt_when_the_list_closes_below_60() {
    let (app, model, _) = panes(40);
    app.click(20, 1);
    assert!(!rows(&app).iter().any(|row| row.contains("Tap ≡")), "the drawer covers the welcome");
    app.click(column_of(&app, 1, '✕').unwrap(), 1);
    assert!(!layout::shown(&model.borrow(), 40));
    assert!(rows(&app).iter().any(|row| row.contains("Tap ≡ to browse files.")));
}

#[test]
fn the_closing_line_breaks_at_its_separator_below_80_and_stays_whole_at_80() {
    for columns in [49, 79] {
        let (app, _, _) = panes(columns);
        let lines = rows(&app);
        let at = lines.iter().position(|row| row.contains("Press ? for every key.")).unwrap();
        assert!(!lines[at].contains("No editing"), "{columns}: the key line stands alone");
        assert!(lines[at + 1].contains("No editing. Just reading."), "{columns}: the reading line is next");
    }
    let (app, _, _) = panes(100);
    assert!(rows(&app).iter().any(|row| row.contains("Press ? for every key.  ·  No editing. Just reading.")));
}
