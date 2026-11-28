use opentui::testing::TestApp;

use super::*;

/// The panes, wired and painted on a headless screen `columns` wide, with nothing open.
fn panes(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 24);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root, None)));
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
    assert!(layout::reader_chevron(&model.borrow(), 79).is_some(), "the reader header shows its chevron");
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
fn a_tap_on_the_reader_header_does_nothing_at_80_columns() {
    let (app, model, ui) = panes(80);
    open_document(&app, &model, &ui);
    app.click(40, 1);
    let m = model.borrow();
    assert!(layout::shown(&m, 80));
    assert_eq!(m.focus, Focus::Reader, "the press focuses the reader, as before");
}

#[test]
fn the_welcome_card_is_drawn_at_80_columns_and_not_in_compact() {
    let (app, _, _) = panes(80);
    assert!(screen(&app).contains("open file or folder"), "80: the key card");
    for columns in [40, 59] {
        let (app, _, _) = panes(columns);
        assert!(!screen(&app).contains("open file or folder"), "{columns}: no welcome in compact");
    }
}

/// The screen's rows as text, for a render test.
fn screen(app: &TestApp) -> String {
    let frame = app.render();
    (0..24).map(|y| format!("{}\n", frame.row(y))).collect()
}

/// The column of `glyph` in row `y`, which the header boxes draw one cell wide.
fn column_of(app: &TestApp, y: u16, glyph: char) -> Option<u16> {
    app.render().row(u32::from(y)).chars().position(|c| c == glyph).map(|x| x as u16)
}

