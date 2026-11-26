---
status: done
created: 2026-11-26T16:00:00+07:00
updated: 2026-11-27T00:10:25+07:00
phases:
    - id: 1
      title: "Key check and deviation records"
      status: done
      note: "Rex probe at 80x24 (fmc on the demo repo): Ctrl+/ gives Char('/') with ctrl; Ctrl+. gives Char('.') with ctrl; . gives Char('.') with no modifiers; a raw 0x1F byte (rex send) gives Char('_') with ctrl. The toggle must match both '/' and '_' under ctrl."
    - id: 2
      title: "Collapsible sidebar: auto-hide, Ctrl+/, border mark"
      status: done
      note: "Rex at 80x24 and 70x24 on the demo repo: the welcome keeps the list with no mark, 80x24 README shows the mark at column 26, Ctrl+/ and the legacy 0x1F byte hide and show it, a Ctrl+/ in the finder does nothing, and at 70 columns README hides the list with the mark at column 0 while Tab does not reach it. The footer shows ^/ sidebar in place of Tab files: it fits at 80 columns, and at 70 it is cut, so the mark is the only cue there. 242 tests pass."
    - id: 3
      title: "Hidden files: Ctrl+. and . in the tree and the finder"
      status: done
      note: "Rex at 80x24 on the demo repo (.gitignore is the dotfile; no demo change needed): the sidebar shows the header badge no dotfiles and drops .gitignore when hidden; the selected dotfile falls to the first row; in the finder a .gitignore query finds it while shown, and Ctrl+. rebuilds the index so it finds nothing. The badge is the header's files badge swapped for no dotfiles, which fits beside a 24-column title. Tests cover the row filter, a dot folder's children, the selection fallback, and the finder index skip. 246 tests pass."
    - id: 4
      title: "Docs sync, ledger close, handover"
      status: done
      note: "Keymaps, README, CHANGELOG, DESIGN, shape, rust.md and the parity records synced; RI-11 and RI-12 landed in the roadmap with persistence left to RI-9; this plan archived; handover-00003 written and handover-00002 archived. The closing commit is the phase 4 commit."
    - id: 5
      title: "Final validation"
      status: done
      note: "Run on 81a735f before the closing commit: just verify (246 tests pass, licenses and format included), cargo build --release, cargo fmt --check. Live Rex runs at 80x24 and 70x24 are in the phase 2 and 3 notes. Owner checks are open: Terminal.app and the kitty-protocol terminals with Ctrl+/ and Ctrl+., and a real terminal narrower than 80 columns."
---

## Add a collapsible sidebar and a hidden-files toggle — roadmap RI-11 and RI-12

### Objective

The owner asked for an adaptive layout with a sidebar that hides and shows (RI-11) and a toggle for dotfiles (RI-12), both after parity (`roadmap-00001-post-parity.md`). After this plan, the sidebar hides on its own below 80 columns. `Ctrl+/` or a click on a mark on the border hides or shows it, and `Ctrl+.` or `.` hides or shows dotfiles in the tree and the finder. Nothing here is saved across runs: persistence waits for RI-9.

### Current state (verified in the tree, 2026-11-26)

- **Sidebar width.** `src/ui/sidebar.rs:13-16` `fit`: a quarter of the terminal within 28..=40, constrained by `constrain` (`:7-10`) to at least 24 columns and leaving at least 28 for the reader. There is no hidden state; the sidebar is always drawn.
- **Noise list.** `src/explorer/sort.rs:11` `NOISE` (7 names) is filtered in `src/services/fs.rs:140`. Dotfiles are otherwise listed, as in Scouter (`scouter:src/core/files.ts:34`).
- **Finder index.** `src/services/find.rs:21` `index(root, skip, limit)` takes a caller-supplied `skip`, so a hidden-files filter can ride on it.
- **Keys.** `.` is unbound (`src/keys/parser.rs`, grep for `"."` finds nothing). `Ctrl+/` and `Ctrl+.` are unbound. The locked `opentui` 0.9.9 enables the Kitty keyboard flags `DISAMBIGUATE | ALTERNATE_KEYS` by default (`opentui-0.9.9/src/host.rs:147`).
- **Size.** `src/app/mod.rs` is 951 lines, over the 800-line rule. New wiring goes in a sibling module (for example `src/app/layout.rs`), not into `mod.rs`.

### Decisions

All picked by the owner on 2026-11-26.

