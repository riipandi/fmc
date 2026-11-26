---
status: open
created: 2026-11-26T12:18:59+07:00
updated: 2026-11-26T12:18:59+07:00
summary: "Scouter draws the stat bar (`+4  ▪▪▪▪`) on a folded diff header and no bar on an unfolded one; fmc draws no bar in either state. The Scouter cause is not established, so the parity target needs the owner (audit-00001 F-06)."
count: 1
---

# The diff header's stat bar

## The folded header has a bar in Scouter and not in fmc

- status: open
  area: changes, stacked diff headers (`src/diff/row.rs` `header_counts`, `src/ui/diff.rs` `paint_headers`)
  summary: At 80x24 on the demo repository, Scouter's folded header of `README.md` reads `+4  ▪▪▪▪` (four bar cells in the added colour, `19-changes-c-fold.html`); fmc's reads `+4`. Scouter's unfolded header shows no bar, before and after a scroll (`15-changes-d`, `52-bar-unfold-after-scroll`, `53-bar-after-scroll`). Scouter's source draws the bar on every header (`scouter:src/diff/diff-blocks.ts:217`, `:299`, `:491`; `statBarSegments` at `:72-96`), so the unfolded absence has no cause in the source. fmc builds the bar in `header_counts` (`src/diff/row.rs`) and it never reaches the screen.
  plan: plan-00001 phase 23 stops here (owner question); no sub-phase
  trigger: the owner picks the target. Options (audit-00001 F-06): (1) match Scouter's visible states exactly, the bar only on folded headers, after finding the cause; (2) bar on every header, as Scouter's source draws it; (3) no bar anywhere. Recommended: (1), falling back to (2) if the cause is not found in a short look at Scouter.
