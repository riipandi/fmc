---
status: in-progress
created: 2026-11-27T16:24:34+07:00
updated: 2026-11-27T17:04:00+07:00
phases:
    - id: 1
      title: The write layer (no UI)
      status: committed (047a58b)
      note: the write layer, 33 tests; trash adds 5 crates on macOS
    - id: 2
      title: The overlay widgets, a context menu and a dialog
      status: committed (17aefe0)
      note: Space opens the menu; the dialog is captured by FMC_OVERLAY in debug builds; items close only
    - id: 3
      title: Create, rename, and delete from keys, the menu, and right-click
      status: committed
      note: keys a, A, r, D, Delete; the menu and right-click; trash refuses a non-UTF-8 name and a swapped entry; the Finder method is kept (architecture.md)
    - id: 4
      title: Copy, cut, and paste
      status: pending
      note: —
    - id: 5
      title: Docs sync, ledger close, handover
      status: pending
      note: —
    - id: 6
      title: Final validation
      status: pending
      note: —
---

## Open the file-manager phase — create, rename, copy, move, and delete from the tree, with a context menu and dialogs

### Objective

RI-4 (`roadmap-00001-post-parity.md`) turns fmc from a read-only browser into a file manager for the five basic operations: create (file or folder), rename, copy, move (cut and paste), and delete. They are reached from keys in the tree, from a context menu (a key or a right-click on a row), and they ask through a dialog where a name or a confirmation is needed. The plan does not cross into editing file contents, permanent delete, undo history, or operations outside the browsed root.

### Current state (verified in the tree, 2026-11-27)

- **fmc is read-only by rule.** `AGENTS.md` ("Read-only until parity is closed") and `rules.md:13` forbid any write to the user's files until the owner opens that phase. plan-00001 closed parity (archived); approving this plan is the owner opening the phase.
- **OpenTUI 0.9.9 has no context-menu, dialog, or modal widget.** A search of `opentui-0.9.9/src` and `examples/` for `menu`, `dialog`, `modal`, `popover`, `overlay` finds only the debug console overlay (`src/console.rs:450`) and the debug-stats corner. The building blocks are there:
  - `Widget::absolute()` and `Widget::z_index()` (`src/widget/mod.rs:970`, `:1005`): an absolutely placed view above the page. The console overlay is built this way: `View::new(app).absolute().z_index(i32::MAX).bg(…)` (`src/console.rs:450`). fmc already uses the same pair for the scrollbar (`src/ui/scrollbar.rs:26-32`) and the rendered-Markdown signs (`src/ui/markdown_signs.rs:29-35`).
  - `Select` (`src/widget/select.rs`): a keyboard and mouse list with `options`, `selected`, `on_change`, `on_select` (`:357-445`) and its own colours: a ready menu body.
  - `Input` (`src/widget/input.rs:24-92`): a one-line editor with `value`, `set_value`, `on_submit`, for a name field.
  - `Painter::color_matrix` (`src/widget/painter.rs:207-218`): dims what is painted under it; its own doc example is "the page behind a dialog" (a scrim).
  - Right-click is parsed: SGR button 2 becomes `MouseButton::Right` (`src/input/parser.rs:322`, `src/input/event.rs:220-226`). fmc handles only `MouseButton::Left` today (`src/app/mouse.rs:19-51`, `src/app/layout.rs:149-201`, `src/app/mod.rs:397-422`).
- **fmc already has a typed prompt.** The finder's prompt is drawn in the footer by `prompt_segments` (`src/ui/find.rs:10`, used by `src/ui/footer.rs:14`), and the key parser knows a typing state (`State.typing`, `src/keys/parser.rs:161-178`).
- **Free keys in the tree.** The tree table (`src/keys/parser.rs:222-240`) binds Return, `l`, `h`, `-`, `p`, `/`, `[`, `]`, the boundary keys, and the list moves; the global table (`:147-183`) binds Tab, Esc, `?`, `.`, `w`, `q`, `m`, `d`, `y`. `a`, `r`, `c`, `x`, `n`, `s`, `o` are bound only in the diff body (`:193-200`), so they are free in the tree. `Delete` is unbound.
- **Background work has a path.** `services::spawn` (`src/services/mod.rs:73`) runs a job on a worker and posts an `Event` back; the poll loop refreshes the tree when the directory changes (`src/app/poll.rs`).
- **Nothing writes to the user's files.** `grep -rn "fs::write\|fs::rename\|fs::remove\|create_dir" src` finds only `src/services/git.rs:261-300`, which removes fmc's own temporary index copy.

### Decisions

Decisions marked _(owner)_ were the agent's recommendations; the owner approved the plan as written on 2026-11-27 ("Commit, push, lanjut eksekusi"), which takes D3, D4 (the `trash` dependency), and D5 as written and opens the file-manager phase.

