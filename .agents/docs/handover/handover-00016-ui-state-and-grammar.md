---
status: done
created: 2026-11-28T16:56:09+07:00
plan: plan-00020-ui-state-and-grammar-rest
---

# Handover: UI state, Lua doc colour, eex, and Markdown front matter; plan-00020 closed

## Closed

- plan-00020 on `feat/ui-state-and-grammar-rest` (branched from `docs/plan-ui-state-grammars`, which held the plan). Commits, in order: `c95faf0` (phase 1, `meta/state.json` read and written), `676d439` (phase 2, the toggles remembered, with the deviation's records), `522fc83` (phase 3, Lua `---` documentation comments and `.eex` and `.leex` as heex), `10e65b9` (phase 4, Markdown front matter as a code block), and the closing commit of phase 5 (the roadmap, the ledger, this handover).
- RI-2, RI-9, and RI-17 are `landed` in `roadmap-00001-post-parity.md`, with their status table rows.
- Phase 6: `just verify` green (867 tests run, 1 skipped), and the live runs of phases 2 to 4 in new Rex tabs in a temporary `XDG_CONFIG_HOME`.

## Decisions taken as planned

- The owner's D1 to D6, D8, D9, D11, and D12 (`plan-00020`, archived). D7 (the file's forgiving read) and D10 (the writes rule grows by one file) are the agent's, and they stand.
- The remembered values are compared at each paint, so a toggle needs no write of its own (`app/remember.rs`). A write waits one second after the last change and runs on a worker; a write still waiting at quit is made before the process ends (joined thread).
- The sidebar's visibility is not saved. The finder's matching in `meta/state.json` wins over `find_mode` in `settings.json` once the state holds one.

## Choices the plan did not settle (for the owner)

- The quit write: the plan says a crash loses at most a second. A clean quit inside that second is written too (`app/remember.rs` `flush`). The alternative is to lose the last change.
- The `---` doc colour: OpenTUI paints a capture over a whole node, and the Lua grammar has no node for a tag inside a comment, so the tag cannot take its own colour (probed in a temporary query, then reverted). The whole `---` line takes `comment.documentation`, which the built-in palettes style as a comment. A `---` line therefore looks like any other comment until a palette gives documentation its own role.
- The front matter fence guard: a block with a fence line inside it is left plain, because the fence would close early. D11 does not name this case.

## Owner checks

1. Read a `---@param` line in a real terminal and decide whether documentation needs its own colour. A new palette role touches `DESIGN.md` and all 16 catalog themes (`architecture.md`, "Lua patterns are an injection; Lua documentation comments take the documentation capture").
2. Press `a` on the changes screen and check that it stacks all files and that a restart keeps it. The live run did not press `a`, because the instructions for it forbade the key; the unit test covers it.
3. Check `Ctrl+.` and `Ctrl+→` on a Kitty-protocol terminal (Ghostty or similar) across a restart; the live run used `.` and `Ctrl+→` under Rex.
4. The installed Scouter is 0.3.1, not the v0.3.4 baseline. The front-matter and the eex parity notes rest on a 0.3.1 live run. If the baseline is to be re-checked, run v0.3.4 (`434d000f`) on the same sample files.
5. The commits carry `Co-Authored-By: Claude Opus 5.5`, as the brief named. The session's attribution reminder named Haiku 5.5; confirm which trailer the owner wants on later commits.

## Open threads

- `handover-00001-parity-w0-w3.md` stays live as `partial`; its threads are closed or listed in handover-00002.
- The handovers 00012 to 00015 were `done` and had stayed live after a newer one landed, which the `agent-docs` rule says they should not; they are archived with this handover.
