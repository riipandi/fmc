//! The reader's git sign strip on a real repository: the signs of an edited, deleted, and clean text, the
//! strip's column, and the answers that are dropped. A text is shown as the app shows it, with its signs read the way
//! the job reads them.

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use fmc::diff::Sign;
use fmc::reader::document::decode;
use opentui::testing::TestApp;
use tempfile::TempDir;

use super::*;
use crate::app::{Ui, build, new_model, paint};
use crate::services::git;

const COLUMNS: u16 = 80;
const SPELLS: &str = "Hermione Granger\nRobert Langdon\nSophie Neveu\n";

fn git_in(dir: &Path, args: &[&str]) {
    let status = Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A repository whose `spells.txt` is committed as `SPELLS`, then rewritten as `edit` when it is given.
fn repository(edit: Option<&str>) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_in(root, &["init", "-q"]);
    git_in(root, &["config", "user.email", "dumbledore@hogwarts.test"]);
    git_in(root, &["config", "user.name", "Albus Dumbledore"]);
    fs::write(root.join("spells.txt"), SPELLS).unwrap();
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-q", "-m", "Harry Potter"]);
    if let Some(text) = edit {
        fs::write(root.join("spells.txt"), text).unwrap();
    }
    dir
}

/// The reader at 80 columns with `spells.txt` from `dir` shown, its signs read as the app reads them: only in a
/// repository, and only with the setting on.
fn reading(dir: &Path, git_gutter: bool) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(COLUMNS, 24);
    let model = Rc::new(RefCell::new(new_model(app.app(), dir.to_path_buf(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    let path = dir.join("spells.txt");
    let text = fs::read_to_string(&path).unwrap();
    let document = decode(text.as_bytes()).expect("a text document");
    {
        let mut m = model.borrow_mut();
        m.git_gutter = git_gutter;
        m.in_repo = git::marks(dir).is_ok_and(|listing| listing.is_some());
        ui.borrow().reader.show(&document, None);
        m.serial += 1;
        let serial = m.serial;
        m.shown = Shown::Text {
            path: path.clone(),
            document,
            language: None,
            markdown: None,
            stamp: Some(String::new()),
            serial,
            signs: Vec::new(),
        };
        if m.git_gutter && m.in_repo {
            apply(&mut m, &path, serial, &git::signs(&path));
        }
    }
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

/// The sign drawn beside the text of `line`. The gutter's sign sits in the eight columns before the text, and the
/// sidebar's own `▎` (its selected row) is far to the left.
fn sign_beside(text: &str, line: &str) -> Option<char> {
    let row = text.lines().find(|row| row.contains(line))?;
    let at = row[..row.find(line)?].chars().count();
    row.chars().skip(at.saturating_sub(8)).take(at.min(8)).find(|c| matches!(c, '▎' | '▁' | '▔'))
}

/// The column the text of `line` starts at.
fn column_of(text: &str, line: &str) -> Option<usize> {
    let row = text.lines().find(|row| row.contains(line))?;
    Some(row[..row.find(line)?].chars().count())
}

#[test]
fn an_edited_line_is_modified_and_a_clean_line_has_no_sign() {
    let dir = repository(Some("Hermione Granger\nLeigh Teabing\nSophie Neveu\n"));
    let (app, _, _) = reading(dir.path(), true);
    let text = app.text();
    assert_eq!(sign_beside(&text, "Leigh Teabing"), Some('▎'), "{text}");
    assert_eq!(sign_beside(&text, "Hermione Granger"), None, "{text}");
    assert_eq!(sign_beside(&text, "Sophie Neveu"), None, "{text}");
}

#[test]
fn a_deletion_is_drawn_on_the_line_above_it() {
    let dir = repository(Some("Hermione Granger\nSophie Neveu\n"));
    let (app, _, _) = reading(dir.path(), true);
    assert_eq!(sign_beside(&app.text(), "Hermione Granger"), Some('▁'), "{}", app.text());
}

#[test]
fn the_git_gutter_off_draws_no_signs_and_no_strip() {
    let edited = repository(Some("Hermione Granger\nLeigh Teabing\nSophie Neveu\n"));
    let (off, _, _) = reading(edited.path(), false);
    let text = off.text();
    assert_eq!(sign_beside(&text, "Leigh Teabing"), None, "{text}");
    let clean = repository(None);
    let (reference, _, _) = reading(clean.path(), false);
    assert_eq!(column_of(&text, "Leigh Teabing"), column_of(&reference.text(), "Hermione Granger"));
}

#[test]
fn the_strip_takes_one_column_in_a_repository_and_none_outside_one() {
    let clean = repository(None);
    let (off, _, _) = reading(clean.path(), false);
    let (on, _, _) = reading(clean.path(), true);
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("spells.txt"), SPELLS).unwrap();
    let (plain, _, _) = reading(outside.path(), true);
    let base = column_of(&off.text(), "Hermione Granger").expect("the text is drawn");
    assert_eq!(column_of(&on.text(), "Hermione Granger"), Some(base + 1), "the strip's column (D6)");
    assert_eq!(column_of(&plain.text(), "Hermione Granger"), Some(base), "no strip outside a repository");
    assert_eq!(
        plain.text(),
        reading(outside.path(), false).0.text(),
        "outside a repository, the setting changes nothing"
    );
}

#[test]
fn an_answer_for_another_showing_or_file_is_dropped() {
    let dir = repository(Some("Hermione Granger\nLeigh Teabing\nSophie Neveu\n"));
    let path = dir.path().join("spells.txt");
    let app = TestApp::new(COLUMNS, 24);
    let mut m = new_model(app.app(), dir.path().to_path_buf(), None);
    m.shown = Shown::Text {
        path: path.clone(),
        document: decode(b"Hermione Granger\n").unwrap(),
        language: None,
        markdown: None,
        stamp: Some(String::new()),
        serial: 2,
        signs: Vec::new(),
    };
    let added = Ok(vec![(0, Sign::Added)]);
    apply(&mut m, &path, 1, &added);
    assert!(drawn(&m).0.is_empty(), "an older showing's answer is dropped");
    apply(&mut m, &dir.path().join("owls.txt"), 2, &added);
    assert!(drawn(&m).0.is_empty(), "another file's answer is dropped");
    apply(&mut m, &path, 2, &added);
    assert_eq!(drawn(&m).0, &[(0, Sign::Added)]);
    apply(&mut m, &path, 2, &Err("git exited with status 143.".into()));
    assert!(drawn(&m).0.is_empty(), "a git failure clears the signs without a notice (D5)");
}
