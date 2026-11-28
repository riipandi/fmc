//! The source-to-block mapping against the blocks OpenTUI draws. The samples are Harry Potter
//! fixtures with the constructs the mapping must follow, and the repository's own Markdown files.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use crate::ui::markdown_images;
use fmc::theme::DARK;
use opentui::style::{Color, Span};
use opentui::testing::TestApp;
use opentui::widget::{Code, Container, InlineSpan, View, Widget, WrapMode};

use super::{Block, split};

/// The blocks of the source as the view draws it: its front matter is a fence (`reader/frontmatter.rs`).
fn rendered(source: &str) -> Vec<Block> {
    split(&markdown_images::prepared(source)).blocks
}
use crate::ui::markdown::MarkdownView;
use crate::ui::styled::color;
use crate::ui::syntax;

const HARRY_POTTER: &str = "# Hogwarts School

The *Sorting Hat* sits on a stool in the Great Hall, and **Harry Potter** waits in line.

## Houses

- Gryffindor: Harry Potter, Hermione Granger
- Slytherin: Draco Malfoy

1. Potions with Severus Snape
2. Herbology with Neville Longbottom

> \"It is our choices, Harry, that show what we truly are.\"
> Albus Dumbledore

| House | Ghost |
| --- | --- |
| Gryffindor | Nearly Headless Nick |
| Ravenclaw | The Grey Lady |

```rust
fn vault() -> u32 {
    713
}
```

```mermaid
flowchart LR
    A[Harry Potter] --> B[Albus Dumbledore]
```

Setext Heading
==============

---

