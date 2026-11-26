---
status: resolved
created: 2026-11-26T06:46:46+07:00
updated: 2026-11-26T12:13:18+07:00
summary: "The rendered Markdown scrollbar and footer percentage are wrong: at 80x24 the end of the rendered README matches Scouter after `G`, but fmc's thumb covers 16 rows where Scouter's covers 4, and one page down reads 32% against 25% (audit-00001 F-03). The count matches the items listed."
count: 1
---

# Rendered Markdown scrollbar and percentage

## The rendered view's thumb and percentage do not match the document

- status: resolved
  area: reader, rendered Markdown (`src/ui/markdown.rs`, `src/app/scroll.rs`)
  summary: At 80x24 `G` in the rendered view now ends where Scouter ends (`Learn more in docs/guide/lumos.md.` and the last list item are both on screen, `10-readme-rendered-bottom`), so the text end is not the defect. The defect is the scrollbar and the percentage: after `G` fmc's thumb covers rows 6 to 21 (16 rows) and Scouter's rows 18 to 21 (4 rows); after one page down (`36-readme-mid`) the footer reads 32% in fmc and 25% in Scouter, from the same top line. The cause is not established (`rust.md`, "Rendered Markdown has rows only after a frame", is the first suspect).
  plan: plan-00001 phase 14 captured the motion keys on text; phase 23 fixes this with the rendered view
  trigger: plan-00001 phase 23, or a reproduction that points at the cause first
  audit: audit-00001 F-03 (2026-11-26): thumb and percentage reproduced (`10-readme-rendered-bottom`, `36-readme-mid`); the text end does not reproduce; fixed in plan-00001 phase 23.

Resolved by plan-00001 phase 23c. The cause was the row count: the rendered view's scrollbar and footer summed `Markdown::block_rows`, which are screen rows, so the total shrank as the view scrolled (the last block's bottom moved up the screen). The count is now the widget's own height (`MarkdownView::rows`, `src/ui/markdown.rs`). At 80x24 the rendered README matches Scouter after `G` (thumb rows 18 to 21, `Bot`) and after one page down (thumb `▗▐▐▐▝`, `25%`), and after three wheel notches (`5%`), captures `76-md-bottom-fix`, `77-md-mid-fix`, `79-md-wheel-3` (fmc and sco).
