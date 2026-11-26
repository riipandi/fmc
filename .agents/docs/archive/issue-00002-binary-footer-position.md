---
status: resolved
created: 2026-11-26T06:14:02+07:00
updated: 2026-11-26T12:29:03+07:00
summary: "Scouter keeps the footer's position from the previous document after a binary file opens (the footer reads `All` where no document is shown). fmc shows no position for a binary. The count matches the items listed."
count: 1
---

# Binary files and the footer position

## Scouter's footer keeps a stale position after a binary file

- status: resolved
  area: reader, footer (`src/app/mod.rs` `paint`, `src/app/reader.rs` `footer_facts`)
  summary: Scouter's binary view does not call `updatePosition`, and `setDocument` does not clear the position, so after an image the footer keeps that image's `All` (`scouter:src/app/app.ts:411-419`, `showBinary`; `research/scouter-inventory.md` section 18). Live at 80x24, Scouter reads `4.0 KB · binary · All` for `assets/seal.bin` opened after `assets/marauder-map.png`; fmc reads `4.0 KB · binary` (`/tmp/fmc-ref/out/w3-11/{fmc,sco}-imgbin-03.txt`, outside the repository). fmc shows no position for a binary.
  plan: plan-00001 phase 13 says report it, not copy it; phase 23 decides
  trigger: plan-00001 phase 23 (the owner may approve the stale value as a deviation, or fmc keeps the clean footer)
  audit: audit-00001 F-07 (2026-11-26): reproduced at 80x24 (`12-png-then-binary`: Scouter `4.0 KB · binary · All`, fmc `4.0 KB · binary`); an owner question, with the options in the audit; no code change until the owner picks.

Resolved by the owner's decision (2026-11-26): fmc keeps its clean binary footer, with no stale position. Recorded as an approved deviation in `architecture.md` ("Approved deviation: a binary file's footer shows no stale position") and in `docs/parity.md` ("Footer"), commit 6cb4373. Audit verdict F-07 is wont_fix (approved deviation).
