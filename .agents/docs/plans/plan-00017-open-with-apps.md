---
status: in-progress
created: 2026-11-28T14:48:05+07:00
updated: 2026-11-28T14:57:39+07:00
phases:
    - id: 1
      title: The launch service
      status: pending
      note: —
    - id: 2
      title: RI-14, open with the default app
      status: pending
      note: —
    - id: 3
      title: RI-15, the editor setting and the folder in the editor or Finder
      status: pending
      note: —
    - id: 4
      title: RI-16, Open With
      status: pending
      note: —
    - id: 5
      title: Docs sync, agent docs sync, ledger close, handover
      status: pending
      note: —
    - id: 6
      title: Final validation and live run
      status: pending
      note: —
---

## Open files and folders in other apps — RI-14, RI-15, and RI-16 on one launch service

### Objective

The owner parked three requests on 2026-11-27 and asked for their plan on 2026-11-28: open a file fmc cannot show with the default app (RI-14), open the browsed folder in Finder or a configured editor (RI-15), and an Open With picker (RI-16) (`roadmap-00001-post-parity.md:190-206`). After this plan, fmc hands a file or folder to macOS through `/usr/bin/open`, lists the apps that open a file through one constant JXA script, and offers each action by key and by the context menu. fmc still writes nothing new: the launched app owns whatever it does with the file.

### Current state (verified in the tree, 2026-11-28)

- **The binary notice has no action** — `src/ui/binary.rs:10` (`HINT`: "fmc only displays UTF-8 text and images."), its lines are `selectable(false)` (`:36`), and nothing in `src/app/mouse.rs` wires it. It shows for a regular file with a NUL in its first MiB (`src/services/fs.rs:84-92`, `src/app/reader.rs:207-214`). Not-UTF-8 text, oversize or unknown images, and non-regular files are footer errors instead (`src/services/fs.rs:46,92,100-118`; `src/app/tree.rs:165-169`).
- **The context menu is write-only** — `src/app/write.rs:139-186` (`menu_rows`): New file, New folder, Rename, Copy, Cut, Paste, Duplicate, Delete. A row is an `Act` (`:30-39`) run by `start` (`:203`). `src/ui/menu.rs` sizes the box to its items (`size`, `:29`) and has no scrolling.
- **Free keys** — `o`, `O`, `e`, and `E` have no default row in the tree or reader contexts (`src/keys/bindings.rs:276-438`); changes binds `o`/`O` (`:318-319`). `Ctrl+o` (`:288`) and `w` (`:296`) are taken everywhere. A new action needs an `Action` variant (`src/keys/mod.rs:36-120`), an `ACTION_NAMES` entry (`bindings.rs:197-266`), a `KEY_SECTIONS` row, and a line in `docs/keymaps.md` (`keys/tests.rs:295-306`).
- **Subprocesses** — fmc spawns only `git` (`src/services/git.rs:420`, argv only, stdin null, output piped, 15 s timeout and 8 MiB cap in `capture`, `:447-488`) and, through the `trash` crate, `osascript` (`src/services/write.rs:221-223`). No shell anywhere. Work runs on `services::spawn` (`src/services/mod.rs:149`), never on the UI thread (`rust.md:22`).
- **Settings** — a key joins `KEYS` (`src/settings.rs:92`), `Settings`, `parse`, and the schema (`schema()`, tests at `src/settings_tests.rs:350-364`); a page row is optional (`file_associations` has none).
- **Footer messages** — `Message { text, error }` (`src/ui/footer.rs:43-46`), cleared after 2 s (`src/app/mod.rs:554-562`).
- **Scouter has no open-externally action** — Scouter 434d000 spawns only `pbcopy` (`src/core/clipboard.ts:28`) and git (`src/diff/git.ts:54`); no `open`, `EDITOR`, or `xdg-open`. Every action here is a new deviation.
- **macOS behavior, measured on this Mac (2026-11-28)** — `open` exits 1 with a message on stderr for a missing file or app; a path starting with `-` is read as a flag unless `--` comes first. A JXA script calling `NSWorkspace.URLsForApplicationsToOpenURL` and `URLForApplicationToOpenURL` lists the apps for a file in 0.06 to 0.13 s; `NSFileManager.displayNameAtPath` gives Finder's name; a nil `NSURL` must be checked with `isNil()`. Arguments passed after the script reach `run(argv)` intact, including quotes and `$(…)`, with nothing executed.

### Decisions

