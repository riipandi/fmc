---
status: open
created: 2026-11-26T07:29:50+07:00
updated: 2026-11-26T07:29:50+07:00
summary: "On the demo README at 80x24 the Markdown table's Language and Owner columns are one cell narrower in fmc than in Scouter 0.9.9. Both HEAD and the W3 tip draw the narrower table, so the gap is not from plan-00001 phase 16."
count: 1
---

# Markdown table column widths differ from Scouter

## The rendered table is one cell narrower in two columns

- status: open
  area: reader, rendered Markdown tables (`src/ui/markdown.rs`, OpenTUI's Markdown)
  summary: The table at `## Modules` of the demo README (`/tmp/fmc-ref/demo-fmc/README.md`, line 13) renders with the Language column 9 cells wide and the Owner column 17 in fmc, against 10 and 18 in Scouter 0.9.9. The cell text and the Module column match. The W3 captures (`/tmp/fmc-ref/out/w3-11/`) recorded equal widths, and the Markdown renderer has not changed since W3, so the cause is not yet established (a change in the demo file or in the table layout of OpenTUI are both open). Evidence: `/tmp/fmc-ref/out/w4-16/readme-head.txt` and `sco-e6-03.txt`.
  plan: plan-00001 phase 22 records it in the audit; phase 23 fixes it with the rendered view
  trigger: plan-00001 phase 22, or a reproduction that shows which side changed
