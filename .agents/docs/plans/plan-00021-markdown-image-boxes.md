---
status: in-progress
created: 2026-11-28T17:09:52+07:00
updated: 2026-11-28T17:09:52+07:00
phases:
    - id: 1
      title: The image paragraphs, found and rewritten as fences
      status: done
      note: —
    - id: 2
      title: The image box in the rendered view
      status: pending
      note: —
    - id: 3
      title: Opening a box's image, and back
      status: pending
      note: —
    - id: 4
      title: Docs sync, agent docs sync, ledger close, handover
      status: pending
      note: —
    - id: 5
      title: Final validation and live run
      status: pending
      note: —
---

## Show Markdown images as boxes in the rendered view — alt, file, size, and a key that opens the picture

### Objective

The owner asked on 2026-11-28 whether the rendered Markdown preview can show images. Today `![alt](path)` draws as its alt text styled as a link, as in Scouter. The owner chose a box over a real picture: after this plan, a paragraph that holds only a local image draws as a box with the alt text, the file, its size and format, and a hint; a key or a click opens the picture in the reader's image view, and back returns to the Markdown where it was. No new dependency, no network, no read outside the browsed root.

### Current state (verified 2026-11-28, research run)

- **Images draw as text** — OpenTUI 0.9.9's Markdown has no image block (`opentui-sys-0.9.9/src/generated.rs:483-493`); `![Marauder map](maps/marauder.png)` draws "Marauder map" as a link and the inline hook sees the link target; `![](a.svg)` draws nothing; inline `<img>` draws as literal HTML.
- **Hooks** — `code_block_renderer` runs only for code blocks (`opentui-0.9.9/src/widget/markdown.rs:631-642`); `inline_hook` runs for paragraphs, headings, and table cells (`:655-663`). fmc registers the `mermaid` and highlighted-language renderers (`src/ui/markdown.rs:91-99`) and rewrites the source before drawing, keeping the line count (`src/reader/headings.rs` `with_markers`; `src/reader/frontmatter.rs` `as_fence`, plan-00020).
- **The Mermaid box is the model** — a measured `Custom` (`src/ui/mermaid.rs:99-108`), drawn from a fence's renderer.
- **Search and signs** — a code renderer must call `count_code` (`src/ui/markdown_search.rs:167-171`) and be registered on the offscreen index (`src/ui/markdown.rs:194-201`); block tops and lines map through `markdown_blocks.rs` and `reader/rendered_signs.rs:22`. A second `inline_hook` rebuilds every block, code renderers included (`opentui-0.9.9/src/widget/markdown.rs:663-672`).
- **The image view** — `src/ui/picture.rs` (`ImageView`, `Contain`, protocol auto), the reader's image path `src/app/reader.rs:192-200`, the image rules `src/reader/image.rs` (png, jpg, jpeg, gif, webp; 10 MiB), the header read `src/services/fs.rs:102-118` (`imagesize`).
- **Scouter** — its Markdown preview has no image handling (`src/reader/code-view.ts:132-142`, `defaultRender()`); this is a deviation.

### Decisions

