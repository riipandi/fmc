---
status: in-progress
created: 2026-11-27T14:00:00+07:00
updated: 2026-11-27T14:00:00+07:00
phases:
    - id: 1
      title: "Key list duplicate and fold on release (P7)"
      status: done
      note: "Fold on release done (P7). Duplicate [ ] row held: it changes the 80x24 ? page, and the audit's open question 4 is unanswered (owner)."
    - id: 2
      title: "Header-row tap and the full-width drawer (P1, P3)"
      status: done
      note: "Header tap (P1) and the drawer below 60 (P3). Live at 40x30, 50x40, 59x30, 80x24 (80x24 unchanged). A tap on a file under the drawer opens it, as Enter does."
    - id: 3
      title: "Tappable footer hints, priority order, Copy hint (P2, P6, P8)"
      status: done
      note: "Taps press the hint key through the key path (D2); priority and Copy below 80 (P6, P8, D3). Live at 40x30, 50x40, 59x30, 80x24 (80x24 unchanged; taps off)."
    - id: 4
      title: "Name-first rows and the narrow welcome (P4, P5)"
      status: pending
      note: —
    - id: 5
      title: "Docs sync, ledger close, handover, final validation"
      status: pending
      note: —
---

## Make fmc usable on a phone: the audit-00002 proposals the owner approved

### Objective

The owner uses Rex on a phone. The phone audit (`audit/audit-00002-mobile.md`) proposed P1 to P8, and the owner approved all of them on 2026-11-27. They are recorded in `architecture.md` as "a phone layout below 80 columns" and "a diff header folds on release, not on press", and both are listed in `docs/parity.md`. After this plan, fmc can be used by touch at 40 to 79 columns. The layout at 80 columns and wider is unchanged, except for fold-on-release.

### Current state (2026-11-27)

- The audit's per-screen findings, with `file:line` evidence, are in `audit-00002-mobile.md`. The captures are in `/tmp/fmc-ref/out/mobile/`.
- `1121d51` already keeps `^/ sidebar` in the footer at phone widths.
- The `?` key list shows `[ ]` twice (`src/keys/mod.rs:102-103`). The agent decided to remove the duplicate.
- Rex block sizes are set with `rex block call resize`, not with `stty` (audit tooling note).

### Decisions

These details are the agent's picks within the owner's records. Change any of them on request.

- **D1: breakpoints.** The drawer applies below 60 columns. The narrow welcome applies below 50 columns. Every other phone rule applies below 80 columns. Each breakpoint is a named constant, and its tests run at 40, 59, 60, 79, and 80 columns.
- **D2: a footer hint tap runs the same `Action` the key runs.** The handler goes through the key table. It does not duplicate dispatch.
- **D3: `Copy` sits first in the footer** while a selection exists, so the fit never drops it.

### Execution rules

- One atomic commit per phase. Code commits carry `Scouter-Base: v0.3.4 (434d000f)`.
- `just verify` must pass before every commit. Never amend. Stage explicit paths. Do not push. No new dependencies. Keep files under 800 lines.
- After each phase, the captures at 80x24 must be unchanged. P7 is the one exception: it changes only timing.
- Live runs use new Rex tabs, resized with `rex block call resize`, at 40x30, 50x40, 59x30, and 80x24. Never touch block:01a2144a-042f-76ae-a6f7-ed8ce2790cd7. Close your tabs and reset the demo repo.

### Phases

1. **Key list duplicate and fold on release.** Remove the second `[ ]` row. A header folds on release, if the pointer has not moved. Add tests: a click folds, and a press-drag does not. _Commit: `fix(diff): fold a header on release and drop the duplicate bracket row`._
2. **Header-row tap and drawer.**
   - While the sidebar is hidden, a tap on the reader's header row shows the sidebar and focuses it.
   - Below 60 columns, the shown sidebar takes the full width.
   - Opening a file closes the drawer.
   - Below 60 columns, the welcome keeps the drawer open, so the list is visible (D7 of plan-00002).

   Add tests at the D1 widths. _Commit: `feat(layout): a header tap and a full-width drawer on narrow screens`._
3. **Footer.**
   - Below 80 columns, a tap on a hint runs its action. `Ctrl+q` is excluded.
   - Each screen keeps a fixed priority order for its hints.
   - While a selection exists, a `Copy` hint shows and does what `y` does.

   Add tests for the hit-testing and the fit order. _Commit: `feat(footer): tappable hints, priority order, and a copy hint on narrow screens`._
4. **Rows and welcome.**
   - Below 80 columns, finder rows and change-list rows show the name first and the folder after it, dimmed.
   - Below 50 columns, the welcome drops its card and wraps its lines.

   Add tests. _Commit: `feat(ui): name-first rows and a wrapped welcome on narrow screens`._
5. **Docs and validation.**
   - Update `docs/keymaps.md` (touch), `README.md`, `CHANGELOG.md`, and `rust.md` (breakpoints).
   - In the audit, give each proposal a verdict.
   - Archive this plan, update the index, and write the handover.
   - Run the full matrix.

   _Commit: `docs: close the phone layout plan`._

### Explicitly out of scope

- Scouter-parity clipping at 80 columns and wider: the header title, the binary notice, and the closing line at 80x24.
- Markdown table cell wrapping inside `opentui`.
- Image drawing on a phone. That is an owner check.
