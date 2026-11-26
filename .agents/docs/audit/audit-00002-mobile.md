---
title: "Audit: fmc on a phone (narrow terminals, touch input)"
status: completed
created_at: 2026-11-27T02:00:00+07:00
updated_at: 2026-11-27T02:49:13+07:00
scope:
    type: feature
    feature: mobile use (portrait 40 to 50 columns, landscape 90x20)
    paths: [src/layout.rs, src/app/layout.rs, src/app/mouse.rs, src/app/mod.rs, src/ui/sidebar.rs, src/ui/hints.rs, src/ui/footer.rs, src/ui/header.rs, src/ui/welcome.rs, src/ui/binary.rs, src/ui/changes.rs, src/ui/tree.rs, src/keys/mod.rs]
    exclusions: [real touch input (Rex sends SGR mouse events only); image pixels (the captures are text); widths 52 to 79 (no captures); Kitty, Sixel, and iTerm protocols]
stack:
    languages: [rust]
    frameworks: [opentui 0.9.9]
    datastores: []
repo:
    branch: "mobile-audit-2026-11-27"
    base: "339d119"
    working_tree_clean: true
environment:
    executed: [Rex live runs at 40x30, 45x35, 50x40, 90x20, 80x24 baseline, and tiny sizes 1x1 to 30x4; SGR taps, wheel, and drag-select in Rex; just verify (283 tests, licenses); cargo build]
    not_run: [real touch on a phone; image rendering (the captures show an empty picture body); widths 52 to 79; text-search results at 40 columns (the query was wrong, see Could not test)]
baseline:
    build: pass
    tests: pass (282 at HEAD, 283 after the fix)
---

## Audit: fmc on a phone

### Summary

- fmc works on a phone-sized terminal. It does not crash or panic at sizes from 1x1 to 30x4, and the 40, 45, 50, and 90x20 layouts draw without overlap.
- Four things block touch use at a phone width:
    1. The sidebar is too narrow to read: 12 cells at 40 columns, 17 at 45 (`src/ui/sidebar.rs:33-41`). File names, folder names, and the changes list are cut to a few letters.
    2. The show-the-sidebar mark is one cell (`src/ui/sidebar.rs:21`). The owner's report is right: it is hard to hit.
    3. Almost every key that matters has no tap path. The footer and the header have no mouse handler (`src/app/mouse.rs`, `src/app/mod.rs`), so `Esc`, `?`, `/`, `:`, `Tab`, `d`, `y`, `q`, and `Ctrl+q` are keyboard-only.
    4. At 40 to 45 columns the footer shows three or four hints. `?`, `/`, `d`, and the search prompt's `Esc` are never visible.
- One clear defect is fixed (commit `1121d51`): the hidden-sidebar hint `^/ sidebar` was dropped at 40 columns, so the only visible way back to the list was gone. The approved record says the footer shows it.
- Most other clipping is Scouter's own behavior (its text uses `wrapMode: "none"`), so fixing it changes parity. Those items are proposals, not fixes.

### Method

- Each tab is a new Rex session made with `rex new`, sized with `rex block call -s SESSION -b BLOCK resize`, and started after its size is set (`/tmp/fmc-ref/mobile/launch.sh`). Captures are `rex capture --trim` text, saved under `/tmp/fmc-ref/out/mobile/`.
- `stty cols N rows M` alone does not work: the Rex emulator stays 80x24, and the captures are garbled (the footer interleaves). The block must be resized, and `block call size` confirms it.
- The owner's Rex app (`ArisM2Pro`) took ownership of two of my blocks and resized them to 187x60 and 57x60. Captures taken then are void. Each capture was taken after `size` showed the intended columns and `owner: null`; the sessions were not reused.
- Touch is modelled as SGR mouse events: a left press and release (`ESC [ < 0 ; x ; y M` and `m`), a wheel step (`64` and `65`), and a drag (`32`). This is not real touch input.

### Findings by screen

Evidence: a capture under `/tmp/fmc-ref/out/mobile/` (`g40-*` is the 40x30 walk, `h45-*` 45x35, `i50-*` 50x40, `j90-*` 90x20, `base80-*` the 80x24 baseline).

