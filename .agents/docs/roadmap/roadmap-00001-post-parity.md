---
status: living
created: 2026-11-26T04:09:25+07:00
updated: 2026-11-28T19:47:10+07:00

references: plan-00001-scouter-parity.md, research/dependency-survey.md, research/editor-study.md, research/sqltui-study.md, research/opentui-binding-risk.md
---

# Roadmap — after Scouter parity

Parked work that the owner named during the planning of `plan-00001`. None of it runs before parity closes. When an unlock fires, the entry becomes a plan.

## Status

One row per entry; the entry's own `status` line carries the evidence. Statuses: `landed` (done, with its commits or plan), `landed in part` (what is left names the follow-up entry), `planned` (a plan exists, waiting for the owner's approval or running), `next` (the owner picked it to start next, no plan yet), `parked` (waiting on its unlock). A change to an entry's status line changes its row here in the same edit.

| ID | Entry | Status | Where | Updated |
| --- | --- | --- | --- | --- |
| RI-1 | Mermaid diagrams in Markdown | landed | plan-00006 phases 1 and 2 | 2026-11-27 |
| RI-2 | The owner's Neovim grammar set | landed | plan-00006 phases 3 and 4, plan-00018, and plan-00020 phase 3 (`522fc83`): `.eex` and `.leex` read as heex | 2026-11-28 |
| RI-3 | Fuzzy file finding | landed | plan-00006 phase 5 | 2026-11-27 |
| RI-4 | Write operations | landed | plan-00012 phases 1 to 5 and the fix `872fd21` | 2026-11-27 |
| RI-5 | SQLite and DuckDB viewer | parked | owner-deferred | 2026-11-26 |
| RI-6 | Platforms beyond macOS aarch64 | parked | owner-deferred | 2026-11-26 |
| RI-7 | An alternate Vim-style keymap | parked | owner-deferred | 2026-11-26 |
| RI-8 | Event-based file watching | parked | unlock: a measured polling cost and MSRV 1.91 | 2026-11-26 |
| RI-9 | Persistent settings | landed | plan-00015 phases 1 to 4 (`3f5673c`, `64c7aea`, `d68af26`): the Settings page writes `settings.json`; plan-00020 phases 1 and 2 (`c95faf0`, `676d439`): the UI state is kept in `meta/state.json`, and the sidebar's visibility is not | 2026-11-28 |
| RI-10 | Theme catalog | landed | plan-00015 phase 2 (`64c7aea`): 16 bundled themes, the theme mode, `dark_theme`, `light_theme`, and the `themes/` folder | 2026-11-27 |
| RI-11 | Adaptive layout and a collapsible sidebar | landed | `55f9b45`, plan-00002 | 2026-11-27 |
| RI-12 | Toggle hidden files with `Ctrl+.` | landed | `81a735f`, plan-00002 | 2026-11-27 |
| RI-13 | Configurable keymap | landed | plan-00015 phase 4 (`1b37a2e`): keys in `keymap.json`; in-UI rebinding is not part of it (D11 (a)) | 2026-11-27 |
| RI-14 | Open an unsupported file with the default app | landed | plan-00017 | 2026-11-28 |
| RI-15 | Open the project folder in Finder or an editor | landed | plan-00017 | 2026-11-28 |
| RI-16 | Open With | landed | plan-00017 | 2026-11-28 |
| RI-17 | The rest of the owner's grammar set | landed | plan-00018 phases 1 to 3; plan-00020 phase 3 (`522fc83`): Lua `---` documentation comments take the documentation capture, and `.eex` and `.leex` read as heex | 2026-11-28 |
| RI-18 | Injected highlighting | landed | `1c3857e`, `d619868`, `d68a7e7`; plan-00013 | 2026-11-27 |
| RI-19 | Multi-select and bulk operations | landed | plan-00014 phases 1 to 4 (`19d119d`, `b669168`, `4c7eac9`, `df88716`); the link refusal (plan-00014 D16) waits for the owner | 2026-11-27 |
| RI-20 | The Settings page on `Ctrl+,` | landed | plan-00015 phase 3 (`d68af26`), the grouped layout `883577d`; owner checks: `Ctrl+,` in Ghostty and Terminal.app | 2026-11-27 |
| RI-21 | The heavier and rarer common grammars | parked (unlock: an owner request) | plan-00024 wave 3 and its dropped set; `research/common-grammars.md` | 2026-11-28 |

