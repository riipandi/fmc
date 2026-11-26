---
status: in-progress
created: 2026-11-27T09:00:00+07:00
updated: 2026-11-27T03:00:00+07:00
phases:
    - id: 1
      title: "Bug fixes: arrow keys pan the reader; Ctrl+/ focuses the shown sidebar"
      status: done
      note: "248 tests; live run at 80x24: arrow on a folder leaves the reader column unchanged; Ctrl+/ focuses the list"
    - id: 2
      title: "settings.json reader (three keys)"
      status: done
      note: "263 tests; serde not added (no derive needed, serde_json alone); live: bad file shows the notice in the footer, good file none"
    - id: 3
      title: "Enter opens and focuses; l/right only expand; click focuses first"
      status: done
      note: "268 tests; live: Enter on README opens and focuses the reader, → on a file does nothing, first click only focuses the list, second click toggles a folder; a click on a file only selects it"
    - id: 4
      title: "Auto-preview with p toggle"
      status: done
      note: "275 tests; live: holding j reads the last file, a folder selection leaves the reader, p shows Auto-preview off/on and gates it"
    - id: 5
      title: "Bracket jumps in the sidebar and the reader"
      status: done
      note: "277 tests; live: ] moves the list two rows and the reader five lines, [ goes back. Footer hint not added: the 160-column footer test pins Scouter's footer (open question)"
    - id: 6
      title: ":q command prompt"
      status: done
      note: "281 tests; live: : opens COMMAND, an unknown word shows Not a command, Esc cancels, :q exits the app"
    - id: 7
      title: "Docs sync, ledger close, handover, final validation"
      status: pending
      note: —
---

## Make browsing smoother: auto-preview, Enter to open, bracket jumps, `:q`, and a settings file

### Objective

The owner tested plan-00002 on 2026-11-27 and asked for a smoother browse flow:

- auto-preview while the list is focused
- Enter that opens the file and moves the focus to it
- `l` and `→` that only expand folders
- a first click from another pane that only focuses the list
- `[`/`]` jumps
- `:q` to quit
- a small `settings.json` for the configurable values

The owner also reported two bugs. When the plan is done, browsing previews files without a key press, and three values are configurable. fmc still never writes a settings file or a user file.

### Current state (verified 2026-11-27)

- **Tree keys.** `src/keys/parser.rs:214-225` `tree`: `return` toggles, `l`/`right` expand, `h`/`left`/`backspace` collapse. `[`/`]` are bound only in `change_list` (`:194-195`). `:` is unbound (grep finds no `":"` in `src/keys`).
- **Bug: an arrow in the tree also pans the reader.** This is an owner report: `→`/`←` on a folder expands or collapses it and also scrolls the reader sideways. Phase 1 finds the double dispatch.
- **Bug: `Ctrl+/` shows the sidebar but leaves the focus in the reader.** This is an owner report. The owner wants the focus on the list.
- **Enter.** It keeps the focus on the list under the superseded record "Enter on a file keeps focus in the sidebar" (`architecture.md`). The owner now wants Scouter's focus move (`scouter:src/app/app.ts:425`).
- **Ctrl+. in Terminal.app.** It cannot be sent, which RI-12 already noted. `.` is the fallback and needs no change.
- **Dependencies.** `serde` (derive) and `serde_json` are approved for after parity (`rules.md`), and `Cargo.toml` does not list them yet.

### Decisions

All decisions are recorded in `architecture.md` on 2026-11-27:

- "Auto-preview in the sidebar, and Enter alone opens"
- "`[` and `]` jump in the sidebar and the reader"
- "a read-only `settings.json` with three keys"
- "`:q` quits, as in Neovim"

They are listed in `docs/parity.md`. Defaults: `auto_preview` true, `sidebar_jump` 2, `reader_jump` 5. `p` toggles auto-preview for the session.

