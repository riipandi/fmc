//! The rendered view keeps each heading's marker in the heading's colour (plan-00011, D1 and D2): the `#` runs show,
//! and emphasis and code markers stay concealed.

use fmc::theme::DARK;
use opentui::testing::Capture;
use opentui::testing::TestApp;

use super::*;

const HOGWARTS: &str = "# Hogwarts\n\n## Gryffindor\n\n### Dumbledore\n\nThe *Sorting Hat* and `Potions`.\n";

/// The rendered view of `markdown` on a screen `columns` wide.
fn rendered(columns: u16, markdown: &str) -> (TestApp, MarkdownView) {
    let app = TestApp::new(columns, 30);
    let view = MarkdownView::new(app.app(), &DARK);
    app.app().root().add(&view.widget());
    view.visible(true);
    view.show(markdown);
    (app, view)
}

/// The row and the column where `text` starts, on the first row that contains it.
fn find(capture: &Capture, text: &str) -> Option<(u32, u32)> {
    (0..capture.height)
        .find_map(|y| capture.row(y).find(text).map(|byte| (y, capture.row(y)[..byte].chars().count() as u32)))
}

#[test]
fn each_heading_level_shows_its_marker_in_the_heading_colour() {
    let (app, _view) = rendered(80, HOGWARTS);
    let capture = app.render();
    let heading = color(DARK.syntax.function);
    for (prefix, title) in [("# ", "Hogwarts"), ("## ", "Gryffindor"), ("### ", "Dumbledore")] {
        let (y, x) = find(&capture, &format!("{prefix}{title}")).unwrap_or_else(|| panic!("{}", app.text()));
        for column in x..x + prefix.len() as u32 + title.len() as u32 {
            let cell = capture.cell(column, y).expect("a cell of the heading");
            assert_eq!(cell.fg, heading, "column {column} of {prefix}{title} in {}", app.text());
        }
    }
}

#[test]
fn emphasis_and_code_markers_stay_concealed() {
    let (app, _view) = rendered(80, HOGWARTS);
    let text = app.text();
    assert!(text.contains("The Sorting Hat and Potions."), "{text}");
    assert!(!text.contains('*') && !text.contains('`'), "{text}");
}

#[test]
fn a_mermaid_fence_beside_a_heading_still_draws() {
    let source = "# Hogwarts\n\n```mermaid\nflowchart LR\n    A[Harry Potter] --> B[Albus Dumbledore]\n```\n";
    let (app, _view) = rendered(100, source);
    let text = app.text();
    assert!(text.contains("# Hogwarts"), "{text}");
    assert!(text.contains('┌') && text.contains("Harry Potter"), "{text}");
    assert!(!text.contains("flowchart LR"), "the diagram replaces its source: {text}");
}
