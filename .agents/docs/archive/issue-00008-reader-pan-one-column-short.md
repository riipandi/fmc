---
status: resolved
created: 2026-11-26T08:57:16+07:00
updated: 2026-11-26T12:17:57+07:00
summary: "At 80x24, panning a long source file to its end leaves fmc's first code column one cell right of Scouter's. The HEAD build pans the same way, so the cause predates plan-00001 phase 19; the changes screen's diff is not involved."
count: 1
---

# The reader's pan ends one column short of Scouter's

## Four pans right on src/main.rs

- status: resolved
  area: reader pan (`src/ui/viewer.rs` `set_wrap`, `src/ui/reader.rs` `set_width`, the gutter of `src/ui/reader.rs`)
  summary: Tab into the reader on `src/main.rs` (demo repository), then four `l` presses. Scouter 0.9.9 puts the first code column at the pane's left edge (`use std::collections::HashMap;` starts in column 0 after the gutter). fmc starts it one cell right (` use std::...`). The HEAD build gives the same result, so the difference is not from the changes screen. Evidence: `/tmp/fmc-ref/out/w5-19/rd/s-01-k:tab.txt` (Scouter), `rd/f-01-k:tab.txt` (fmc), `rd/h-01-k:tab.txt` (HEAD). The `G` and `g` steps keep the pan in both apps.
  plan: measure the widest line's width and the pane's inner width at 80x24 against Scouter's `ScrollBox` content width, then fix the content width by the difference. Not in plan-00001 phase 19.
  trigger: the parity audit (phase 22), or the next change to the reader's width
  audit: audit-00001 F-05 (2026-11-26): reproduced at 80x24 on `src/main.rs` after four `l` presses (`34-tree-main`); fixed in plan-00001 phase 23.

Resolved by plan-00001 phase 23e. The unwrapped width was the pane less the bar and three columns, one column short of Scouter's pan range; it is now the pane's inner width with the bar's column (`src/app/mod.rs` paint, `set_width(columns - (width + 2))`). Measured at 80x24 on `src/main.rs`: the first `l` moves the code to column 30 in both apps, and the second stops at column 29 in both, where the old width stopped at 30 (`90-pan-open`, `94-pan-open-y`, `97-pan-four`: the panned pane matches Scouter except the approved footer label).
