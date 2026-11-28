//! `file_associations` in the reader: an association decides how a file is highlighted and whether
//! `m` renders it as Markdown, and a file with no association keeps Scouter's lookup.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use fmc::explorer::{Entry, Kind};
use fmc::reader::document::decode;
use fmc::reader::markdown::View;
use fmc::theme::DARK;
use opentui::style::Color;
use opentui::testing::TestApp;

use super::*;
use crate::ui::styled::color;

/// Keywords in a Rust file, which a Rust highlight colours and a plain file does not.
const VAULT: &str = "fn vault() {\n    let key = 1;\n}\n";

const OWLS: &str = "# Hedwig\n\n**Harry Potter** receives a letter.\n";

/// `name` open in the reader on a screen 100 columns wide, with `associations` set as `settings.json` sets them.
fn reading(name: &str, text: &str, associations: &[(&str, &str)]) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(100, 30);
    let root = PathBuf::from("/hogwarts");
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    capture::wire(app.app(), &model, &ui);
    let file = root.join(name);
    {
        let mut m = model.borrow_mut();
        for (pattern, value) in associations {
            m.associations.add(pattern, value).expect("a valid entry");
        }
        m.browser.listed(&root, vec![Entry::new(&root, name, Kind::File)]);
        m.browser.select(&file);
    }
    let document = decode(text.as_bytes()).expect("a text document");
    reader::show_text(&mut model.borrow_mut(), &ui.borrow(), &file, document, String::new());
    model.borrow_mut().focus = Focus::Reader;
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

/// The text of the cells drawn in `colour` as foreground.
fn drawn_in(app: &TestApp, colour: Color) -> String {
    let capture = app.render();
    let mut out = String::new();
    for y in 0..capture.height {
        for x in 0..capture.width {
            if let Some(cell) = capture.cell(x, y).filter(|cell| cell.fg == colour) {
                out.push_str(&cell.symbol);
            }
        }
    }
    out
}

fn language(model: &Rc<RefCell<Model>>) -> Option<&'static str> {
    model.borrow().shown.text().and_then(|(_, language, _)| language)
}

#[test]
fn an_association_highlights_a_file_its_extension_would_not() {
    let (app, model, _ui) = reading("vault.rs.bak", VAULT, &[("*.rs.bak", "rust")]);
    assert_eq!(language(&model), Some("rust"));
    assert!(drawn_in(&app, color(DARK.syntax.keyword)).contains("fn"), "{}", app.text());
}

#[test]
fn without_an_association_the_same_file_is_plain_text() {
    let (app, model, _ui) = reading("vault.rs.bak", VAULT, &[]);
    assert_eq!(language(&model), None);
    assert!(drawn_in(&app, color(DARK.syntax.keyword)).is_empty(), "{}", app.text());
}

#[test]
fn plaintext_turns_off_the_highlighting_of_a_file_that_has_one() {
    let (app, model, _ui) = reading("vault.rs", VAULT, &[("*.rs", "plaintext")]);
    assert_eq!(language(&model), None);
    assert!(drawn_in(&app, color(DARK.syntax.keyword)).is_empty(), "{}", app.text());
}

#[test]
fn an_association_to_markdown_lets_m_render_the_file() {
    let (app, model, _ui) = reading("owls.hedwig", OWLS, &[("*.hedwig", "markdown")]);
    assert_eq!(model.borrow().shown.text().and_then(|(_, _, view)| view), Some(View::Rendered));
    app.press("m");
    assert_eq!(model.borrow().shown.text().and_then(|(_, _, view)| view), Some(View::Source));
}

#[test]
fn m_does_nothing_to_a_file_with_no_markdown_association() {
    let (_app, model, _ui) = reading("owls.hedwig", OWLS, &[]);
    assert_eq!(model.borrow().shown.text().and_then(|(_, _, view)| view), None);
}
