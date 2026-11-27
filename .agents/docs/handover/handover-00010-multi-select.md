---
status: done
created: 2026-11-27T23:00:00+07:00
plan: plan-00014-multi-select
---

# Handover: multi-select and bulk operations landed; plan-00014 closed

## Closed

- plan-00014 phases 1 to 4 on `feat/ri-19-multi-select` (not pushed): the marked set `19d119d`, the keys `s` and `S` with the mark glyph and the footer count `b669168`, the bulk copy, cut, and paste `4c7eac9`, the bulk Trash `df88716`. The plan itself is `ede3e3f`; the docs commit is named by its message, `docs: multi-select and bulk operations`.
- The keys: `s` marks the row and moves down, `S` clears the marks. `c`, `x`, `D`, Delete, and the menu's Copy, Cut, and Delete act on every marked row, or the selected row when none is marked. `r` and `R` stay single-row.
- The runs: a copy, move, duplicate, or Trash of several entries runs on the worker one item at a time, and the first error stops it. The footer names the count (`copying 2 of 5…`) and says how many landed (`Copied 2 of 5. <error>`). The marks clear only when every item landed.
- Security (`security-review` on phases 3 and 4). Finding 1 is fixed: the clipboard keeps each entry's device and inode, and a changed entry is refused with a footer line (`An item changed after it was copied or cut, so it was skipped.`). Findings 1 and 2 (the windows between the checks and Finder's call) are recorded as a `shortcut:` in `services/write.rs` (`trash_with`). Finding 3 is the owner's decision: the five-name confirm stays. Findings 4 (escaping) and 5 (links) were checked live: a quoted name trashed intact, a link refused; see the owner checks.
- A link cannot go to the Trash. Finder refuses it (error -5000), and a plain file in the same folder goes. `D` on a link refuses before its confirm. This amends plan-00012 D6, which said a link is trashed as the link (`architecture.md`, "Deleted entries go to the Trash through Finder"; plan-00014 D16).
- Docs: `docs/keymaps.md`, `docs/usage.md` ("Managing files"), `DESIGN.md` (the mark), `docs/parity.md` and `README.md` (a link is not trashed; the marks are fmc's own), `CHANGELOG.md` `[Unreleased]`, `architecture.md` (the marks, the bulk run, the Trash amendment), `rules.md` ("Writes"), the roadmap (RI-19 landed), the index, the plan archived as `done`.
- Tests: 673 pass with `KACHE=false just verify` (63 in `app::write`, 8 in `explorer`, the key and tree and footer tests).

## Owner checks

These need the owner's hand or a real terminal.

1. **Empty or restore these Trash items.** The live runs of this session put them in `~/.Trash`: `katherine-solomon.md`, `remy-legrand.md`, `bezu-fache.md`, `jacques-sauniere.md` (the two scenarios of phase 4), `langdon "quote" \ back.md` (the quoted-name check, arrived intact), and `plain-test.md` (a reproduction of the link refusal, a plain file). Nothing else was trashed.
2. **Decide the link route (plan-00014 D16).** A link cannot go to the Trash through Finder. The options: (1) `NsFileManager` for links only, which the crate's table says gives no Put Back (not checked here); (2) keep the refusal, which is the current state; (3) another route. Until you pick, `D` on a link says so and moves nothing.
3. **Check the mark in a real terminal.** The `✓` is drawn in `accent_soft` (`#c7d2fe` dark) in the bar cell. Check it in Ghostty and Terminal.app; the Rex capture shows the glyph but not the colour.
4. **The first delete from a terminal.** The live runs did not show the macOS Automation prompt (fmc controls Finder). If a prompt shows on your machine, allow it, then check that Finder's Put Back restores a test file.
5. **The copy footer.** `copying 2 of 5…` is not seen in the live captures: a copy of a few small files finishes in milliseconds. Check it on a large folder in a real terminal if the status matters to you.

## Decisions the agent took (cheap to undo; each has its record)

- The clipboard holds a list of paths, each with its device and inode (plan-00014 D11; `architecture.md`, "The clipboard is fmc's own").
- Every bulk write runs as one queue on the worker, one item per job, and the Trash joins it, so a single delete is a run of one (D12; `architecture.md`, "A bulk run goes to the worker one item at a time").
- The marks clear only when a run lands every item; a run that stops keeps them (D13).
- `-` clears the marks; hiding dotfiles keeps them, because the confirm names every marked entry (D14).
- A mark is `✓` in `accent_soft`, in the bar cell, in place of `▎`; a marked and selected row keeps its background (D3).
- A link refuses the Trash before the confirm (D16, see owner check 2).
- Findings 1 and 2 of the phase 4 review are a recorded `shortcut:` in `services/write.rs` (`trash_with`): a same-user process can swap an entry or a parent between the checks and Finder's call. No descriptor-based Trash call is available.

## Validation

- `KACHE=false just verify` green at each commit (`19d119d`, `b669168`, `4c7eac9`, `df88716`), with `cargo build` and the 672 and 673 test counts recorded in the commit runs.
- Live Rex runs, each captured and checked: marks at 120x40 and 45x30 (`b669168`); copy of three into a folder, with `copy` and `copy 2` names and no overwrite, at 120x40 and 45x30 (`4c7eac9`); the delete of two marks at 120x40 and 45x30, each trashed through Finder (`df88716`); the link refusal in the rebuilt app (the link stays, the footer names it).

## Where things live

- The plan: `.agents/docs/archive/plan-00014-multi-select.md` (done).
- The records: `architecture.md`, "Marks are a set of paths in the browser", "A bulk run goes to the worker one item at a time", and the amended "The clipboard is fmc's own" and "Deleted entries go to the Trash through Finder".
- The rules: `rules.md`, "Writes" (the bulk run, the Trash on the worker, the link refusal).
- The roadmap: RI-19 landed; the link route is the open question.