- **D1 (owner, 2026-11-28): a box, not a picture.** Rejected: drawing the picture inside the preview (an overlay `ImageView` placed from block tops, a decode cache, clipping under scroll; larger, and unproven in the terminals); an inline `[image: alt]` marker only.
- **D2 (owner, 2026-11-28): local files inside the browsed root only.** A path is resolved against the Markdown file's folder; one that leaves the root, a missing file, or a format fmc cannot show draws the box with the reason (`outside the folder`, `not found`, `not an image fmc shows`) and no open hint.
- **D3 (owner, 2026-11-28): no remote images and no SVG.** An `http(s)` source draws the box marked `remote`; an `.svg` marked `SVG`. No HTTP client, no rasteriser.
- **D4 (agent, 2026-11-28): only a top-level paragraph that is a single image becomes a box.** An image inside a sentence, a list, a quote, or a table keeps today's alt text. The rewrite turns that paragraph's line into a fence with an internal info string (for example ```` ```fmc-image ````, the original line inside), keeping the line count, as the front matter rewrite does; the source view (`m`) shows the file as it is.
- **D5 (agent, 2026-11-28): the box's content.** First line: `▣` and the alt text (or the file name when the alt is empty). Second: the path as written, `W×H`, and the format, read from the header with `imagesize` on a worker and cached by path and mtime. Third: `Enter opens it` (or the reason from D2/D3). Border and colours as the code block's; the box fits the pane's width and is three rows plus its border.
- **D6 (agent, 2026-11-28): opening.** `Enter` in the rendered view opens the first box whose top is in view (the reader's other targets do not use `Enter`, verified free in the reader context), and a click opens the clicked box. The picture shows in the reader's image view as when the file is opened from the tree; `Ctrl+o` (jump back) returns to the Markdown at the same scroll. The tree's selection does not move.
- **D7 (agent, 2026-11-28): search counts the alt text.** The box's alt line is part of the rendered text for `/`, so a search for the alt finds the box.
- **D8 (2026-11-28): the deviation is recorded** in `architecture.md` and `docs/parity.md` with the Scouter baseline (v0.3.4, `434d000f`), in the commit that draws the box.

### Execution rules

- **The owner gave the go-ahead for all phases on 2026-11-28, with D4 to D7 as written.** It runs after the current work ships.
- **Every commit ends with** `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; comments never cite a plan, roadmap entry, decision ID, or doc file.
- **No new dependency; no new write; no network.** Reads stay inside the browsed root.
- **Branch:** stacked on the branch that is current when it starts, so it ships with the open work.

### Phases

1. **The image paragraphs** — `src/reader/images.rs` (pure): find top-level single-image paragraphs with tree-sitter-md, classify each source (local, outside the root, remote, SVG, other), and rewrite the line as the D4 fence with the same line count. Tests: a lone image, an image in a sentence (untouched), in a list and a quote (untouched), two images in a row, an image with a title, `<img>` (untouched), remote, SVG, `../` leaving the root; the line count holds. Validation: `cargo test images`. _Commit: `feat(markdown): find image paragraphs in the rendered view`._
2. **The box** — a renderer for the D4 fence drawing the D5 box (a measured `Custom`, as the Mermaid box), the header read on a worker with a path-and-mtime cache, `count_code` and the offscreen index (D7), the block mapping and the signs kept aligned. Records in this commit: `architecture.md`, `docs/parity.md`, `docs/usage.md`, `CHANGELOG.md`. Tests: the box's rows at 40, 80, and 120 columns; every reason; search finds the alt and the count is unchanged elsewhere; signs below the box on the same rows. Validation: `cargo test markdown`; a live run of a Markdown file with each case at 120 and 50 columns. _Commit: `feat(markdown): show an image paragraph as a box`._
3. **Opening** — `Enter` and a click per D6; back with `Ctrl+o`. Tests through `TestApp`: Enter opens the top box's image; a click opens the clicked one; back restores the Markdown and its scroll; a box with a reason does nothing and says why in the footer. Records: `docs/keymaps.md` (with the tie test), `docs/usage.md`. Validation: `cargo test`; live run: open, back. _Commit: `feat(markdown): open an image box's picture`._
4. **Docs sync, agent docs sync, ledger close, handover** — `rust.md` (the fence contract, the renderer's search registration), `shape.md`, roadmap entry if one fits, the plan archived with its index row, a handover. _Commit: `docs: image boxes in the Markdown preview`._
5. **Final validation and live run** — the matrix; Rex tab at 120 and 50 columns. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl fmt --check --quiet .` | No drift |
| Unit | `cargo test images`, `cargo test markdown` | Detection, classification, line count, box rows, search, signs |
| App | `cargo test` | Enter, click, back, reasons |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Full suite | `just verify` | All green |
| Live run | Rex tab, 120 and 50 columns | Boxes drawn, open and back work |
| E2E | n/a beyond the live run: no CLI change | — |

### Out of scope

- Drawing the picture inside the preview (a later plan can add it behind the box).
- Remote images, SVG, `<img>` tags, and images inside lists, quotes, tables, or sentences.