/// The screen's rows as text, for a render test.
fn rows(app: &TestApp) -> Vec<String> {
    let frame = app.render();
    (0..24).map(|y| frame.row(y)).collect()
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
fn the_welcome_prompt_is_the_same_at_60_79_and_80() {
    for columns in [60, 79, 80] {
        let (app, _, _) = panes(columns);
        assert!(
            rows(&app).iter().any(|row| row.contains("Select a file and press Enter.")),
            "{columns}: keeps its text"
        );
    }
}

#[test]
fn the_closing_line_breaks_at_its_separator_below_80_and_stays_whole_at_80() {
    for columns in [60, 79] {
        let (app, _, _) = panes(columns);
        let lines = rows(&app);
        let at = lines.iter().position(|row| row.contains("Press ? for every key.")).unwrap();
        assert!(!lines[at].contains("Space opens"), "{columns}: the key line stands alone");
        assert!(lines[at + 1].contains("Space opens the menu."), "{columns}: the reading line is next");
    }
    let (app, _, _) = panes(100);
    assert!(rows(&app).iter().any(|row| row.contains("Press ? for every key.  ·  Space opens the menu.")));
}

#[test]
fn the_reader_chevron_shows_the_hidden_list_at_60_and_79() {
    for columns in [60, 79] {
        let (app, model, ui) = panes(columns);
        open_document(&app, &model, &ui);
        let x = column_of(&app, 1, '›').unwrap_or_else(|| panic!("{columns}: no reader chevron"));
        assert!(column_of(&app, 1, '‹').is_none(), "{columns}: the list is hidden, so no list chevron");
        app.click(x, 1);
        let m = model.borrow();
        assert!(layout::shown(&m, columns), "{columns}: the chevron shows the list");
        assert_eq!(m.focus, Focus::List, "{columns}: and focuses it");
    }
}

#[test]
fn the_list_chevron_hides_the_list_at_60_and_79() {
    for columns in [60, 79] {
        let (app, model, ui) = panes(columns);
        open_document(&app, &model, &ui);
        let x = column_of(&app, 1, '›').unwrap_or_else(|| panic!("{columns}: no reader chevron"));
        app.click(x, 1);
        let x = column_of(&app, 1, '‹').unwrap_or_else(|| panic!("{columns}: no list chevron"));
        app.click(x, 1);
        assert!(!layout::shown(&model.borrow(), columns), "{columns}: the list chevron hides the list");
        assert!(column_of(&app, 1, '›').is_some(), "{columns}: and the reader chevron is back");
    }
}

#[test]
fn the_chevrons_are_absent_on_the_welcome_screen_and_at_80_columns() {
    for columns in [60, 79] {
        let (app, _, _) = panes(columns);
        assert!(column_of(&app, 1, '‹').is_none() && column_of(&app, 1, '›').is_none(), "{columns}: nothing to hide");
    }
    let (app, model, ui) = panes(80);
    open_document(&app, &model, &ui);
    assert!(column_of(&app, 1, '‹').is_none() && column_of(&app, 1, '›').is_none(), "80: the marks are the controls");
}

#[test]
fn the_chevron_glyphs_measure_one_cell() {
    let app = TestApp::new(40, 24);
    let context = app.app().session().context();
    let width = |text: &str| context.text_width(text, app.app().width_method()).unwrap();
    assert_eq!(width("‹"), 1, "the list chevron");
    assert_eq!(width("›"), 1, "the reader chevron");
}

#[test]
fn a_tap_on_a_medium_chevron_its_gap_or_its_title_toggles_the_list() {
    for columns in [60, 79] {
        let (app, model, ui) = panes(columns);
        open_document(&app, &model, &ui);
        // Offsets from the chevron: the chevron itself, the two gap columns after it, and the title's letters.
        for offset in [0, 1, 2, 4] {
            let x = column_of(&app, 1, '›').unwrap_or_else(|| panic!("{columns}: no reader chevron"));
            app.click(x + offset, 1);
            assert!(layout::shown(&model.borrow(), columns), "{columns}: offset {offset} shows the list");
            let x = column_of(&app, 1, '‹').unwrap_or_else(|| panic!("{columns}: no list chevron"));
            app.click(x + offset, 1);
            assert!(!layout::shown(&model.borrow(), columns), "{columns}: offset {offset} hides the list");
        }
    }
}

#[test]
fn a_compact_title_tap_goes_back_a_page_as_the_chevron_does() {
    let (app, model, ui) = panes(49);
    open_document(&app, &model, &ui);
    assert_eq!(model.borrow().pages.top(), Page::Keys);
    app.click(8, 1);
    assert_eq!(model.borrow().pages.top(), Page::Files, "the whole title box is the back target, as the chevron is");
}

#[test]
fn the_welcome_card_goes_at_medium_widths_when_the_reader_is_narrower_than_it() {
    for (columns, card) in [(60, false), (70, false), (74, false), (75, true), (79, true), (80, true)] {
        let (app, _, _) = panes(columns);
        let text = screen(&app);
        assert_eq!(text.contains("open file or folder"), card, "{columns}: the card");
        assert!(text.contains("a quiet terminal file manager"), "{columns}: the tagline stays");
        assert!(text.contains("█▀▀ █▀▄▀█ █▀▀"), "{columns}: the wordmark stays");
        assert!(text.contains("Select a file and press Enter."), "{columns}: the prompt stays");
    }
}

#[test]
fn the_welcome_tagline_and_menu_line_draw_at_80_and_60_by_24() {
    let (app, _, _) = panes(80);
    let lines = rows(&app);
    assert!(lines.iter().any(|row| row.contains("a quiet terminal file manager")), "80: the tagline is whole");
    // The closing line is cut at the reader's edge at 80 columns, as Scouter's is.
    assert!(lines.iter().any(|row| row.contains("Press ? for every key.  ·  Space")), "80: the closing line starts");
    let (app, _, _) = panes(60);
    let lines = rows(&app);
    assert!(lines.iter().any(|row| row.contains("a quiet terminal file manager")), "60: the tagline is whole");
    assert!(lines.iter().any(|row| row.contains("Space opens the menu.")), "60: the menu line is whole");
}
