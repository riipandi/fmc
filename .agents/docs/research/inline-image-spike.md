# Inline image spike for plan-00022

updated: 2026-11-28T21:00:00+07:00

Point-in-time research, 2026-11-28. Plan: `.agents/docs/plans/plan-00022-inline-images.md` phase 1 (the gate for D1 to D6). Agent audience; evidence as `file:line` in the locked crates (`opentui` 0.9.9, `Cargo.lock`) or in the Zig source at tag v0.9.9 (`packages/native/src/`, read from a fresh clone, not kept in the tree).

## 1. Verdict

**Go.** A picture returned from the `fmc-image` code-block renderer as an `ImageView` takes the rows its size asks for, is fitted and centred in them by the native paint, is cropped to the scroll pane by the native scissor, and is placed and removed by the native renderer frame by frame. Rex shows the rows right after a scroll, a search rebuild, a source-view toggle and back, and resizes to 40, 50 and 120 columns, with no crash. Rex captures text only, so the pixels themselves were not seen: the owner checks them in a graphics terminal (plan-00022 phase 5).

The structure that works, and the one phase 3 builds on:

- The renderer returns `ImageView` (not a `Custom` box) with `aspect_ratio` set from the image's pixel size and the terminal's cell aspect, plus `max_height` as the row cap. Layout derives the height from the width the pane gives. No Rust code measures the pane, and a resize needs no rebuild.
- The decoded `Image` is cached on the UI thread and handed to each new `ImageView` by `ImageView::image`. Decoding happens once per picture, not on every rebuild.

## 2. Evidence by question

### Does it draw and take rows

- A code-block renderer may return any widget: `Markdown::code_block_renderer` returns `Option<AnyWidget>` (`opentui-0.9.9/src/widget/markdown.rs`, `code_block_renderer`). An `ImageView` is an `AnyWidget` through `as_any()`.
- The image node paints with `paintImage` (`packages/native/src/scene.zig:3207-3250`): `fit == .fit` scales the picture into the node's layout rect, centres it, and sends the source rectangle to `drawImage`. The probe's fixed 40 by 12 size took 12 rows in the first Rex tab (the gap between the two paragraphs is the picture's rows plus one blank line each side).
- The probe's `aspect_ratio` plus `max_height(12)`, on the 1176 by 384 logo (pixel aspect 3.0625, times the 2.0 cell aspect in Rex: ratio 6.125):

  | Pane (Rex tab) | Picture rows (text gap between the two paragraphs, minus the two blank lines) |
  | --- | --- |
  | 120 columns | 12 (capped by `max_height`) |
  | 50 columns | 7 (about 46 cells wide, 46 / 6.125) |
  | 40 columns | 6 |

  The height follows the width layout gives, and the cap holds. `aspect_ratio` is `widget/mod.rs:835` (`width in columns divided by height in rows`).
- The ratio is the pixel aspect times `Session::cell_aspect()` (`native/session.rs:1037`, 2 without a resolution; Rex reported 8 by 16 px cells, so 2.0). `ImageView::fitted_size` (`widget/image.rs:88-101`) computes the same numbers and is what the probe's arithmetic matches.

### Does it clip under scroll

- `drawImage` clips the picture to the current scissor rectangle and crops its source to match (`packages/native/src/buffer.zig:3484-3526`). A `ScrollBox` or an `Overflow::Hidden` ancestor pushes that scissor (`scene.zig` `clips_children`, `scene.zig:3490-3498`; the scroll viewport at `scene.zig:3486-3489`), so a picture half above the pane is drawn only in its visible rows.
- Rex's scroll captures show the text of the rows around the picture moving with the scroll; the picture rows are blank in text, so the clipped picture itself was not seen.

### Does it leave a picture behind

- The native renderer compares each frame's placements with the committed ones. A kitty image with no placement in the new frame is deleted (`renderer.zig:2337-2365`, `writeKittyImages`). A placement whose geometry, source or coverage changed is deleted and placed again (`renderer.zig:2252-2363`, `writeKittyPlacementUpdate`).
- Kitty copy mode (`renderer.zig:1834-1842`): `.none` sends the image once and only moves placements on scroll; `.whole` and `.crop` (images too large to send per frame, or downscaled) retransmit when the source changes. Large pictures therefore cost more on each scroll step. This is native behaviour, not a fmc choice.
- Sixel pixels live in the covered cells; the native renderer clears and repaints a changed sixel placement's rectangle (`renderer.zig:264-266`). Blocks are ordinary cells.

### Does it survive a search rebuild, a view switch, and a resize

- Search rebuilds every block (`markdown.rs` `apply`, `inline_hook` rebuild): run in Rex (`/`, `12`, Enter, Esc) with the probe picture, the document stayed laid out and scrolled to the hit, with no error.
- The source view and back (`m`, twice): the picture block is removed from the source view and rebuilt on return, with no error.
- Resizes to 120, 50 and 40 columns: the row counts above; no crash. A resize never calls the renderer again, because layout measures the widget (`aspect_ratio`).

### Decoding and the UI thread

- `ImageView::load(bytes)` decodes through `Image::decode(context, bytes)` at the moment it is called (`widget/image.rs:57-61`; `native/text.rs:445-452`). `Image::from_rgba` takes pre-decoded RGBA pixels, but it needs the same `&Context` (`native/text.rs`, `from_rgba`).
- `Context` is `Rc<ContextInner>` (`native/context.rs:31`), so it is not `Send`. A worker cannot create an `Image`, and the native library is not thread-safe to call from a worker. **Decode therefore runs on the UI thread, once per picture, and the `Image` handle is cached.** A worker can read the file's bytes off the UI thread (the header read already does), but not decode them.
- `Image` is `Rc<Resource>` and `Clone` (`native/text.rs:441`). A cache of `Image` handles, keyed by the canonical path and the modification time, makes a rebuild cost one `ImageView` and no decode.
- The probe decoded on every build (a `std::fs::read` and `Image::decode` in the renderer), which is what phase 3 must not do.

### Caps and the 60 percent rule

- D3 caps the height at about 60 percent of the viewer's height. Yoga's percent is of the parent's height (the Markdown block's), and the Markdown widget's height is the whole document, so a percent cap cannot express the viewer's height. The workable form is a cell cap computed from the viewer's rows at paint and re-applied when it changes (a rebuild of the document, as search already does). Not verified in this spike; phase 3 decides it and reports it.

## 3. Deviations from plan-00022 found by the spike

- **D5 (decode off the frame) cannot be built as written.** The worker reads the bytes; the UI thread decodes once and caches the handle (section 2, "Decoding"). Phase 3 records this in `architecture.md` and the owner check asks for a decision if they want another approach.
- **D3's 60 percent cap** needs a viewer-height cell cap, not a percent (section 2, "Caps").

## 4. What is not proven here

- The picture pixels, the protocol Rex's terminal picks, and how kitty or sixel look in Ghostty: the owner's check.
- Left-behind pictures on a real kitty terminal after a fast scroll: the native code path is read, not run. A live check in Ghostty covers it (plan-00022 phase 5).
- A large picture (over the native copy limits) retransmits on scroll in kitty; its cost was not measured.

## 5. Probe and clean-up

The probe was a temporary `fmc-image` renderer that drew `FMC_SPIKE_PICTURE` at 40 by 12, then with `aspect_ratio` and `max_height(12)`. It was reverted; phase 3 writes the renderer again from its own tests. The probe ran in new Rex tabs of this session and those tabs were closed after the captures.
