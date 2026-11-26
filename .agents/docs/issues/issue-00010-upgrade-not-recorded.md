---
status: open
created: 2026-11-26T12:18:59+07:00
updated: 2026-11-26T12:18:59+07:00
summary: "fmc's `--help` omits Scouter's `--upgrade` line, which plan-00001 excludes, but no architecture record or parity line records the drop (audit-00001 F-08)."
count: 1
---

# The `--upgrade` drop is not recorded

## Scouter's `--upgrade` has no fmc record

- status: open
  area: command line (`src/args.rs`), agent and user docs (`.agents/docs/architecture.md`, `docs/parity.md`)
  summary: Scouter's `--help` lists `scouter --upgrade` (`scouter:src/index.ts:11`, handled at `:26-28`). fmc's `--help` omits it (`help-sco.txt` against `help-fmc.txt`). The plan's "Explicitly out of scope" names the installer and `--upgrade` (plan-00001), but `architecture.md` and `docs/parity.md` do not, so under the parity rule the difference is unrecorded. Run outside a terminal, `fmc --upgrade` prints `Run fmc in an interactive terminal.` (exit 1); Scouter prints its installed-command message (exit 1).
  plan: plan-00001 phase 23 stops here (owner question); no sub-phase
  trigger: the owner decides. Options (audit-00001 F-08): (1) record it as an approved deviation (an `architecture.md` record citing the plan's out-of-scope list, and a `docs/parity.md` line), recommended, since fmc ships as one binary; (2) implement `--upgrade`, which needs Scouter's installer; (3) other.
