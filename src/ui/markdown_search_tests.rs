//! The rendered view's search marks: every occurrence the index counts is marked where the index
//! says, the current one in the current colour, and a fence the renderer does not draw is counted but not marked.

use fmc::reader::rendered_search::Slot;
use fmc::theme::DARK;
use opentui::style::Color;
use opentui::testing::{Capture, TestApp};
use opentui::widget::{Container, ScrollBox};

use super::*;
use crate::ui::markdown::MarkdownView;
use crate::ui::styled::color;

const SHOW: &str = "# Hogwarts School

The *Sorting Hat* waits for **Harry Potter** in the Great Hall.

- Gryffindor: Harry Potter, Hermione Granger

| House | Ghost |
| --- | --- |
| Ravenclaw | The Grey Lady |

```rust
fn vault() -> u32 {
    713
}
```

```make
echo vault
```

> Albus Dumbledore keeps the vault key.
";

fn current_colour() -> Color {
    color(DARK.target_bg)
}

fn match_colour() -> Color {
    color(DARK.match_bg)
}

/// A view of `source` on a screen `columns` wide, with no marks.
fn shown(columns: u16, source: &str) -> (TestApp, MarkdownView) {
    let app = TestApp::new(columns, 60);
    let view = MarkdownView::new(app.app(), &DARK);
    app.app().root().add(&view.widget());
    view.visible(true);
    view.show(source);
    (app, view)
}

/// The text of the cells drawn in `colour`, row by row, as runs.
fn runs_in(capture: &Capture, colour: Color) -> Vec<String> {
    (0..capture.height)
        .map(|y| {
            (0..capture.width)
                .filter_map(|x| capture.cell(x, y))
                .filter(|cell| cell.bg == colour)
                .map(|cell| cell.symbol.clone())
                .collect::<String>()
        })
        .filter(|run| !run.is_empty())
        .collect()
}

#[test]
fn every_counted_occurrence_is_marked_where_the_index_puts_it() {
    for query in ["harry potter", "vault", "grey lady", "e"] {
        let (app, view) = shown(200, SHOW);
        let rendered = view.index(SHOW);
        let hits = rendered.hits(query);
        assert!(!hits.is_empty(), "{query}");
        for hit in hits.iter().filter(|hit| hit.slot.is_some()) {
            view.mark(Some(Mark { query: query.into(), current: hit.slot.map(|slot| (slot, hit.at.clone())) }));
            let capture = app.render();
            let piece = rendered.pieces.iter().find(|piece| piece.slot == hit.slot).expect("the hit's piece");
            let expected = piece.text[hit.at.clone()].to_owned();
            let current = runs_in(&capture, current_colour());
            assert_eq!(current.concat(), expected, "{query}: the current mark, in {:?}\n{}", hit.slot, app.text());
        }
    }
}

#[test]
fn a_match_in_a_bold_span_counts_once_and_is_marked_across_spans() {
    let (app, view) = shown(200, SHOW);
    let hits = view.index(SHOW).hits("harry potter");
    assert_eq!(hits.len(), 2, "the bold `**Harry** Potter` and the list item");
    view.mark(Some(Mark { query: "harry potter".into(), current: None }));
    let capture = app.render();
    let marked: String = runs_in(&capture, match_colour()).concat();
    assert_eq!(marked, "Harry PotterHarry Potter", "{}", app.text());
}

#[test]
fn a_fence_the_renderer_does_not_draw_is_counted_but_not_marked() {
    let (app, view) = shown(200, SHOW);
    let rendered = view.index(SHOW);
    let plain = rendered.pieces.iter().find(|piece| piece.slot.is_none() && piece.text.contains("vault"));
    assert_eq!(plain.map(|piece| piece.text.as_str()), Some("echo vault"));
    assert_eq!(rendered.hits("vault").len(), 3, "the rust fence, the make fence, and the quote");
    view.mark(Some(Mark { query: "vault".into(), current: None }));
    let capture = app.render();
    let marked = runs_in(&capture, match_colour());
    assert!(marked.iter().all(|run| run == "vault"), "{marked:?}");
    assert_eq!(marked.len(), 2, "the rust fence and the quote are marked: {}", app.text());
}

#[test]
fn clearing_the_mark_draws_no_mark() {
    let (app, view) = shown(200, SHOW);
    view.mark(Some(Mark { query: "vault".into(), current: Some((Slot::Code(0), 3..8)) }));
    assert!(!runs_in(&app.render(), current_colour()).is_empty());
    view.mark(None);
    let capture = app.render();
    assert!(runs_in(&capture, current_colour()).is_empty() && runs_in(&capture, match_colour()).is_empty());
}

#[test]
fn a_shown_document_drops_the_marks_of_the_one_before() {
    let (app, view) = shown(200, SHOW);
    view.mark(Some(Mark { query: "vault".into(), current: None }));
    view.show("# Diagon Alley\n\nNo spells here.\n");
    let capture = app.render();
    assert!(runs_in(&capture, match_colour()).is_empty(), "{}", app.text());
}

#[test]
fn block_tops_follow_the_blocks_above_the_view() {
    let app = TestApp::new(60, 8);
    let scroll = ScrollBox::new(app.app()).axes(true, true).scrollbars(false).grow(1.0);
    app.app().root().add(&scroll);
    let view = MarkdownView::new(app.app(), &DARK);
    scroll.content_view().add(&view.widget());
    view.visible(true);
    view.show(SHOW);
    app.render();
    let first = view.block_tops();
    scroll.scroll_to(0.0, 6.0);
    app.render();
    let scrolled = view.block_tops();
    assert_eq!(first, scrolled, "the tops do not depend on the scroll position");
    let blocks = view.widget().block_rows();
    let origin = view.widget().rect().screen_y;
    for (index, rows) in blocks.iter().enumerate() {
        if rows.start > 0 {
            assert_eq!(f64::from(first[index].0), f64::from(rows.start) - origin, "block {index}");
        }
    }
}

const FRONT: &str = "---\ntitle: Hermione Granger\n---\n\n# Hogwarts\n\nHermione reads in the library.\n";

#[test]
fn a_hit_in_the_front_matter_and_one_below_it_are_marked_where_the_fence_is_drawn() {
    let (app, view) = shown(120, FRONT);
    let hits = view.index(FRONT).hits("hermione");
    assert_eq!(hits.len(), 2, "the front matter's title and the paragraph below it");
    view.mark(Some(Mark { query: "hermione".into(), current: None }));
    let capture = app.render();
    assert_eq!(runs_in(&capture, match_colour()).concat(), "HermioneHermione", "{}", app.text());
}
