---
status: open
created: 2026-11-26T07:45:00+07:00
updated: 2026-11-26T09:30:00+07:00
summary: "The owner reports that scrolling looks glitchy: the scrollbar does not move smoothly. Rex captures match Scouter 0.9.9 frame by frame for keys and the wheel; the cause is not found, so the owner checks the feel by hand."
count: 1
---

# Scrolling and the scrollbar are not smooth

## The scrollbar glitches while scrolling

- status: open
  area: scrollbar and scroll rendering (`src/ui/scrollbar.rs`, `src/ui/viewer.rs`, `src/app/scroll.rs`, the tree rows in `src/ui/tree.rs`)
  summary: Owner report, 2026-11-26, from a manual run: while scrolling, the scrollbar looks glitchy rather than smooth. Which pane and which input the owner used is not recorded; whether the thumb jumps, flickers, or lags the text is not recorded either.
  evidence: |
  Investigated 2026-11-26 on fmc at `main` (`42e22ae`) against Scouter 0.9.9 (core 0.9.9, `/tmp/fmc-ref/scouter-099`), 80x24, `chapters.txt` (200 lines) and wrapped text, in new Rex tabs:
  - Thumb math equals Scouter's `renderScrollbar` (`scouter:src/ui/styled.ts:28-51`): the size floor of two half-cells, the rounded start, the `▐ ▝ ▗` choice. `ui/tree.rs` `thumb` takes the same arithmetic. The colour is the same token (`theme.muted`).
  - Keys (`j` one at a time, a burst of twenty `j`, `ctrl+d`, `ctrl+u`, `G`, `k`, `gg`): the text and the thumb glyphs agree with Scouter on every frame.
  - Wheel (the wheel is injectable: a raw SGR report, `ESC [ < 65 ; 40 ; 12 M` for one notch down and `< 64` for up, sent with `rex send`, reaches both apps): one notch moves one row in both; a burst of five moves five; the thumb frames agree with Scouter's.
  - Wrapped text (`w`, `G`, a wheel notch up, `ctrl+u`): agrees with Scouter at the bottom and back at the top.
  - Immediate captures (no settle time after each notch): the thumb matches the text line of the same capture, so no one-frame lag shows in captures.
  - Wheel step: OpenTUI's default is one cell per step on both sides (`opentui-0.9.9/src/widget/scroll.rs:584-606` with `LinearScroll`; `opentui/packages/core/src/renderables/ScrollBox.ts:478-497` with `LinearScrollAccel`). fmc's list wheel moves one row per event (`src/app/mod.rs`, `wire_mouse`).
  - Output per wheel notch (a `script` log, 40 notches): fmc writes about 2.7 KB, Scouter about 1.4 KB. The extra is colour runs in the reader rows (about 3.3 cursor moves a row for fmc, 1.9 for Scouter), not extra rows: the header and border rows are re-emitted only a few times in 40 notches. A redraw of about 2.7 KB per notch is small, so this alone does not explain a visible stutter; it is the first candidate if the owner sees it in a slow terminal.
  - Scouter's ScrollBox citation corrected: the ScrollBox is in `scouter:src/app/reader-pane.ts:31-48` (`src/reader/reader-pane.ts` does not exist at `434d000`).
    No defect is reproduced, so no fix lands and this item stays open.
  owner check: |
  Run fmc and Scouter 0.9.9 (or 0.3.4) in the same terminal on the same long file (more than 60 lines, for example a file from the demo repository), one after the other:
    1. One wheel notch at a time over the reader: each notch should move the text one row and the thumb by a half cell at most. Compare where the thumb changes glyph in each app.
    2. A fast trackpad flick over the reader in each app: say whether fmc stutters more than Scouter, and whether the thumb or the text stutters.
    3. The same in the sidebar on a folder with more rows than the pane.
    4. Note the terminal app, its font, and whether the thumb glyphs (`▐ ▝ ▗`) render as half cells in both apps. If fmc stutters only under a flick, the candidate is the redraw volume above; if the thumb alone lags, reply with the terminal and the file.
  plan: investigated before plan-00001 W5 phase 18; no defect reproduced, so no fix commit
  trigger: the owner's report (2026-11-26)
