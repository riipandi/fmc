---
status: living
created: 2026-11-26T04:09:25+07:00
updated: 2026-11-27T16:45:00+07:00
references: plan-00001-scouter-parity.md, research/dependency-survey.md, research/editor-study.md, research/sqltui-study.md, research/opentui-binding-risk.md
---

# Roadmap — after Scouter parity

Parked work that the owner named during the planning of `plan-00001`. None of it runs before parity closes. When an unlock fires, the entry becomes a plan.

## Rendering

### RI-1 — Mermaid diagrams in Markdown

- status: landed (plan-00006 phases 1 and 2, 2026-11-27; `architecture.md`, the Mermaid records) · updated 2026-11-27

Scouter draws a `mermaid` fence as plain text (`research/scouter-inventory.md` §7), and upstream OpenTUI has no Mermaid renderer. The binding exposes a code-block renderer hook for a fence language (`research/rendering-research.md`, `markdown.rs:631` upstream). Candidate: `mermaid-text` 0.58 (flowcharts only, box drawing); `mermaid-rs-renderer` needs SVG rasterising.

**Unlock:** plan-00001 done. **First move:** a spike that renders the demo README's flowchart through the hook with `mermaid-text`, and an owner decision on sequence diagrams.

### RI-2 — The owner's Neovim grammar set

- status: landed in part (plan-00006 phases 3 and 4, 2026-11-27): twelve grammars are highlighted, `architecture.md`, "the owner's extra grammars". The seven without a crate are RI-17; the injections are RI-18 · updated 2026-11-27

The owner's Neovim config (`~/.config/nvim/lua/plugin_treesitter.lua:14-50`) adds, beyond Scouter's 16: astro, diff, eex, elixir, hcl, heex, jsdoc, json5, just, lua, luadoc, luap, query, regex, sql, svelte, vim, vimdoc, xml. On crates.io (2026-11-26): no crate for astro or eex; `tree-sitter-sequel` 0.3.13 for SQL; old crates that need an ABI check against `tree-sitter` 0.27: `tree-sitter-diff` 0.1.0, `tree-sitter-svelte-ng` 1.0.2, `tree-sitter-xml` 0.7.0; `luadoc`, `luap`, `query`, `json5` crates are barely used. Each grammar also needs a `highlights.scm` that uses the palette's capture names.

**Unlock:** plan-00001 done. **First move:** build each candidate crate against `opentui` 0.9.9 and list the ones that load.

## Finding

### RI-3 — Fuzzy file finding

- status: landed (plan-00006 phase 5, 2026-11-27; `find_mode` and `Ctrl+F`, `architecture.md`) · updated 2026-11-27

Phase 1 copies Scouter's substring AND match and order (`architecture.md`). A fuzzy toggle could use `neo_frizbee` (1 crate, MIT, MSRV 1.89; the matcher inside fff) or `nucleo-matcher` (MPL-2.0). `fff-search` (96 crates, LMDB frecency, git signals, MSRV 1.91) only if its grep or index is wanted (`research/dependency-survey.md`).

**Unlock:** plan-00001 done and the owner asks for fuzzy matching. **First move:** a toggle in the finder that swaps the matcher, with Scouter's order as the default.

## File manager

### RI-4 — Write operations

- status: parked (owner-deferred 2026-11-26; unlock: the owner opens the file-manager phase) · updated 2026-11-26

Create, rename, move, copy, delete. Ideas from `research/editor-study.md`: atomic writes through a temp file and `rename`. Candidates: `ignore` 0.4 for walks, `trash` 5.4 for delete-to-trash; `fs_extra` is stale (2023). Every write path needs a `security-review` pass (symlinks, path traversal).

**Unlock:** the owner opens the file-manager phase (`rules.md`, read-only rule). **First move:** a plan with the operation list, the keymap additions, and the confirm and undo rules.

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

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done) · updated 2026-11-26

Scouter has no configuration by design (`research/scouter-inventory.md` §15), so this is a post-parity deviation the owner asked for on 2026-11-26. fmc remembers the UI state the user changes: sidebar width, reader wrap, diff layout (unified or split), diff context (`x`), and the theme mode (`auto` follows the terminal as Scouter does, or `dark`, `light`). Path: `~/.config/fmc/settings.json` (owner's choice). Crates approved for this item: `serde` with `derive` and `serde_json` (`rules.md`, "Dependencies"). The write is fmc's own file, not a user file: temp file plus `rename`; a file that fails to parse is never overwritten, fmc starts with defaults and shows a notice.

**Unlock:** plan-00001 done. **First move:** a plan with the schema (every key, type, default), the load and save points, and the `architecture.md` deviation record.

### RI-10 — Theme catalog in `~/.config/fmc/themes/*.json`

- status: parked (owner-deferred 2026-11-26; unlock: RI-9 landed) · updated 2026-11-26

User themes as JSON files, one per theme, each with a dark and a light variant or a declared mode, selected from settings (RI-9). The starting catalog comes from bjarneo/editor (MIT; keep its notice): editor dark, Catppuccin Mocha and Latte, Tokyo Night and Tokyo Night Day, Gruvbox and Gruvbox Light, Everforest, Kanagawa, Nord, One Dark and One Light, Flexoki Light (source: `src/theme.rs` in github.com/bjarneo/editor). Each theme maps onto fmc's palette tokens (`DESIGN.md`, `theme.rs`) and passes the contrast tests the Scouter palettes pass.

**Unlock:** RI-9 landed. **First move:** map editor's theme fields onto fmc's tokens and list the tokens with no source value.

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

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done and RI-9 landed) · updated 2026-11-26