- **D1: the auto-preview debounce is about 100 ms.** A held `j` must not read every file it passes. A preview replaces the reader's document as an open does, but it does not push jump history and does not move the focus.
- **D2: a preview of a binary, an image, or a refused file shows what an open shows.** It uses the same views and the same notices.

### Execution rules

- One atomic commit per phase. Code commits carry `Scouter-Base: v0.3.4 (434d000f)`.
- `just verify` must pass before each commit. Never amend. Stage explicit paths. Do not push.
- The only new dependencies are `serde` with derive and `serde_json` in phase 2. Check their licenses with `cargo deny`.
- Keep files under 800 lines. `src/app/mod.rs` is already over that limit, so new code goes in sibling modules.
- Live runs happen in new Rex tabs. Never touch block:01a2144a-042f-76ae-a6f7-ed8ce2790cd7. Close your tabs and reset the demo repo.
- For the settings tests, use a temporary `$XDG_CONFIG_HOME`. Never touch the owner's real `~/.config/fmc`.

### Phases

1. **Bug fixes.**
   - Find why `→`/`←` in the tree also pan the reader, and fix it so that a key reaches one pane only.
   - Make `Ctrl+/` focus the list when it shows the sidebar.
   - Add tests for both.
   - Validation: a live run in which an arrow on a folder leaves the reader's column unchanged.
   - _Commit: `fix(keys): keep tree arrows out of the reader and focus the shown sidebar`._
2. **settings.json.**
   - Add `src/settings.rs`, a pure parse from a string to the three values, with every error collected.
   - The app side loads `$XDG_CONFIG_HOME/fmc/settings.json` or `~/.config/fmc/settings.json` once at start. Bad keys fall back to their defaults and show a notice once. Unknown keys are ignored.
   - Unit tests cover a missing file, an empty object, every key, a wrong type, zero, invalid JSON, and an unknown key.
   - Document the file in `docs/configuration.md`.
   - _Commit: `feat(settings): read settings.json`._
3. **Enter, `l`/`→`, and clicks.**
   - Enter on a file opens it and focuses the reader. Enter on a folder toggles it.
   - `l`/`→` on a file does nothing.
   - While another pane has the focus, a click on the list only focuses the list, and the selection does not move.
   - Update the tests and the key list.
   - _Commit: `feat(explorer): open with Enter only and focus the list on the first click`._
4. **Auto-preview.**
   - While the list is focused, a change of selection to a file previews it after the debounce. A folder leaves the reader alone.
   - `p` toggles auto-preview and shows a short notice or badge, whichever fits 80 columns. The default comes from settings.
   - Add tests for the debounce, folder rows, the toggle, and no history push.
   - Live run: holding `j` through the demo repo shows the last file, and the reader keeps the previous document while a folder is selected.
   - _Commit: `feat(explorer): preview the selected file`._
5. **Brackets.**
   - While browsing, `]`/`[` move the list by `sidebar_jump` rows and scroll the reader by `reader_jump` lines.
   - On the changes screen they still step hunks.
   - Add tests, and update the key list and the footer hints.
   - _Commit: `feat(keys): jump with brackets`._
6. **`:q`.**
   - `:` outside a prompt opens a footer prompt. `q`, `q!`, `qa`, `qa!`, and `quit` quit on Enter. `Esc` or an empty Enter cancels. Anything else shows `Not a command: <text>`.
   - Reuse the existing prompt widget.
   - Add tests and update the key list.
   - _Commit: `feat(keys): quit with :q`._
7. **Docs and validation.**
   - Update `docs/keymaps.md`, `docs/configuration.md`, `README.md`, `CHANGELOG.md`, and `rust.md`.
   - Archive this plan, update the index, and write the handover.
   - Run the full matrix: fmt, clippy `-D warnings`, `just verify`, a release build, and `cargo deny`.
   - _Commit: `docs: close the browse UX plan`._

### Explicitly out of scope

- Saving UI state and the theme override, the rest of RI-9.
- Open with the default app, open in Finder or an editor, and Open With (RI-14 to RI-16).
- Other `:` commands.
