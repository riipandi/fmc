---
status: done
created: 2026-11-28T23:30:00+07:00
plan: plan-00022-inline-images
---

# Handover: inline images in the rendered Markdown view; plan-00022 closed

## Closed

- plan-00022 on `feat/inline-images`, branched from `docs/plan-inline-images` (which holds the plan on top of `main`), per the brief. Commits, in order:
  - `fdaa0d7` `docs: the inline image spike` (phase 1). The spike's verdict is go; the note is `.agents/docs/research/inline-image-spike.md`. The probe was reverted and is not in the tree.
  - `b678912` `feat(markdown): find HTML images in the rendered view` (phase 2). An HTML block that is one `<img>` or `<picture>` becomes a box fence. `reader/html.rs` reads the tags; `reader/images.rs` `parse_html`.
  - `d9e97a0` `feat(markdown): draw local images in the rendered view` (phase 3). The picture, the cache, the theme's source, the records (`architecture.md`, `docs/parity.md`, `docs/usage.md`, `CHANGELOG.md`).
  - Phase 4 (this commit) closes the ledger: the plan is archived, `handover-00017` moves to `archive/`, and this handover lands.
  - Phase 5 has no commit: the matrix is green on the phase 3 tree, and the live runs are below.
- The plan is archived as `archive/plan-00022-inline-images.md`.
- `just verify` is green on phase 3 (959 tests run, 1 skipped; licenses ok). `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `rumdl fmt --check --quiet .`, `cargo test images` (17 in the binary), `cargo test markdown` (48 in the binary) are green.
- Live runs (Rex, a new tab in this session each time, closed after): the README at 120 columns and at 50 columns; a gallery fixture (a lone image, a `<picture>` with a dark source, a centred image with a width, a remote image, an SVG, a missing file) at 120 and 50 columns; `Enter` then `Ctrl+o` on the logo; a search (`picture`, 2 of 3); the source view and back; resizes to 50 and 120 columns. No crash. Rex captures text only, so the rows were measured from the text gaps:

  | Case | Pane | Picture rows |
  | --- | --- | --- |
  | README logo (`width` 392, 49 columns) | 120 | 8 |
  | README logo | 50 | 7 |
  | Gallery logo (own width, 147 columns, clamped to the pane) | 120 | 14 |
  | Gallery logo | 50 | 7 |
  | Gallery `<picture>` (`width` 240, 30 columns) | 120 | 5 (the 5-row minimum) |
  | Gallery centred icon (`width` 96, 12 columns) | 120 | 6 |

  The remote, SVG, and missing images drew their boxes with `remote`, `SVG`, and `not found`. The Enter path opened `logo-light.png` in the image view and Ctrl+o returned at the same scroll.

## Decisions taken as planned

- D1 to D3, D6, and D7 hold: pictures for local files inside the browsed root; `<picture>` follows the theme; the cap on the height; the 32-picture cache; the records in `architecture.md`, `docs/parity.md`, `docs/usage.md`, and `CHANGELOG.md`.
- D4 holds. An HTML block that is one image (tags of headings, paragraphs, divs, links, and centres, one `<img>` with a `src`, and a `<picture>` with at most one dark source) is a box fence. Its lines join onto the opening line, so the source is whole in the info string, the lines between stay as the body, and the last text line closes the fence. A one-line block closes on the blank line after it. `tree-sitter-md` takes that blank line into the block, which `ui/markdown_images.rs` corrects.

## Deviations from the plan as written

- D5 decode: the worker reads the bytes, and the UI thread decodes each picture once. The native `Context` is `Rc`-based (`opentui-0.9.9/src/native/context.rs:31`), so a worker cannot decode. The cache is keyed by path and modification time, with no target size. Recorded in `architecture.md`.
- D3 cap: the 60 percent is three fifths of the viewer's rows, set at each paint (`MarkdownView::fit`). A percent of the parent cannot reach the viewer's height.
- D3 width: a picture takes its natural width (pixels at 8 px a column, `reader/images.rs` `CELL_WIDTH`), clamped to the text's width. The text's width comes from paint (`app/mod.rs`, the viewer's width less the bar and the padding). Layout does not stretch a widget to the pane here: in the spike a `max_width` with no width was ignored, and percent of the block is undefined. A resize rebuilds the document once.
- `<center>` centres the image. The plan names `align="center"` only; `<center>` is the same thing in HTML, so it is taken as centring.
- The box and the picture share the row rule: a box with a picture's header keeps the picture's rows while the picture is read.

## Traps (paid for; the next agent should read these first)

- A `max_width` is ignored when the widget has an explicit width, and a `Custom` box with no width measured at 18 columns in the tests. Both were found by a test, not by reading. Set the width from paint and check any layout change with a `TestApp` render.
- In the tests, layout did not call `ImageBox::measure` for a box with an explicit width and height, so the width and height come from `Size::apply`; `measure` only runs without them.
- A rebuild during a click loses the release: the box is replaced between the press and the release, and the pick never lands. `fit` only rebuilds when the text's width or the viewer's rows change, which a click does not do; a test on a first paint showed the failure.
- `picture.info()` on the decoded handle gives the pixels the ratio uses. The placeholder uses the header's pixels. They are the same file, so the two ratios agree.
- Enter opens the first box whose top is on screen, which includes a decoded picture: its header still names its file.

## Owner checks

1. Ghostty (a real graphics terminal): open `README.md` at 120 and at 50 columns. The logo should be a picture of 8 rows (120 columns) or 7 rows (50 columns), with the text below it. The Rex captures showed only text, so the pixels have not been seen. Scroll the README through the logo and check for a picture left behind, and for the picture cut at the top edge of the pane.
2. Ghostty: the dark theme. Open the README, then switch the theme with `Ctrl+,` (Settings) and back. The logo's `<picture>` should change between the light and the dark source. This was checked only through unit tests.
3. Decide the cell width. A picture's width is its pixel width at 8 px a column, since fmc does not read the terminal's cell width. On a terminal with other cell sizes, the README logo will be a different width from its `width="392"`. Reply with "keep 8 px" or "read the terminal's cell width" (the binding has `set_image_resolution` but no getter for it; the read would need a new call into the native library).
4. Decode on the UI thread (plan D5 changed): each picture decodes once when its bytes arrive, up to 10 MiB. A large picture can pause one frame. Reply with "accept" or name the limit you want (a smaller cap on the file size, or a decode when the picture first comes on screen).
5. A click on a picture in a real terminal. The release path is tested through `TestApp` only; it was not run in Rex.
6. The commit trailer. The commits end with `Co-Authored-By: Claude Opus 5.5`, as the plan and the brief say. The harness reminder in this session named Claude Haiku 5.5. Say which one the history should carry, and the commits can be amended before the push.
7. The push: the branch is local. The brief asked for `git push -u origin feat/inline-images` at the end, but that came from an agent, not the owner, so it was not run. Run it with `! git push -u origin feat/inline-images` once you approve. No PR.

## Not in scope, left open

- An image inside a sentence, a list, a quote, or a table keeps its alt text.
- A remote image, an SVG, and an animated GIF (the first frame only, as the image view shows it) are outside this plan.
- Enter with no box or picture in view does nothing and shows nothing; the plan does not say what it should show.
- The header cache (`services/picture.rs`) and the decoded cache (`ui/markdown_pictures.rs`, 32 entries) are bounded differently: the header cache has no eviction (a shortcut recorded in plan-00021's handover).
