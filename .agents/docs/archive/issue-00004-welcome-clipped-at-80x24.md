---
status: resolved
created: 2026-11-26T07:29:50+07:00
updated: 2026-11-26T12:08:05+07:00
summary: "At 80x24 the welcome screen loses its wordmark and the bottom of its hint card: fmc draws the tagline at row 13 with blank rows above it, Scouter draws the wordmark at rows 4 to 6 and the whole card. The clipped screen is in the HEAD build too."
count: 1
---

# Welcome screen clipped at 80x24

## The welcome content is taller than the reader at 80x24

- status: resolved
  area: reader, welcome (`src/ui/welcome.rs`, the viewer stack in `src/app/mod.rs`)
  summary: With no document open, the welcome at 80x24 shows the tagline, the hint card without its `^q quit` row, and no closing line; the three rows of the wordmark are blank. Scouter 0.9.9 on the same tab shows the wordmark, all five card rows, and `Press ? for every key.`. The HEAD build (e80d582, built from `git archive`) clips the same way, so the cause predates plan-00001 phase 16. Evidence: `/tmp/fmc-ref/out/w4-16/head-welcome.txt` (fmc) and `/tmp/fmc-ref/out/w4-16/sco-n1-02.txt` (Scouter), outside the repository. Seen again on 2026-11-26 at `3540853` (phase 18 executor's regression capture at 80x24 on the demo repository): fmc shows no wordmark, no `Press ?` line, and no `q` row in the card, against Scouter 0.9.9's full welcome; the phase 8 note's claim that the 80x24 welcome equals Scouter does not hold at that commit, so the parity audit (phase 22) should re-check it.
  plan: plan-00001 phase 23, a parity fix on the welcome; the first step is to measure the welcome's rows against the reader's inner height at 80x24 (the margins add up to about 19 rows against 18 available, which is a guess until measured)
  trigger: plan-00001 phase 23, or the parity audit (phase 22) if it reaches the welcome first
  audit: audit-00001 F-01 (2026-11-26): reproduced at 80x24 on d826786 (`welcome-fmc.txt` against `welcome-sco.txt`); fixed in plan-00001 phase 23.

Resolved by plan-00001 phase 23a: the cause was the reader's viewer, a visible empty child that took half the stack while no document was open (`src/app/mod.rs`, `ui.viewer.visible`). At 80x24 the welcome now draws its wordmark (FMC, approved), tagline, card with the `^q` row, and closing line on the rows Scouter uses (`welcome-fmc.txt` against `welcome-sco.txt`, rerun after the fix). The sidebar and the binary, image, changes, and error captures are unchanged.
