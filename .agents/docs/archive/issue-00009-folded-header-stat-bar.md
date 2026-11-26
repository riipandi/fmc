---
status: resolved
created: 2026-11-26T12:18:59+07:00
updated: 2026-11-26T12:28:57+07:00
summary: "Scouter draws the stat bar (`+4  ▪▪▪▪`) on a folded diff header and no bar on an unfolded one; fmc draws no bar in either state. The Scouter cause is not established, so the parity target needs the owner (audit-00001 F-06)."
count: 1
---

# The diff header's stat bar

## The folded header has a bar in Scouter and not in fmc

- status: resolved
  area: changes, stacked diff headers (`src/diff/row.rs` `header_counts`, `src/ui/diff.rs` `paint_headers`)
  summary: At 80x24 on the demo repository, Scouter's folded header of `README.md` reads `+4  ▪▪▪▪` (four bar cells in the added colour, `19-changes-c-fold.html`); fmc's reads `+4`. Scouter's unfolded header shows no bar, before and after a scroll (`15-changes-d`, `52-bar-unfold-after-scroll`, `53-bar-after-scroll`). Scouter's source draws the bar on every header (`scouter:src/diff/diff-blocks.ts:217`, `:299`, `:491`; `statBarSegments` at `:72-96`), so the unfolded absence has no cause in the source. fmc builds the bar in `header_counts` (`src/diff/row.rs`) and it never reaches the screen.
  plan: plan-00001 phase 23 stops here (owner question); no sub-phase
  trigger: the owner picks the target. Options (audit-00001 F-06): (1) match Scouter's visible states exactly, the bar only on folded headers, after finding the cause; (2) bar on every header, as Scouter's source draws it; (3) no bar anywhere. Recommended: (1), falling back to (2) if the cause is not found in a short look at Scouter.

Resolved by plan-00001 phase 23f, on the owner's instruction (2026-11-26) to match Scouter's visible states. The cause is the stack's width. Scouter's stack is as wide as its widest visible block, and each header is 100% of its block: an unfolded diff is the pane plus its gutter, so every header is that wide and its counts and bar fall past the pane edge; a folded block is pane-wide. fmc sized every block to the pane plus gutter regardless of fold. `src/ui/diff.rs` `apply_widths` now does what Scouter does, with the rule in `diff/stack.rs` `widens_stack` (test `an_unfolded_diff_widens_the_stack_and_a_folded_diff_or_a_notice_does_not`). Live at 80x24 against Scouter 0.9.9, the header rows match in every state: one-file folded (`+4  ▪▪▪▪`), one-file unfolded (no bar), stacked with README folded (`+4 ▐` while another diff is unfolded), `seal.bin` after README folds (`new`), and after `c` `c` (`q6`, `q7`, `q8`, `q2`, `q5`, `r1`, `r2`). The fallback (a bar on every header) was not needed.