The owner asked on 2026-11-26 for user keybindings in the style of Zed, persisted at `~/.config/fmc/keymap.json`. Zed's shape is a JSON array of blocks, each with an optional `context` predicate and a `bindings` map from a key sequence (`"ctrl-q"`, `"g g"`) to an action name; user blocks override the defaults, and `null` unbinds a key. fmc already dispatches every key from one table through a pure pending-key parser (`architecture.md`, "Keys are dispatched from one key table"), so the file maps onto it: contexts are the table's focus and screen values, actions are its `Action` names. The defaults stay Scouter's keys plus the approved deviations; `--help`, the `?` card, and the footer hints keep reading the merged table. Crates: `serde` and `serde_json` (approved for after parity, `rules.md`). A file that fails to parse or names an unknown action or context is not applied: fmc keeps the defaults and shows a notice naming the line.

Traps: keys that terminals cannot send in the legacy encoding (`Ctrl+/`, `Ctrl+.`; RI-11, RI-12) need a per-terminal note; a user binding can shadow a prefix (`g`), so the parser's `More` state must follow the merged table.

**Unlock:** plan-00001 done and RI-9 landed (shared config directory, load and error rules). **First move:** freeze the action names and context names as a public list, then write the schema with Zed's file as the model.

## Opening with other apps

### RI-14 — Open an unsupported file with the default app

- status: parked (owner-deferred 2026-11-27) · updated 2026-11-27

The owner asked on 2026-11-27 for a button or action that opens a file fmc cannot show (the binary notice) with the system's default app (`open <path>` on macOS). Open points: the key and the button on the binary notice; whether it also works for files fmc can show.

### RI-15 — Open the project folder in Finder or an editor

- status: parked (owner-deferred 2026-11-27; unlock: settings.json landed) · updated 2026-11-27

The owner asked on 2026-11-27 for an action that opens the browsed folder in Finder or in a chosen editor or IDE (Zed, VS Code), with the editor set in `settings.json`. Open points: the key, the settings key and its shape (an app name or a command line), and quoting of the path.

### RI-16 — Open With

- status: parked (owner-deferred 2026-11-27; unlock: settings.json landed) · updated 2026-11-27

The owner asked on 2026-11-27 for `Open With …` on text, code, and other supported files: a picker of relevant apps, or the ones set in `settings.json`. Open points: how relevant apps are found on macOS (Launch Services) versus a configured list per extension, and the picker's UI. Shares the launch code and the security rules (no shell interpolation of paths) with RI-14 and RI-15.

## Grammars and highlighting (the owner's set, after plan-00006)

### RI-17 — The rest of the owner's Neovim grammar set

- status: parked, after RI-18 (owner, 2026-11-27: RI-18 is the next grammar work; unlock: a crate that loads on the locked `tree-sitter` 0.27, per language) · updated 2026-11-27

Twelve of the nineteen filetypes in RI-2 load and are highlighted (`architecture.md`, "the owner's extra grammars are highlighted"). Seven have no crate that loads. Six have a crate that pins another `tree-sitter` (json5, luadoc, luap, and query pin 0.20; just and vimdoc pin 0.25), which clashes on the `links` key with 0.27. eex has no published crate. Each also needs a highlight query for fmc's capture names. Nothing is dropped from a loaded set: the phase 3 spike dropped none (`research/grammar-load-spike.md`, section 5).

**Unlock:** a crate for one of the seven that loads with `Grammar::compile`, and the owner's go-ahead for its dependency. **First move:** load the candidate with its query, as the spike did.

### RI-18 — Injected highlighting for the owner's grammars

- status: next (owner, 2026-11-27: RI-18 before RI-17) · updated 2026-11-27

The owner set RI-18 as the next grammar work on 2026-11-27, ahead of RI-17. It improves files fmc already highlights (a Svelte `<script>` and `<style>` body, HEEx inside Elixir, JSDoc, and regex) and adds no dependency: the injection queries run on the grammars already pinned. RI-17 needs new crates, several of which pin another `tree-sitter`.

The owner's grammars highlight their own tokens, but not the code they hold: heex inside elixir, jsdoc inside JavaScript, regex inside JavaScript and Python, and a Svelte `<script>` body, which the spike's sample showed plain (`research/grammar-load-spike.md`, section 3). OpenTUI takes an injection query per grammar (`Grammar::injections`, `opentui-0.9.9/src/syntax/highlight.rs:65`); the Markdown fences already use one. A fence in one of the new languages is highlighted by its name, without an injection.

**Unlock:** the owner's choice of RI-18 next (2026-11-27). **First move:** the injection query for one pair, checked against the palette on a sample.