- **D1 (2026-11-28): every launch is `/usr/bin/open` with argv, never a shell.** `services/launch.rs` builds `["-a", app, "--", path]`, `["-R", "--", path]`, or `["--", path]` and runs it on a worker with stdin null and stdout and stderr captured, so nothing reaches the TUI. The path is always absolute and comes after `--`. `capture` moves from `services/git.rs` to `services/mod.rs` (git keeps its call), so both use the same timeout and cap. Without `-g`, the app comes to the front, as a Finder double-click does. Rejected: `std::process::Command::new(app_cli)` (`zed`, `code`) — a CLI is often missing from `PATH` (here, `code` and `cursor` are), and `open -a` goes through Launch Services.
- **D2 (2026-11-28): Open With lists apps through one constant JXA script.** The script text is a `const` in `services/launch.rs`, run as `osascript -l JavaScript -e <script> -- <path>`; the path reaches it only as `argv[0]`. It prints one tab-separated line per app: default mark, display name, bundle id, app path. Rejected: a Launch Services binding (`core-foundation`, `objc2`) — a new dependency; `lsregister -dump` — slow and undocumented.
- **D3 (2026-11-28): the list shows apps from the application folders only.** Paths under `/Applications`, `/System/Applications`, `~/Applications`, and `/System/Library/CoreServices/Applications` stay; helpers and caches (a Chrome for Testing under `~/Library/Caches` appeared in the measurement) are dropped. The default app comes first, the rest by name. An app is launched by its path (`open -a <app path>`), not its display name, so two apps with one name stay apart.
- **D4 (2026-11-28): a launch reports in the footer.** Success: `Opened <name> in <app>.` (2 s, as other messages). Failure: the first line of `open`'s stderr, as an error. A launch never blocks input: the worker sends an `Event` back.
- **D5 (2026-11-28): the target is the selected row or the shown document.** In the tree, the selected file or folder; in the reader, the shown file. A link is handed to `open` as the link path, which macOS resolves; fmc does not follow it itself. The target is always inside the browsed root, because the tree and the reader only reach entries there.
- **D6 (2026-11-28): the actions are parity deviations, recorded once.** `architecture.md` gets one record for the launch service and the three actions, and `docs/parity.md` one line, naming the baseline (v0.3.4, `434d000f`). `rules.md` gets a "Launches" section: only `/usr/bin/open` and `/usr/bin/osascript` with the constant script, argv only, `--` before a path, no shell.
- **D7 (2026-11-28): no new dependency.**
- **O1 (owner, 2026-11-28: (a)): the keys.** (a) `o` opens the target with the default app (RI-14); `O` opens the Open With picker (RI-16); `e` opens the browsed root folder in the editor (RI-15); `E` reveals the target in Finder (`open -R`) — in the tree and reader contexts, rebindable in `keymap.json` [recommended: the four free letters, `o` as in `open`, `e` as in editor, and the shifted form for the wider choice]. (b) Only menu rows, no default keys. (c) Other letters, named by the owner.
- **O2 (owner, 2026-11-28: (a)): where RI-14 applies.** (a) `o` works on any file or folder, and the binary notice's hint adds `Press o to open it in the default app.` [recommended: one rule for every file; a folder opens in Finder]. (b) `o` works only on the binary notice and the footer refusals.
- **O3 (owner, 2026-11-28: (a)): the editor setting.** (a) `editor`: a string, an app name (`Zed`) or an app path (`/Applications/Zed.app`), passed to `open -a`; empty (the default) means `e` opens the folder in Finder. Not on the Settings page, as `file_associations` [recommended: `open -a` takes both forms, and an empty default needs no guess]. (b) A Settings-page row that picks from the installed editors. (c) A command line (`zed {path}`), which needs a shell or a parser for quoting.
- **O4 (owner, 2026-11-28: (a)): the context menu rows.** (a) A file or folder row adds an "Open" group above the write rows: `Open` (o), `Open With…` (O, files only), `Show in Finder` (E); empty space adds `Open Folder in Editor` (e) [recommended: each key has a row, as the write actions do]. (b) No menu rows.
- **O5 (owner, 2026-11-28: (a)): the picker's size.** (a) The picker is the context menu filled with app rows, and the menu gains scrolling (the selection moves past the last visible row and the rows follow), so a long list (19 apps for a `.md` file here) fits any terminal [recommended: one overlay to learn, and the scrolling helps the menu on a short compact screen too]. (b) The list stops at the rows that fit, with no scrolling.

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The launch rules | `rules.md`, a new "Launches" section | `/usr/bin/open` and the constant JXA script only, argv only, `--`, no shell, worker thread |
| The reasoning and deviation | `architecture.md`, one record; `docs/parity.md`, one line | D1 to D7 and O1 to O5, with the rejected alternatives and the Scouter baseline |
| The Rust contract | `rust.md`, "Contracts and traps" | `capture` is shared; the JXA script is a `const` and paths go in argv; a nil `NSURL` is checked in the script |
| The keys | `docs/keymaps.md`, `KEY_SECTIONS` | `o`, `O`, `e`, `E` per O1 |
| The setting | `docs/configuration.md`, `settings::KEYS`, `schema()` | `editor` per O3 |
| The roadmap | `roadmap-00001-post-parity.md` | RI-14, RI-15, RI-16 `planned` now, `landed` at the close |