#### Welcome (`g40-welcome`, `h45-welcome`, `j90-welcome`, `base80-welcome`)

- At 40 columns the tagline (`a quiet, read-only file vi`), the prompt (`Select a file and press En`), the key card's descriptions (`open fil`, `find a f`), and the closing line are cut mid-word. The card text is clipped by its own text rows (`src/ui/welcome.rs:21-27`, `WrapMode::None`).
- At 80x24 (the baseline) the closing line already ends `Just readin`. Scouter draws the same line with `wrapMode: "none"` (`scouter:src/ui/welcome.ts:38`), so this is parity. It is listed under "Not fixed".
- At 90x20 the wordmark's top row and the closing line are both clipped. The view is centred and `Overflow::Hidden` (`src/ui/welcome.rs:31-39`), with no fallback for short heights.
- The sidebar keeps its full width on the welcome screen at any width (approved record, amended 2026-11-27). At 40 columns that is 12 cells.

#### List, sidebar shown (`g40-image-list-shown`, `h45-code-main`, `k40-5-tap-row5`)

- 40 columns: the sidebar is 12 cells (`src/ui/sidebar.rs:33-41`: `constrain` caps the width at terminal − 28 below 56 columns). Names are cut to about 6 letters, and the status glyph is pushed out (`mai M`). The title shows `de files` for `demo-fmc`.
- 45 columns: 17 cells, names cut to about 10 letters (`ledger`, `owl/`, `main.r M`).
- Tree rows: `src/ui/tree.rs:175`, `WrapMode::None`, no ellipsis. The row is clipped silently.
- A tap on a row works: a tap on `owl/` selected it and expanded it (`k40-5-tap-row5`).

#### List, sidebar hidden (reader open, under 80 columns)

- The mark `»` is one cell at the reader's top-left corner (`src/ui/sidebar.rs:21`, `src/app/layout.rs:82-95`). A tap on it shows the sidebar and focuses it (`k40-1-tap-mark-show`); a tap on `«` hides it (`k40-2-tap-mark-hide`). Both work.
- The footer at 40 columns was `j k scroll   h l pan   ^q quit`. The hint `^/ sidebar`, the only key that shows the list, was dropped. This is the fixed defect (commit `1121d51`). After the fix: `j k scroll   ^/ sidebar   ^q quit` (`fx40-hidden-main-after-fix`).

#### Reader: code (`g40-code-main`, `h45-code-main`)

- Readable. Line numbers, the header, and the footer band all fit at 40 and 45 columns.
- Footer: `READER …/src/main.rs  All`, then the hints. At 40 columns `/`, `d`, `?`, `m`, and `Tab` are not visible.

#### Reader: Markdown (`g40-markdown-rendered`, `h45-markdown-rendered`, `g40-markdown-source`)

- Tables wrap mid-word: `Modul|e`, `Langu|age` (45 and 40 columns). The table is drawn by `opentui`'s Markdown widget, not by fmc code.
- The rendered text reads well at 40 columns. The `m` source toggle is reachable only by key.

#### Reader: image (`g40-image`, `h45-image`, `j90-code-main`)

- The picture body is empty in every text capture. `src/ui/picture.rs:14` draws with `ImageProtocol::Auto` (Kitty, Sixel, or blocks). The captures cannot show pixels, so this is not verified.
- The header title is cut without an ellipsis: `assets/marauder-map.` (40 columns). Scouter's header is the same (`scouter:src/ui/header.ts`, `wrapMode: "none"`, `minWidth: 0`), so this is parity.

#### Reader: binary (`g40-binary`, `h45-binary`)

- The notice `Binary files are not supported.` is cut to `Binary files are not suppo` at 40 columns. `src/ui/binary.rs:35` uses `WrapMode::None`. Scouter's binary view uses the same mode (`scouter:src/reader/binary-view.ts:40-46`). Parity.
- The footer shows the size and `binary` correctly.

#### Finder, name mode (`f40-finder-name`, `h45-finder-name`)

- The result row shows the folder prefix and cuts the file name: `assets/m` at 45 columns, `ass` at 40 (the match is `assets/marauder-map.png`). The user cannot see what matched. The row keeps its start, so the name is at the end where it is lost.
- The footer at 40 and 45 shows `type filter   ↑ ↓ move   Enter open`. `Esc cancel` is not visible.

