//! Search in the rendered Markdown view through the key path (plan-00010 phase 2): `/` opens the prompt, the query counts
//! the occurrences, `n` and `N` step through them, Esc clears the marks, and `m` clears the search.

use std::path::{Path, PathBuf};

use fmc::reader::document::decode;
use fmc::theme::DARK;
use opentui::style::Color;
use opentui::testing::TestApp;

use super::*;
use crate::ui::styled::color;

const NOTES: &str = "# Hogwarts\n\nThe *Sorting Hat* waits for **Harry Potter** in the Great Hall.\n\n- Gryffindor: Harry Potter, Hermione Granger\n";

/// `notes.md` open in the rendered view, with the panes wired to the keys, on a screen `columns` wide.
fn reading(columns: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    reading_text(columns, NOTES)
}

/// The same, with `markdown` as the file.
fn reading_text(columns: u16, markdown: &str) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(columns, 30);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), Path::new("/hogwarts"), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    wire_mouse(&model, &ui);
    capture::wire(app.app(), &model, &ui);
    let notes = root.join("notes.md");
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&root, vec![fmc::explorer::Entry::new(&root, "notes.md", fmc::explorer::Kind::File)]);
        m.browser.select(&notes);
    }
    let document = decode(markdown.as_bytes()).expect("a text document");
    reader::show_text(&mut model.borrow_mut(), &ui.borrow(), &notes, document, String::new());
    model.borrow_mut().focus = Focus::Reader;
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

fn footer(model: &Rc<RefCell<Model>>) -> String {
    model.borrow().message.as_ref().map_or_else(String::new, |message| message.text.clone())
}

/// The text of the cells drawn in `colour`, as one string.
fn drawn_in(app: &TestApp, colour: Color) -> String {
    let capture = app.render();
    let mut out = String::new();
    for y in 0..capture.height {
        for x in 0..capture.width {
            if let Some(cell) = capture.cell(x, y).filter(|cell| cell.bg == colour) {
                out.push_str(&cell.symbol);
            }
        }
    }
    out
}

#[test]
fn slash_counts_the_occurrences_in_the_rendered_view() {
    let (app, model, _ui) = reading(100);
    app.press("/");
    assert!(model.borrow().search.as_ref().is_some_and(|search| search.rendered), "the rendered view searches itself");
    app.type_text("harry potter");
    assert_eq!(footer(&model), "/harry potter  1 of 2", "{}", app.text());
    assert_eq!(drawn_in(&app, color(DARK.target_bg)), "Harry Potter", "the first hit is the current one");
}

#[test]
fn n_and_capital_n_step_through_the_hits_and_wrap() {
    let (app, model, _ui) = reading(100);
    app.press("/");
    app.type_text("harry potter");
    // The prompt takes plain keys until Esc closes it, keeping the query (the code view's rule).
    app.press("escape");
    app.press("n");
    assert_eq!(footer(&model), "/harry potter  2 of 2");
    app.press("n");
    assert_eq!(footer(&model), "/harry potter  1 of 2", "n wraps to the first hit");
    app.press("N");
    assert_eq!(footer(&model), "/harry potter  2 of 2", "N steps back, wrapping to the last hit");
}

#[test]
fn escape_clears_the_marks_and_the_search() {
    let (app, model, _ui) = reading(100);
    app.press("/");
    app.type_text("harry");
    app.press("escape");
    app.press("escape");
    assert!(model.borrow().search.is_none());
    assert!(drawn_in(&app, color(DARK.target_bg)).is_empty(), "{}", app.text());
    assert!(drawn_in(&app, color(DARK.match_bg)).is_empty(), "{}", app.text());
}

#[test]
fn toggling_the_source_clears_the_search() {
    let (app, model, _ui) = reading(100);
    app.press("/");
    app.type_text("harry");
    // An open prompt takes `m` as text, so the prompt closes first.
    app.press("escape");
    app.press("m");
    assert!(model.borrow().search.is_none(), "leaving the rendered view clears the search (D8)");
    assert!(drawn_in(&app, color(DARK.match_bg)).is_empty(), "{}", app.text());
}

#[test]
fn counts_stay_refused_in_the_rendered_view() {
    let (app, model, _ui) = reading(100);
    app.press("3");
    assert!(model.borrow().parser.pending_count().is_none(), "a count is refused in the rendered view");
}

#[test]
fn a_hit_below_the_view_scrolls_into_view() {
    let mut markdown = String::from("# Ledger\n\n");
    for line in 1..=40 {
        markdown.push_str(&format!("Paragraph {line} of the Hogwarts ledger.\n\n"));
    }
    markdown.push_str("The last word belongs to Albus Dumbledore.\n");
    let (app, model, ui) = reading_text(100, &markdown);
    assert_eq!(ui.borrow().viewer.position().1, 0.0);
    app.press("/");
    app.type_text("dumbledore");
    app.render();
    assert!(ui.borrow().viewer.position().1 > 0.0, "the view moves to the hit");
    assert_eq!(drawn_in(&app, color(DARK.target_bg)), "Dumbledore", "{}", app.text());
    assert_eq!(footer(&model), "/dumbledore  1 of 1");
}
