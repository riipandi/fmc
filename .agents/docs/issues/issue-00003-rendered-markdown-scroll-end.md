---
status: open
created: 2026-11-26T06:46:46+07:00
updated: 2026-11-26T06:46:46+07:00
summary: "The rendered Markdown view stops two rows short at the bottom: Scouter's last lines (`Learn more in docs/guide/lumos.md.`) show after `G`, fmc's do not. The count matches the items listed."
count: 1
---

# Rendered Markdown scroll end

## The rendered view does not scroll to its last rows

- status: open
  area: reader, rendered Markdown (`src/ui/markdown.rs`, `src/app/scroll.rs`)
  summary: On the demo README at 80x24, `G` in the rendered view shows the table, the rule, and the next blank rows in fmc, but Scouter's last line, `Learn more in docs/guide/lumos.md.`, is visible below them (`/tmp/fmc-ref/out/w3-11/{fmc,sco}-mdG-01.txt`, outside the repository). The footer reads `Bot` in both. The source view (`m`) and the plain text scroll to the same end as Scouter. The cause is not established: the rendered rows reported to the scrollbar and the scroll range may come from different layouts.
  plan: plan-00001 phase 14 captured the motion keys on text; phase 23 fixes this with the rendered view
  trigger: plan-00001 phase 23, or a reproduction that points at the cause first