#### Finder, text mode (`f40c-finder-text-harry`)

- The sidebar is 12 cells wide, so the snippet rows cannot be read. The capture shows an empty list because the query I typed was wrong (`HarryHarry`); the results were not verified at 40 columns.

#### In-file search (`g40-search-owners`, `h45-search-owners`)

- Readable: `SEARCH  /owners  2 of 2  …`. The footer at 45 shows `type search   Enter next   ↑ ↓ prev / next`, and `Esc close` is dropped. Esc is the only way out (no tap path, see Touch below).

#### Help `?` (`g40-help-card`, `h45-help-card`, `j90-help-card`)

- The key list is a Markdown table. At 40 and 45 columns the `Key` column wraps mid-word: `Backspa|ce`. At 90 columns it reads fine.
- The footer shows `j k scroll   gg G top/bottom   ^q quit` at 40 and 45. `? help` and `q close` are not visible, and neither has a tap path.
- Duplicate row: `[ ]` is listed twice in `src/keys/mod.rs:102-103` (also in `--help`). Not fixed (see Not fixed).

#### `:` prompt and error notice (`g40-colon-prompt`, `g40-colon-notice`)

- Readable. `COMMAND :` and `Esc cancel` fit. `Not a command: xyz` shows in the footer.

#### Changes, one file (`g40-changes-one`, `h45-changes-one`)

- The header reads `README modified one file`. The list names are cut: `RE`, `lu`, `unl`, `o` at 40 columns (`src/ui/changes.rs:29`, `WrapMode::None`). The list is not usable at 40 columns.
- Footer at 40: `j k move   Enter view   ^q quit`. `a`, `d`, `?`, and `Tab` are not visible.

#### Changes, all files (`g40-changes-all`, `y40-all-after-header-press2`)

- The header title is cut (`READM modified all files`). The diff pane is 26 columns, and lines are cut at the right edge (pan with `h l`).
- A press on a file's header folds it: `▾ M ○ README.md` became `▸ M ○ README.md` (`y40`). See Touch below.

#### Landscape 90x20 (`j90-*`)

- The layout is sound: sidebar 28 cells, reader 58 cells, header, footer band, and hints all fit on one line. The only defect is the welcome's clipped wordmark and closing line.

#### Tiny sizes (`pz10x5`, `pz8x4`, `pz1x1`, `tiny*`)

- No panic and no exit at 1x1, 6x3, 8x4, 10x5, 12x6, 20x10, or 30x4. Each stayed up through the welcome, reader, finder, changes, help, and the `:` prompt. The captures are clipped, as expected.

### Touch and keys

#### What a tap or gesture does (tested)

| Gesture | Result | Evidence |
| --- | --- | --- |
| Tap the `»` mark (1 cell) | Shows the sidebar and focuses it | `k40-1-tap-mark-show` |
| Tap the `«` mark (1 cell) | Hides the sidebar | `k40-2-tap-mark-hide` |
| Tap a tree row | Selects it, expands a folder | `k40-5-tap-row5` |
| Wheel, 5 notches, in the reader | Scrolls 5 lines | `x40-1-wheel5` (1 to 6) |
| Press and drag in the reader, then `y` | Selects 2 lines; `Copied 2 lines.` | `x40-2-drag`, `x40-2` footer |
| Press on a diff file header | Folds the file (`▾` to `▸`) | `y40-all-after-header-press2` |

#### What has no tap path (source and captures)

- The footer and header have no mouse handler. `grep on_mouse src` lists the list (`src/app/mouse.rs:9`), the changes list (`:43`), the reader and the stack (`src/app/mod.rs:319-349`), the bars (`src/app/mod.rs:381`), the sidebar edge and mark (`src/app/layout.rs:97`, `:120`), and the pane presses (`src/app/mod.rs:358`). Nothing else.
- Keys reachable only from the keyboard: `Esc` (closes search, help, and the prompt, and clears a selection), `?`, `/`, `:`, `Tab` (pane switch), `d` (changes), `a` (all files), `m` (source or render), `y` (copy the selection), `q` (close), `p` (preview), `n` `N` (hunks), `c` (fold), `x` (context), `w` (wrap), `o` (open file), `s` (split), `gg` `G`, `z z`, `^d` `^u`, `Ctrl+q` (quit), and `Ctrl+←/→` (resize).
- A drag selects text, but nothing is copied until `y` (`docs/parity.md`, "Copying text"). A phone user cannot copy without a keyboard.

