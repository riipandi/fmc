# Source-to-block mapping and inline marks for the rendered Markdown view

Point-in-time research, 2026-11-27. Plan: `.agents/docs/plans/plan-00010-markdown-preview-search-signs.md`, phase 1 (the spike). Agent audience; evidence as `file:line` or the test that measured it. Code: `src/ui/markdown_blocks.rs` (the splitter), `src/ui/markdown_blocks_tests.rs` (the measurements, kept as assertions). Local toolchain: rustc and cargo 1.100.0, macOS arm64, OpenTUI 0.9.9 (the locked source, `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opentui-0.9.9`).

## 1. Verdict

The STOP condition does not hold. tree-sitter-md's top-level blocks, read with two rules, match the blocks OpenTUI draws on every sample and at every width tested (40, 60, 80, and 120 columns). Without the rules, two constructs shift the block order (section 3). The mapping is good enough for phase 2 (search: marks and jumps) and phase 3 (signs), which should keep the test in `markdown_blocks_tests.rs` as the guard.

The remaining risk is a construct the samples do not contain. A future mismatch shows up as a failed count in `the_blocks_match_opentui_at_each_width`, not as a silent wrong sign.

## 2. Method

- The splitter parses `"\n" + source` with `tree-sitter-md` (block grammar), flattens each `section` into its children, drops `link_reference_definition` nodes, and returns each block's zero-based source lines (`src/ui/markdown_blocks.rs`).
- OpenTUI's blocks are the `Markdown::block_rows()` ranges of a TestApp render (`opentui-0.9.9/src/widget/markdown.rs:758`), drawn through `MarkdownView::show`, which applies the heading rewrite (`reader/headings.rs`). The rewrite keeps every line, so the source lines still map.
- Each block must show its first and last word of at least four letters in its own screen rows, with spaces removed (a long hash wraps mid-word). A mermaid fence is one block in both, but its render is a diagram, so its text is not probed.
- Samples: the Harry Potter fixture (heading, emphasis, list, ordered list, quote, table, Rust fence, mermaid fence, setext heading, rule, link), an edge fixture (front matter, link definition, HTML, a loose list), a second edge fixture (HTML comment, footnote, a pipe table without outer bars, `+++` front matter, HTML `div`, a soft break, a rule, a trailing link definition), and seven of the repository's Markdown files: `README.md`, `DESIGN.md`, `docs/cli.md`, `docs/configuration.md`, `docs/development.md`, `docs/keymaps.md`, `docs/parity.md`.

## 3. Numbers

| Sample | tree-sitter blocks | OpenTUI blocks | Probe failures, 40 to 120 columns |
| --- | --- | --- | --- |
| Harry Potter fixture | 12 | 12 | 0 |
| Edge fixture | 9 | 9 | 0 |
| Second edge fixture | 9 | 9 | 0 |
| `README.md` | 18 | 18 | 0 |
| `DESIGN.md` (with its front matter) | 44 | 44 | 0 |
| `docs/cli.md` | 8 | 8 | 0 |
| `docs/configuration.md` | 11 | 11 | 0 |
| `docs/development.md` | 29 | 29 | 0 |
| `docs/keymaps.md` | 21 | 21 | 0 |
| `docs/parity.md` | 21 | 21 | 0 |

Before the two rules, at 80 columns: the edge fixture had 9 and 9 blocks but 4 failed probes (the order was shifted), and `DESIGN.md` had 43 against 44 with 37 failed probes.

Test: `the_blocks_match_opentui_at_each_width` (asserts the counts and the probes at four widths). Fixture kinds are asserted in `front_matter_and_link_definitions_follow_opentui`.

### The two differences, and the rules

