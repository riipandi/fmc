---
status: done
created: 2026-11-27T14:30:00+07:00
updated: 2026-11-27T20:00:00+07:00
references: plan-00008-git-gutter.md, ui/markdown.rs, app/search.rs, plan-00011 (headings rewrite, 21cb066)
phases:
    - id: 1
      title: "Spike: map source lines to rendered blocks, and mark matches"
      status: done
      note: "Verdict: the mapping matches OpenTUI's blocks on 10 samples at 40, 60, 80, and 120 columns, after two rules (a leading blank line disables tree-sitter's front matter; link reference definitions are dropped). The STOP condition does not hold. Numbers and the D2 correction: research/markdown-preview-mapping.md. Tests: ui/markdown_blocks_tests.rs (4 assertions, 1 ignored timing test)."
    - id: 2
      title: "Search in rendered Markdown"
      status: done
      note: "Search runs in the rendered view: `/`, `n`, `N`/`p`, Esc, and the footer's `k of N` (the code view's text; D1 refined). The index (`ui/markdown_search.rs`, `reader/rendered_search.rs`) numbers the occurrences per block; the hook and renderers mark them by build order. Records: architecture.md, 'Approved deviation: search runs in the rendered Markdown view'; parity.md Reader and Help; keymaps; CHANGELOG; DESIGN.md. Tests: 6 mark tests (every counted occurrence drawn where the index puts it), 6 app key-path tests (including a hit below the view), 4 core tests, 4 block-top and mapping tests. just verify: 450 tests, 1 ignored. Live (Rex, DESIGN.md, a new tab, closed): 70 and 120 columns; `/palette` 1 of 3 with the current mark on target-bg and the others on match-bg; Enter and shift+n step; Esc keeps then clears; m shows the source with no marks. Refinements: D8 (m clears the search), D9 (a changed document keeps the query, as the code view does)."
    - id: 3
      title: "Git change signs beside rendered blocks (needs plan-00008)"
      status: done
      note: "Signs per D6 and D7: `reader/rendered_signs.rs` maps the file's signs to the blocks (6 core tests after the phase's own mapping tests), `ui/markdown_signs.rs` draws them in an absolute column over the viewer's left edge at each paint. Refinements (D6): a bar is the strongest sign of the block's lines (modified over added); a deletion is drawn under the block that holds its line, or the block above a blank line, and a deletion above line 1 over the first block; an added blank line between blocks draws nothing. Records: architecture.md, 'Approved deviation: the rendered view shows git change signs beside its blocks'; parity.md Reader; README; configuration; CHANGELOG; DESIGN.md. Tests: 4 temporary-repository app tests (an edited paragraph, an added section, a deletion, the setting off) and a scroll test. Live (Rex, a scratch repository under /tmp, 70 and 120 columns): an edited paragraph and an appended section drawn modified and added (#e5b880, #8fd3a8); a commit cleared them; a deleted section drew `▁` under the block above. just verify: 462 tests, 1 ignored. Refinement to the mapping test: its frame is 1200 rows (DESIGN.md at 40 columns is taller than 400)."
    - id: 4
      title: "Docs, live checks, close"
      status: done
      note: "The records, parity lines, CHANGELOG, DESIGN.md, README, and configuration landed with phases 2 and 3, because a deviation's record lands with its behaviour (rules.md). This commit archives the plan and updates index.md. The phase 2 commit (dafc20b) recorded a longer keymaps row that the key-list test does not carry; d8a96ae restores the row. Owner checks are in the run report."
---

## Search and git signs in the rendered Markdown view

### Objective

In the rendered view (`m`), the owner can:

- Search with `/` and step through matches with `n`, `N`, and `p` without switching back to the source. The matches are marked in place, and the footer counts them.
- See the git change signs of plan-00008 beside the blocks whose source lines changed.

Both are deviations from Scouter v0.3.4 (`434d000f`). Scouter refuses search in the rendered view, and plan-00008 limits its signs to the text view (D3).

### Current state (2026-11-27)

- **Rendered view.** `src/ui/markdown.rs` wraps OpenTUI's `Markdown` widget with `padding_x(2)`. Search, counts, and line motions are refused while previewing (`keys/parser.rs` `State::previewing`, `absorb_count`). Search lives in `src/app/search.rs` and `reader::Search`, and is drawn in the code view with the `search.match` style.
- **OpenTUI 0.9.9.** It offers these hooks (`opentui-0.9.9/src/widget/markdown.rs`):
  - `inline_hook` (line 663) rewrites the spans of every paragraph, heading, and table cell, including those inside lists and quotes. Code blocks never reach it.
  - `rerun_inline_hook(select)` (line 694) rebuilds only the blocks it selects.
  - `block_rows()` (line 758) gives each top-level block's screen rows from the last frame.
  - `code_block_renderer(language, …)` (line 631) replaces a fenced block of one language. It is registered per language, with no wildcard.

  No API gives a block's source line range or the screen row of a byte inside a wrapped paragraph.
