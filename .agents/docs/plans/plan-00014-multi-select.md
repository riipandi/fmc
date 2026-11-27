---
status: in-progress
created: 2026-11-27T21:00:00+07:00
updated: 2026-11-27T21:00:00+07:00
references: roadmap-00001-post-parity.md (RI-19), plan-00012-write-operations.md (D2, D7, D8), architecture.md (the write records)
phases:
    - id: 1
      title: The selection model (no UI)
      status: committed (feat(explorer): a set of marked rows that survives scrolling and collapse)
      note: a set of marked paths in the browser, pruned as listings change; 8 tests in explorer/tests.rs
    - id: 2
      title: Mark rows with s, clear them with S
      status: pending
      note: the keys, the mark glyph, the footer count, Esc; Rex capture at 120x40 and 45x30
    - id: 3
      title: Copy, cut, and paste several marked rows
      status: pending
      note: the clipboard holds a list; one bulk queue on the worker; the menu counts; Rex run
    - id: 4
      title: Delete several marked rows to the Trash
      status: pending
      note: the confirm names the entries; the Trash joins the queue; security review; the full Rex scenario
    - id: 5
      title: Docs sync, ledger close, handover
      status: pending
      note: index rows, roadmap RI-19 landed, this plan archived, handover-00009; the one docs commit the brief names
    - id: 6
      title: Final validation
      status: pending
      note: the gates over the tree as it stands; no commit if green
---

## Mark several rows and act on them together — the tree's selection, and copy, cut, paste, and delete over it

### Objective

RI-19 (`roadmap-00001-post-parity.md`) asks for multi-select: plan-00012 acts on one row at a time (D7). The owner chose the keys on 2026-11-27: `s` marks the selected row and moves down one row, and `S` clears every mark. The marked rows take `c`, `x`, `D`, Delete, and the context menu's Copy, Cut, and Delete. This plan makes the tree hold a set of marked paths, draws them, runs a copy, move, or Trash over the set one item at a time on the worker, and stops at the first error. It does not add mouse selection, range keys, select-all, marks in the finder or the changes screen, or any write outside the browsed root.

### Current state (verified in the tree, 2026-11-27, `main` at `b929627`)

- **The browser has no selection beyond the cursor.** `src/explorer/browser.rs:26-36` holds `selected`, `top`, `height`, `flat`, and `flat_at`. `selected_entry` (`:50`) is the only read of the row the keys act on.
- **The name `marks` is taken.** `src/app/mod.rs` `Model.marks: Marks` holds git's change marks (`fmc::diff::Marks`), refreshed by `src/app/tree.rs:178` `refresh_marks`. The new set is named `marked`, and no new code says "mark" for git's marks.
- **The keys `s` and `S` are free in the tree.** The tree table (`src/keys/parser.rs:222-250`) binds Return, `l`, `h`, `-`, `p`, `/`, `Space`, `a`, `A`, `R`, `r`, `D`, Delete, `c`, `x`, `v`, `[`, `]`, and the list moves. `s` is bound only on the changes screen (`parser.rs`, `changes()`: `("s", _) => ToggleSplit`), which the tree never sees while the changes screen shows.
- **Esc in the list drops the clipboard first.** `src/app/capture.rs:96` `Action::Escape if m.focus == Focus::List && m.clipboard.is_some() && m.changes.is_none()`; otherwise Esc only moves the focus to the list (`:97`).
- **The clipboard holds one path.** `src/app/write.rs:34-38` `Clip { path, cut }`; `src/app/mod.rs:203` `clipboard: Option<write::Clip>`. A cut row is muted by `src/ui/tree.rs` `paint` (its `cut: Option<&Path>` parameter).
- **A paste runs on one worker job.** `src/app/write.rs:188-209` `paste` sets `Model.pasting` (`src/app/mod.rs:205`) and spawns one `Event::Pasted { source, into, moved, result }` (`src/services/mod.rs:59-61`). `duplicate` (`write.rs:318`) does the same with a typed name.
- **A delete is one path, inline.** `src/app/write.rs` `Task::Delete { path, next, identity }` (`:41-60`) confirms one entry; `submit` (`:263`) checks its identity and calls `trash` on the UI thread. plan-00012 D8 says Trash runs inline (`rules.md`, "Writes", the fourth bullet).
- **The menu's labels are fixed.** `src/app/write.rs:67-79` `menu_rows` builds `New file…`, `New folder…`, `Rename…`, `Copy`, `Cut`, `Paste`, `Duplicate…`, `Delete…` from one row.
- **The tree's leading column is the selection bar.** `src/ui/tree.rs:57` `vec![Seg::new(if selected { "▎" } else { " " }, p.accent)]`; the git mark is a letter at the row's right end (`ui/tree.rs` `RowView.mark`).
- **The footer's facts are for documents.** `src/ui/footer.rs:167` `candidate_facts` builds facts from the document and the diff; `:218` `fit_facts` drops them to fit. Nothing in a plain tree says how many rows are chosen.
- **The compact bottom bar lists the page's keys.** `src/ui/bars.rs:28` `page_segments`: on the Files page, `Find /`, `Changes d`, `Keys ?`.
- **The tree is rebuilt on `-`.** `src/app/tree.rs` `go_up` replaces `m.browser` with `Browser::new`, so a set held in the browser starts empty for a new root.
- **Nothing selects more than one row today.** `grep -rn "marked\|selected_set" src` finds only the text-match `marked_text` (`ui/tree.rs`, the fuzzy highlight) and `reader/headings.rs`.

