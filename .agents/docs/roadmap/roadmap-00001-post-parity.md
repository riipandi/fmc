---
status: living
created: 2026-11-26T04:09:25+07:00
updated: 2026-11-26T05:02:31+07:00
references: plan-00001-scouter-parity.md, research/dependency-survey.md, research/editor-study.md, research/sqltui-study.md, research/opentui-binding-risk.md
---

# Roadmap — after Scouter parity

Parked work that the owner named during the planning of `plan-00001`. None of it runs before parity closes. When an unlock fires, the entry becomes a plan.

## Rendering

### RI-1 — Mermaid diagrams in Markdown

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done) · updated 2026-11-26

Scouter draws a `mermaid` fence as plain text (`research/scouter-inventory.md` §7), and upstream OpenTUI has no Mermaid renderer. The binding exposes a code-block renderer hook for a fence language (`research/rendering-research.md`, `markdown.rs:631` upstream). Candidate: `mermaid-text` 0.58 (flowcharts only, box drawing); `mermaid-rs-renderer` needs SVG rasterising.

**Unlock:** plan-00001 done. **First move:** a spike that renders the demo README's flowchart through the hook with `mermaid-text`, and an owner decision on sequence diagrams.

### RI-2 — The owner's Neovim grammar set

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done) · updated 2026-11-26

The owner's Neovim config (`~/.config/nvim/lua/plugin_treesitter.lua:14-50`) adds, beyond Scouter's 16: astro, diff, eex, elixir, hcl, heex, jsdoc, json5, just, lua, luadoc, luap, query, regex, sql, svelte, vim, vimdoc, xml. On crates.io (2026-11-26): no crate for astro or eex; `tree-sitter-sequel` 0.3.13 for SQL; old crates that need an ABI check against `tree-sitter` 0.27: `tree-sitter-diff` 0.1.0, `tree-sitter-svelte-ng` 1.0.2, `tree-sitter-xml` 0.7.0; `luadoc`, `luap`, `query`, `json5` crates are barely used. Each grammar also needs a `highlights.scm` that uses the palette's capture names.

**Unlock:** plan-00001 done. **First move:** build each candidate crate against `opentui` 0.9.9 and list the ones that load.

## Finding

### RI-3 — Fuzzy file finding

- status: parked (owner-deferred 2026-11-26; unlock: plan-00001 done) · updated 2026-11-26

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
