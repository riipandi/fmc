---
name: parity-audit
description: >
    Screen-by-screen parity audit of fmc against Scouter, the read-only reference terminal app
    (github.com/indrazm/scouter), using Scouter's source and a live run of both apps through Rex. Use when
    checking a screen, key, view, or feature for parity, reviewing a port, or before claiming a phase-1 area is
    at parity. An optional screen or feature name narrows the scope.
metadata:
    argument-hint: "[screen | feature]"
---

# Parity audit

Find where fmc differs from Scouter, prove each difference from both apps, and report it in the ledger. Scouter is the reference (`.agents/docs/rules.md`, "Parity with Scouter"); the Rust contracts are `.agents/docs/rust.md`. A difference is a defect unless the owner approved it and `.agents/docs/architecture.md` records the approval.

The report shape, numbering, statuses, confirmation protocol, and fix phase are `deep-audit`'s: `.agents/skills/deep-audit/SKILL.md` ("Confirmation protocol", "Phase 5", "Phase 6") and `references/report-template.md`. Save to `.agents/docs/audit/audit-<nnnnn>-parity-<topic>.md`. The report file is English; chat follows the user's language.

## Sources

- **Scouter source**: clone `https://github.com/indrazm/scouter` into a fresh directory outside this repository (for example `/tmp/scouter-ref/<date>`). Its `docs/decision/` records (0001 to 0037) are the first map of its behavior. Its `src/` is organized as `app/`, `core/` (including `keymap.ts`), `explorer/`, `reader/`, `diff/`, `find/`, and `ui/`. Its `tests/` is a second source for expected behavior. Read it; do not install or run it outside the live run.
- **Scouter live run**: through the `rex` skill, in its own directory, started with the command its README gives. Record the command used.
- **fmc live run**: `cargo run` through the `run` skill or Rex, at the same terminal size as the Scouter run.

## Phase 0: Scope

Determine scope from the argument: a screen (Scouter's welcome, explorer, reader, diff, find, help), a feature (for example the Markdown preview or the git diff), or the whole app. A screen name that matches nothing: offer the closest candidates; never guess.

Write features are out of scope in phase 1 (`rules.md`, "Read-only until parity is closed"). Record a Scouter write feature as a deferred item, not as a finding.

## Phase 1: Inventory from Scouter

List every screen, view, mode, key, and state transition in scope, each with its `file:line` in Scouter and its decision record where one exists. Record the list in the report's scope section before running either app.

## Phase 2: Live runs

For each inventory item, capture the same state in both apps at the same terminal size: 80x24 and one larger size. Capture the terminal text through Rex and keep it as evidence. Capture a screenshot only when a color or a style is in question.

For each item, cover: the initial state, the effect of each key, the empty state, a long-content state, the error state, and a resize.

## Phase 3: Compare by dimension

| Dimension | What to compare |
| --- | --- |
| Layout | Pane positions, borders, sizes, the active-pane marker, header and footer rows, at each size |
| Text | Labels, messages, empty and error states, casing, truncation |
| Colour and style | Foreground and background roles, bold and dim, active and inactive panes, light and dark following the terminal (Scouter decision 0027) |
| Keys | The effect of each key, focus after it, count prefixes, the key list in the help screen |
| State | Selection, scroll position, expand and collapse, sort, jump history, search matches |
| Content | Markdown and Mermaid preview, code highlighting, binary and image views, diff layout, hunk navigation |
| Resize and exit | Behavior when the size shrinks, terminal state restored on exit |

For each difference record: the Scouter evidence (`file:line` and the captured output), the fmc evidence (`file:line` and the captured output), and its classification.

## Phase 4: Classify

- **Defect**: fmc differs and no approval exists. The fix is parity work.
- **Approved deviation**: the owner approved it and `architecture.md` records it. Not a finding; cite the record.
- **Scouter defect**: Scouter's behavior is clearly a bug (a crash, a broken layout at a size). Do not copy it silently: ask the owner whether to copy it or to fix it in fmc.
- **Unverified**: the Scouter behavior could not be established from its source or a live run. A hypothesis, not a finding.

## Phase 5: Report

Write the report with `deep-audit`'s template. Scope `type` is `feature` for a named screen or feature, `system` for the whole app. The baseline records the fmc gates from `shape.md` ("Commands") and the Scouter run command.

## Phase 6: Fix plan

Group fixes by screen. A group that touches more than one module becomes a plan (`agent-docs`). Fixes start only on the owner's approval.

## Rules

- Phases 0 to 5 change no production code. Adding a test is fine.
- Scouter's behavior is read from its source or a live run, never from memory. Cite it.
- Never write to the user's files during a live run: use a temporary directory with fixture files. Fixture names follow the sample-data rule (`rules.md`).
- Never commit the report or the captures unless asked.