## Rendering

### RI-1 — Mermaid diagrams in Markdown

- status: landed (plan-00006 phases 1 and 2, 2026-11-27; `architecture.md`, the Mermaid records) · updated 2026-11-27

Scouter draws a `mermaid` fence as plain text (`research/scouter-inventory.md` §7), and upstream OpenTUI has no Mermaid renderer. The binding exposes a code-block renderer hook for a fence language (`research/rendering-research.md`, `markdown.rs:631` upstream). Candidate: `mermaid-text` 0.58 (flowcharts only, box drawing); `mermaid-rs-renderer` needs SVG rasterising.

**Unlock:** plan-00001 done. **First move:** a spike that renders the demo README's flowchart through the hook with `mermaid-text`, and an owner decision on sequence diagrams.

### RI-2 — The owner's Neovim grammar set

- status: landed (plan-00006 phases 3 and 4, 2026-11-27: twelve grammars are highlighted, `architecture.md`, "the owner's extra grammars"; plan-00018 for the rest, RI-17; plan-00020 phase 3 for eex and leex, `522fc83`). The injections landed in RI-18 (plan-00013) · updated 2026-11-28

The owner's Neovim config (`~/.config/nvim/lua/plugin_treesitter.lua:14-50`) adds, beyond Scouter's 16: astro, diff, eex, elixir, hcl, heex, jsdoc, json5, just, lua, luadoc, luap, query, regex, sql, svelte, vim, vimdoc, xml. On crates.io (2026-11-26): no crate for astro or eex; `tree-sitter-sequel` 0.3.13 for SQL; old crates that need an ABI check against `tree-sitter` 0.27: `tree-sitter-diff` 0.1.0, `tree-sitter-svelte-ng` 1.0.2, `tree-sitter-xml` 0.7.0; `luadoc`, `luap`, `query`, `json5` crates are barely used. Each grammar also needs a `highlights.scm` that uses the palette's capture names.

**Closed:** the candidate crates were built against `opentui` 0.9.9 and the ones that load are highlighted (plan-00018); eex is read as heex by the owner's decision (plan-00020 D9, `architecture.md`, "`.eex` and `.leex` are read as HEEx").

## Finding

### RI-3 — Fuzzy file finding

- status: landed (plan-00006 phase 5, 2026-11-27; `find_mode` and `Ctrl+F`, `architecture.md`) · updated 2026-11-27

Phase 1 copies Scouter's substring AND match and order (`architecture.md`). A fuzzy toggle could use `neo_frizbee` (1 crate, MIT, MSRV 1.89; the matcher inside fff) or `nucleo-matcher` (MPL-2.0). `fff-search` (96 crates, LMDB frecency, git signals, MSRV 1.91) only if its grep or index is wanted (`research/dependency-survey.md`).

**Unlock:** plan-00001 done and the owner asks for fuzzy matching. **First move:** a toggle in the finder that swaps the matcher, with Scouter's order as the default.

## File manager

### RI-4 — Write operations

- status: landed (plan-00012, phases 1 to 5: `047a58b` the write layer, `17aefe0` the menu and dialog, `46f13a0` create, rename, and delete, `738501b` copy, cut, and paste, `872fd21` the fix for a replaced source and the dialog hover, and the docs commit `docs: the file-manager phase and its write operations`; `architecture.md`, the records of D10, D5 and D6, and the clipboard) · updated 2026-11-27

Create, rename, move, copy, delete. Ideas from `research/editor-study.md`: atomic writes through a temp file and `rename`. Candidates: `ignore` 0.4 for walks, `trash` 5.4 for delete-to-trash; `fs_extra` is stale (2023). Every write path needs a `security-review` pass (symlinks, path traversal).

**Unlock:** the owner opens the file-manager phase (`rules.md`, read-only rule). **First move:** done, plan-00012 (2026-11-27): the operation list, the keys, the confirm and Trash rules, and the context menu and dialog built on OpenTUI's absolute views (OpenTUI 0.9.9 has no menu or dialog widget).

### RI-19 — Multi-select and bulk operations

- status: landed (plan-00014, phases 1 to 4: `19d119d` the marked set, `b669168` the keys and the mark, `4c7eac9` the bulk copy, cut, and paste, `df88716` the bulk Trash; `architecture.md`, the records of the marks and the bulk run) · updated 2026-11-27

