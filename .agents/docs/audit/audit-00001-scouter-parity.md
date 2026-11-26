---
title: "Audit: Scouter parity, whole system (plan-00001 phase 22)"
status: completed
created_at: 2026-11-26T12:01:10+07:00
updated_at: 2026-11-26T12:29:18+07:00
completed_at: 2026-11-26T12:29:18+07:00
scope:
    type: system
    feature: null
    paths: [src/ui, src/app, src/keys, src/explorer, src/reader, src/find, src/diff, src/services, src/args.rs]
    exclusions: [mouse click, drag, and drag-copy (Rex injects no mouse events except the SGR wheel); 160x48 (Rex tabs stay at 80x24); Kitty image pixels; split view at 60+ columns]
stack:
    languages: [rust]
    frameworks: [opentui 0.9.9 (opentui-sys =0.9.9, tree-sitter feature)]
    datastores: []
repo:
    branch: "main"
    commit: "d826786"
    working_tree_clean: true
environment:
    executed: [just verify (216 tests, licenses), cargo build, Scouter 0.9.9 and fmc live in Rex at 80x24 on the demo repository and fixtures, Scouter source at 434d000, Scouter --help diff]
    not_run: [mouse click and drag (no injection in Rex), 160x48 captures (Rex tabs are 80x24 only), Kitty image rendering (owner check), split view at 60+ columns]
baseline:
    build: pass
    tests: pass
    lint: not_run
    dependency_scan: pass
    secret_scan: not_run
    race_detector: not_available
findings:
    total: 8
    by_severity: { critical: 0, high: 0, medium: 5, low: 3, info: 0 }
    by_status:
        { open: 0, fixed: 6, deferred: 0, wont_fix: 2, false_positive: 0 }
    verified: 8
    unverified: 0
cleanup_candidates: 0
optimization_opportunities: 0
open_questions: 3
tests_added: 0
standards: []
generated_by: parity-audit
---

## Audit: Scouter parity, whole system (plan-00001 phase 22)

### Executive summary

fmc matches Scouter 0.9.9 cell for cell on most states: the code reader, Markdown source, the finder and its text search, in-file search, tree keys and sidebar resize, the changes list, hunk keys, one-file mode, split refusal, the error notices, and the file-open, quit, and help paths. Each remaining difference is either an approved deviation (`architecture.md`) or one of eight findings below. Five are defects to fix toward Scouter in phase 23 (issues 00003, 00004, 00005, 00007, 00008, with the scrollbar part of 00003 and the wrap width of 00005 recorded here as one cause each). Three need the owner before any code changes (F-06, F-07, F-08).

Findings per severity (the frontmatter matches): medium 5 (F-01 to F-04, F-06), low 3 (F-05, F-07, F-08). None is critical or high.

The three things to fix first:

1. F-01: the welcome screen at 80x24 loses its wordmark and the hint card's last row and closing line (issue-00004). It is the first screen a user sees.
2. F-04: the tree does not scroll with the mouse wheel (issue-00007); each notch is undone by the next frame.
3. F-02 and F-03: the rendered Markdown is two cells narrower than Scouter's, and its scrollbar thumb and footer percentage are wrong (issue-00005 and the open part of issue-00003). Both are in the reader's most-read view.

### Scope and flow inventory

Flows audited: welcome, open a file from the finder and from the tree, read text, Markdown rendered and source, image, binary, in-file search, the finder with name and text modes, the changes screen (list, stacked body, one-file, fold, context, hunk keys), the key list (`?`), `--help`, the error notices (not UTF-8, symbolic-link loop, not a git repository, file no longer on disk), quit.

Trust boundaries: none changed; the audit is read-only (fixtures outside the repository, the demo repository rebuilt by `scripts/demo-repo.sh`).

Assumptions: Scouter 0.9.9 (`/tmp/fmc-ref/scouter-099`, the 434d000 source with `@opentui/core` 0.9.9, the parity reference of `architecture.md`) is the live reference; its source is `/tmp/fmc-ref/scouter` at 434d000 (`Release 0.3.4`), and every `scouter:` citation below is checked against that tree.

