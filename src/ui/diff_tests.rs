//! The stacked body's width on a real frame: a diff whose lines fit the pane has no overflow, a longer line widens the
//! stack to that line, and a folded block or a wrapped body stays pane-wide.

use fmc::diff::{Change, ChangeKind, FilePatch, Folds, Patch};
use fmc::theme::{DARK, Hex, Mode};
use opentui::testing::{Capture, TestApp};

use super::*;

const PANE: u32 = 70;
const DIGITS: usize = 2;

/// One changed file whose hunk replaces its second line with `added`.
fn files(added: &str) -> Vec<FilePatch> {
    let text = format!("@@ -1,2 +1,2 @@\n [package]\n-version = \"0.0.1\"\n+{added}\n");
    let patch = Patch::Text { text, hunks: 1, added: 1, removed: 1 };
    let change = Change {
        path: "ledger.toml".into(),
        old_path: None,
        kind: ChangeKind::Modified,
        added: Some(1),
        removed: Some(1),
        binary: false,
        staged: None,
        unstaged: Some(true),
    };
    vec![FilePatch { change, patch }]
}

/// The body of `files` on a frame `PANE` columns wide, with `wrap` and `split` set and `folded` files folded.
fn frame(files: &[FilePatch], wrap: bool, split: bool, folded: bool) -> (TestApp, DiffBody, Capture) {
    let app = TestApp::new(PANE as u16, 24);
    let body = DiffBody::new(app.app());
    app.app().root().add(&body.root);
    body.root.visible(true);
    body.set_pane(PANE);
    body.set_split(split);
    body.sync(app.app(), (1, 0, DIGITS, Mode::Dark), files, &DARK, &|_| None);
    body.set_wrap(wrap);
    let mut folds = Folds::default();
    if folded {
        folds.toggle("ledger.toml");
    }
    body.paint_headers(files, None, &folds, Hex::rgb(0, 0, 0), &DARK);
    let capture = app.render();
    (app, body, capture)
}

/// The rightmost column the header's stat bar reaches, on the row that draws it.
fn bar_edge(capture: &Capture) -> Option<u32> {
    (0..capture.height)
        .find_map(|y| (0..capture.width).filter(|&x| capture.cell(x, y).is_some_and(|cell| cell.symbol == "▪")).max())
}

/// The columns a line of `text` needs beside its gutter: the digits, the gutter's four cells, and the text.
fn need(text: &str) -> usize {
    DIGITS + 4 + text.chars().count()
}

#[test]
fn a_diff_whose_lines_fit_the_pane_is_pane_wide_and_its_bar_is_inside_it() {
    let short = "name = \"ledger\"";
    for (wrap, split) in [(false, false), (false, true), (true, false)] {
        let (_app, body, capture) = frame(&files(short), wrap, split, false);
        assert_eq!(body.stack_width(), PANE as usize, "wrap {wrap}, split {split}: no overflow");
        let edge = bar_edge(&capture).unwrap_or_else(|| panic!("no stat bar:\n{}", capture.text()));
        assert!(edge < PANE, "wrap {wrap}, split {split}: the bar ends at column {edge}, inside the pane");
    }
}

#[test]
fn a_line_longer_than_the_pane_widens_the_stack_to_its_need() {
    let long = "# ".to_string() + &"Hermione keeps the vault key and the hour of its last audit. ".repeat(2);
    let (_app, body, _capture) = frame(&files(&long), false, false, false);
    assert_eq!(body.stack_width(), need(&long), "the stack is the widest line's need, gutter and sign included");
}

#[test]
fn a_folded_or_wrapped_long_line_keeps_the_stack_pane_wide() {
    let long = "x".repeat(120);
    let (_app, body, _capture) = frame(&files(&long), false, false, true);
    assert_eq!(body.stack_width(), PANE as usize, "a folded block takes no part");
    let (_app, body, _capture) = frame(&files(&long), true, false, false);
    assert_eq!(body.stack_width(), PANE as usize, "a wrapped body is pane-wide");
}