#### Gesture conflicts

- Wheel and drag-select do not conflict in the reader. A wheel step scrolls one line, and a press-drag selects text (tested above).
- A press on a diff header folds the file at once (`src/app/mod.rs:335-349`, `changes::click_header`). A swipe that starts on a header folds the file too. Scouter folds on click (`createHeader`), so a click matches; a drag does not, and touch swipes are drags.
- The sidebar edge is a one-column strip (`src/app/layout.rs:70-80`). It is a drag target but a hard one to hit.

#### Target sizes

- Mark: 1 by 1 cell (`src/ui/sidebar.rs:21`). Too small for a finger.
- Tree and changes rows: the full sidebar width (12 to 17 cells at phone widths) and one row high. Usable.
- Footer and hints: no target at all.
- Sidebar edge: 1 column. Too small.

### Layout code

- The breakpoint is 80 columns (`src/layout.rs:6`). A document under 80 hides the sidebar; a manual choice holds until a crossing (`src/layout.rs:28-33`).
- The sidebar width is a quarter of the terminal, clamped to 28 to 40 (`src/ui/sidebar.rs:40`). `constrain` keeps the reader at 28 or more (`src/ui/sidebar.rs:33-37`). Below 56 columns the sidebar is the terminal minus 28, so 12 cells at 40 and 17 at 45.
- The footer's hints are fitted in order: `fit_hint_list` stops at the first hint that does not fit (`src/ui/hints.rs:165-180`). The reader's list is ordered for 80 columns, so at 40 only the first three or four survive (`src/ui/hints.rs`, `hints_for`).
- The footer's band fits its facts before its location (`src/ui/footer.rs:97-104`).
- Header title and badges: `WrapMode::None`, `minWidth: 0`, no ellipsis (`src/ui/header.rs:81`). Scouter's header is the same (`scouter:src/ui/header.ts`).

### Fix in this audit

| Commit | Change | Tests | Parity |
| --- | --- | --- | --- |
| `1121d51` `fix(ui): the hidden-sidebar hint stays in the footer at phone widths` | While the sidebar is hidden, the reader's and the changes body's footers place `^/ sidebar` second. It was seventh, so the fit dropped it at 40 columns. | `ui::footer::tests::a_hidden_sidebar_hint_survives_a_phone_width` (new); the existing hidden-sidebar test still passes | The shown state is unchanged (the reorder runs only while hidden). The 80x24 welcome is byte-identical to the baseline (`base80-welcome` vs `new80-welcome`). The 80x24 reader footer in the shown state is unchanged by construction. |

Live check after the fix: `fx40-hidden-main-after-fix` shows `j k scroll   ^/ sidebar   ^q quit` at 40x30.

### Not fixed (parity or the owner's call)

- Header titles and badges are cut without an ellipsis (`src/ui/header.rs:81`). Scouter's header is the same (`scouter:src/ui/header.ts`). Changing it is a deviation.
- The welcome's closing line is cut at 80x24 (`base80-welcome`). Scouter's welcome is the same (`scouter:src/ui/welcome.ts:38`). Changing it changes the 80x24 capture.
- The binary notice is cut at 40 columns. Scouter's binary view is the same (`scouter:src/reader/binary-view.ts:40-46`).
- The `[ ]` row is listed twice in `src/keys/mod.rs:102-103`, and so in `--help` and the `?` card. It is a clear duplicate, but removing one row changes the `?` card at 80x24. The owner's rule keeps 80x24 captures fixed, so this waits for the owner.
- The diff header fold on press (see Gesture conflicts). It matches Scouter's click, so it is a deviation only for drags.

### Proposals (not implemented)

#### P1. A tap target that shows the hidden sidebar

