---
status: done
created: 2026-11-26T12:30:53+07:00
plan: plan-00001-scouter-parity
wave: W0–W6 done
---

# Handover: plan-00001 closed; parity audit, fixes, and owner decisions landed

## What this session did

Ran plan-00001 phase 22 (the parity audit) and phase 23 (the audit's fixes), then the owner's decisions, then phases 24 and 25. The audit is `audit/audit-00001-scouter-parity.md` (completed): 80x24 live runs in Rex against Scouter 0.9.9 for every §3 screen and §4 key, eight findings. Fixed in phase 23 (F-01 to F-05, and F-06 on the owner's instruction, sub-phase 23f); the owner decided F-07 (binary footer, issue-00002) and F-08 (no `--upgrade`, issue-00010) on 2026-11-26 and recorded both as approved deviations in `architecture.md` and `docs/parity.md` (commit `6cb4373`).

Ledger moves: issue-00002, 00003, 00004, 00005, 00007, 00008, 00009, and 00010 are in `archive/` (resolved); issue-00006 (scrollbar feel) stays open for the owner check in `issues/`; plan-00001 is in `archive/` (done). The plan's phase 23 sub-phases are 23a to 23f in its body.

The fix for F-06 (issue-00009) needed no fallback. The folded header's bar was clipped by the stack's width: Scouter's stack is as wide as its widest visible block, so an unfolded diff (the pane plus its gutter) makes every header that wide, and a folded block is pane-wide (`ui/diff.rs` `apply_widths`, `diff/stack.rs` `widens_stack`). Live captures match Scouter in each fold state.

## Commits (this slice)

- `b409a76` docs(audit): record the scouter parity audit (phase 22).
- `498958c` fix(reader): show the welcome in full at 80x24 (23a, F-01, issue-00004).
- `784caf4` fix(reader): draw the rendered Markdown at Scouter's width (23b, F-02, issue-00005).
- `d9bd454` fix(reader): count the rendered Markdown rows from its height (23c, F-03, issue-00003).
- `e3d6c73` fix(explorer): keep the tree's wheel scroll across repaints (23d, F-04, issue-00007).
- `10ced9a` fix(reader): pan unwrapped text as far as Scouter does (23e, F-05, issue-00008).
- `2f75167` docs(issues): file the parity audit's owner questions.
- `78cad71` fix(changes): size each stacked header as Scouter's stack does (23f, F-06, issue-00009).
- `docs: close the scouter parity plan` (phase 24, this handover's commit).

Owner decisions: `6cb4373` (before this slice) records F-07 and F-08 in `architecture.md` and `docs/parity.md`.

## What stayed open / the trap

- **issue-00006, the scrollbar's feel**: open for the owner check. Keys and wheel match Scouter in Rex; no cause found.
- **Owner checks** (Rex cannot inject mouse events or draw pixels, and cannot make a tab larger than 80x24): see the list below.
- **Traps**:
  - `block_rows` on a Markdown widget are screen rows, so their last end moves up as the view scrolls; the total is the widget's own height (`ui/markdown.rs` `rows`).
  - A visible child takes a share of its stack. The reader's viewer, shown while the welcome is up, halved the stack and clipped the welcome (`rust.md`).
  - The stack's width is the widest visible unfolded diff, not the pane (`rust.md`, `widens_stack`).
  - `rex send-key` sends `J` and `G` as lowercase; use `shift+j` and `shift+g`. `?` is a text key: `rex send -b BLOCK '?'`.
  - A finder Enter once opened the wrong file in Scouter after keys sent 0.4 s after `/`; three reruns did not repeat it. Wait about a second after `/` before the typed text.
  - Sandbox shell: `rm` can be denied; use `git rm` as a separate command.
  - `cargo fmt` must run before `just verify`, or the format gate fails on new code.
- `handover-00001-parity-w0-w3.md` is `partial` and older; its open threads (W4 next, owner checks for W2 and W3) are now done or listed here.

### Owner-check list

1. **Scrollbar feel (issue-00006)**: in the same terminal, run fmc and Scouter 0.9.9 on one long file (over 60 lines, for example a demo file). Wheel one notch at a time, then a fast trackpad flick, over the reader and over a long folder. Say whether fmc stutters more, and whether the thumb or the text lags. Record the terminal, its font, and whether `▐ ▝ ▗` render as half cells.
2. **Mouse, not injectable in Rex**: click a tree row (opens or toggles); click a changes row (selects, focuses the diff); click a diff header (folds); drag the sidebar edge (resizes, `col-resize` cursor); drag across text (selects, `y` copies, `Esc` clears).
3. **Wheel in the changes list and diff body**: one row per notch in both directions (the audit saw it only through Rex SGR reports).
4. **The image** (`assets/marauder-map.png`): the picture shows in a Kitty-capable terminal (Rex draws no pixels).
5. **Split view** at 60 or more columns (`s` on the changes screen): the two columns line up with Scouter's.
6. **160x48 live run** (the plan's second size): run fmc and Scouter in a terminal of that size on the demo repository with the same keys. Rex cannot open it.

## Suggested for the next session

- `run` for any live check, `parity-audit` before claiming a screen at parity, `agent-docs` for ledger moves, `rust-verify-harden` before a Rust change is called done.
- The next plan (post-parity, `roadmap/roadmap-00001-post-parity.md`) starts only on the owner's go-ahead.