- **Core crate.** `tree-sitter-md` is already a dependency, so fmc can split the source into top-level blocks with line ranges without a new crate.
- **Signs.** plan-00008 builds the per-line signs for the open file (`diff/signs.rs`, still to land).

### Decisions (agent, cheap to change)

- **D1.** The keys are the code view's: `/` opens the prompt, `n` goes to the next match, `N` and `p` to the previous one, and `Esc` clears. The footer shows `3/12` as it does in the code view. Counts and line motions stay refused in the rendered view.
- **D2.** Matching runs on the rendered text, not the source. Searching `Harry Potter` finds `**Harry** Potter`, because concealed markers are not in the text. The match is case-insensitive, as the code view's rule is: always, with no smart case (phase 1 refined this; `research/markdown-preview-mapping.md` section 7).
- **D3.** Every match is drawn in the `match` colours. The current match is drawn with `target_bg`, so it stands out.
- **D4.** A jump scrolls the block that holds the current match into view. A match inside a block taller than the viewport scrolls to the block's top plus the match's estimated row. The estimate is the match's byte offset over the block's text, times the block's rows. The plan states this precision openly, because OpenTUI does not expose a byte's row.
- **D5.** Fenced code blocks: in the languages fmc highlights, matches are marked through a `code_block_renderer` that draws a `Code` widget with the search layer, the same as the code view. A fence in another language is searched for the count and the jump but not marked. Phase 1 confirms the renderer keeps the default look byte for byte when no search is active. If it does not, code blocks are counted and jumped to but not marked.
- **D6.** Signs in the rendered view sit in the left padding column, beside each top-level block's rows:
  - a whole-block bar in the colour of the block's strongest change (modified over added);
  - `▁` under the block above a deletion.

  A block changes when any of its source lines has a sign.
- **D7.** Signs follow plan-00008's `git_gutter` setting. There is no second key.
- **D8 (phase 2).** `m` clears the search, in both directions: the view it searched is no longer the one shown, and the marks name the document they were counted in.
- **D9 (phase 2).** A document that changes keeps the query, and its hits are counted again, as the code view's `follow` does. The phase text said the marks clear on a new document; the code view keeps the query, and one rule for both views is simpler. Cheap to change.

### Phases

#### Phase 1: Spike

- In a test module, split sample documents (README.md, DESIGN.md, docs/*.md, and a Harry Potter sample with a list, a quote, a table, a fence, and a Mermaid fence) into top-level blocks with `tree-sitter-md`. Compare the count and order with `block_rows()` from a TestApp render. Record every mismatch, such as link definitions, HTML, front matter, or a list's loose items.
- Mark a match through `inline_hook`, and refresh only the blocks that change with `rerun_inline_hook`. Measure a refresh on DESIGN.md.
- Check D5's code-block renderer for a byte-identical look with no search active.
- Write `.agents/docs/research/markdown-preview-mapping.md`. If the block mapping cannot match OpenTUI's blocks reliably, stop and report the options to the owner. Otherwise continue.

#### Phase 2: Search

- Allow `/`, `n`, `N`, and `p` while previewing (parser and `app/search.rs`). Search state for the rendered view: the matches per block, the current match, and a jump per D4.
- Mark matches per D3 and D5. Clear the marks on `Esc`, on a new document, and when leaving the rendered view.
- Tests: counting across concealed markers, case rules, jumps across blocks, a match in a table cell and in a fence, and `Esc`. A TestApp test that renders and finds the marked cells.
- Live run in a new tab of the agent's own Rex session on DESIGN.md at 70 and 120 columns.

#### Phase 3: Signs (after plan-00008)

- Map plan-00008's line signs to top-level blocks through the phase 1 mapping, and draw them per D6 in an overlay column over the left padding. They refresh with plan-00008's refresh.
- Tests in a temporary repository: an edited paragraph, an added section, a deleted block, and the setting off.
- Live run: edit a Markdown file in a scratch repository and watch the signs in the rendered view.

#### Phase 4: Docs and close

- `architecture.md` records (both deviations, D1 to D7), `docs/parity.md` (Reader), `docs/keymaps.md`, CHANGELOG, and `DESIGN.md` (the marks and signs in the rendered view).
- Archive the plan, update `index.md`, and list the owner checks.

### Risks

- If `tree-sitter-md`'s blocks do not line up with OpenTUI's native parser, the mapping is the weak point. Phase 1 measures it before any feature code.
- The jump position inside a tall block is an estimate (D4).
- Rebuilding blocks on each keystroke could be slow on large documents. Phase 1 measures it, and the prompt can debounce.

### Order

Phases 1 and 2 can run any time. Phase 3 needs plan-00008.