plan-00012 acted on one row at a time. plan-00014 gives the browser a set of marked paths (`s`, `S`), and copy, cut, paste, and Trash run over it as one queue on the worker, one item at a time, stopping at the first error. The clipboard holds a list, each path with its device and inode (`architecture.md`, the records of the marks and the bulk run). Mouse selection and range keys were left out (the owner, 2026-11-27).

**Done:** the owner asked on 2026-11-27 and chose `s` and `S`; plan-00014 landed the marks, the bulk copy, cut, paste, and Trash, and a link is refused, since Finder cannot trash one. Open: whether a link gets another route to the Trash (plan-00014 D16).

### RI-5 — SQLite and DuckDB viewer

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done and the owner opens it) · updated 2026-11-26

A read-only database view for `.sqlite`/`.db` and DuckDB files. sqltui (GPL-3.0: ideas only, `rules.md`) offers a table sidebar, schema tab, paged grid with preview, multi-statement runner, autocomplete, and export (`research/sqltui-study.md`). Crates: `rusqlite` 0.43 with `bundled`, opened with `SQLITE_OPEN_READ_ONLY`; `duckdb` 1.20001 with `bundled` (a long C++ build, 103 crates) opened with `AccessMode::ReadOnly`. A cell grid needs a custom widget.

**Unlock:** plan-00001 done and the owner opens it. **First move:** measure the binary size and build time of `rusqlite` and `duckdb` bundled, then decide whether DuckDB is a cargo feature.

## Platforms

### RI-6 — Platforms beyond macOS aarch64

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done) · updated 2026-11-26

macOS x86_64 is unverified (test under Rosetta, then a `lipo` universal binary). Linux x86_64 musl built statically and ran in an Alpine container in the probe (`research/opentui-binding-risk.md` §3). Linux glibc and arm64 are untested. Windows is high risk: upstream only compile-checks it.

**Unlock:** plan-00001 done. **First move:** build the native library for `x86_64-macos` with zig and run the binary under Rosetta.

## Input and watching

### RI-7 — An alternate Vim-style keymap

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done) · updated 2026-11-26

The key table and pending-key parser (`architecture.md`) can carry a second keymap with operators, motions, and text objects (`research/editor-study.md` §4). Scouter's keys stay the default; `Tab`, `?`, `q`, `w`, `d`, `y`, `m` conflict with Vim's meanings.

**Unlock:** plan-00001 done and the owner asks for it. **First move:** a conflict table that maps each Scouter key to its Vim-mode replacement.

### RI-8 — Event-based file watching

- status: parked (unlock: polling cost measured as a problem, and MSRV at or above 1.91) · updated 2026-11-26

Scouter polls (`research/scouter-inventory.md` §12) and fmc copies it. `notify` 9 would cut idle work but raises the MSRV to 1.91.

**Unlock:** a measured polling cost on a large tree, and the toolchain at 1.91 or later. **First move:** measure CPU per poll tick on a 20 000-file tree.

## Settings and themes

### RI-9 — Persistent settings in `~/.config/fmc/settings.json`

- status: landed (plan-00015 phases 1 to 4, `3f5673c`, `64c7aea`, `d68af26`: the Settings page on `Ctrl+,` saves each change to `settings.json` through a temp file and a rename, and the theme keys are in the file; plan-00020 phases 1 and 2, `c95faf0` and `676d439`: the sidebar's width, wrap, the changes view's split, wide context, and all files, dotfiles, and the finder's matching are kept in `meta/state.json`, and the sidebar's visibility is not, `architecture.md`, "fmc remembers its UI state in meta/state.json") · updated 2026-11-28