Final line with `Potions` and a [Gringotts link](https://gringotts.test/vault).
";

/// Front matter that OpenTUI draws as Markdown, and a link definition it draws as nothing.
const EDGES: &str = "---
title: Hogwarts
---

# Diagon Alley

A paragraph before a link definition.

[gringotts]: https://gringotts.test/vault \"Vault 713\"

<details>
<summary>Hermione Granger's notes</summary>

Hidden text.

</details>

- Loose item one for Ron Weasley

- Loose item two for Luna Lovegood

Text after the loose list.
";

const MORE_EDGES: &str = "<!-- a comment for Hermione Granger -->

Footnote text with a note.[^1]

[^1]: The note for Dobby the house-elf.

Name | House
--- | ---
Luna Lovegood | Ravenclaw

## Heading with trailing hashes ##

+++
title = \"Hogwarts\"
+++

<div>
Inside a div for Neville Longbottom.
</div>

Line one for Minerva McGonagall
Line two after a soft break.

***

[gringotts]: https://gringotts.test/vault
";

/// Fences inside a list item and a quote, which OpenTUI's default block draws in the parent's text colour.
const NESTED_FENCES: &str = "Intro for Hermione Granger.

- Potions list:

  ```rust
  fn potion() -> u32 {
      42
  }
  ```

> Quote with a fence:
>
> ```rust
> fn spell() -> u32 {
>     7
> }
> ```

Outro for Albus Dumbledore.
";

/// The fixtures above and the repository's Markdown files.
fn samples() -> Vec<(String, String)> {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut out = vec![
        ("harry potter".to_owned(), HARRY_POTTER.to_owned()),
        ("edges".to_owned(), EDGES.to_owned()),
        ("more edges".to_owned(), MORE_EDGES.to_owned()),
    ];
    for path in [
        "README.md",
        "DESIGN.md",
        "docs/cli.md",
        "docs/configuration.md",
        "docs/development.md",
        "docs/keymaps.md",
        "docs/parity.md",
    ] {
        out.push((path.to_owned(), std::fs::read_to_string(format!("{root}/{path}")).expect("a sample file")));
    }
    out
}

/// The text of a block as the render shows it: a fence loses its fence line.
fn shown(kind: &str, source: &str) -> String {
    if kind == "fenced_code_block" {
        source.split_once('\n').map_or("", |(_, rest)| rest).to_owned()
    } else {
        source.to_owned()
    }
}

/// The first and the last word of at least four letters in a block: the words its render must show.
fn probes(kind: &str, source: &str) -> Vec<String> {
    let text = shown(kind, source);
    let words = text.split(|c: char| !c.is_alphanumeric()).filter(|word| word.chars().count() >= 4);
    let mut found: Vec<String> = words.clone().take(1).chain(words.clone().next_back()).map(str::to_owned).collect();
    found.dedup();
    found
}

/// Renders `source` in a tall app and returns the rows of each top-level block OpenTUI drew, with their text.
fn drawn(source: &str, width: u16) -> Vec<(Range<u32>, String)> {
    // Tall enough for the design sample at 40 columns, which renders past 400 rows.
    let app = TestApp::new(width, 1200);
    let view = MarkdownView::new(app.app(), &DARK, true);
    app.app().root().add(&view.widget());
    view.visible(true);
    view.show(source);
    let capture = app.render();
    view.widget()
        .block_rows()
        .into_iter()
        .map(|rows| {
            let text = (rows.start..rows.end).map(|y| capture.row(y)).collect::<Vec<_>>().join("\n");
            (rows, text)
        })
        .collect()
}

#[test]
fn the_blocks_match_opentui_at_each_width() {
    for width in [40u16, 60, 80, 120] {
        for (name, source) in samples() {
            let blocks = rendered(&source);
            let prepared = markdown_images::prepared(&source);
            let drawn = drawn(&source, width);
            assert_eq!(blocks.len(), drawn.len(), "{name} at {width} columns: the block count");
            for (index, (block, (_, screen))) in blocks.iter().zip(&drawn).enumerate() {
                let text =
                    prepared.lines().skip(block.lines.start).take(block.lines.len()).collect::<Vec<_>>().join("\n");
                // A box is drawn as its box, which its own tests check; a Mermaid diagram is drawn as its rows.
                if text.starts_with("```mermaid") || text.starts_with("```fmc-image") {
                    continue;
                }
                // A word longer than the row wraps mid-word, so the screen is compared without its spaces and breaks.
                let screen: String = screen.chars().filter(|c| !c.is_whitespace()).collect();
                for word in probes(&block.kind, &text) {
                    assert!(
                        screen.contains(&word),
                        "{name} at {width}: block {index} ({}) shows no {word:?}",
                        block.kind
                    );
                }
            }
        }
    }
}

/// The front matter is a fence in the view (`reader/frontmatter.rs`), so OpenTUI does not read it as a rule and a heading;
/// the blocks after it follow OpenTUI.
#[test]
fn front_matter_is_a_fence_and_link_definitions_follow_opentui() {
    let kinds: Vec<(String, Range<usize>)> = rendered(EDGES).into_iter().map(|b| (b.kind, b.lines)).collect();
    let expected = [
        ("fenced_code_block", 0..3),
        ("atx_heading", 4..5),
        ("paragraph", 6..7),
        ("html_block", 10..13),
        ("paragraph", 13..14),
        ("html_block", 15..17),
        ("list", 17..21),
        ("paragraph", 21..22),
    ];
    let expected: Vec<(String, Range<usize>)> =
        expected.into_iter().map(|(kind, lines)| (kind.to_owned(), lines)).collect();
    assert_eq!(kinds, expected);
}

/// Splits the hook's spans at the matches of `query` in the rendered text (ASCII, ignoring case) and gives each matched
/// piece `bg`. A match across spans, such as `**Harry** Potter`, marks each of its pieces.
fn mark(spans: Vec<InlineSpan>, query: &str, bg: Color) -> Vec<InlineSpan> {
    let full: String = spans.iter().map(|span| span.span.content.as_ref()).collect();
    let (lower, needle) = (full.to_lowercase(), query.to_lowercase());
    if needle.is_empty() || lower.len() != full.len() {
        return spans;
    }
    let hits: Vec<Range<usize>> = lower.match_indices(&needle).map(|(at, found)| at..at + found.len()).collect();
    let mut out = Vec::new();
    let mut offset = 0;
    for inline in spans {
        let text = inline.span.content.to_string();
        let mut points = vec![0, text.len()];
        for edge in hits.iter().flat_map(|hit| [hit.start, hit.end]) {
            if edge > offset && edge < offset + text.len() {
                points.push(edge - offset);
            }
        }
        points.sort_unstable();
        points.dedup();
        for pair in points.windows(2) {
            let (from, to) = (pair[0], pair[1]);
            let inside = hits.iter().any(|hit| hit.start <= offset + from && offset + to <= hit.end);
            let style = if inside { inline.span.style.bg(bg) } else { inline.span.style };
            let mut piece = InlineSpan::from(Span::styled(text[from..to].to_owned(), style));
            piece.code = inline.code;
            piece.target = inline.target.clone();
            out.push(piece);
        }
        offset += text.len();
    }
    out
}

#[test]
fn an_inline_hook_marks_a_match_across_spans() {
    let bg = Color::rgb(0x44, 0x55, 0x22);
    let app = TestApp::new(80, 40);
    let view = MarkdownView::new(app.app(), &DARK, true);
    app.app().root().add(&view.widget());
    view.visible(true);
    view.widget().inline_hook(move |spans, _| mark(spans, "harry potter", bg));
    view.show(HARRY_POTTER);
    let capture = app.render();
    let row = (0..capture.height).find(|&y| capture.row(y).contains("Harry Potter")).expect("the bold row");
    let column = capture.row(row).find("Harry Potter").map(|byte| capture.row(row)[..byte].chars().count() as u32);
    let column = column.expect("the column of the match");
    let marked = (column..column + 12).filter(|&x| capture.cell(x, row).is_some_and(|cell| cell.bg == bg)).count();
    assert_eq!(marked, 12, "{}", app.text());
}

#[test]
fn a_code_block_renderer_draws_the_default_look() {
    let text = color(DARK.text);
    let theme = Arc::new(syntax::theme(&DARK));
    let frame = |custom: bool| {
        let app = TestApp::new(60, 24);
        let view = MarkdownView::new(app.app(), &DARK, true);
        app.app().root().add(&view.widget());
        view.visible(true);
        if custom {
            let theme = Arc::clone(&theme);
            view.widget().code_block_renderer("rust", move |block| {
                let code = Code::new(block.app)
                    .syntax_theme(Arc::clone(&theme))
                    .fg(text)
                    .filetype(block.language)
                    .set_code(block.code)
                    .wrap(WrapMode::Word)
                    .focusable(false);
                Some(View::new(block.app).bg(Color::TRANSPARENT).child(&code).as_any())
            });
        }
        view.show(NESTED_FENCES);
        app.render()
    };
    let (default, custom) = (frame(false), frame(true));
    let differ: Vec<(u32, u32)> = (0..default.height)
        .flat_map(|y| (0..default.width).map(move |x| (x, y)))
        .filter(|&(x, y)| default.cell(x, y) != custom.cell(x, y))
        .collect();
    assert!(differ.is_empty(), "{} cells differ, first {:?}", differ.len(), differ.first());
}

/// The refresh cost on the design sample, for the timing report (`cargo test -- --ignored --nocapture`).
#[test]
#[ignore = "a timing report, not a check"]
fn refresh_cost_on_design_md() {
    let design = samples().into_iter().find(|(name, _)| name == "DESIGN.md").expect("DESIGN.md").1;
    let query = Rc::new(RefCell::new(String::from("the")));
    let bg = Color::rgb(0x44, 0x55, 0x22);
    let app = TestApp::new(80, 400);
    let view = MarkdownView::new(app.app(), &DARK, true);
    app.app().root().add(&view.widget());
    view.visible(true);
    let hook_query = Rc::clone(&query);
    view.widget().inline_hook(move |spans, _| mark(spans, &hook_query.borrow(), bg));
    let started = Instant::now();
    view.show(&design);
    app.render();
    eprintln!("refresh: DESIGN.md built with the hook in {:?}", started.elapsed());
    let started = Instant::now();
    view.widget().rerun_inline_hook(|source| source.to_lowercase().contains("the"));
    app.render();
    eprintln!("refresh: rerun for the blocks containing \"the\" in {:?}", started.elapsed());
    let started = Instant::now();
    view.widget().rerun_inline_hook(|_| true);
    app.render();
    eprintln!("refresh: rerun for every block in {:?}", started.elapsed());
}