- **D1 (2026-11-27): the menu and the dialog are fmc widgets on OpenTUI primitives.** An absolute view at a high z-index holds a `Select` (menu) or a bordered box with text, an `Input`, and buttons (dialog); a scrim component dims the page with `color_matrix` while a dialog is open. This is how OpenTUI's own console overlay is built, and fmc's scrollbar already uses the same pair. Cost: fmc owns the placement (flip at the screen edges, the compact tier) and the focus trap. Rejected: drawing the prompt in the footer only, as the finder does: it fits a name but not a delete confirmation with the list of paths, and a context menu needs a floating list. Rejected: waiting for an upstream widget: none is planned in the locked source.
- **D2 (2026-11-27): the operations.** New file, new folder, rename, copy, cut, paste, delete. Each acts on the selected row; paste goes into the selected folder, or the folder of the selected file. Duplicate is paste of a copy into the same folder. Out: multi-select (D7), permanent delete, chmod, symlink creation.
- **D3 (2026-11-27) _(owner)_: the keys, in the tree only.** `a` new file (a name ending in `/` makes a folder, as nvim-tree and yazi do), `A` new folder, `r` rename, `c` copy, `x` cut, `v` paste, `D` (Shift+d) and `Delete` delete (`d` stays the changes screen; the parser sees `shift`, `src/keys/mod.rs:16`), and `Space` opens the context menu on the selected row (free in the tree; the reader's `Space` pages and stays). Right-click on a row selects it and opens the menu at the pointer. Rejected: `y`/`p` for copy and paste as yazi does: `y` copies the path and `p` toggles the preview (`src/keys/parser.rs:183`, `:236`).
- **D4 (2026-11-27) _(owner)_: delete goes to the macOS Trash through the `trash` crate (5.4.0).** It is recoverable with Finder's Put Back, which stands in for undo. The confirm dialog names the entries and defaults to Cancel. Cost: a new dependency (owner approval, `rules.md` "Dependencies"); phase 1 measures its crate count. Rejected: `std::fs::remove_*`: not recoverable. The method is Finder through `osascript`, the crate's default (owner, 2026-11-27, after the phase-3 security review): Put Back needs it, the crate passes the path as an argument with no shell and escapes `"` and `\`, and the first delete asks macOS for Automation permission to control Finder. The per-delete subprocess is the cost; the record is `architecture.md`, "Deleted entries go to the Trash through Finder".
- **D5 (2026-11-27) _(owner)_: never overwrite.** Create, rename, and move refuse an existing name with a notice and keep the dialog open for a new name. Paste into a folder that has the name adds `copy`, `copy 2`, … before the extension, as Finder does. Rejected: a Replace / Keep both / Cancel dialog: more surface for the first cut; it can follow.
- **D6 (2026-11-27): safety rules for every write.** Every target path stays inside the browsed root after joining (no `..` escape); a name is one path segment, not empty, not `.` or `..`, with no `/` or NUL; a symlink is renamed, moved, copied, and trashed as the link, never followed; a folder copy recurses without following links and refuses to copy a folder into itself; a rename across volumes falls back to copy then trash of the source. Existence is checked right before the call; the race with another process is accepted and recorded (`shortcut:` comment) because std has no no-clobber rename without `libc`. A `security-review` pass runs before phases 1, 3, and 4 claim done.
- **D7 (2026-11-27): one entry at a time.** No multi-select in this plan; it needs a selection model in the tree and the finder and is its own roadmap entry.
- **D8 (2026-11-27): copy and move run on the worker.** A large folder copy must not block the frame. The footer shows `copying …` while it runs; a second paste waits. Create, rename, and trash run inline (one syscall each).
- **D9 (2026-11-27): after a write, the tree reveals the result.** A created, renamed, or pasted entry is selected; after a delete, the next row is. The existing reveal path does it.
- **D10 (2026-11-27): the read-only rule is amended, not removed.** `AGENTS.md`, `rules.md`, and `architecture.md` say the file-manager phase is open for the operations in D2, inside the browsed root; anything else that writes still needs the owner. fmc still never writes its own settings.

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The opened phase | `AGENTS.md`, `rules.md` ("Read-only") | D10: what may write, and the root boundary |
| The write rules | `rules.md`, a "Writes" section | D5, D6: never overwrite, one segment, links not followed, root only |
| The overlay | `architecture.md`, a record | D1: absolute view, z-index, scrim, focus trap, tier placement |
| The keys | `docs/keymaps.md`, the `?` page, `--help` | D3 |
| The dependency | `Cargo.toml` comment, `rules.md` "Dependencies" | D4 |
| Components | `DESIGN.md` | the menu and the dialog: colours, border, sizes per tier |

### Execution rules

- Atomic commit per phase; each phase names its message. The owner's go-ahead per phase authorizes that phase's commit only.
- Work on the branch `feat/write-operations`; the work reaches `main` through a PR.
- New dependency: `trash` only, and only with the owner's approval of D4.
- Tests that write use a temporary directory; nothing touches the repository tree or the owner's files. Sample names come from Dan Brown and Harry Potter.
- `just verify` green at every commit; a live run through Rex in a new tab of the agent's session for phases 2 to 4.

### Phases

1. **The write layer (no UI).** `src/services/write.rs`: `create_file`, `create_dir`, `rename`, `copy`, `move_to`, `trash`, the name check, the root check, the `copy N` naming, each returning a typed error the UI can show. Measure `trash`'s crate count and record it. Validation: unit tests in a temp dir for each operation, each refusal in D5 and D6 (existing name, `..`, `/` in a name, a link to outside the root, a folder into itself, a cross-volume move simulated by the fallback path); `security-review` on the module; `just verify`. _Commit: `feat(services): the write layer for the file-manager phase`._
2. **The overlay widgets.** `src/ui/menu.rs` (a `Select` in a bordered absolute box, placed at a cell and flipped to fit the screen) and `src/ui/dialog.rs` (title, message, optional `Input`, buttons, the scrim). Palette tokens from the existing set. The compact tier draws the dialog across the width minus the border, the menu as a bottom sheet. Keys while open: arrows, `j` `k`, Return, Esc; Tab between field and buttons; everything else is swallowed. Validation: unit tests for placement at the four edges and the three tiers; a Rex capture of each at 120×40, 70×24, and 45×30 with a test harness key that opens them. _Commit: `feat(ui): a context menu and a dialog over the page`._
3. **Create, rename, and delete.** The keys in D3, `Space`, and right-click on a tree row; the menu lists the actions that apply to the row (a folder and a file differ; paste is greyed out until something is copied); the dialogs call the write layer; the tree reveals the result (D9); errors show in the dialog or the footer. Validation: tests through the model for each action and each refusal; a Rex run that creates `langdon/` and `robert-langdon.md`, renames it to `sophie-neveu.md`, and trashes it, with the tree captured after each step; `security-review`. _Commit: `feat(tree): create, rename, and delete to Trash`._
4. **Copy, cut, and paste.** An fmc-internal clipboard of one path and its mode; paste on the worker (D8) with the footer status; the `copy N` naming; a cut entry is dimmed in the tree until pasted or Esc clears it. Validation: tests for copy and move of a file, a folder with a link inside, a paste into the same folder, a folder into itself refused; a Rex run that copies a folder of 2 000 files and keeps scrolling while it runs; `security-review`. _Commit: `feat(tree): copy, cut, and paste`._
5. **Docs sync, ledger close, handover.** `AGENTS.md`, `rules.md`, `architecture.md` (D1, D6, D10), `rust.md` if a contract changed, `DESIGN.md` (menu and dialog), `docs/keymaps.md`, `docs/usage.md`, `README.md`, `CHANGELOG.md` `[Unreleased]`, the roadmap (RI-4 landed), the index, this plan archived, a handover. Validation: each doc claim checked against the code; `rumdl` green. _Commit: `docs: the file-manager phase and its write operations`._
6. **Final validation.** The matrix below over the tree as it stands. _Commit: none if green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Gates | `just verify` (format, check, clippy, nextest, rumdl, licenses) | All green |
| Write layer | `cargo nextest run services::write` | Every operation and every refusal in D5, D6 |
| Overlay placement | unit tests in `ui::menu`, `ui::dialog` | In bounds at every edge and tier |
| Model flows | tests through the model for each key and the menu | The right call, the reveal, the notice |
| Live run | Rex at 120×40, 70×24, 45×30 | Menu, dialog, scrim, and the tree after each write, captured |
| Right-click | Rex live run; Ghostty and Terminal.app by the owner | The menu opens at the pointer, or the key is the fallback |
| Security | `security-review` on phases 1, 3, 4 | No path escapes the root; links are not followed |
| Parity | n/a: Scouter is read-only and has no writes; the menu and dialog are post-parity features | — |
| E2E | n/a: no E2E harness; the Rex runs stand in | — |

### Explicitly out of scope

- **Multi-select and bulk operations** (D7): a roadmap entry of their own.
- **Permanent delete and emptying the Trash**: Trash is the only delete.
- **Undo history**: Trash's Put Back covers delete; rename and move have no undo in this plan.
- **Writes from the finder or the changes screen**: the tree only.
- **Editing file contents**: fmc stays a reader.
- **The system clipboard for files** (Finder's copy and paste between apps): the clipboard is fmc's own.
