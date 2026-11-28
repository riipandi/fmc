---
status: in-progress
created: 2026-11-28T14:00:33+07:00
updated: 2026-11-28T14:08:05+07:00
phases:
    - id: 1
      title: The schema, built in the pure core
      status: committed
      note: —
    - id: 2
      title: The startup check and write of meta/config.schema.json
      status: pending
      note: —
    - id: 3
      title: The $schema link in settings.json (per O1)
      status: pending
      note: runs, O1 (b)
    - id: 4
      title: Docs sync, agent docs sync, ledger close, handover
      status: pending
      note: —
    - id: 5
      title: Final validation and live run
      status: pending
      note: —
---

## Ship a JSON Schema for settings.json — fmc checks or writes `<config folder>/meta/config.schema.json` at every start

### Objective

The owner asked (2026-11-28) that the settings file have a JSON Schema, and that fmc check for it, or create it, at startup in `~/.config/fmc/meta/config.schema.json`. After this plan, every `fmc` start that opens the UI leaves that file byte-equal to the schema of the running build, so an editor that reads `$schema` completes and checks `settings.json`. The plan writes only that one file (and, if the owner picks it in O1, the `$schema` key through the existing Settings save path); keymap and theme schemas stay out.

### Current state (verified in the tree, 2026-11-28)

- **Nine keys, parsed by name from a `serde_json::Value`** — `src/settings.rs:58-87` (`parse`): `auto_preview`, `sidebar_jump`, `reader_jump`, `git_gutter`, `file_associations`, `find_mode`, `theme_mode`, `dark_theme`, `light_theme`. Defaults are the constants at `src/settings.rs:12-17` and `:40-44`. Unknown keys, `$schema` included, are ignored without a notice (`:75-85`).
- **Value domains live in code** — numbers are whole and `>= 1` within `u32` (`src/settings.rs:178-186`); `find_mode` is `substring`/`fuzzy` (`src/find/mod.rs:39`); `theme_mode` is `auto`/`dark`/`light` (`src/theme/mod.rs:34`); theme ids are any string, chosen against the bundled catalog and user themes later (`src/settings.rs:156-166`, `src/theme/catalog.rs:11,77`); `file_associations` maps a glob to a name from `HIGHLIGHTED`, `ALIASES`, or `plaintext` (`src/reader/associations.rs:22-32`, `src/reader/language.rs:9`). Help lines per key are `Row::help` (`src/settings.rs:258-265`); `file_associations` has none.
- **Saves keep unknown keys and sort them** — `merged` (`src/settings.rs:92-106`) inserts only the changed keys over the file's object; `serde_json`'s map is a `BTreeMap` in the runtime build (no `preserve_order`), so a `$schema` key survives a save and sorts first.
- **An atomic write exists** — `services::settings::save(path, text)` (`src/services/settings.rs:60-83`): resolves a link, `create_dir_all`, temp file with `create_new`, `sync_all`, rename, temp removed on error. It is not settings-specific except its doc comment. `services::read_config` (`src/services/mod.rs:48-60`) reads with the size cap and the FIFO check.
- **Config folder rule** — `services::config_folder` (`src/services/mod.rs:28-34`): absolute `XDG_CONFIG_HOME`, else `HOME/.config`, then `/fmc`; `None` without `HOME`.
- **Startup reads** — `src/main.rs:55-60` reads settings, themes, and keymap for `Command::Open` only, after the TTY check (`:48-51`); `--help` and `--version` read nothing. Their notices chain into one footer message (`src/app/mod.rs:370-376`).
- **No schema anywhere** — `grep -rn -i schema src docs README.md` finds only SQL fixtures and the SQL highlight query; no JSON Schema, no `$schema` in `docs/configuration.md`.
- **Writes are bounded** — `AGENTS.md` ("Parity and read-only") and `rules.md:13,29` allow fmc to write only `settings.json`, only from the Settings page. Any other config file waits for the owner.
- **Scouter has no config file** — Scouter `docs/decision/0001-read-only-and-minimal.md:9` ("no configuration file"), `README.md:20`; its tree (v0.3.4, `434d000f`) ships no schema. The settings file is already an approved deviation (`architecture.md:847-894`); this plan extends it.

### Decisions

