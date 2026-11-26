---
status: open
created: 2026-11-26T07:45:00+07:00
updated: 2026-11-26T07:45:00+07:00
summary: "The owner reports that scrolling looks glitchy: the scrollbar does not move smoothly. Not reproduced by an agent yet (Rex cannot inject wheel events)."
count: 1
---

# Scrolling and the scrollbar are not smooth

## The scrollbar glitches while scrolling

- status: open
  area: scrollbar and scroll rendering (`src/ui/scrollbar.rs`, the reader, tree, preview, and diff scroll paths in `src/app/scroll.rs`)
  summary: Owner report, 2026-11-26, from a manual run: while scrolling, the scrollbar looks glitchy rather than smooth. Not yet known whether it is the wheel, the keys, or both; which pane; or whether the thumb jumps, flickers, or lags the text. Scouter draws the thumb in half-block glyphs (`▐ ▝ ▗`, `scouter:src/ui/styled.ts:28-51`) and scrolls with OpenTUI's `ScrollBoxRenderable` (`scouter:src/reader/reader-pane.ts:31-48`); fmc paints its own thumb (`src/ui/scrollbar.rs`, `paint`). Candidates to check: the thumb position and size rounding against Scouter's formula, a repaint of the whole bar per tick instead of only on change, the thumb painted before the text has scrolled (one frame of lag), and wheel ticks that scroll more than one row and skip thumb steps. Compare with Scouter in the same terminal by hand: the agent cannot drive the wheel.
  plan: investigated before plan-00001 W5 phase 18; a fix lands as its own commit
  trigger: the owner's report (2026-11-26)
