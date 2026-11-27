---
status: done
created: 2026-11-27T15:10:00+07:00
updated: 2026-11-27T12:20:00+07:00
references: ui/markdown.rs, reader/markdown.rs, reader/headings.rs
phases:
    - id: 1
      title: "Rendered headings keep their `#` markers"
      status: done
      note: "D3 option 2: the inline hook cannot tell the level (its `source` has no run; levels 2 to 6 share one style), so `reader/headings.rs` rewrites the source with entities (`&#35;`, which take the heading colour; `\\#` takes the string colour). 21 core tests (`headings_tests.rs`), 3 TestApp tests (`ui/markdown_tests.rs`). Live run 2026-11-27 on README.md and DESIGN.md at 70 and 120 columns. Owner decisions recorded in architecture.md."
---

## Heading markers in the rendered Markdown view

### Objective

A terminal cannot change the font size, so in the rendered view (`m`) the heading levels are hard to tell apart. Headings keep their level marker: `# Title`, `## Section`, `### Part`, and so on. The heading colours and styles stay as they are, and the marker takes the heading's own style. This is a deviation from Scouter v0.3.4 (`434d000f`), whose `MarkdownRenderable` conceals the markers (owner request, 2026-11-27).

### Current state (2026-11-27)

- `src/ui/markdown.rs` sets the heading colour (`p.syntax.function`) and the syntax theme. In the theme, `markup.heading.1` to `.6` are bold `function`, and Core's fallback underlines level 1. The widget conceals markers, which is the default.
- OpenTUI 0.9.9 `Markdown::conceal(false)` would show every marker (`**`, backticks, link brackets), not only the heading ones. `inline_hook` sees heading spans but gets only `InlineContext { source, conceal, syntax, table_cell }`. Whether `source` holds the `#` run, or whether the level can be read from the span style, is unverified.

### Decisions (agent)

- **D1.** Only headings keep markers. Emphasis, code, and link markers stay concealed.
- **D2.** The marker is the ATX run for the heading's level plus one space, drawn in the heading's style. A setext heading (`Title` over `===` or `---`) shows the ATX marker of its level (`#` or `##`), so every heading reads the same way.
- **D3.** Implementation, in this order of preference:
  1. The inline hook, if it can tell a heading and its level apart (from `InlineContext::source` or the span style). It prepends one span with the marker.
  2. Otherwise, a pure source rewrite in the core crate before `set_markdown`. Each ATX heading line `## Title` becomes `## \#\# Title`, and each setext heading is handled the same way. Lines inside fenced code blocks and HTML blocks are left alone. The line count does not change, so the source-to-block mapping of plan-00010 still holds.

  The executor checks option 1 first, in a test, and records which option it used and why.
- **D4.** Copying selected text from the rendered view includes the marker, because the marker is on screen.
- **D5.** The source view (`m` again) is unchanged.

### Phase 1

- Implement per D3, with unit tests: ATX levels 1 to 6, a closing `##` run (`## Title ##`), setext levels 1 and 2, a `#` inside a fence, an escaped `\#` paragraph, a heading in a list or quote (if OpenTUI renders it as a heading), and a Mermaid fence beside a heading. Sample text from Harry Potter or Dan Brown.
- A TestApp test renders a document and finds `# `, `## `, and `###` at the start of the heading rows, in the heading colour.
- Live run in a new tab of the agent's own Rex session on README.md and DESIGN.md at 70 and 120 columns.
- Docs in the same commit:
  - `architecture.md` record (the deviation, D1 to D5);
  - `docs/parity.md` (Reader);
  - `DESIGN.md` (the rendered heading look);
  - CHANGELOG (Unreleased);
  - archive this plan and update `index.md`.

### Order

Runs after plan-00008 and before plan-00010, so plan-00010's spike sees the final rendered text.