- Problem: the mark is 1 cell (`src/ui/sidebar.rs:21`). It is the only tap path to the list while a document is open under 80 columns.
- Options:
    1. A three-cell label `» list` on the reader's top border. A target of six cells by one row. Keeps the glyph.
    2. While hidden, a press on the reader's header row (two rows, full width) shows the sidebar. The header press already focuses the reader, so the rule changes only while hidden.
    3. A tap on the footer's mode pill (`READER`) shows the sidebar. Needs a footer handler (see P2).
- Recommendation: option 2. It is the largest target, needs no new glyph, and sits at the top-left where the thumb reaches on a phone. Option 1 is the fallback if the owner wants the mark visible.
- Cost: small. One mouse handler on the header while hidden, and a test.
- Parity at 80 and wider: none while the sidebar shows (the default). A manual hide at 80 or more is an approved state, so the rule applies there too; it needs an `architecture.md` note.

#### P2. Tap paths for the keys

- Problem: `Esc`, `?`, `/`, `:`, `Tab`, `d`, `y`, `q`, and `Ctrl+q` have no tap path (see Touch above).
- Options:
    1. Make the footer's hints tappable: a press on a hint's cells sends its key through the same dispatch. Uses the hints already shown.
    2. A narrow-only key row under the footer (below 60 columns) with `Esc`, `Tab`, `?`, `/`, `:`, `d`, `y`, `q`. Costs one row of height.
    3. Tap on the mode pill opens the key list (`?`). Covers help only.
- Recommendation: option 1, narrow widths first, with explicit tap spans. It adds no rows, and it reuses the keys' own hints. `Ctrl+q` should stay keyboard-only or move into the `:` prompt, since a stray tap should not quit.
- Cost: medium. A hit-test for each hint span, and a dispatch from the footer.
- Parity at 80 and wider: footer clicks are new behaviour that Scouter does not have. It needs owner approval and a `parity.md` line. Limit it to widths under 80 to keep the 80x24 captures.

#### P3. A readable sidebar at phone widths

- Problem: the sidebar is the terminal minus 28 below 56 columns (12 cells at 40, 17 at 45). Names are cut to about six or ten letters (`src/ui/sidebar.rs:33-41`).
- Options:
    1. Below about 60 columns, a full-width drawer: the list covers the reader when shown; a row tap opens the file and hides the list (the usual phone pattern).
    2. Keep the split and raise the minimum sidebar to 20 cells. The reader gets 20 cells, which is too narrow to read.
    3. Keep the split; rely on the finder for names.
- Recommendation: option 1. The list is readable and the reader gets the full width.
- Cost: medium. A layout rule, focus rules, and a changed record. It amends the 80-column record (`architecture.md`).
- Parity at 80 and wider: none, if the drawer applies only below 80. It changes the approved record below 80, so it needs the owner's approval.

#### P4. Finder and changes rows keep the file name

- Problem: a row keeps its start and cuts its end. The finder shows `assets/m` for `assets/marauder-map.png`, and the changes list shows `RE` for `README.md` (`src/ui/tree.rs:175`, `src/ui/changes.rs:29`).
- Options:
    1. Name first, then the folder in dim; cut the folder first.
    2. Keep the row and cut the start with `…` (`…/marauder-map.png`), as the footer does.
    3. Leave it.
- Recommendation: option 1, under 80 columns only. It keeps the name, which is what the user searched for. Note `issue-00012` (the finder row) is about the same row.
- Cost: small.
- Parity at 80 and wider: none if gated under 80. At 80 the row order is Scouter's, and it changes only when the owner approves.

#### P5. Welcome at phone widths and short heights

- Problem: at 40 columns the tagline, the prompt, the card text, and the closing line are cut. At 90x20 the wordmark's top row is lost (`src/ui/welcome.rs:21-39`).
- Options:
    1. Under about 50 columns, drop the key card and wrap the lines; keep the wordmark. Under 24 rows, drop the closing line.
    2. Keep the card, and shorten its text (`open file`, `find`).
    3. Leave it.
- Recommendation: option 1, gated under 80 columns and below the height the content needs.
- Cost: small to medium.
- Parity at 80 and wider: the 80x24 welcome already clips its closing line, and Scouter does too (`scouter:src/ui/welcome.ts:38`). Changing the 80x24 welcome needs the owner's approval; gating under 80 keeps the capture.

#### P6. Footer priority at phone widths