- **D1 (2026-11-28): the owner's request opens one new write: `<config folder>/meta/config.schema.json`.** fmc owns that file and the `meta/` folder; the record lands in `rules.md` ("Writes"), `architecture.md`, and the `AGENTS.md` line. A hand edit to the schema is overwritten at the next start, and the docs say so. Rejected: write next to `settings.json` as `settings.schema.json` — the owner named the path.
- **D2 (2026-11-28): the schema is built in code from the same constants, not hand-written.** `settings::schema()` in `src/settings.rs` (pure core, no I/O) returns the text from `serde_json::json!`: defaults from the constants, enums from the same lists `named` reads (`Matching`, `ModeChoice`, `HIGHLIGHTED` + `ALIASES` + `plaintext`, the bundled theme ids), descriptions from `Row::help`. One source, so a new key or name cannot drift from the schema. Cost: the text is generated at each start (microseconds). Rejected: a checked-in `config.schema.json` with `include_str!` — two sources, needs a sync test for every enum. Rejected: `schemars` — a new dependency (`rules.md`, "Dependencies"), and `Settings` has no serde derives to feed it.
- **D3 (2026-11-28): JSON Schema draft-07.** VS Code's and Zed's JSON language servers (vscode-json-languageservice) support draft-07 fully; 2020-12 support is partial in older builds. `$id` is omitted (no public URL). Rejected: 2020-12 — no feature here needs it.
- **D4 (2026-11-28): the schema is strict on keys, open on theme ids.** `additionalProperties: false` with `$schema` listed as an allowed string property, so an editor flags a typo the parser would silently ignore. `dark_theme`/`light_theme` are `anyOf: [{enum: <bundled ids>}, {type: string}]`: completion for bundled ids, no error for a user theme in `themes/`. Numbers are `integer`, `minimum: 1`, `maximum: 4294967295`. `file_associations` is an object whose values are an `enum` of the known names. Rejected: `additionalProperties: true` — loses the typo check, the main value of the schema.
- **D5 (2026-11-28): check means compare bytes; write only on a difference.** At start, read the file (`read_config`); when it is missing, unreadable, or differs from `schema()`, write it atomically. A start with an equal file does no write. Rejected: write every start — needless disk writes and mtime churn for dotfile managers.
- **D6 (2026-11-28): the write reuses the existing atomic save.** `services::settings::save` moves to `services/mod.rs` as `save_atomic` (callers renamed, no alias; early-stage rule), and the schema write calls it, so link write-through and temp cleanup behave as for `settings.json`. Rejected: a second write function — duplicates the guarded code.
- **D7 (2026-11-28): the schema is written for `Command::Open` only, before the UI, in `src/main.rs` beside the config reads.** `--help`, `--version`, and a non-TTY refusal write nothing. No `HOME` means no write and no notice (as for the reads).
- **O1 (owner, 2026-11-28: (b)): how `settings.json` points at the schema.** (a) fmc does not touch `settings.json`; `docs/configuration.md` tells the user to add `"$schema": "./meta/config.schema.json"`. (b) The Settings page's save adds that `$schema` key when it is missing (the existing write path, no new write) [recommended: zero setup for page users, no new write rule]. (c) fmc adds `$schema` at startup — a second startup write to the user's file, against `rules.md:29`. Phase 3 runs only for (b) or (c).
- **O2 (owner, 2026-11-28: (a)): what a failed schema write does.** (a) A footer notice in the startup chain, every start until it succeeds [recommended: a silent failure leaves the user wondering why the editor shows nothing]. (b) Silent; the failure is lost. A read-only config folder (for example, one a dotfile manager owns) would show the notice at every start under (a).

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The new write | `rules.md`, "Writes"; `AGENTS.md`, "Parity and read-only" | fmc writes `meta/config.schema.json` at start when it differs; nothing else new |
| The reasoning | `architecture.md`, a new record under the settings records | D1 to D7, O1, O2, the rejected alternatives |
| The parity line | `docs/parity.md` | Scouter has no config; fmc ships a schema for its own |
| The Rust contract | `rust.md`, "Contracts and traps" | `schema()` is generated from the parse lists; a new key needs a schema property (the test fails otherwise) |
| The user docs | `docs/configuration.md`, `CHANGELOG.md` `[Unreleased]` | Where the schema lives, that fmc rewrites it, how `$schema` points at it |
| The layout | `shape.md` | `settings.rs` gains `schema()`; `services/mod.rs` gains `save_atomic` and the schema path |

### Execution rules