Scouter has no configuration by design (`research/scouter-inventory.md` §15), so this is a post-parity deviation the owner asked for on 2026-11-26. fmc remembers the UI state the user changes: sidebar width, reader wrap, diff layout (unified or split), diff context (`x`), and the theme mode (`auto` follows the terminal as Scouter does, or `dark`, `light`). Path: `~/.config/fmc/settings.json` (owner's choice). Crates approved for this item: `serde` with `derive` and `serde_json` (`rules.md`, "Dependencies"). The write is fmc's own file, not a user file: temp file plus `rename`; a file that fails to parse is never overwritten, fmc starts with defaults and shows a notice.

**Closed:** the schema, the load and save points, and the deviation record are in plan-00015 and plan-00020 (`architecture.md`).

### RI-10 — Theme catalog in `~/.config/fmc/themes/*.json`

- status: landed (plan-00015 phase 2, `64c7aea`: 16 bundled themes, the theme mode, `dark_theme` and `light_theme` in `settings.json`, and user themes in `themes/`; `architecture.md`, "fmc offers a theme catalog") · updated 2026-11-27

User themes as JSON files, one per theme, each with a dark and a light variant or a declared mode, selected from settings (RI-9). The starting catalog comes from bjarneo/editor (MIT; keep its notice): editor dark, Catppuccin Mocha and Latte, Tokyo Night and Tokyo Night Day, Gruvbox and Gruvbox Light, Everforest, Kanagawa, Nord, One Dark and One Light, Flexoki Light, and, as the source has them, Rosé Pine, Rosé Pine Dawn, and editor light (source: `src/theme.rs` in github.com/bjarneo/editor). Each theme maps onto fmc's palette tokens (`DESIGN.md`, `src/theme/catalog.rs`). Twelve of the sixteen fall short of 7:1 on a tint or a match pair; the pairs are listed in `src/theme/theme_tests.rs` (plan-00015, A16), and the owner has not yet decided on them.

**Unlock:** RI-9 landed. **First move:** map editor's theme fields onto fmc's tokens and list the tokens with no source value.

### RI-20 — The Settings page on `Ctrl+,`

- status: landed (plan-00015 phase 3, `d68af26`; the grouped layout `883577d`, the owner's request on 2026-11-27; `architecture.md`, "Approved deviation: the Settings page edits settings.json on Ctrl+, and saves each change at once") · updated 2026-11-27

The owner asked on 2026-11-27 for a settings UI opened with `Ctrl+,`, and only that key (plan-00015 D12). The page edits the eight saved keys of RI-9 and RI-10 in two groups, Browsing and Appearance; each change applies and saves at once (D5). It is not in RI-9's text, so it has its own entry. Left out: rebinding keys on the page (RI-13 stays file-only, D11), a mouse on the page (A21), and a route for a terminal without the kitty keyboard protocol, which cannot send `Ctrl+,`.

**Unlock:** none; landed. **Follow-ups:** the owner's live check of `Ctrl+,` in Ghostty and Terminal.app; footer hints that follow `keymap.json`.

## Layout

### RI-11 — Adaptive layout and a collapsible sidebar

- status: landed (`55f9b45`, plan-00002 phases 1 and 2; persistence waits for RI-9) · updated 2026-11-27

Scouter always shows the sidebar: its width is `min(40, max(28, floor(termWidth × 0.25)))` and `Ctrl+←`/`Ctrl+→` resize it (`research/scouter-inventory.md` §2, §4.2); it has no hide or collapse. The owner asked on 2026-11-26 for:

- Auto-hide: below a width breakpoint the sidebar hides and the reader takes the full width; above it the sidebar comes back at its last width. The breakpoint is an open design point (a candidate: the sidebar minimum of 28 plus a usable reader width).
- A toggle: `Ctrl+/` (shown as `^/` on macOS) and a mouse click (candidate targets: the sidebar title, or a collapse mark on the panel border) hide or show the sidebar. A manual choice wins over auto-hide until the width crosses the breakpoint again.
- With RI-9, the collapsed state and the width persist in `settings.json`.

Trap: in the legacy keyboard encoding `Ctrl+/` arrives as `0x1F`, the same byte as `Ctrl+_` (and `Ctrl+7` in some terminals). Telling them apart needs the kitty keyboard protocol; check what the locked `opentui` parser reports for each terminal fmc supports before binding it, and keep a fallback key if a terminal cannot send it.

**Landed:** plan-00002. The 80-column breakpoint, the welcome-screen rule, `Ctrl+/` with its legacy `Ctrl+_`, and the border mark are in `55f9b45`; the deviation record is in `architecture.md`. Terminal.app and the other terminals are an owner check.

### RI-12 — Toggle hidden files with `Ctrl+.`

- status: landed (`81a735f`, plan-00002 phase 3; persistence waits for RI-9) · updated 2026-11-27

Scouter lists every entry except a fixed noise list (`.git`, `.hg`, `.svn`, `.DS_Store`, `.AppleDouble`, `Thumbs.db`, `desktop.ini`; `research/scouter-inventory.md` §9.1, `scouter:src/core/files.ts:34`) and has no hidden-file toggle. The owner chose on 2026-11-26:

- Default: dotfiles and dotfolders shown, as Scouter does.
- `Ctrl+.` (shown as `^.` on macOS) hides or shows them in the tree and the finder.
- The noise list stays hidden in both states.
- With RI-9, the state persists in `settings.json`.

Trap: `Ctrl+.` has no legacy encoding; only terminals with the kitty keyboard protocol (Ghostty, Rio, kitty, WezTerm) send it, and Terminal.app cannot. It shares the terminal check and the fallback-key decision with RI-11's `Ctrl+/`.

**Landed:** plan-00002 phase 3. `Ctrl+.` and `.` toggle dotfiles; `.` is the fallback for terminals that cannot send `Ctrl+.`, and it is free in the key table. Kitty-protocol terminals and Terminal.app are an owner check.

### RI-13 — Configurable keymap in `~/.config/fmc/keymap.json`

- status: landed (plan-00015 phase 4, `1b37a2e`: the default keys are one table, and `keymap.json`, read at start, binds and unbinds over it; the `?` card and `--help` show the merged keys; in-UI rebinding is not part of it, D11 (a)) · updated 2026-11-27

The owner asked on 2026-11-26 for user keybindings in the style of Zed, persisted at `~/.config/fmc/keymap.json`. The file is one flat JSON object, `"<context>.<key>": "<action>"` to bind and `null` to unbind a default, not Zed's array of blocks (plan-00015, A22). fmc dispatches every key from one table through a pure pending-key parser (`architecture.md`, "Keys are dispatched from one key table"), so the file edits that table: contexts are the table's focus and screen values, actions are its `Action` names. The defaults stay Scouter's keys plus the approved deviations; the `?` card and `--help` read the merged table, and the footer hints keep their own list. Crates: `serde_json` only, already declared; `serde` is approved but not used. A file that fails to parse keeps the defaults. A bad entry (an unknown context, action, or key name) is skipped with a notice that names its line, and the other entries apply (plan-00015, A11).

Traps: keys that terminals cannot send in the legacy encoding (`Ctrl+/`, `Ctrl+.`; RI-11, RI-12) need a per-terminal note; a user binding can shadow a prefix (`g`), so the parser's `More` state must follow the merged table.

**Unlock:** plan-00001 done and RI-9 landed (shared config directory, load and error rules). **First move:** freeze the action names and context names as a public list, then write the schema with Zed's file as the model.

## Opening with other apps

### RI-14 — Open an unsupported file with the default app

- status: landed (plan-00017, phases 1 to 6: `12fdaa1`, `52639c3`, `7adc043`, `8b57fda`, and the docs commit) · updated 2026-11-28

The owner asked on 2026-11-27 for a button or action that opens a file fmc cannot show (the binary notice) with the system's default app (`open <path>` on macOS). Open points: the key and the button on the binary notice; whether it also works for files fmc can show.

### RI-15 — Open the project folder in Finder or an editor

- status: landed (plan-00017, the same commits as RI-14; `editor` is in `settings.json`) · updated 2026-11-28

The owner asked on 2026-11-27 for an action that opens the browsed folder in Finder or in a chosen editor or IDE (Zed, VS Code), with the editor set in `settings.json`. Open points: the key, the settings key and its shape (an app name or a command line), and quoting of the path.

### RI-16 — Open With

- status: landed (plan-00017, the same commits as RI-14; `open_with` and the picker are the amendment D8 and D9) · updated 2026-11-28

The owner asked on 2026-11-27 for `Open With …` on text, code, and other supported files: a picker of relevant apps, or the ones set in `settings.json`. Open points: how relevant apps are found on macOS (Launch Services) versus a configured list per extension, and the picker's UI. Shares the launch code and the security rules (no shell interpolation of paths) with RI-14 and RI-15.

## Grammars and highlighting (the owner's set, after plan-00006)

### RI-17 — The rest of the owner's Neovim grammar set

- status: landed (plan-00018, 2026-11-28: query and json5 from crates, just and vimdoc vendored under `vendor/grammars/` with their queries, and luap patterns inside Lua; plan-00020 phase 3, `522fc83`: a Lua `---` documentation comment takes the documentation capture as a whole line, and `.eex` and `.leex` are read as heex, `architecture.md`, "Lua patterns are an injection; Lua documentation comments take the documentation capture" and "`.eex` and `.leex` are read as HEEx") · updated 2026-11-28

Twelve of the nineteen filetypes in RI-2 load and are highlighted (`architecture.md`, "the owner's extra grammars are highlighted"). Of the seven that had no crate loading on `tree-sitter` 0.27, four are now highlighted: query and json5 through crates that load with no vendored C, and just and vimdoc through parsers vendored at pinned commits. luadoc and luap are injections, not filetypes: luap is injected into Lua's pattern arguments, and luadoc is not, as below. Nothing was dropped from a loaded set. The `links` clash is the 0.25 pins' (vimdoc and just); the 0.20 pins (json5, luadoc, luap, and query) link a second C runtime (`research/grammar-load-spike.md`, section 5).

**Closed:** the owner chose the two routes on 2026-11-28 (plan-00020 D8 and D9). A `---` comment takes `comment.documentation` as a whole line, because OpenTUI paints a capture over a whole node and the grammar has no node for a tag, so the tag cannot be coloured on its own; the palettes give documentation the comment's colour. eex and leex are read as heex, with no licence needed.

### RI-18 — Injected highlighting for the owner's grammars

- status: landed (plan-00013 phases 1 and 2, commits `1c3857e` and `d619868`, and the fix `d68a7e7`; `architecture.md`, the injection records for HTML, JavaScript, TypeScript, Python, and the owner's pairs) · updated 2026-11-27

The owner set RI-18 as the next grammar work on 2026-11-27, ahead of RI-17. It improves files fmc already highlights (a Svelte `<script>` and `<style>` body, HEEx inside Elixir, JSDoc, and regex) and adds no dependency: the injection queries run on the grammars already pinned. RI-17 needs new crates, several of which pin another `tree-sitter`.

The owner's grammars highlight their own tokens, but not the code they hold: heex inside elixir, jsdoc inside JavaScript, regex inside JavaScript and Python, and a Svelte `<script>` body, which the spike's sample showed plain (`research/grammar-load-spike.md`, section 3). OpenTUI takes an injection query per grammar (`Grammar::injections`, `opentui-0.9.9/src/syntax/highlight.rs:65`); the Markdown fences already use one. A fence in one of the new languages is highlighted by its name, without an injection.

**Landed:** the pairs in `plan-00013-injected-highlighting.md`, each checked against the palette on a sample (`src/ui/syntax/tests.rs`, `injected_code_is_painted_in_its_own_languages_colours` and `jsdoc_and_regex_are_painted_in_their_own_languages_colours`). Svelte, Astro, HTML, Elixir, and HEEx in `1c3857e`; JavaScript, TypeScript, and Python in `d619868`; the Elixir `~r` body in `d68a7e7`. The Markdown fences in these languages follow each pair by name.

### RI-21 — The heavier and rarer common grammars

- status: parked (unlock: an owner request naming the languages) · updated 2026-11-28

plan-00024 highlighted Dockerfiles, build and config files, the git family, and C++, C#, Ruby, PHP, Kotlin, Swift, and Dart. Its research loaded the rest on the locked `tree-sitter` 0.27.2 and measured them (`research/common-grammars.md`, sections 3 and 6): Scala (`tree-sitter-scala` 0.26.14), Haskell (`tree-sitter-haskell` 0.24.1), OCaml (`tree-sitter-ocaml` 0.26.0), Objective-C (`tree-sitter-objc` 3.0.2), R (`tree-sitter-r` 1.3.0), PowerShell (`tree-sitter-powershell` 0.26.6), and Groovy (`dekobon-tree-sitter-groovy` 0.6.0) all load, and together add about 21 MiB to the release binary. Scala, PowerShell, and Objective-C want nvim-treesitter's query (the crates' are thin or use unstyled names).

Not available on the current terms: SSH config (the one loading crate is CC0-1.0, not on the licence allowlist; the MIT grammar Neovim uses would have to be vendored, which the owner declined on 2026-11-28), Perl (no crate loads: `tree-sitter-perl` 1.1.2 pins `tree-sitter` ^0.26, and upstream commits no `parser.c`), and git commit messages (the one grammar is WTFPL).

**First move:** a plan that takes the named languages from section 3's "Pick" column, as plan-00024 phase 3 did.