### Execution rules

- **The owner chose every recommendation and gave the go-ahead for all phases in order (2026-11-28)**; each phase's go-ahead authorizes its one commit with the message named.
- **Every commit ends with the trailer** `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. No `Scouter-Base` trailer: nothing is ported.
- **No new dependency; no new write.** A launch hands a path to macOS; fmc writes no file.
- **Tests never launch a GUI app.** The launcher is a function the tests stand in for (the pattern of the Trash in `app/write_tests.rs`); the argv builder and the JXA output parser are pure and tested directly. One test runs the JXA script on a temporary `.txt` file and checks only the output's shape (macOS CI has `osascript`).
- **Branch:** `feat/open-with-apps`, from `main`.

### Phases

1. **The launch service** — `services/launch.rs`: `open_args(target, how)` (pure: `Default`, `App(path)`, `Reveal`), `run_open`, the `APPS_SCRIPT` const and `parse_apps` (pure, D3 filter and order), `list_apps`; `capture` moved to `services/mod.rs`. Tests: argv for each `how`, a path starting with `-` comes after `--`; `parse_apps` on a captured sample (default first, cache helpers dropped, a nil row skipped); the script on a temporary `.txt` returns well-formed lines or `(none)`. Validation: `cargo test launch`; `cargo test git` (the moved `capture`); clippy. _Commit: `feat(launch): hand files to macOS through open and list their apps`._
2. **RI-14, open with the default app** — `Action::OpenDefault` on `o` (per O1/O2) in tree and reader; the binary notice hint line; the menu's `Open` row (per O4); the footer messages (D4). Tests through `TestApp` with the launcher stood in: the selected file, the shown document, a folder, a failure's message. Validation: `cargo test`; the `docs/keymaps.md` tie test. _Commit: `feat(open): open the selected file with the default app`._
3. **RI-15, the editor setting and Finder** — `editor` in `settings.rs` (`KEYS`, parse, schema, per O3); `Action::OpenInEditor` on `e` (the root; Finder when `editor` is empty) and `Action::RevealInFinder` on `E`; menu rows. Tests: parse and schema for `editor`; `e` with and without an editor; `E` on a file and on a folder. Validation: `cargo test`. _Commit: `feat(open): open the folder in the editor or Finder`._
4. **RI-16, Open With** — `Action::OpenWith` on `O`: a worker lists the apps, the menu opens with one row per app (default marked), Enter launches the picked app by path; menu scrolling per O5 (`src/overlay.rs`, `src/ui/menu.rs`); no apps gives a footer notice. Tests: the menu's rows from a stood-in list; scrolling keeps the selection visible at 40 and 120 columns; Esc closes without a launch. Validation: `cargo test`; a live run (Rex tab) of the picker drawn at compact and wide sizes, where the live run closes the picker with Esc, so nothing launches. _Commit: `feat(open): Open With picks an app for the file`._
5. **Docs sync, agent docs sync, ledger close, handover** — `docs/usage.md`, `docs/keymaps.md`, `docs/configuration.md`, `docs/parity.md`, `CHANGELOG.md`; `rules.md` "Launches", `architecture.md`, `rust.md`, `shape.md` (`services/launch.rs`); roadmap RI-14, RI-15, RI-16 to `landed`; plan archived with its index row; a handover. Validation: `rumdl fmt --check --quiet .`; every path named resolves. _Commit: `docs: open files and folders in other apps`._
6. **Final validation and live run** — the matrix below over the tree; a live run in a new Rex tab of this session: `?` lists the four keys, the menu shows the Open group, `O` draws the picker, Esc closes it. Launching a real app is the owner's check. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl fmt --check --quiet .` | No drift |
| Unit | `cargo test launch`, `cargo test settings`, `cargo test keys` | argv, parser, filter, `editor`, the key tie |
| App | `cargo test` (`TestApp` with the launcher stood in) | Each action's target, message, and failure |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Build | `cargo build` | Builds |
| Full suite | `just verify` | All green |
| Live run | Rex tab: `?`, Space menu, `O` picker, Esc | Drawn as designed, nothing launched |
| Owner | A real launch: `o` on a PDF, `e` with `editor: "Zed"`, `E`, `O` then an app | The app opens the file; focus stays sensible |
| E2E | n/a beyond the live run: no CLI change | — |

### Out of scope

- A per-extension app list in `settings.json` for Open With (the picker uses Launch Services' list).
- Changing the default app of a file type, and Finder's "Other…" app chooser.
- Opening from the changes screen and the finder (their `o` keys stay as they are).
- Any platform other than macOS (RI-6).
