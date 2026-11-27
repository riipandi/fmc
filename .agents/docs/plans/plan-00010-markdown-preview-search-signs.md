---
status: draft
created: 2026-11-27T14:30:00+07:00
updated: 2026-11-27T14:30:00+07:00
references: plan-00008-git-gutter.md, ui/markdown.rs, app/search.rs
phases:
    - id: 1
      title: "Spike: map source lines to rendered blocks, and mark matches"
      status: pending
    - id: 2
      title: "Search in rendered Markdown"
      status: pending
    - id: 3
      title: "Git change signs beside rendered blocks (needs plan-00008)"
      status: pending
    - id: 4
      title: "Docs, live checks, close"
      status: pending
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
- **D2.** Matching runs on the rendered text, not the source. Searching `Harry Potter` finds `**Harry** Potter`, because concealed markers are not in the text. The match is case-insensitive unless the query has an upper-case letter, the same as the code view's rule. Phase 1 confirms what that rule is.
- **D3.** Every match is drawn in the `match` colours. The current match is drawn with `target_bg`, so it stands out.
- **D4.** A jump scrolls the block that holds the current match into view. A match inside a block taller than the viewport scrolls to the block's top plus the match's estimated row. The estimate is the match's byte offset over the block's text, times the block's rows. The plan states this precision openly, because OpenTUI does not expose a byte's row.
- **D5.** Fenced code blocks: in the languages fmc highlights, matches are marked through a `code_block_renderer` that draws a `Code` widget with the search layer, the same as the code view. A fence in another language is searched for the count and the jump but not marked. Phase 1 confirms the renderer keeps the default look byte for byte when no search is active. If it does not, code blocks are counted and jumped to but not marked.
- **D6.** Signs in the rendered view sit in the left padding column, beside each top-level block's rows:
  - a whole-block bar in the colour of the block's strongest change (modified over added);
  - `▁` under the block above a deletion.

  A block changes when any of its source lines has a sign.
- **D7.** Signs follow plan-00008's `git_gutter` setting. There is no second key.

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