1. **Front matter.** OpenTUI has no front matter. A `---` on line 0 is a thematic break, and `title: Hogwarts` followed by `---` is a setext heading (`opentui` renders the edge fixture's lines 0 to 2 as a rule and an `##` heading). tree-sitter-md reads the top block as `minus_metadata`, one block. The leading blank line in `"\n" + source` keeps the grammar from detecting front matter; the rows shift by one and map back. This affects `DESIGN.md`, whose 84-line front matter OpenTUI draws as a rule and paragraphs. That is the same engine Scouter uses, so it is not a fmc deviation, but Scouter's live output was not captured here.
2. **Link reference definitions.** OpenTUI draws no block for `[label]: url`; the line is invisible in the rendered view (the edge fixture's line 8, and the second fixture's last line). tree-sitter-md makes one block per definition. The splitter drops those nodes.

Other constructs behaved the same in both parsers: setext headings, a list with loose items (one block in both), a pipe table, a fence (plain and mermaid), a block quote, HTML blocks (`<details>`, `<!-- -->`, `<div>`), a footnote definition (a paragraph in both), a soft line break (one paragraph), and a rule.

### Placement

`tree_sitter` is not a direct dependency. `opentui::syntax::tree_sitter` re-exports it (`opentui-0.9.9/src/syntax/highlight.rs:17`), and `src/ui/syntax/mod.rs` already uses it. The core crate may not import opentui (`rust.md`, "Layout rules"), so the splitter lives in `src/ui/`. No new crate (`rules.md`, "Dependencies").

`Block.kind` is a `String`: in tree-sitter 0.27 `Node::kind()` borrows from the tree, so a `&'static str` does not compile.

## 4. Inline marks

OpenTUI's `inline_hook` gets the spans of every paragraph, heading, and table cell, in lists and quotes too (`opentui-0.9.9/src/widget/markdown.rs:663`). A hook that splits spans at the matches of a query, over the rendered text (concealed markers are not in it), marks a match across spans.

- Test `an_inline_hook_marks_a_match_across_spans`: the query `harry potter` in `**Harry** Potter` marks 12 of 12 cells with the mark colour.
- Code blocks never reach the hook (the widget doc, `opentui-0.9.9/src/widget/markdown.rs:656`). Table cells do (`markdown.rs:654`, "every paragraph, heading and table cell"); not measured beyond that.
- `InlineSpan` is `#[non_exhaustive]` (`markdown.rs:182`), so a hook builds a piece with `InlineSpan::from(Span)` and sets its fields. `Span.link` and `InlineSpan.target` must be copied from the span they split.

## 5. Refresh cost

`refresh_cost_on_design_md` (ignored; `cargo test --bin fmc refresh_cost_on_design_md -- --ignored --nocapture`). Each figure includes one render.

| Step | Debug | Release |
| --- | --- | --- |
| Build `DESIGN.md` with the hook | 8.2 ms | 7.5 ms |
| `rerun_inline_hook` for blocks whose source contains `the` | 6.4 ms | 3.2 ms |
| `rerun_inline_hook` for every block | 6.1 ms | 2.9 ms |

Reading: a per-keystroke refresh of the whole document costs about 3 ms in a release build on this machine, well under a frame. The plan's debounce is not needed for documents of this size. A 1 MiB file (the reader's cap) is not measured.

`rerun_inline_hook` selects by the block's inline source (`markdown.rs:694`), which holds markup such as `**`. A selector on rendered text cannot be applied through it, so phase 2 selects by source and accepts a superset.

## 6. Code block look (D5)

`a_code_block_renderer_draws_the_default_look`: a `rust` renderer that draws `View::new(app).bg(TRANSPARENT).child(Code::new(app).syntax_theme(theme).fg(text).filetype(lang).set_code(code).wrap(Word).focusable(false))` produces a 60 by 24 frame with 0 cells different from the default, for a top-level fence, a fence in a list item, and a fence in a quote.

The default block is `Code::themed` with the Markdown's style table, and `set_closed` (`opentui-0.9.9/src/widget/markdown.rs:1143-1190`). `Code::themed` is `pub(crate)` (`src/widget/code.rs:491`), so a custom renderer cannot share that table or call `set_closed`. The frame matched without them in these samples. Two limits:

- A parent with its own text style (a list item or a quote whose `base` style differs from the palette text) would be drawn in the palette text, not the parent's `base.fg` (`markdown.rs:1181` and `:1185`). `MarkdownView::restyle` sets the quote colour to the text colour (`src/ui/markdown.rs:44`), so the sample cannot show the difference.
- A streaming document would hand `closed` to the renderer (`CodeBlock::closed`), which fmc does not use (the document never streams).

Verdict: D5 holds for the search layer. Phase 2 can mark matches in the highlighted fences through this renderer. A fence in another language keeps the default block and is counted and jumped to, not marked (D5).

## 7. Corrections to the plan's decisions

- **D2 (the case rule).** The plan says "case-insensitive unless the query has an upper-case letter, the same as the code view's rule". The code view's rule is always case-insensitive: `reader/search.rs` (`matching_lines`, `match_ranges`, both lower-case both sides) and Scouter's `src/reader/document-search.ts:151-170` (lower-cased `includes`, no smart case). Rendered search follows the code view: always case-insensitive. Recorded here, to be made in `architecture.md` with the phase 2 record.
- **D2 (the text searched).** Confirmed: the rendered text, so `Harry Potter` finds `**Harry** Potter`. The inline hook supplies it for prose and tables; code blocks supply their text through the renderer (section 6).
- **D4 (a jump).** Not measured in phase 1. OpenTUI gives a block's rows (`block_rows`), not a byte's row, as the plan says. Phase 2 measures the estimate on DESIGN.md.

## 8. Unverified

- Constructs outside the samples (nested tables in lists, a table with an inline link, math, a footnote's back reference, a link definition inside a list or quote). The mapping test is the guard, and a new mismatch is a failed count.
- The release timings are one machine (macOS arm64, 2026-11-27).
- Scouter's live rendering of front matter was not captured. The claim that Scouter shows it the same way rests on the shared engine (`@opentui/core` 0.9.9 in the parity reference, `architecture.md`).
- Phase 1 changes no drawn screen, so no Rex run was made. The live run is phase 2's.