- **D1: the sidebar auto-hides below 80 columns.** At 80 or more columns the layout is Scouter's, so the 80x24 parity captures do not change. Below 80, the reader takes the full width. Rejected: below 60 columns, which leaves a cramped reader from 60 to 79. Rejected: below 100 columns, which would hide the sidebar at 80x24 and change every parity capture.
- **D2: a manual toggle wins until the width crosses 80 again.** `Ctrl+/` or the mark flips the current state. A resize that crosses the breakpoint resets to the automatic state, as RI-11 describes.
- **D3: the mouse target is a mark on the border.** When the sidebar is shown, the mark (`«`) sits on the sidebar panel's border. When it is hidden, a mark (`»`) sits on the reader's left border. Rejected: the sidebar title, which is hard to see and leaves nothing to click once the sidebar is hidden.
- **D4: `.` is the fallback for `Ctrl+.`.** `Ctrl+.` reaches fmc only from terminals with the Kitty keyboard protocol; Terminal.app cannot send it. `.` is free in the key table, and Yazi uses it for the same toggle. `Ctrl+/` needs no fallback: in the legacy encoding it arrives as `0x1F`, which fmc does not use for anything else (phase 1 confirms this).
- **D5: dotfiles are shown by default, and the noise list stays hidden in both states.** This keeps Scouter's default.
- **D6: no persistence in this plan.** The state resets on every start; RI-9 adds `settings.json`.
- **D7 (2026-11-27): the welcome screen keeps the sidebar.** While no document is open, the sidebar stays shown at any width; D1 applies once a document opens, and `q` brings the list back. Rejected: hiding it and rewording the welcome; hiding it with the welcome unchanged (its copy names a list that is not on screen).
- **D8 (2026-11-27): phase 2 details.** The mark sits on the top border row (`«` left of the sidebar's top-right corner, `»` at the reader's top-left corner). While hidden, the footer shows `^/ sidebar` and drops `Tab files`, and `Tab`, `Esc`, and `Ctrl+←`/`Ctrl+→` neither focus nor resize the sidebar. `Ctrl+_` is bound with `Ctrl+/` (legacy `0x1F`, phase 1 note).

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| Deviations | `architecture.md`, `docs/parity.md` | Collapsible sidebar (D1 to D3), hidden-files toggle (D4, D5), each with the Scouter baseline |
| Keys | `src/keys/`, `docs/keymaps.md`, the `?` key list, `--help` | `Ctrl+/`, `Ctrl+.`, `.`, and the border-mark click |
| Breakpoint and key traps | `rust.md` | The 80-column rule, and how each key arrives (Kitty versus legacy) |
| Roadmap | `roadmap-00001-post-parity.md` | RI-11 and RI-12 marked done, with persistence left to RI-9 |

### Execution rules

- **Atomic commit per phase**, with the message each phase names and the trailer `Scouter-Base: v0.3.4 (434d000f)` on code commits. The owner's go-ahead on this plan authorizes its phases' commits.
- **No new dependency.** Read-only stays in force.
- **Files under 800 lines.** Do not grow `src/app/mod.rs`; put new code in sibling modules.
- **Live runs** happen in new Rex tabs. Resize a tab with `stty` where possible; where Rex cannot change the width, cover the breakpoint with unit tests and leave an owner check.
- **Never amend; stage explicit paths; no push without the owner's word.**

### Phases

1. **Key check and deviation records.** Run fmc in a new Rex tab and log the `KeyEvent` that opentui reports for `Ctrl+/`, `Ctrl+.`, and `.`. Rex is a Kitty-capable terminal. Read opentui's legacy parser for `0x1F`. Write the two deviation records in `architecture.md` and `docs/parity.md`. Validation: the logged events are recorded in the phase note. _Commit: `docs(parity): record the collapsible sidebar and the hidden-files toggle`._
2. **Collapsible sidebar.** Add a pure layout state in core (auto or manual, shown or hidden, the 80-column breakpoint) with unit tests: auto-hide at 79 and show at 80; a manual toggle holds; crossing the breakpoint resets to auto.
   - `Ctrl+/` toggles from every screen except prompts. In a prompt, the key is not typed as text.
   - Draw the border mark on the sidebar's border when it is shown and on the reader's left border when it is hidden; a click toggles.
   - Focus moves to the reader when the focused sidebar hides.
   - Update the footer hints and the key list.

   Validation: unit tests; live at 80x24 with no change from the current captures; a narrower tab if Rex allows. _Commit: `feat(layout): hide and show the sidebar`._
3. **Hidden files.** `Ctrl+.` and `.` toggle dotfiles in the tree and the finder index; the noise list stays hidden in both states.
   - The selection falls back as it does for a removed row.
   - The finder rebuilds its index on a toggle.
   - Add a footer badge, or a hint in the sidebar title, while dotfiles are hidden. Pick the smallest form that fits 80 columns, and show it in the phase note.

   Validation: unit tests for the filter (a dotfile, a dot folder, a noise name in each state) and the selection fallback; a live run on the demo repo, which needs a dotfile, so add one to `scripts/demo-repo.sh` if there is none. _Commit: `feat(explorer): toggle hidden files`._
4. **Docs sync, ledger close, handover.** Sync `docs/keymaps.md`, `README.md`, `CHANGELOG.md`, `rust.md`, and the roadmap (RI-11 and RI-12 done, persistence left to RI-9); archive this plan; update the index rows. Validation: every key in `docs/keymaps.md` is in the key table. _Commit: `docs: close the sidebar and hidden-files plan`._
5. **Final validation.** Run the matrix below over the tree. _Commit: none if green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check` | No drift |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Unit and full suite | `just verify` | All green, new layout and filter tests included |
| Build | `cargo build --release` | Builds |
| Live run | Rex, 80x24 | Parity captures unchanged; toggles visible |
| Owner check | A real terminal narrower than 80 columns; Terminal.app with `.` | Auto-hide and the fallback work |

### Explicitly out of scope

- **Persistence** of the sidebar state, its width, and the hidden-files state: RI-9.
- **A configurable breakpoint or configurable keys**: RI-9 and RI-13.
- **Other layout changes**, such as a stacked layout for very narrow terminals: not asked for.