Evidence: captures are in `/tmp/fmc-ref/out/w6-22/` (outside the repository): `fmc/` and `sco/` hold text and HTML captures with the same numbers, for example `fmc/10-readme-rendered-bottom.txt` and `sco/10-readme-rendered-bottom.txt`. Each step's keys are in the report under its finding.

### Baseline

- Commit `d826786` (main, clean tree). `just verify`: 216 tests pass, `licenses ok`, exit 0 (`/tmp/fmc-ref/out/w6-22-verify-base.txt`). `cargo build` passes.
- Scouter run: `cd /tmp/fmc-ref/scouter-099 && bun --config=/dev/null --no-env-file src/index.ts <dir>` in a Rex tab at 80x24 (`stty cols 80 rows 24`).
- fmc run: `/Users/ariss/Developer/github.com/riipandi/fmc/target/debug/fmc <dir>` in a Rex tab at 80x24.
- Demo repository: `scripts/demo-repo.sh /tmp/fmc-ref/demo-fmc` (rebuilt for this audit, `git status` as in the script's marker).

### Verdicts by screen (`research/scouter-inventory.md` §3)

| # | Screen | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Welcome | Defect F-01 (issue-00004); name, version, `^q` label approved | `01-welcome`, `welcome-fmc.txt`, `welcome-sco.txt` |
| 2 | Browse (tree + reader) | Match, apart from the approved root row, reveal, and focus | `02-readme-open`, `34-tree-main`, sweep `tree-*` |
| 3 | Reader: text | Match (text and footer); pan is F-05 | `04-finder-main`, `40`–`49` (`main-*`) |
| 4 | Reader: Markdown rendered | Defects F-02 (width) and F-03 (thumb, percent) | `02`, `07`, `10`, `36`, `37` |
| 5 | Reader: Markdown source | Match | `08-readme-source`, `09-readme-source-bottom` |
| 6 | Reader: image | Match in footer (`64×32 · png · All`); pixels not drawable in Rex (owner check) | `11-png-finder` |
| 7 | Reader: binary notice | Text matches; footer position is F-07 (issue-00002) | `12-png-then-binary` |
| 8 | Reader: error | Match: not UTF-8, symbolic-link loop, not a git repository, no longer on disk | `26-latin1-open`, `28-loop-up-fmc`, `29-not-a-repo`, `31-owl-deleted` |
| 9 | Find file | Match (prompt, match count, rows, Esc) | `03-finder-empty`, `05-finder-ledger`, `04` |
| 10 | Find text and preview | Match (rows, status, preview, Enter opens the file) | `06-finder-ledger-tab`; phase 17 note |
| 11 | In-file search | Match (footer, count, Enter); colours per phase 15 note | `14-src-search` |
| 12 | Changes | Match, apart from the approved footer (`a one file` dropped by `^q`) and F-06 (folded bar) | `15-changes-d`, `16-changes-n`, `18-changes-a` |
| 13 | Changes, split | Match: refused below 60 columns (`Split needs 60+ columns.`); the split view itself needs an owner check | `17-changes-split` |
| 14 | Changes, empty | Not re-run here; phase 19 note records clean and unborn repositories as matching | phase 19 note |
| 15 | Help overlay | Approved deviation: the key list is a Keys page. Its key content matches `src/core/keymap.ts` except approved rows (`--help` diff) | `23-help`, `24-help-bottom`, `help-sco.txt` vs `help-fmc.txt` |
| 16 | Empty reader | Defect F-01 (the welcome it shows) | `01-welcome` |

### Verdicts by key (`research/scouter-inventory.md` §4)

§4.1 global keys:

| Key | Verdict | Evidence |
| --- | --- | --- |
| `q` | Approved: closes the document (`architecture.md`) | `--help` diff; `docs/keymaps.md` |
| `Ctrl+c` | Match: both apps quit | live check, both tabs exit to the shell |
| `?` | Approved: the Keys page (`architecture.md`) | `23-help` |
| `Tab` | Match: focus moves between panes | phases 8, 10, 19 notes; `33`/`34` runs |
| `Esc` | Match: keeps then clears a kept search; focus to list | phases 15, 17 notes |
| `Ctrl+←` / `Ctrl+→` | Match: divider 28, 30, 32, then 30 in both | sweep `tree-*` (ctrl+right, ctrl+left) |
| `Ctrl+o` / `Alt+i` | Match (jump history of 100) | phase 14 note |
| `w` | Match: wrap toggles; footer `Top`/`Bot` as Scouter | `40`–`49` |
| `m` | Match: rendered and source | `08`, `09`, `10` |
| `d` | Match: enter and leave the changes screen | `15`, `29`, `50` |
| `y` | Match in the footer; the copied-path notice is not visible in either capture after one second (not decided) | `51-yank-path`; phase 14 note |

§4.2 sidebar tree:

| Key | Verdict | Evidence |
| --- | --- | --- |
| `j` `k` `↓` `↑` | Match | sweep `tree-*`; `27`, `28` |
| `J` `K` | Match: five rows in both | sweep `tree-*` (shift+j, shift+k) |
| `PageDown` `PageUp` | Match: to the last and first row | sweep `tree-*` |
| `g` `gg` `Home` | Match: top | sweep `tree-*` (g, g, Home) |
| `G` `End` | Match: last row | sweep `tree-*` (shift+g, End) |
| `Enter` | Match for files and folders; the approved focus rule on files | `02`, `28` |
| `l` `→` | Match (phase 10 note) | phase 10 note |
| `h` `←` `Backspace` | Match: Backspace collapses the root; top-level rows do nothing (approved root row) | sweep `up-*` (`-`, Backspace) |
| `-` | Match: the parent becomes the root | sweep `up-*` |
| `/` | Match | `03`, `05` |

§4.3 reader:

| Key | Verdict | Evidence |
| --- | --- | --- |
| `j` `k` `↓` `↑` | Match (one line) | phase 14 note |
| `h` `l` `←` `→` | Defect F-05 (pan ends one column short) | `34-tree-main` |
| `Space` `PageDown` `b` `PageUp` | Match | `41`, `42` |
| `Ctrl+d` `Ctrl+u` | Match | `43`, `44` |
| `g` `gg` `Home` | Match | `46`, `47` |
| `G` `End` | Match | `45`, `49` |
| `0`–`9` count (`120G`) | Match (phase 14 note: 120G, 40G) | phase 14 note |
| `{` `}` | Match (phase 14 note) | phase 14 note |
| `z` `z` | Match (phase 14 note) | phase 14 note |
| `/` | Match in text; refused in rendered Markdown with Scouter's footer text | `14`, `13` |
| `n` `N` `p` | Match (phase 15 note) | phase 15 note; `14` |
| `m` `w` `y` `d` `Tab` `Esc` | As global | §4.1 |

§4.4 changes screen:

| Key | Verdict | Evidence |
| --- | --- | --- |
| `s` | Match: refused below 60 columns | `17` |
| `a` | Match, with the approved landing of the selected file at the top | `18` |
| `c` | Defect F-06 (folded header bar) | `19`, `21`, `22` |
| `x` | Match, with the approved landing | `20` |
| `r` | Match (phase 21 note) | phase 21 note |
| `o` | Match: opens the file at the hunk, footer `Bot` as Scouter | `50` |
| `n` `]` `N` `p` `[` | Match: next and previous hunk (phase 20 note) | `16`; phase 20 note |
| `d` | Match | `15` |
| Changes list keys (`j` `k` `J` `K` `g` `G` `Enter` `l` `→`) | Match (phase 19 note) | phase 19 note |
| Diff body keys (`j` `k` `h` `l` `Ctrl+d` `Ctrl+u` `Space` `b` `w`) | Match (phases 19 and 20 notes) | phases 19, 20 notes |

§4.5 prompts:

| Prompt | Verdict | Evidence |
| --- | --- | --- |
| Finder: typed text, Backspace | Match (typed text); Backspace in phase 16 note | `05` |
| Finder: `Tab` | Match | `06` |
| Finder: `↑` `↓` in text mode (preview) | Match (phase 17 note) | phase 17 note |
| Finder: `Enter` | Match: opens at the line in text mode; focus to the reader | `04`, `35b` |
| Finder: `Esc` | Match (phase 16 note) | phase 16 note |
| In-file search: `Enter` `↓` `Shift+Enter` `↑` `Esc` | Match (phase 15 note) | phase 15 note |

§4.6 mouse: the wheel is injected (`rex send` with SGR reports). The tree wheel is F-04; the reader wheel matches Scouter's thumb at the top (`37`). Click, drag, sidebar edge drag, and header click are not run here; they are owner checks (below).

§4.7 help card versus key handling: the Keys page is generated from one table; its content equals Scouter's key table except the approved rows.

### Findings

| ID | Severity | Category | Location | Title | Evidence | Status |
| --- | --- | --- | --- | --- | --- | --- |
| F-01 | medium | layout | `src/ui/welcome.rs`, viewer stack in `src/app/mod.rs` | Welcome clipped at 80x24 | verified | fixed (498958c) |
| F-02 | medium | layout | `src/app/mod.rs` (bar), `src/ui/markdown.rs` | Rendered Markdown two cells narrower than Scouter | verified | fixed (784caf4) |
| F-03 | medium | scroll state | `src/ui/markdown.rs` (`rows`) | Rendered Markdown thumb and percentage wrong | verified | fixed (d9bd454) |
| F-04 | medium | input | `src/explorer/browser.rs` (`set_height`, `keep_visible`) | Tree wheel scrolls back | verified | fixed (e3d6c73) |
| F-05 | low | layout | `src/app/mod.rs` (unwrapped width) | Reader pan ends one column short | verified | fixed (23e) |
| F-06 | medium | chrome | `src/ui/diff.rs` (`apply_widths`), `src/diff/stack.rs` (`widens_stack`) | Folded diff header draws no stat bar | verified | fixed (phase 23f) |
| F-07 | low | chrome | `src/app/reader.rs` (`footer_facts`) | Scouter's stale binary footer position | verified | wont_fix (approved deviation, 6cb4373; owner, 2026-11-26) |
| F-08 | low | docs | `docs/parity.md`, `.agents/docs/architecture.md` | `--upgrade` drop is not recorded as a deviation | verified | wont_fix (approved deviation, 6cb4373; owner, 2026-11-26) |

#### F-01 Welcome clipped at 80x24

- **Category**: layout
- **Location**: `src/ui/welcome.rs`; the viewer stack in `src/app/mod.rs`
- **Problem**: the welcome content is taller than the reader's inner height. fmc draws the tagline at row 13 with the wordmark rows blank, and the hint card loses its `q` row; the closing line `Press ? for every key.` is missing.
- **Scenario**: start `fmc` on the demo repository at 80x24, no file open.
- **Impact**: the first screen has no app name and no closing hint.
- **Evidence**: `welcome-fmc.txt` against `welcome-sco.txt` (verified, captures in `/tmp/fmc-ref/out/w6-22/`). Scouter: `src/ui/welcome.ts:7-87`, `:53`, `:78`. Same as issue-00004.
- **Fix**: measure the welcome's rows against the reader's inner height at 80x24, then reduce the margins or the wordmark's offset so all rows fit (issue-00004).
- **Confidence**: high
- **Status**: fixed in plan-00001 phase 23 (see the commit named in the findings table).

#### F-02 Rendered Markdown two cells narrower than Scouter

- **Category**: layout
- **Location**: `src/ui/viewer.rs` (the scrolling content is `width 100%`), `src/ui/markdown.rs` (`padding_x(2)`)
- **Problem**: in the reader, fmc's rendered Markdown is at most 43 cells wide; Scouter's is at least 45. The paragraph `A small ledger and symbology toolkit for the` is 44 cells: Scouter keeps `the` on that line, fmc wraps it. The `## Modules` table is 43 cells in fmc (Language 9, Owner 17) and 45 in Scouter (Language 10, Owner 18), which is issue-00005. The horizontal rule is 43 cells in fmc and 45 in Scouter. The Keys page's tables wrap a row earlier for the same reason (`scroll the pane under the / pointer`).
- **Scenario**: open `README.md` from the finder at 80x24 (`/`, `README.md`, Enter); the rendered view wraps line 3 before `the`.
- **Impact**: every rendered paragraph, table, and rule in the reader is narrower; the text is still correct.
- **Evidence**: `02-readme-open` and `07-readme-finder` (fmc, sco); `10-readme-rendered-bottom` (rule, 43 against 45). The find preview of the same file (`06-finder-ledger-tab`) wraps as Scouter's, so the preview width is right and the reader's width is not.
- **Fix**: find where the reader's Markdown loses two cells (the scrolling content, the Markdown padding, or the scrollbar reserve) and make the reader's Markdown width match the preview's. The cause is not established.
- **Confidence**: high on the difference; medium on the cause.
- **Status**: open; fix in phase 23 (issue-00005 records the table part).

#### F-03 Rendered Markdown thumb and percentage wrong

- **Category**: scroll state
- **Location**: `src/ui/scrollbar.rs`, `src/ui/markdown.rs` (`block_rows`, read by the bar and the footer)
- **Problem**: at the end of the rendered `README.md` (`G`), the thumb in fmc covers 16 rows (rows 6 to 21); Scouter's covers 4 rows (18 to 21). After one page down, the footer reads `32%` in fmc and `25%` in Scouter, from the same top line. At the top the thumbs agree (`37`).
- **Scenario**: open `README.md`, press `G` (`10`), or `Space` (`36`).
- **Impact**: the thumb and the percentage tell the user the wrong place in the rendered document. The text itself is not wrong.
- **Evidence**: `10-readme-rendered-bottom` and `36-readme-mid` (fmc, sco); the thumb column read from col 78 of each capture. The likely cause is `rust.md` "Rendered Markdown has rows only after a frame": the row count is read before every block has rows. Not confirmed.
- **Fix**: count the rendered rows after the document is laid out (or from the layout that the scroll range uses), and test the thumb at the top, middle, and end.
- **Confidence**: high on the difference; medium on the cause.
- **Status**: open; fix in phase 23 (the thumb and percentage part of issue-00003). The text end of issue-00003 does not reproduce at HEAD in this run.

#### F-04 Tree wheel scrolls back

- **Category**: input
- **Location**: `src/explorer/browser.rs` (`set_height`, `keep_visible`), `src/app/mod.rs` (`wire_mouse`)
- **Problem**: `set_height` brings the scroll back to the selected row on each paint, so a wheel notch is undone by the next frame.
- **Scenario**: `hogwarts-30` (30 files) at 80x24; three wheel-down reports (`ESC [ < 65 ; 10 ; 8 M`) (`32-tree-wheel-3down`).
- **Impact**: the mouse does nothing in the tree.
- **Evidence**: fmc keeps `hogwarts-01` at the top; Scouter moves to `hogwarts-03` at the top, one row per notch. The thumb size is equal (11 rows at col 26) in both. Issue-00007 has the same captures from HEAD.
- **Fix**: make `set_height` only clamp the scroll, as the change list does (`src/diff/list.rs`), with a test that a repaint keeps a wheel scroll (issue-00007).
- **Confidence**: high
- **Status**: fixed in plan-00001 phase 23 (see the commit named in the findings table).

#### F-05 Reader pan ends one column short

- **Category**: layout
- **Location**: `src/ui/reader.rs` (`set_width`), `src/ui/viewer.rs` (`set_wrap`)
- **Problem**: after four `l` presses on `src/main.rs` at 80x24, Scouter's first code column is at the pane's left edge; fmc's is one cell right.
- **Scenario**: open `src/main.rs`, press `l` four times (`34-tree-main` after the same keys).
- **Impact**: a small offset at the end of a pan; the text is complete.
- **Evidence**: fmc `│ use std::collections::`, Scouter `│use std::collections::`. Same as issue-00008.
- **Fix**: measure the widest line and the pane's inner width against Scouter's `ScrollBox` content width, then correct the content width by the difference (issue-00008).
- **Confidence**: high
- **Status**: fixed in plan-00001 phase 23 (see the commit named in the findings table).

#### F-06 Folded diff header draws no stat bar (owner question)

- **Category**: chrome
- **Location**: `src/diff/row.rs` (`stat_bar`, `header_counts`), `src/ui/diff.rs` (`paint_headers`)
- **Problem**: Scouter's folded header of `README.md` shows `+4  ▪▪▪▪` (four bar cells, the added colour); fmc's folded header shows `+4` and no bar. Scouter's unfolded header shows no bar, before and after a scroll; fmc shows no bar in either state. Scouter's source draws the bar on every header (`src/diff/diff-blocks.ts:217`, `:299`, `:491`; `statBarSegments` at `:72-96`), so the unfolded absence is not explained by the source.
- **Scenario**: `d`, `c` on the first file (`19-changes-c-fold`, Scouter `+4  ▪▪▪▪`). `c` again, then `j`/`k` (`52`, `53`): no bar in Scouter.
- **Impact**: the folded header shows fewer counts than Scouter's.
- **Evidence**: `19-changes-c-fold.html` (Scouter: four `&#9642;` cells in `rgb(143, 211, 168)`; fmc: none), `15-changes-d`, `52`, `53`. The fmc bar is in `header_counts` (`src/diff/row.rs:78-82`), so the bar is lost between the segments and the screen; the cause is not established.
- **Why a question**: the owner's standing rule covers chrome that lags a layout change until the next scroll. Here the bar does not appear after a scroll in the unfolded header, so it is not a lag that settles. Matching the folded state while leaving the unfolded one bare needs a cause, and the source says the bar belongs on every header.
- **Options**:
  1. Match Scouter's visible states exactly: the bar only on folded headers. Needs the cause in Scouter first.
  2. Show the bar on every header, as Scouter's source draws it. Differs from Scouter's unfolded header.
  3. Show no bar anywhere. Differs from Scouter's folded header.
- **Recommendation**: option 1 (parity first), with option 2 if Scouter's cause cannot be found within phase 23.
- **Confidence**: high on the difference; low on the cause.
- **Status**: open; owner question. Not changed in phase 23 until the owner picks.

#### F-07 Scouter's stale binary footer position (owner question, issue-00002)

- **Category**: chrome
- **Location**: `src/app/reader.rs` (`footer_facts`)
- **Problem**: after the PNG, Scouter opens `assets/seal.bin` and its footer keeps the image's `All`: `4.0 KB · binary · All`. fmc's footer reads `4.0 KB · binary` with no position.
- **Scenario**: open `assets/marauder-map.png`, then `assets/seal.bin` from the finder (`12-png-then-binary`).
- **Impact**: none in fmc. Scouter shows a position for a view that has none.
- **Evidence**: `12-png-then-binary.txt` (both, footer row 23). Scouter source: `src/app/app.ts:411-419` (`showBinary` does not update the position), as issue-00002 records.
- **Options**:
  1. Keep fmc's footer with no position, and approve it as a deviation (an `architecture.md` record and a `docs/parity.md` line). Recommended: the stale `All` describes an image that is no longer shown.
  2. Copy Scouter's stale value. Matches the screen exactly but repeats a stale value on purpose.
  3. Other: a design you want.
- **Confidence**: high
- **Status**: open; owner question. The clean footer stays until the owner picks.

#### F-08 `--upgrade` drop is not recorded (owner question)

- **Category**: docs
- **Location**: `docs/parity.md`, `.agents/docs/architecture.md`
- **Problem**: Scouter's `--help` lists `scouter --upgrade` (`scouter:src/index.ts:11`, `:26-28`); fmc's `--help` omits it. The plan's "Explicitly out of scope" names it, but neither `architecture.md` nor `docs/parity.md` records it, so under this audit's rule it is an unrecorded difference.
- **Scenario**: `fmc --help` against `scouter --help` (`help-sco.txt`, `help-fmc.txt`). Run outside a terminal, `fmc --upgrade` prints `Run fmc in an interactive terminal.` (exit 1); `scouter --upgrade` prints its installed-command message (exit 1).
- **Impact**: docs only; the drop is intended.
- **Options**:
  1. Record it as an approved deviation (an `architecture.md` record citing plan-00001's out-of-scope list, and a `docs/parity.md` line). Recommended: the plan already excludes it, and fmc ships as one binary.
  2. Implement `--upgrade`. Needs Scouter's installer, which the plan excludes.
  3. Other.
- **Confidence**: high
- **Status**: open; owner question.

### Optimization opportunities

None recorded by this audit.

### Cleanup candidates

None recorded by this audit.

### Decisions log

- 2026-11-26: plan-00001 phase 22 go-ahead (owner). Phase 23 fixes F-01 to F-05 (fixed, sub-phases 23a to 23e); F-06 to F-08 wait for the owner.

### Coverage and limitations

Covered live at 80x24 against Scouter 0.9.9: §3 rows 1 to 16, §4.1 to §4.5 (rows marked "phase N note" were checked live in that phase, not re-run here), §4.6 wheel only.

Not covered: mouse click, drag, sidebar edge drag, header click (owner checks); 160x48 (Rex tabs stay 80x24; a 190x61 check in phase 8 was by eye); Kitty pixels; the split view at 60+ columns; the empty changes state (phase 19 note).

Not reproduced: a finder Enter in one Scouter run (keys sent right after `/`) opened `assets/marauder-map.png` instead of the highlighted `src/main.rs`. Three later runs with the same keys, with and without a pause, opened `src/main.rs` in both apps. Recorded as a timing flake in the harness, not a finding.

Not checked against Scouter: the copied-path notice after `y` (not visible after one second in either app).

### Owner checks

- Issue-00006 (scrollbar feel): the owner checks by hand (`issue-00006-scrollbar-not-smooth.md`). This audit's thumb captures agree with Scouter in the tree and the reader at the top (`32`, `37`); F-03 is a separate rendered-view defect.
- Mouse: click, drag-select, drag-copy, sidebar edge drag, header fold (§4.6).
- Image: the PNG in a Kitty-capable terminal.
- Split view at 60+ columns.

### Open questions

1. F-06: the folded and unfolded header bar. Options and recommendation under F-06.
2. F-07: the binary footer (issue-00002). Options and recommendation under F-07.
3. F-08: the `--upgrade` drop. Options and recommendation under F-08.

### Change log

- 2026-11-26T12:01:10+07:00: status draft -> in_review; eight findings (five defects for phase 23, three owner questions); live runs at 80x24 on d826786.
- 2026-11-26T12:18:05+07:00: status in_review -> in_review; F-01 to F-05 fixed in plan-00001 phase 23 (498958c, 784caf4, d9bd454, e3d6c73, and 23e); F-06 to F-08 remain open for the owner.
- 2026-11-26T12:29:18+07:00: status in_review -> completed. F-06 fixed in plan-00001 phase 23f (the stack's width, per Scouter's widest visible block); F-07 and F-08 are approved deviations recorded in 6cb4373 (owner, 2026-11-26), issues 00002 and 00010 archived.
