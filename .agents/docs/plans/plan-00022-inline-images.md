---
status: in-progress
created: 2026-11-28T18:29:53+07:00
updated: 2026-11-28T23:00:00+07:00
phases:
    - id: 1
      title: Spike, a picture inside the rendered view
      status: done
      note: fdaa0d7; go; `.agents/docs/research/inline-image-spike.md` (D5's decode cannot run on a worker, see its section 3)
    - id: 2
      title: HTML images found and rewritten
      status: done
      note: the phase 2 commit (see git log); the box shows HTML images; the picture comes in phase 3
    - id: 3
      title: The picture drawn in place of the box
      status: done
      note: the phase 3 commit (see git log); deviations in architecture.md (decode on the UI thread once, the cap in rows, the width set at paint)
    - id: 4
      title: Docs sync, agent docs sync, ledger close, handover
      status: pending
      note: —
    - id: 5
      title: Final validation and live run
      status: pending
      note: —
---

## Draw Markdown images in the rendered view — real pictures for local images, from `![]()` and HTML

### Objective

On 2026-11-28 the owner opened `README.md` and saw its logo as raw HTML (`<h1><picture><source media="(prefers-color-scheme: dark)" srcset=…><img src=… width="392"></picture></h1>`), and asked for the best image UX possible. plan-00021 drew a box for a Markdown image paragraph; this plan draws the picture itself. After it, a local image inside the browsed root, written as `![alt](path)` or as an HTML `<img>` or `<picture>` block, draws as pixels in the rendered view (kitty graphics, sixel, or coloured blocks, as the image view already chooses), sized to its `width` or the pane, with a height cap. Remote images, SVG, paths outside the root, and unreadable files keep the plan-00021 box with its reason. No new dependency.

### Current state (verified 2026-11-28)

- **The box** — `src/reader/images.rs` (classify, inside the root, percent-decode), `src/ui/markdown_images.rs` (tree-sitter scan, the `fmc-image` fence rewrite with the same line count), `src/ui/image_box.rs` (a measured `Custom`), `src/services/picture.rs` (header read on a worker, cache by path and mtime); `Enter` and a click open the picture in the image view, `Ctrl+o` returns (plan-00021, `0c01579`).
- **HTML blocks are drawn as text** — `<img>` and `<picture>` are out of plan-00021's scope; OpenTUI 0.9.9 has no hook for HTML blocks (`opentui-0.9.9/src/widget/markdown.rs:655-663`: the inline hook covers paragraphs, headings, and table cells).
- **The picture widget** — `src/ui/picture.rs` (`ImageView`, `ImageFit::Contain`, `ImageProtocol::Auto`); `ImageView::size` and `fitted_size(w, h, cell_aspect)` (`opentui-0.9.9/src/widget/image.rs:31-37, 88-101`); decode is synchronous where it is set (`:57-61`). A research probe on the locked 0.9.9 had a code-block renderer return an `ImageView` with `height(5)`, and it took 5 rows; whether it clips under scroll, and how kitty and sixel images behave inside a scrolling view, is unproven.
- **Rebuilds** — a second `inline_hook` rebuilds every block, code renderers included (`opentui-0.9.9/src/widget/markdown.rs:663-672`), so a renderer that decodes would decode again on every search step unless it reuses `CodeBlock::previous` (`:172-174`) or a cache.
- **Scouter** — no image handling in its preview (`src/reader/code-view.ts:132-142`); this extends plan-00021's deviation.

### Decisions

- **D1 (owner, 2026-11-28): real pictures for local images.** Rejected: boxes for HTML images too (the previous level); an inline marker.
- **D2 (owner, 2026-11-28): `<picture>` follows the theme mode.** The dark palette takes the `<source>` whose `media` is `(prefers-color-scheme: dark)`, the light palette takes `<img src>`; a theme change redraws. Other `media` queries and `srcset` densities: the first candidate of `srcset` is used.
- **D3 (owner, 2026-11-28): size.** Width is the HTML `width` (pixels, at the terminal's cell width; 8 px a cell when the terminal does not report it) or the image's own width, capped at the pane; height follows the ratio and is capped at about 60% of the viewer's height; `Enter` still opens the full image.
- **D4 (agent, 2026-11-28): which HTML counts.** An HTML block (tree-sitter-md `html_block`) whose only content, after tags like `<h1>`, `<p>`, `<div>`, `<a>`, `<center>`, is one `<img>` or one `<picture>` becomes an image block; `align="center"` or a `<p align="center">` wrapper centres it. Any other HTML block draws as today. The rewrite keeps the line count, as the front matter and `fmc-image` rewrites do: the block's first line becomes the fence opener carrying the source, the last becomes the closer, the lines between are its body.
- **D5 (agent, 2026-11-28): decode off the frame, then draw.** The worker reads and decodes into the cache (path, mtime, target size); until it lands, the plan-00021 box shows as the placeholder with the same height, so the layout does not jump twice. A decode failure keeps the box with its reason.
- **D6 (agent, 2026-11-28): memory and count.** At most 32 decoded pictures are kept (least recently drawn dropped); a picture over the image view's 10 MiB file cap keeps the box.
- **D7 (2026-11-28): records** — the plan-00021 deviation record in `architecture.md` grows to pictures and HTML images, with the Scouter baseline; `docs/parity.md`, `docs/usage.md`, and `CHANGELOG.md` follow.

### Execution rules

- **The owner gave the go-ahead for all phases in order on 2026-11-28, with D4 to D6 as written.** Phase 1 is a spike: if a picture cannot draw and scroll correctly inside the rendered view (clipping, a kitty image left behind after scrolling, a crash), the plan stops after phase 1 and reports, and the owner chooses again; the box stays.
- **Every commit ends with** `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; comments never cite a plan, roadmap entry, decision ID, or doc file.
- **No new dependency; reads stay inside the browsed root; no network.**
- **Live runs need a graphics terminal**: the Rex capture shows text only, so phase 1 and phase 5 record what Rex can show (layout, rows, no leftovers in text) and leave the picture check to the owner in Ghostty.
- **Branch:** `feat/inline-images`, from `main`.

### Phases

1. **Spike** — in a scratch branch of the code (not committed unless it becomes phase 3's base): the `fmc-image` renderer returns an `ImageView` sized by `fitted_size` instead of the box; scroll a long document past it, search (which rebuilds), resize, and switch views; check rows, clipping, and leftovers through Rex captures, and the native library's protocol choice in code. Write the result to `research/inline-image-spike.md` (what works, what does not, and the chosen structure). Validation: the research note; a go or stop recorded in the plan. _Commit: `docs: the inline image spike`._
2. **HTML images** — `reader/images.rs` parses `<img>` (`src`, `alt`, `width`, `height`, `align`) and `<picture>` (`<source media srcset>`, `<img>`), D4's block test, and the rewrite in `ui/markdown_images.rs`. Tests: the README's exact block; `<img>` alone; `<p align="center"><img></p>`; `<a href><img></a>`; HTML with text (untouched); a remote or SVG `src` (box with reason); line count held. Validation: `cargo test images`. _Commit: `feat(markdown): find HTML images in the rendered view`._
3. **The picture** — the renderer draws the decoded picture (D3, D5, D6), the box as placeholder and for the reasons, the theme switch for `<picture>` (D2), `Enter` and click unchanged, search numbering and git signs unchanged. Records (D7) in this commit. Tests: the size rule at 40, 80, 120 columns and with `width`; the cap; the cache keeps decodes across a rebuild; the theme switch picks the other source; a failed decode keeps the box. Validation: `cargo test`; Rex live run of `README.md` and a fixture with several images at 120 and 50 columns. _Commit: `feat(markdown): draw local images in the rendered view`._
4. **Docs sync, agent docs sync, ledger close, handover** — `rust.md` (the renderer and cache contracts), `shape.md`, the plan archived with its index row, a handover. _Commit: `docs: inline images in the Markdown preview`._
5. **Final validation and live run** — the matrix; Rex at 120 and 50 columns. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl fmt --check --quiet .` | No drift |
| Unit | `cargo test images`, `cargo test markdown` | HTML parsing, D4, sizes, cache, theme source |
| App | `cargo test` | Enter, click, back, placeholder to picture |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Full suite | `just verify` | All green |
| Live run | Rex at 120 and 50 columns | Rows and layout right, nothing left behind on scroll |
| Owner | Ghostty: `README.md` and a fixture | The logo and pictures draw, scroll cleanly, follow the theme |

### Out of scope

- Remote images, SVG, animated GIF frames (the first frame only), and images inside lists, quotes, tables, or sentences.