- **Atomic commit per phase** is authorized by the owner's per-phase go-ahead; each phase names its message.
- **The owner gave the go-ahead for all phases in order (2026-11-28)**; O1 (b) and O2 (a) are chosen.
- **Writes:** only `meta/config.schema.json` (D1), and the `$schema` key through the Settings save only under O1 (b). No other write, no new dependency (`serde_json` is already declared, `Cargo.toml`).
- **Branch:** `feat/settings-schema`. Port commits need no `Scouter-Base` trailer: nothing is ported.
- **Tests use temporary config folders** (`tempfile`, the pattern in `src/services/settings_tests.rs`); no test touches the real `~/.config`.
- **Per-phase validation runs before each phase's commit.**

### Phases

1. **The schema, built in the pure core** — `settings::schema() -> String` in `src/settings.rs` (draft-07, D2 to D4), pretty-printed with a trailing newline, stable across calls. Tests in `src/settings_tests.rs`: the schema parses as JSON; its `properties` equal the keys `parse` reads plus `$schema` (a key list shared by `parse` and `schema`, so adding a key without a property fails); each `default` equals the constant; the `find_mode`, `theme_mode`, and association enums accept exactly what `named` accepts (round-trip each enum value through `named`); bundled theme ids appear. Validation: `cargo test settings`; `cargo clippy --all-targets -- -D warnings`. _Commit: `feat(settings): build a JSON Schema for settings.json`._
2. **The startup check and write** — move `save` to `services::save_atomic` (callers in `src/services/settings.rs` and `src/app/settings.rs` renamed); add `services::settings::schema_path` (`<config folder>/meta/config.schema.json`) and `ensure_schema(path) -> Option<String>` (a notice under O2 (a)); call it from `src/main.rs` for `Command::Open`, before `app::run`, and chain its notice with the others. Tests in `src/services/settings_tests.rs`: missing file and folder → created with `schema()`; stale file → rewritten; equal file → not rewritten (mtime unchanged); read-only folder → notice, no panic; a link → written through, link kept; relative `XDG_CONFIG_HOME` → `HOME/.config`. `tests/cli.rs`: `fmc --version` with a temporary `HOME` writes no `meta/`. Validation: `cargo test`; `cargo clippy --all-targets -- -D warnings`. _Commit: `feat(settings): write meta/config.schema.json at startup`._
3. **The `$schema` link (O1 (b) or (c) only)** — under (b): `merged` adds `"$schema": "./meta/config.schema.json"` when the object has no `$schema`, and keeps a user's own value. Tests in `src/settings_tests.rs` and `src/app/settings_tests.rs`: a save into an empty file writes `$schema` first; an existing `$schema` is kept unchanged; parse still ignores it. Validation: `cargo test settings`. _Commit: `feat(settings): link settings.json to its schema on save`._ Recorded n/a under (a).
4. **Docs sync, agent docs sync, ledger close, handover** — `docs/configuration.md` (the schema file, its rewrite, the `$schema` line, the editors it was checked with), `docs/parity.md`, `CHANGELOG.md`; `rules.md` "Writes", `AGENTS.md`, `architecture.md` record, `rust.md` trap, `shape.md` rows; roadmap entry if one applies (none today); the plan moves to `archive/` with its index row; a handover. Validation: `rumdl check`; every path the docs name resolves; `grep -rn "config.schema.json" AGENTS.md .agents/docs docs` lists each record. _Commit: `docs: the settings JSON Schema`._
5. **Final validation and live run** — the matrix below; a live run in a new Rex tab of this session with a temporary `XDG_CONFIG_HOME`: start fmc, quit, check the file; start again, check no rewrite. Validation: the matrix green. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl check` | No drift |
| Unit | `cargo test settings` | Schema keys, defaults, enums tied to the parser; the write cases of phase 2 |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Build | `cargo build` | Builds |
| Full suite | `cargo test` (or `just verify`) | All green |
| CLI | `tests/cli.rs`: `--version` writes no `meta/` | Pass |
| Live run | Rex tab, `XDG_CONFIG_HOME=$(mktemp -d) cargo run`; `ls`/`stat` the schema before and after a second start | Created once, untouched the second time |
| Editor | Owner check: open `settings.json` in VS Code or Zed with `$schema` set | Completion for keys and enums; a typo key flagged |
| E2E | n/a beyond the CLI and live run: the schema is not drawn | — |

### Out of scope

- Schemas for `keymap.json` and theme files (`themes/<id>.json`); a later plan can add `meta/keymap.schema.json` with the same builder.
- Validating `settings.json` against the schema inside fmc: `parse` stays the validator, with its notices.
- Publishing the schema at a URL or in the repository (`$id`, SchemaStore).
- Creating `settings.json` when it is missing.