- Problem: at 40 and 45 columns the footer keeps three or four hints. `?`, `/`, `d`, and the prompts' `Esc` are never visible (`src/ui/hints.rs:165-180`).
- Options:
    1. A priority order per width: `?` and `/` (and `Esc` in a prompt) before the paging keys.
    2. A two-row hint bar under 60 columns.
    3. Leave it; rely on the key list.
- Recommendation: option 1, under 80 columns only.
- Cost: small.
- Parity at 80 and wider: option 1 changes the 80-column footer order, so it is gated under 80.

#### P7. A diff header's fold on release

- Problem: a press folds the file at once (`src/app/mod.rs:335-349`). A swipe that starts on a header folds it.
- Options:
    1. Fold on release if the pointer did not move (a touch slop of a few cells).
    2. Fold only from the chevron cell (`▾` or `▸`).
    3. Leave it; it matches Scouter's click.
- Recommendation: option 1. A click behaves the same; only a drag changes.
- Cost: small.
- Parity at 80 and wider: a click is unchanged, a drag that starts on a header no longer folds. That is a deviation, so it needs an `architecture.md` record and a `parity.md` line.

#### P8. Copy without a key

- Problem: a drag-select copies only on `y` (tested: `Copied 2 lines.`), and `y` is a key.
- Options:
    1. A `Copy` control in the footer while a selection exists, under 80 columns.
    2. Copy on release. Changes the approved "nothing is copied until you press y".
    3. Leave it.
- Recommendation: option 1. It keeps the approved behaviour and adds a tap path.
- Cost: medium.
- Parity at 80 and wider: a control that appears only with a selection, which Scouter does not have. It needs approval; limit it under 80 columns.

### Owner checks

- Real touch on a phone: a tap on the mark, a swipe in the reader (wheel or drag), and a swipe that starts on a diff header.
- Image on a phone: whether the picture draws (the protocol the terminal supports).
- The `[ ]` duplicate in `?` and `--help`: approve the one-row removal, or keep it.

### Could not test

- Real touch (SGR events only).
- Image pixels in Rex (the picture body is blank in text captures).
- Widths 52 to 79 (no captures).
- Text-search results at 40 columns (the query was wrong).
- Real Rex clients: while my tabs ran, the owner's Rex app took two of them over and resized them (187x60, 57x60). Each capture was checked against `size` before it was saved.

### Open questions

1. P1: header tap (option 2) or the visible `» list` label (option 1)?
2. P2: approve tappable footer hints under 80 columns?
3. P3: approve a full-width drawer below about 60 columns, which amends the 80-column record?
4. The `[ ]` duplicate row: remove it (changes the `?` card at 80x24)?

## Verdicts

The owner approved P1 to P8 on 2026-11-27. Plan-00004 landed them; the answers to the open questions are in `architecture.md` ("a phone layout below 80 columns") and the welcome amendment in `cc7ae66`.

| Proposal | Verdict | Where |
| --- | --- | --- |
| P1 header tap (option 2) | Landed | `f4f46bd`; the welcome below 60 in `6be0b8e` |
| P2 tappable footer hints (option 1) | Landed under 80 columns; `^q` stays keyboard-only | `14b7078` |
| P3 full-width drawer below 60 (option 1) | Landed; amends the 80-column rule below 60 | `f4f46bd`, `6be0b8e` |
| P4 name first, folder dimmed (option 1) | Landed for finder rows under 80; the change list already drew the name first | `355957e` |
| P5 narrow welcome (option 1) | Landed below 50: card dropped, lines wrapped. The short-height rule (drop the closing line under 24 rows) is not implemented | `5ec24e4` |
| P6 footer priority (option 1) | Landed under 80; the order is the agent's pick, in `ui/hints.rs` `phone_rank` | `14b7078` |
| P7 fold on release (option 1) | Landed at every width | `f46692d` |
| P8 Copy hint (option 1) | Landed under 80; leads the hints while text is selected | `14b7078` |

Open questions: 1, option 2 (the header tap). 2, yes: footer taps under 80 columns. 3, yes: the drawer below 60. 4, the duplicate `[ ]` row is removed (`18b623f`).

Owner checks (real touch on a phone) are not run here; they are listed in `handover-00005-phone-layout.md`.