### Decisions

Decisions marked _(owner)_ are the owner's, given on 2026-11-27 ("kerjakan RI-19", and the keys). Decisions marked _(agent)_ are the agent's pick where the owner's list was silent; each is one line to change, and the owner can overrule any of them before phase 3 lands.

- **D1 _(owner)_: the keys.** `s` toggles the mark on the selected row and moves the selection down one row (a last row stays). `S` (Shift+s) clears all marks. Both work in the tree only; the finder and the changes screen keep their own `s` (the finder types it, the changes screen splits its diff). Rejected: Space for the mark, because Space opens the menu (plan-00012 D3). Rejected: Ctrl+click or Cmd+click, because terminals do not send them (Shift+click is the terminal's own selection).
- **D2 _(owner)_: marks are a set of paths in the browser.** `Browser` holds a `BTreeSet<PathBuf>`. A mark survives scrolling, collapse, and a folder's re-listing. A mark is pruned when its entry is no longer in the tree (`Tree::entry` returns `None` after a listing or a poll), so a renamed, moved, or trashed entry leaves the set. Rejected: a mark on the row index: a row moves when a folder collapses, so a mark would name a different entry.
- **D3 _(owner)_: a marked row shows a mark, the selection keeps its bar.** The leading column of a marked row is `✓` in `accent_soft` (`palette` token, `#c7d2fe` dark), and the name is unchanged. The selected row keeps `▎`, its row background, and its bold accent name; a marked and selected row shows `✓`, with the background. A cut row stays muted, as now. Rejected: a background tint for marks: `selection_bg` is the tint the selection and the text hits use, and a second tint on the same row would be read as a selection. _(agent: the glyph and the token.)_
- **D4 _(owner)_: the actions over the marks.** With marks, `c`, `x`, `D`, Delete, and the menu's Copy, Cut, and Delete act on every marked entry, in path order. With no marks they act on the selected row, as now. `r` and `R` stay single-row, and `a`, `A`, and `v` take the selected folder as they do now. The marks of the selected row are not required: a mark elsewhere still makes `c` act on the marks.
- **D5 _(owner)_: a marked folder and its marked descendant collapse to the folder.** Copying or trashing the folder already takes its children, so a marked child inside a marked folder is dropped from the action. The rule is applied to the set before the action runs.
- **D6 _(owner)_: the labels and the names say the count.** The menu reads `Copy 3 items`, `Cut 3 items`, `Delete 3 items…`, and `Paste 3 items` when the clipboard holds three; one entry keeps the existing labels. The Trash confirm names up to five entries, one per line, then `and K more` when there are more; its title reads `Move 3 items to the Trash?`.
- **D7 _(owner)_: a paste of several entries runs one at a time on the worker.** The footer says `copying 2 of 5…` (or `moving`); the first error stops the run, and the footer reports how many succeeded: `Copied 2 of 5. <error>`. Names are never overwritten: each copy takes its free name (D5 of plan-00012). A cut pasted where it lies is skipped, and a cut that leaves nothing to move ends the cut.
- **D8 _(owner)_: the footer shows the count while marks exist.** Three marked entries show `3 marked` in the footer's facts (medium and wide tiers). In the compact tier the bottom bar's Files page shows it as its first segment.
- **D9 _(owner)_: Esc clears the clipboard first, then the marks.** In the list with a clipboard, Esc drops the clipboard (as now). Otherwise, with marks, Esc clears them. Otherwise it does what it does now.
- **D10 _(owner)_: the finder and the changes screen have no marks.** A mark is drawn and counted only in the tree. Opening the finder hides the marks' glyphs but keeps the set; closing it shows them again.
- **D11 _(agent)_: the clipboard is a list of paths.** `Clip { paths: Vec<PathBuf>, cut }` replaces the one path. This amends the record "The clipboard is fmc's own, and the footer names what it holds" (the one-path clause). Rejected: one clipboard entry per marked item: a second `c` would then add to the clipboard and `v` would have to know which entries came together.
- **D12 _(agent)_: every bulk write is a queue, and the Trash joins it.** Copy, move, duplicate, and Trash run through one queue on the worker, one item per job, and a single entry is a queue of one. The Trash moves from inline to the worker (plan-00012 D8 said inline). Rejected: inline Trash for several entries: each `trash` call waits for Finder through `osascript` (`architecture.md`, "Deleted entries go to the Trash through Finder"), so ten entries freeze the frame for seconds. The owner can take the inline path back for a single entry, which changes one line in `write.rs`.
- **D13 _(agent)_: marks clear only when a bulk op finishes with every item done.** A copy's marks clear when all copies succeed; a move's and a Trash's marks clear the same way (the moved and trashed entries have left, so their marks are pruned anyway). A partial run keeps the marks, so a retry picks up the rest. `c` and `x` do not clear the marks; they copy the marked list into the clipboard, and the marks stay until a bulk op ends. Rejected: clearing at `c`: the user may want to paste somewhere else and still see what they marked.
- **D14 _(agent)_: `-` clears the marks, and a dotfile toggle keeps them.** `-` rebuilds the browser, so the set starts empty for the new root (`src/app/tree.rs` `go_up`). Hiding dotfiles keeps a mark on a hidden entry: the confirm names every marked entry, so a hidden mark is named before it is trashed. Rejected: pruning on hide: a user who hides dotfiles to tidy the list would lose marks without a sign.
- **D15 _(owner)_: writes stay in the tree, inside the root.** Every marked path comes from a row under the browsed root. Each item still runs the write layer's checks (`services/write.rs` `plan`, `trash_with`): the root, the name, the vacant destination, and a folder into itself. The Trash confirm rechecks each item's device and inode before the first call, as a single delete does now (plan-00012 D6).

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The marked set and its pruning | `src/explorer/browser.rs`, `explorer/tests.rs` | D2, D5, D14 |
| The keys `s`, `S`, and Esc's order | `src/keys/parser.rs`, `src/keys/mod.rs` (`KEY_SECTIONS`, `?`), `src/app/capture.rs` | D1, D9 |
| The mark glyph, the count | `src/ui/tree.rs`, `src/ui/footer.rs`, `src/ui/bars.rs`, `DESIGN.md` | D3, D8 |
| The clipboard as a list | `src/app/write.rs`, `architecture.md` ("The clipboard is fmc's own") | D11 |
| The bulk queue and the worker event | `src/app/write.rs`, `src/services/mod.rs` (`Event::Written`), `architecture.md` (a new record) | D7, D12, D13 |
| The rules of a bulk write | `rules.md`, "Writes" (the fourth bullet changes: the Trash is on the worker) | D12, D15 |
| The keys for the user | `docs/keymaps.md`, `docs/usage.md` ("Managing files"), `?`, `--help`, `tests/cli.rs` | D1, D4 |
| The changelog | `CHANGELOG.md` `[Unreleased]` | every decision the user sees |

### Execution rules

- One atomic commit per phase, with the message the phase names. A phase's commit carries the docs for its own behaviour (`rules.md`, "Where a fact lives"); the index and the roadmap move only in phase 5, as the brief asks.
- Work on `feat/ri-19-multi-select` in the worktree `.claude/worktrees/agent-a7347a7a35d1b41bd`. Nothing is pushed.
- No new dependency. The glyph `✓` and the palette token `accent_soft` are existing things.
- Tests that write use a temporary folder. The Trash is the test's stand-in (`app/write.rs` `trash`, `#[cfg(test)]`), so no test writes to the owner's Trash. The sample names come from Dan Brown and Harry Potter.
- `KACHE=false just verify` green before every commit, and `cargo build`. A phase that changes what is drawn or how a key is handled runs a live Rex capture in a new tab at 120×40 and 45×30 (`AGENTS.md`, "Rex live runs"); the owner's panes and windows are not touched.
- Each phase names its commit trailer: none. These phases port no Scouter behaviour (the feature is post-parity, like plan-00012), so no `Scouter-Base` trailer applies.

### Phases

1. **The selection model (no UI).** `Browser` gets `marked: BTreeSet<PathBuf>` with `toggle_mark(path)`, `clear_marks()`, `marked()`, and `targets()`, which returns the set in path order with the descendants of a marked folder dropped (D5). `listed` and `refreshed` prune the set against `Tree::entry` (D2). `Browser::new` starts it empty (D14). Validation: unit tests in `explorer/tests.rs` for each rule (the toggle, the order, the collapse rule, the prune after a folder's re-listing drops a child, a collapsed folder keeps its marked children, a root change starts empty); `KACHE=false just verify`; `cargo build`. _Commit: `feat(explorer): a set of marked rows that survives scrolling and collapse`._
2. **Mark rows with s, clear them with S.** `Action::ToggleMark` and `Action::ClearMarks` in the tree table; `app/tree.rs` `toggle_mark` (the selected row, then one row down) and `clear_marks`; `capture.rs` Esc order (D9); `ui/tree.rs` `row_segments` takes `marked` and draws `✓` in the leading column; `paint` takes the set and draws nothing in the finder (D10); the footer's `3 marked` fact and the compact bar's first segment (D8); `KEY_SECTIONS` and the `?` page; `docs/keymaps.md`; `CHANGELOG` `[Unreleased]`; `DESIGN.md` (the mark, the count); `architecture.md` (a record: "Marks are a set of paths in the browser", D1 to D5, D9, D10, D14). Validation: parser tests (`s` and `S` in the tree, not in the reader, not on the changes screen); model tests (toggle and move, clear, Esc order, count on the footer, the finder hides the glyphs); a Rex run at 120×40 and 45×30 that marks three files, shows the glyph, the count, and Esc clearing them, captured. _Commit: `feat(tree): mark rows with s and clear them with S`._
3. **Copy, cut, and paste several marked rows.** `Clip { paths, cut }` (D11); `take` copies the marked targets or the selected row; `paste` queues one item per path on `Model.bulk` (D12) and the worker event `Event::Written { kind, source, into, result }` replaces `Event::Pasted`; each answer starts the next item, stops at the first error (D7), and on the last one reveals the last copy and clears the marks when all were done (D13); `menu_rows` labels with the counts (D6); a cut pasted into its own folder is skipped per item; `duplicate` joins the queue as a copy with a typed name; the cut dim takes the list. Validation: model tests for three marked files copied into a folder (the three copies, the marks cleared, the reveal), a cut of three (the sources gone, the clipboard empty), a same-folder cut (skipped), a stop at the first error (`Copied 1 of 3. …`, the marks kept), a folder with a marked child (one copy), a paste while one runs (refused), `R` still single-row; `security-review` on the bulk copy path; `architecture.md` (the clipboard record amended; a record for the queue, D12 and D13); `rules.md` "Writes" (the queue); `docs/usage.md` "Managing files"; `docs/keymaps.md`; `CHANGELOG`. Rex run at 120×40 and 45×30: mark three, `c`, select a folder, `v`, capture the footer's `copying 2 of 3…` and the tree. _Commit: `feat(tree): copy, cut, and paste several marked rows`._
4. **Delete several marked rows to the Trash.** `Task::Delete { items, next }`, the confirm's title and its names (D6); the identity of each item is rechecked before the first call, and a changed one refuses the whole batch with the existing message (D15); the Trash joins the queue (D12), so a single delete also runs on the worker and its footer says `trashing Robert Langdon…`; the reveal selects the successor of the selected row, or the parent when that row was trashed; the menu's Delete reads its count. Validation: model tests for three marked files trashed (the Trash stand-in), the confirm's names and `and K more` at six entries, a changed item refusing all, a stop at the first error, a folder's marked child dropped by D5; `security-review` on the bulk delete path; the full Rex scenario at 120×40 and 45×30: create three unique files, mark them, copy and paste into a folder, mark two, delete them to the Trash; each step captured, names reported. _Commit: `feat(tree): delete several marked rows to the Trash`._
5. **Docs sync, ledger close, handover.** `docs/usage.md`, `docs/keymaps.md`, `DESIGN.md`, `README.md` if the front page names the keys, `CHANGELOG` `[Unreleased]` (one entry per user-visible change), `architecture.md` (the records of D11 and D12 read against the code), `rules.md` ("Writes" matches D12 and D15); the roadmap: RI-19 `landed` in its status line and its table row; `index.md` rows for `plan-00014` and `handover-00009`; this plan archived with `status: done`; `handover-00009-multi-select.md`. Validation: each doc claim checked against the code with a `file:line`; `rumdl` green through `just verify`. _Commit: `docs: multi-select and bulk operations`._
6. **Final validation.** The matrix below over the tree as it stands. _Commit: none if green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Gates | `KACHE=false just verify` | All green |
| Build | `cargo build` | Builds |
| The set | `cargo test explorer` | D2, D5, D14 for each rule |
| Keys | `cargo test keys` | `s` and `S` in the tree only; the reader and the changes screen unchanged |
| The mark and the count | `cargo test app::tree`, `ui::tree`, `ui::footer` | D3, D8; the glyph and the footer fact |
| Esc order | `cargo test app::capture` | D9: clipboard, then marks, then the list |
| Bulk copy and cut | `cargo test app::write` | D4, D7, D11, D13: the copies, the moves, the stop, the skip |
| Bulk delete | `cargo test app::write` | D6, D15: the names, the identity refusal, the stop |
| Writes stay in the root | `cargo test services::write` (unchanged) and the bulk tests | Every bulk path refuses a target outside the root |
| Live run | Rex, 120×40 and 45×30, a new tab of the agent's session | Each phase's capture as its own phase lists |
| Security | `security-review` on phases 3 and 4 | No path leaves the root; a swapped item is refused; no shell is built from a name |
| Parity | n/a: post-parity feature; Scouter has no marks | — |
| E2E | n/a: no E2E harness; the Rex runs stand in | — |

### Explicitly out of scope

- **Mouse multi-select.** A terminal sends Shift+click to its own selection and no Cmd+click (the owner, 2026-11-27).
- **Range select and select all.** No key was chosen; a range is its own decision.
- **Marks in the finder and on the changes screen.** D10: the tree only.
- **Persisted marks and marks across restarts.** The set lives in the browser and is lost on exit, as the clipboard is.
- **Undo for a bulk write.** The Trash's Put Back covers a delete; a copy or move has no undo, as plan-00012 has none.
- **Copy and move across browsed roots, or to the system clipboard.** plan-00012 D2 keeps the clipboard fmc's own.
- **A Replace, Keep both, or Cancel dialog for a taken name.** A paste names its copy freely (D5 of plan-00012); the owner kept that deferred.
