> Point-in-time research, 2026-11-26. Source: agent research for plan-00001 (Scouter at commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`). `scouter:<path>` is relative to the Scouter repository at that commit. Other scratch paths (`<scratch>/…`) were local working copies and probes that are not kept. The tree and the locked crates win over this file.

# bjarneo/editor: study for fmc

Date: 2026-11-26. Repo cloned with `--depth 1` to `<scratch>/editor` (HEAD `43820ba`, "Release 0.3.2"). Nothing was built or run. The repo has no `build.rs`. Paths below are relative to `<scratch>/editor`.

## 0. What the project is, and what Rune is

- **editor**: "A lightweight modal editor for reading code fast and making quick edits, built on Rune." (`Cargo.toml:5`). MIT (`LICENSE:1-3`, "Copyright (c) 2026 The editor authors"). Version 0.3.2, edition 2024.
- Maturity: the GitHub repo was created 2026-11-24, has 39 stars, and is 2 days old at the time of this study. The design is still moving.
- **Rune is a native GPU-rendered Rust UI framework**, not the Rune scripting language. Source: `github.com/anomalyco/rune`, description "Rune: Native UI framework", MIT, Rust, pushed 2026-11-25 (GitHub API). Its README says it has its own renderer, text, layout and springs, and runs headless for screenshots. It is pre-release: editor pins a git rev (`Cargo.toml:13`) and patches winit, wgpu-hal, parley_engine and accesskit (`Cargo.toml:46-51`). The crate `rune` on crates.io is a different project (rune-rs scripting language, keywords `scripting`). Do not confuse the two.
- Rune README also says "We don't take pull requests" (contribution policy, not a licence issue).

## 1. Stack

### 1.1 Manifest, and what each crate does here

| Crate (editor pin) | Latest on crates.io (checked 2026-11-26) | Used for (evidence) |
| --- | --- | --- |
| `rune`, `rune-highlights` (git rev `bf3f9d4`) | n/a (git only) | GUI, layout, text shaping, and the syntax-colour lexers (80 languages). `src/highlight.rs:11`. |
| `ropey = "1.6"` (1.6.1, 2025-08) | 1.6.1 | Rope. The text lives in `Rope` (`src/buffer.rs:10`, field `text: Rope`). |
| `serde`, `serde_json` | 1.x / 1.0.154 | Config structs with `#[serde(default)]` (`src/config.rs`); IPC JSON (`src/ipc/*`); LSP JSON-RPC (`src/services/lsp/*`). |
| `toml = "0.8"` (latest 1.2.3) | 1.2.3 | `config.toml` read and write (`src/config.rs`, `src/app/settings.rs`, `src/app/session.rs`). |
| `dirs = "6"` (latest 7.0.0) | 7.0.0 | `~/.config`, `~/.local/state` paths (`src/config.rs`, `src/theme.rs`). |
| `ignore = "0.4"` (0.4.35, 2026-11-22) | 0.4.35 | Project walk honouring `.gitignore` for finder and tree (`src/services/index.rs:10`). |
| `globset = "0.4"` (0.4.22) | 0.4.22 | `.editorconfig` glob matching (`src/services/editorconfig.rs`). |
| `notify = "8"` (latest 9.0.0) | 9.0.0 | Recursive file watcher for outside changes (`src/services/watch.rs:8`, `:43`). |
| `similar = "2"` (latest 3.2.1) | 3.2.1 | Line diff for the git gutter and for agent proposals, with a time limit (`src/services/git.rs:157`, `src/app/review.rs:34`). |
| `unicode-width = "0.2"` (0.2.3) | 0.2.3 | Column width for the mouse and the text view (`src/ui/text.rs`, `src/app/mouse.rs`). |
| `unicode-segmentation = "1"` (1.13.3) | 1.13.3 | Declared; no direct use found by grep. |
| `tree-sitter = "0.25"` (latest 0.27.2) + 12 grammars (`Cargo.toml:27-39`) | 0.27.2 | Code structure only: text objects `af` `ic` `aa` and motions `]f` (`src/syntax/mod.rs:1-10`). Not used for highlighting. Node-kind name lists per language, not `.scm` queries (`src/syntax/langs.rs:76-160`). Files over 4 MiB are skipped (`src/syntax/mod.rs:5`). |
| `tempfile` (dev only) | | Tests. |

Not used, although often expected: no `clap` (arguments come from `rune::tooling::extra_args()`, `src/args.rs:13`), no LSP crate (`src/services/lsp/server.rs:1`, JSON-RPC over stdio by hand), no `regex` (its own Vim-style engine, `src/vim/regex.rs:1-6`), no `similar`-based editor diff beyond the above, no `rope` other than ropey.

### 1.2 Subsystems

- **Rendering**: Rune. `view(&App)` builds an element tree each time state changes (`docs/architecture.md`, "One frame"). Scrolling is the editor's own (`src/editor.rs:450`, `scroll_to_cursor`). Not portable to a terminal.
- **Text buffer**: ropey rope plus a char-index edit log (`src/buffer.rs:10`, `:51-68`). Marks move with edits (`src/buffer.rs:94`).
- **Syntax highlighting**: `rune_highlights` lexers (`src/highlight.rs:1-11`). Tree-sitter is not used for colours (`README.md`, "Not built yet").
- **LSP**: hand-written client, one thread for all servers (`src/services/lsp/mod.rs:1-4`). Diagnostics, definition, references, hover, completion, rename, code actions.
- **File watching**: `notify` recursive on the project root (`src/services/watch.rs:43`), plus a sleep loop (`:40`).
- **Fuzzy finder**: hand-written scorer (`src/fuzzy.rs:1-10`, `score` at `:27`), parallel above 16 Ki items. Not a crate.
- **Git**: shells out to `git` (`src/services/git.rs:82`). Always passes `--no-optional-locks` so a status read does not block the user's commit (`src/services/git.rs:80-81`). Gutter signs from `similar` against `HEAD`.
- **Project search**: ripgrep with `--vimgrep` or a plain walk (`docs/architecture.md`, "Workers").
- **Config format**: TOML, one struct per section, every field defaulted (`src/config.rs:1-30`, `#[serde(default)]`). Settings is a buffer that writes the file (`docs/configuration.md`).
- **Plugin / scripting**: none. The extension points are the IPC socket (`editor ctl`, `editor mcp`, `docs/agents.md`) and the Ex command line (`src/vim/ex.rs`).
- **Markdown / Mermaid preview**: none. `grep -i markdown|mermaid src` hits only lexer and LSP/setup text (`src/highlight.rs`, `src/services/lsp/`, `src/app/setup.rs`).

### 1.3 Crate versions behind latest

editor pins `notify` 8 (latest 9.0.0), `similar` 2 (3.2.1), `dirs` 6 (7.0.0), `tree-sitter` 0.25 (0.27.2), `toml` 0.8 (1.2.3). Their lock file is the reference. Check the MSRV of any newer version before adopting it (fmc is edition 2024).

## 2. Modal editing design

### 2.1 Modes

- `enum Mode { Normal, Insert, Visual(VisualKind), Command }` (`src/editor.rs:11-17`). `VisualKind` is Char, Line or Block (`src/editor.rs:23-27`). Labels are in `src/editor.rs:30-40`.
- The mode lives in `Editor`, not in a keymap object. Keymaps are free functions: `vim::handle_key(&mut Editor, KeyInput) -> Vec<Action>` (`src/vim/mod.rs:221`) and `classic::handle_key` (`src/classic/mod.rs:13`).
- Dispatch by mode: `vim::dispatch` (`src/vim/mod.rs:243-258`) matches `Command`, `Insert`, `Visual`, `Normal`. A `:s///c` confirm prompt is checked first (`:244`).

### 2.2 Key input and sequence parsing

- `KeyInput { code: KeyCode, ctrl, alt, shift, cmd }` (`src/editor.rs:51-58`). `KeyCode` is `Char(char)` or a named key (`src/editor.rs:60-78`).
- `KeyInput::ctrl` sets `cmd` too, so Vim bindings read `ctrl` and classic bindings read `cmd` (`src/editor.rs:46-49`, `:88-91`).
- **Pending keys, not a trie.** `VimState.pending: Vec<KeyInput>` (`src/vim/mod.rs:57`). Each key is appended and the whole buffer is re-parsed (`src/vim/normal.rs:331-350`). The parser returns `Parse::More` (keep reading), `Parse::Bad` (clear), or `Parse::Done(cmd)` (run and clear). `enum Parse<T>` is at `src/vim/mod.rs:198-202`. Esc clears (`src/vim/normal.rs:332-336`).
- Each parser is small and pure: `normal::parse` (`src/vim/normal.rs:144`), `motion::parse` (`src/vim/motion.rs:114`), `textobj::parse` (`src/vim/textobj.rs:38`), `parse_count` (`src/vim/normal.rs:103`), `parse_prefix` for `"x` and count (`src/vim/normal.rs:125`).
- Grammar: `"reg` `count` `operator` `count` `target`. `Cmd { reg, count, act }` (`src/vim/normal.rs:14-18`). `Target` is Motion, Obj (text object), Lines (`dd`, `yy`), Search (`d/pat`), or Gn (`src/vim/normal.rs:21-31`). The count multiplies across the operator and target (`mul`, `src/vim/normal.rs:117`).
- Doubling the operator letter means "lines" (`src/vim/normal.rs:288-290`). Counts are allowed everywhere, `0` only after a non-zero digit (`src/vim/normal.rs:107`).
- Ctrl chords go first (`src/vim/normal.rs:156-176`), then named keys, then single chars (`match c`, `:194-265`). Two-key prefixes (`g`, `z`, `Z`, `m`, `q`, `@`, `r`) use `need2`, which returns `More` until the second key (`:184-192`).
- Leader: a lone leader key returns `Act::Leader` (`src/vim/normal.rs:153-155`). The leader is Space or backslash (`src/vim/mod.rs:54`). The leader groups themselves are app code (see 2.4).

### 2.3 Dispatch of commands

- `enum Act` is the command set (`src/vim/normal.rs:56-112`): `Move`, `Op`, `Put`, `Insert(Ins)`, `Visual`, `Cmdline`, `Record`, `Play`, `Undo`, `Redo`, `Dot`, `App(Action)` and others.
- `exec` is one big `match` (`src/vim/normal.rs:447-589`). Changing commands set `ed.vim.dot` for `.` (`:586-588`).
- `Op` (`src/vim/ops.rs:12-23`) is applied to a `Range` (`:27-34`): `Chars`, `Lines`, or `Block`.
- `Motion` (`src/vim/motion.rs:15-80`) has about 40 variants. Each motion returns a `Reach { pos, kind, want, jump }` (`src/vim/motion.rs:89`). The `kind` (Excl, Incl, Line) decides the operator's range (`motion_range`, `src/vim/normal.rs:376-398`). This is the Vim exclusive-motion rule.
- `TextObj { inner, kind }` (`src/vim/textobj.rs:13`, `:24`) covers word, sentence, para, pairs, quotes, tag and tree-sitter (`function`, `class`, `param`, `comment`).

### 2.4 Keymap representation, which-key, command palette, Ex

- **No user keymap file.** Bindings are code: match arms in `normal::parse`, `insert`, `visual`, `classic`. The only settings are `keys.map = "neovim" | "classic"` and `keys.leader = "space" | "backslash"` (`src/config.rs:101-112`, `:265-270`). `docs/configuration.md` does not offer custom bindings.
- **Leader menu (which-key style)**: static tables `LEADER`, `LEADER_GIT`, `LEADER_LSP`, `LEADER_AGENT` (`src/app/keys.rs:59-81`). The popup reads them (`src/ui/overlays.rs:18-25`). Dispatch is a separate `match (group, c)` in `leader_key` (`src/app/keys.rs:207-270`). The display table and the dispatch match are two copies that must agree. There is no test tying them together.
- **Key list (`F1`, `SPC ?`, `:Keys`)**: parses `docs/keymaps.md` at compile time (`src/app/key_list.rs:6` `include_str!`, `:20` `rows()`). The doc is the single source of truth for help, and a Markdown table is the data format. Whole-word search (`src/app/key_list.rs:85`).
- **Command palette (classic)**: `COMMANDS: &[(label, id)]` (`src/app/picker.rs:60-79`), then `run_command(id)` maps each id to an `Action` (`src/app/picker.rs:309`).
- **Ex commands**: one `match` on the command name with aliases (`src/vim/ex.rs:60-247`). Unknown names give `E492` (`:247`). Ranges parse as `.`, `$`, `'m`, `/pat/`, offsets (`src/vim/ex.rs:298-386`).
- **Key routing order** (`src/app/keys.rs:107-190`): setup, F1, overlay (leader, picker, menu), agent proposal, command line, file tree (when focused), Settings, no-document keys, popup, F2 (rename), then the keymap. This is the same kind of list as Scouter's dispatch order (inventory §4).

### 2.5 Undo and redo

- `Edit { at, removed, inserted }` is one change; `Txn { id, edits, before, after }` is one undo step (`src/buffer.rs:51-68`).
- `begin(cursor)` / `commit(cursor)` wrap a step (`src/buffer.rs:366-390`). An empty transaction leaves no step. A new commit clears redo (`:383`). Every change without an open transaction gets its own step (`:396-412`).
- Caller rule: `begin`/`commit` in the keymap layer (`src/vim/mod.rs:315-330`), so one command is one step, and an Insert session is one step (`src/vim/normal.rs:592-605`).
- Cap: `UNDO_LIMIT = 1000` (`src/buffer.rs:98`).
- `undo()` and `redo()` replay the edit list and return the cursor position (`src/buffer.rs:451-489`).
- Persistence: `history()` and `restore_history()` (`src/buffer.rs:537-559`). A history is applied only when the text hash matches (`:543`). The hash is `std::collections::hash_map::DefaultHasher` (`:527-534`). Its algorithm is not guaranteed stable across Rust releases. A mismatch fails safe: the history is dropped, not applied. Use a stable hash (FNV or blake3) if history must survive a toolchain change.

### 2.6 Registers and clipboard

- `Registers { unnamed, named a-z, yank 0, numbered 1-9, small -, inserted . }` (`src/editor.rs:225-236`). `get` and `set` at `:240-290`. Uppercase appends (`:255-262`). Deletes of a line or more shift `1`-`9`, smaller deletes go to `-` (`:275-284`).
- `"+`, `"*` and the unnamed register also send text to the system clipboard (`src/vim/mod.rs:348-362`, `Action::Copy`).
- Paste is an `Action::Paste` answered later (`src/vim/normal.rs:626-630`).

### 2.7 Macros and dot repeat

- `q{reg}` records every key into `rec_keys` (`src/vim/mod.rs:226-229`). `@{reg}` replays them with `replaying` set so they do not re-record.
- `.` repeats `Dot { cmd, keys }` (`src/vim/mod.rs:158-163`). Insert keys are stored with the command.

### 2.8 Separation from the GUI

- Clean. `src/lib.rs` exposes `buffer`, `classic`, `editor`, `syntax`, `vim`. `src/lib.rs:1-3` says the core has no window code. The binary (`src/main.rs`, `src/app`, `src/ui`) draws it.
- Contract: a keymap changes text and cursor through `&mut Editor` and returns `Vec<Action>` for I/O (`src/editor.rs:116-179`). Save, open, shell, clipboard, LSP and quit all come back as actions. Tests drive the core without a window (`src/vim/tests.rs`, `keys("...")` helper at `:59`).
- Test count: about 250 `#[test]` attributes across `src` (grep, 2026-11-26).

## 3. Conflicts with Scouter parity

Do not reuse the Neovim grammar as-is. It mutates text, and several keys mean something else in Scouter. Scouter's keys are in `.agents/docs/research/scouter-inventory.md` §4.

| Key | Scouter (inventory §4) | editor, Neovim keymap |
| --- | --- | --- |
| `Tab` | Switch pane (`src/app/app.ts:771`) | Jump forward in list (`src/vim/normal.rs:175`) |
| `?` | Help overlay | Backward search start (`src/vim/normal.rs:264`) |
| `q` | Quit | Macro record (`src/vim/normal.rs:232-233`), so it waits for a register |
| `w` | Toggle wrap | Word motion |
| `d`, `y` | Changes screen, copy path or selection | Delete and yank operators |
| `m` | Toggle Markdown rendered/source | Set mark (`src/vim/normal.rs:235`) |
| counts | Only before `G` in the reader; ignored in the file list (inventory §4.3) | Before every motion and operator |
| `x`, `p`, `u`, `.`, `J`, `r` | not bound (read-only) | Edit the buffer |

Keys that already match: `gg`, `G` with count, `Ctrl-d` and `Ctrl-u` (half page), `Ctrl-o` (jump back), `zz`, `{` and `}`, `/` and `n`/`N`.

## 4. Recommendations for fmc

Ranked by value for fmc, with the milestone where each fits. "Adopt" means reuse the idea, written fresh, since the repo is MIT and the design is small.

### Adopt

1. **Pure pending-key parser for Scouter's keys (M1).** Keep a `Vec<Key>` of pending keys and a `fn parse(&[Key], ctx) -> Parse<Command>` returning `More | Bad | Done`. The dispatcher runs `Done` and clears on `Bad`. This replaces ad hoc key matching without changing behaviour, because the table is Scouter's, not Vim's. It is testable with key-sequence tables (`src/vim/tests.rs:59` is a model). Ref: `src/vim/mod.rs:198-202`, `src/vim/normal.rs:331-350`.
2. **Explicit dispatch order (M1).** A written list of focus layers (help overlay, find bar, pane focus, reader, list). Scouter already has one (inventory §4). Ref: `src/app/keys.rs:107-190`.
3. **One key table for dispatch, `?` help and footer hints (M1).** Scouter keeps `src/core/keymap.ts` and tests the README against it. editor keeps the leader menu in two copies (`src/app/keys.rs:59` and `:207`) and has no test for that. Use one table and a test. Ref: `src/app/key_list.rs:20`.
4. **`git --no-optional-locks` (M1, one line).** The reason is in `src/services/git.rs:80-81`. Scouter polls git every 2 s, so this matters more for fmc.
5. **Atomic write: temp file beside the target, then rename (M2).** Ref: `src/services/io.rs:177-191`. Copy the permission step too. A crate such as `tempfile::NamedTempFile::persist` does the same and is fine if already in the tree.
6. **Command palette as an id table (M2, only if fmc gets a command prompt).** Ref: `src/app/picker.rs:60-79`, `:309`. Skip if fmc stays key-driven.
7. **Undo as transactions with before/after cursor, capped (M3, in-place editing).** Ref: `src/buffer.rs:51-68`, `:366-390`, `:451-489`. First check whether opentui's `EditBuffer` covers it: `<scratch>/opentui_rust/src/text/edit.rs:93-99` has grouped undo with a max depth, in the 0.2.3 checkout. If the 0.9.9 API matches, use it and skip this.
8. **Time-limited diff (M3, if fmc previews edits).** `similar` with `timeout(200 ms)` (`src/services/git.rs:157`). Only the timeout pattern is worth copying. The opentui `Diff` widget may render the result.

### Dependencies

| Crate | Latest | Where | Recommendation |
| --- | --- | --- | --- |
| `ignore` | 0.4.35 (active) | `src/services/index.rs:10` | M2, for file-manager walks and grep. Not M1: Scouter's finder uses its own noise list, and `ignore` would change results. The dependency survey already chose `std::fs` for M1. |
| `similar` | 3.2.1 | `src/services/git.rs:156` | M3 only, if edit previews need a line diff. Not for M1: Scouter feeds git patches to the Diff widget. |
| `ropey` | 1.6.1 (stale since 2025-08) | `src/buffer.rs:10` | Skip as a direct dep. `opentui_rust` 0.2.3 already depends on `ropey = "1.6"` (`<scratch>/opentui_rust/Cargo.toml:39`). Add it only if `EditBuffer` is not enough at M3. |
| `notify` | 9.0.0 | `src/services/watch.rs:8` | Skip for M1, since polling is decided. Reconsider at M2 if live refresh is wanted. |
| `toml` + `dirs` | 1.2.3 / 7.0.0 | `src/config.rs` | Skip. Scouter has no config (`docs/decision/0001`). Revisit if fmc gets a config file. |
| `globset` | 0.4.22 | `src/services/editorconfig.rs` | Skip unless `.editorconfig` support is added (M3). |
| `tree-sitter` (direct) | 0.27.2 | `src/syntax/` | Skip. opentui already brings it with its `tree-sitter` feature. |
| `unicode-width` | 0.2.3 | `src/ui/text.rs` | Already in the graph via opentui. No direct dep. |
| `clap` | | none (`rune` args) | Not from this repo. fmc keeps clap. |

### Skip (GUI-specific or covered by opentui)

- Rune, rune-highlights, the winit/wgpu patches. fmc has opentui, which covers rendering and the highlighted code view.
- Settings-as-a-buffer, the five-step setup wizard, Omarchy theme sync, font and opacity settings, the mouse context menu.
- The agent socket: `editor ctl` and `editor mcp` (`src/ipc/`, `docs/agents.md`). Out of scope for all fmc milestones listed.
- LSP client (`src/services/lsp/`). No fmc milestone needs it.
- The Vim regex engine (`src/vim/regex.rs`) and the fuzzy scorer (`src/fuzzy.rs`). Scouter's search and ranking are plain, so match Scouter's behaviour instead.
- The Vim Normal grammar as a whole (see §3).

## 5. License and reuse

- editor: MIT, "Copyright (c) 2026 The editor authors" (`LICENSE:1-3`). Copying code is allowed if the copyright and permission notice are kept. The default for fmc is to write from the ideas above.
- Rune: MIT per GitHub API. It is not in fmc's plan, so no copy is needed.
- Scouter's license is not in this study's scope; check it before copying any Scouter-derived idea.

## 6. Checks and limits

- Read: manifest, `Cargo.lock` (Rune rev, crate versions), `AGENTS.md`, `README.md`, `PRODUCT.md`, `docs/architecture.md`, `docs/configuration.md`, `docs/keymaps.md`, `src/vim/*`, `src/editor.rs`, `src/buffer.rs`, `src/app/keys.rs`, `src/app/key_list.rs`, `src/app/menu.rs`, `src/app/picker.rs`, `src/config.rs`, `src/services/{git,index,watch,io,grep,lsp}`, `src/syntax/*`, `src/highlight.rs`, `src/args.rs`.
- Not read in detail: `src/vim/insert.rs`, `visual.rs`, `ops.rs` bodies, `src/classic/multi.rs`, `src/ui/*`, `src/ipc/*`, `src/app/setup.rs` beyond the leader import (`:438`).
- Not checked: whether the Rune rev still builds, since nothing was compiled. The crates.io data is from the API on 2026-11-26. The Scouter line references come from `.agents/docs/research/scouter-inventory.md` and were not re-verified against Scouter source.
- Discrepancy found: `Cargo.toml` declares `opentui_rust = "0.2.3"`, while the brief and `.agents/docs/research/dependency-survey.md` assume `opentui` / `opentui-sys` 0.9.9. The two need to be reconciled before §4 item 7 is decided, because the EditBuffer history was checked in the 0.2.3 checkout only.
