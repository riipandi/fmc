---
status: done
created: 2026-11-27T00:10:25+07:00
plan: plan-00002-sidebar-and-hidden-files
---

# Handover: sidebar and hidden files landed; plan-00002 closed

## Closed

- plan-00002 phases 1 to 5. Commits: `b071ec7` (deviation records and the key check), `55f9b45` (the sidebar: 80-column rule, `Ctrl+/` and `Ctrl+_`, the border mark), `81a735f` (hidden files: `Ctrl+.` and `.`), and the phase 4 closing commit. `f90486d` is outside the plan: it fixed a `rumdl` indent in an archived issue, which blocked `just verify` on clean HEAD.
- Owner decisions D7 (the welcome screen keeps the sidebar) and D8 (the phase 2 details) are applied; `architecture.md` carries the amendment.
- Final validation on `81a735f`: `just verify` (246 tests), `cargo build --release`, `cargo fmt --check`. Live runs on the demo repo at 80x24 and 70x24 are in the plan's phase 2 and 3 notes.
- RI-11 and RI-12 are `landed` in `roadmap-00001-post-parity.md`. Persistence waits for RI-9.

## Open

- Owner checks: Terminal.app and the kitty-protocol terminals with `Ctrl+/` and `Ctrl+.`, and a real terminal narrower than 80 columns. Rex can drive only its own terminal.
- Footer: at 70 columns `^/ sidebar` is cut, so the border mark is the only cue there. At 80 columns on a rendered Markdown file, the hint takes the place of `m source`.
- Not reproduced: once, in a tab resized to about 200 columns, the finder query showed `..gitignore` for a `.gitignore` search. A fresh 80x24 tab did not repeat it. Worth a look if it recurs.
- Decided by consequence, not by D1 to D8, and open to change: `Tab`, `Esc`, and `Ctrl+←/→` do nothing while the sidebar is hidden (D8 names these); an open finder holds the sidebar; the mark is drawn and clickable only where a document is open and no finder is open; `Ctrl+.` works inside a prompt and `.` does not, because a prompt takes `.` as text; the sidebar header swaps `files` for `no dotfiles` while dotfiles are hidden.
- The demo repo is unchanged: its `.gitignore` is the dotfile the live run needs.

## Next

- RI-9 (`~/.config/fmc/settings.json`): persist the sidebar's state and width, and the dotfile state. Each is a new persisted format, so it needs the owner's go-ahead first.
