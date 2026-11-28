//! The git signs beside the rendered view's blocks on a real repository: an edited paragraph, an
//! added section, a deletion, the setting off, and a bar that follows its block when the view scrolls.

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use fmc::explorer::{Entry, Kind};
use fmc::reader::document::decode;
use fmc::theme::Palette;
use opentui::style::Color;
use opentui::testing::{Capture, TestApp};
use tempfile::TempDir;

use super::gutter::apply;
use super::*;
use crate::services::git;
use crate::ui::styled::color;

const NOTES: &str = "# Hogwarts\n\nHermione Granger keeps the vault key.\n\nSophie Neveu owns the Louvre.\n";

fn git_in(dir: &Path, args: &[&str]) {
    let status = Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A repository whose `notes.md` is committed as `NOTES`, then rewritten as `edit` when it is given.
fn repository(edit: Option<&str>) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_in(root, &["init", "-q"]);
    git_in(root, &["config", "user.email", "dumbledore@hogwarts.test"]);
    git_in(root, &["config", "user.name", "Albus Dumbledore"]);
    fs::write(root.join("notes.md"), NOTES).unwrap();
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-q", "-m", "Harry Potter"]);
    if let Some(text) = edit {
        fs::write(root.join("notes.md"), text).unwrap();
    }
    dir
}

/// `notes.md` from `dir` open in the rendered view, its signs read as the app reads them, `rows` tall.
fn reading(dir: &Path, git_gutter: bool, rows: u16) -> (TestApp, Rc<RefCell<Model>>, Rc<RefCell<Ui>>) {
    let app = TestApp::new(80, rows);
    let root = dir.to_path_buf();
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
    let path = dir.join("notes.md");
    let text = fs::read_to_string(&path).unwrap();
    let document = decode(text.as_bytes()).expect("a text document");
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&root, vec![Entry::new(&root, "notes.md", Kind::File)]);
        m.browser.select(&path);
        m.git_gutter = git_gutter;
        m.in_repo = git::marks(dir).is_ok_and(|listing| listing.is_some());
    }
    reader::show_text(&mut model.borrow_mut(), &ui.borrow(), &path, document, String::new());
    {
        let mut m = model.borrow_mut();
        let serial = m.serial;
        if m.git_gutter && m.in_repo {
            apply(&mut m, &path, serial, &git::signs(&path));
        }
    }
    // The app repaints 20 ms after each layout (`repaint_after_layout`), so the marks follow the first frame.
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    app.render();
    (app, model, ui)
}

/// The sign cells the frame draws: the row, the glyph, and the colour of each, in the sign colours.
fn signs_drawn(capture: &Capture, p: &Palette) -> Vec<(u32, String, Color)> {
    let colours = [color(p.added), color(p.modified), color(p.removed)];
    let mut out = Vec::new();
    for y in 0..capture.height {
        for x in 0..capture.width {
            if let Some(cell) = capture.cell(x, y)
                && matches!(cell.symbol.as_str(), "▎" | "▁" | "▔")
                && colours.contains(&cell.fg)
            {
                out.push((y, cell.symbol.clone(), cell.fg));
            }
        }
    }
    out
}

/// The rows where `text` shows.
fn rows_of(capture: &Capture, text: &str) -> Vec<u32> {
    (0..capture.height).filter(|&y| capture.row(y).contains(text)).collect()
}

#[test]
fn an_edited_paragraph_gets_a_modified_bar_beside_its_rows() {
    let dir = repository(Some(&NOTES.replace("Sophie Neveu", "Robert Langdon")));
    let (app, model, _) = reading(dir.path(), true, 24);
    let palette = model.borrow().palette().clone();
    let p = &palette;
    let drawn = signs_drawn(&app.render(), p);
    let row = rows_of(&app.render(), "Robert Langdon");
    assert_eq!(drawn.iter().map(|(y, _, _)| *y).collect::<Vec<_>>(), row, "{}", app.text());
    assert!(drawn.iter().all(|(_, glyph, colour)| glyph == "▎" && *colour == color(p.modified)), "{drawn:?}");
}

#[test]
fn an_added_section_gets_bars_and_a_deletion_is_drawn_under_the_block_above() {
    let edit = "# Hogwarts\n\n## Crypt\n\nLeigh Teabing keeps the Holy Grail.\n\nSophie Neveu owns the Louvre.\n";
    let dir = repository(Some(edit));
    let (app, model, _) = reading(dir.path(), true, 24);
    let palette = model.borrow().palette().clone();
    let p = &palette;
    let capture = app.render();
    let heading = rows_of(&capture, "Crypt");
    let added = rows_of(&capture, "Leigh Teabing");
    let drawn = signs_drawn(&capture, p);
    let bars: Vec<u32> = drawn.iter().filter(|(_, glyph, _)| glyph == "▎").map(|(y, _, _)| *y).collect();
    assert!(heading.iter().chain(&added).all(|row| bars.contains(row)), "{drawn:?} {}", app.text());
    // The heading takes the line the old paragraph had, so it is modified; the new paragraph is added.
    let colour_at = |row: u32| drawn.iter().find(|(y, glyph, _)| *y == row && glyph == "▎").map(|(_, _, c)| *c);
    assert_eq!(colour_at(heading[0]), Some(color(p.modified)), "{}", app.text());
    assert_eq!(colour_at(added[0]), Some(color(p.added)), "{}", app.text());

    let dir = repository(Some("# Hogwarts\n\nSophie Neveu owns the Louvre.\n"));
    let (app, model, _) = reading(dir.path(), true, 24);
    let palette = model.borrow().palette().clone();
    let p = &palette;
    let capture = app.render();
    let under = signs_drawn(&capture, p).into_iter().find(|(_, glyph, _)| glyph == "▁");
    let heading = rows_of(&capture, "Hogwarts")[0];
    assert_eq!(under.map(|(y, _, colour)| (y, colour)), Some((heading + 1, color(p.removed))), "{}", app.text());
}

#[test]
fn the_setting_off_draws_no_bars() {
    let dir = repository(Some(&NOTES.replace("Sophie Neveu", "Robert Langdon")));
    let (app, model, _) = reading(dir.path(), false, 24);
    assert!(signs_drawn(&app.render(), model.borrow().palette()).is_empty(), "{}", app.text());
}

#[test]
fn a_bar_moves_with_its_block_when_the_view_scrolls() {
    let mut text = String::from("# Ledger\n\n");
    for line in 1..=30 {
        text.push_str(&format!("Entry {line} of the Hogwarts ledger.\n\n"));
    }
    text.push_str("The last entry belongs to Albus Dumbledore.\n");
    let dir = repository(None);
    fs::write(dir.path().join("notes.md"), &text).unwrap();
    git_in(dir.path(), &["commit", "-q", "-am", "Ledger"]);
    let edited = text.replace("Albus Dumbledore", "Minerva McGonagall");
    fs::write(dir.path().join("notes.md"), &edited).unwrap();
    let (app, model, ui) = reading(dir.path(), true, 14);
    ui.borrow().viewer.scroll_to(0.0, 200.0);
    paint(app.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
    let capture = app.render();
    let palette = model.borrow().palette().clone();
    let p = &palette;
    let bars: Vec<u32> = signs_drawn(&capture, p).into_iter().map(|(y, _, _)| y).collect();
    let row = rows_of(&capture, "Minerva McGonagall");
    assert!(!row.is_empty() && bars == row, "{bars:?} {row:?}\n{}", app.text());
}
