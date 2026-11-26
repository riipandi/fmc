---
status: open
created: 2026-11-26T16:07:05+07:00
updated: 2026-11-26T16:07:05+07:00
summary: "The owner's hand evaluation of fmc (2026-11-26) found four items: the scrollbar thumb does not drag and a track click does nothing; the changes screen opens in all-files; the binary notice names Scouter; a click does not move the focus. The owner asked for all four to be fixed, one commit each."
count: 4
---

# Owner's hand evaluation, 2026-11-26

## The scrollbar thumb does not drag, and a track click does nothing

- status: resolved (2026-11-26, `fix(scrollbar): drag the bar's thumb and jump on a track click`)
  area: every scrollbar: the reader and its rendered and preview views (`src/ui/scrollbar.rs`, `src/app/mod.rs` paint), the file tree (`src/ui/tree.rs` `TreeView::paint`), the changes list (`src/ui/changes.rs`), and the stacked diff body (the reader's `narrow_bar`)
  summary: The owner holds a click on the thumb and drags; nothing scrolls. The wheel and the trackpad are smooth. The cause: fmc paints each bar from `Text` cells with no mouse handler, so a press on the bar reaches the pane's row handler (the tree and changes list select a row under the bar) or nothing.
  evidence: |
  Scouter 0.9.9 live run (`/tmp/fmc-ref/scouter-099`, 80x24, `README.md` of the demo repository, rendered view, 78 lines, the thumb on rows 4 to 7): a raw SGR press on the thumb at column 79 and row 6, a motion to row 14, and a release moved the footer from `top` to `57%`. A press on the track at row 8, released there, moved it to `22%` (the row fraction (4 / 18) times the 60-row range, as `Slider.updateValueFromMouseDirect` computes it). A wheel notch over the bar column (column 79) moved one row per notch (`22%` to `25%` after two notches).
  Scouter's mechanism: each bar is the `ScrollBoxRenderable`'s vertical `ScrollBarRenderable`, whose `SliderRenderable` is interactive and has a transparent track (`scouter:src/ui/styled.ts:24-26`); the custom thumb is painted over it (`:28-51`). `@opentui/core` 0.9.9 `chunk-bun-qkerj9a2.js:12249-12286` (`setupMouseHandling`: a press on the thumb keeps the grab offset, a press on the track jumps to the pointer and then drags, a release ends the drag), `:12287` and `:12298` (`updateValueFromMouseDirect`, `updateValueFromMouseWithOffset`).
  The locked `opentui` 0.9.9 crate offers the same drag as `widget::Slider` (`src/widget/slider.rs:61` `set_and_notify`, `:81` `value_at`, `:129` `mouse`): a press outside the thumb jumps, a press on it grabs with the offset, and a drag or release moves the value. Its `range` and `thumb_size` set the native thumb as `ScrollBox` sets its own bars (`src/widget/scroll.rs:384-391`).
  plan: none; the owner asked for the fix (2026-11-26). Related and still open: issue-00006 (the bar's feel while scrolling).
  resolution: A transparent `Slider` lays over each bar (`src/ui/scrollbar.rs` `track`, `sync`), and `wire_bar` (`src/app/mod.rs`) moves the pane it belongs to. Live in Rex at 80x24 against Scouter 0.9.9 on the demo repository: the rendered README drag to row 14 reads `57%` in both apps and the frames match; a track press at row 8 reads `22%` in both; the diff body's drag reads `56%` in both with the same rows; the wheel over the bar moves one row per notch in both. Tests: `src/ui/scrollbar_tests.rs` (a track click, a drag on the thumb, the text height in the range, a press on the thumb).
  trigger: None — closed.

## The changes screen opens in all-files

- status: open
  area: the changes screen's default scope (`src/app/changes.rs` `Options`, `src/app/mod.rs` `run`)
  summary: The approved deviation (architecture.md, "the changes screen opens in one-file mode"; docs/parity.md) says the screen opens showing one file. The code does not: `Options` derives `Default`, so `one_file` is `false` and every changed file is stacked. Scouter's default is `scope = "all"` (`scouter:src/diff/diff-screen.ts:82`).
  evidence: |
  `src/app/changes.rs:40-47`: `#[derive(Debug, Default)] pub struct Options { split, one_file, wide }`. The badge (`changes.rs` `badges`, `one file` or `all files`) and the footer hint (`src/ui/hints.rs:71`) both follow `one_file`.
  plan: none; the owner asked for the fix (2026-11-26).
  trigger: resolved by the commit `fix(changes): open the changes screen in one-file mode`.

## The binary notice names Scouter

- status: open
  area: the binary notice (`src/ui/binary.rs:55`)
  summary: The notice reads "Scouter only displays UTF-8 text and images." The product name is fmc (approved deviation, architecture.md, "the product name is fmc wherever Scouter says scouter"). A grep of `src` for `scouter` outside comments, test names, and vendored query comments finds this string and no other user-visible one.
  plan: none; the owner asked for the fix (2026-11-26).
  trigger: resolved by the commit `fix(binary): name fmc in the binary notice`, with a test that pins the notice.

## A click does not move the focus

- status: open
  area: the focus of the tree, the reader, the changes list, and the diff body (`src/app/mod.rs` `wire_mouse` and the viewer and sidebar panels, `src/app/changes.rs`)
  summary: Approved deviation (architecture.md, "a click focuses the pane under the pointer", commit a91844c): a left click anywhere in a pane focuses it, including its scrollbar. Today only a click on a tree or changes row sets the focus, so a click in the reader text, its header, or the diff header leaves the focus where it was. Row clicks keep their actions; in the reader a press still starts a drag-select.
  plan: none; the owner asked for the change (2026-11-26) and approved it in a91844c.
  trigger: resolved by the commit `feat(focus): a click focuses the pane under the pointer`; the thumb press also focuses its pane (item 1).
