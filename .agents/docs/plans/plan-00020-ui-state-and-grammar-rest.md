---
status: awaiting-approval
created: 2026-11-28T16:31:56+07:00
updated: 2026-11-28T16:31:56+07:00
phases:
    - id: 1
      title: The state file, read at start and written after a change
      status: pending
      note: —
    - id: 2
      title: Each remembered toggle reads and writes the state
      status: pending
      note: —
    - id: 3
      title: Lua documentation comments and eex through heex
      status: pending
      note: —
    - id: 4
      title: Docs sync, agent docs sync, ledger close, handover
      status: pending
      note: —
    - id: 5
      title: Final validation and live run
      status: pending
      note: —
---

## Finish the roadmap's partly landed entries — RI-9's remembered UI state, RI-17's luadoc and eex, RI-2's status

### Objective

An audit of the roadmap on 2026-11-28 found three entries `landed in part`: RI-2, RI-9, and RI-17. RI-2's remainder is RI-17, so it only needs its status. RI-9 left the UI state unsaved (plan-00015 D3 (a)); the owner now chose which state fmc remembers and where. RI-17 left luadoc in Lua comments and eex parked; the owner chose a doc colour for the first and the heex grammar for the second. After this plan, the three entries are `landed`, fmc remembers the chosen toggles across runs in `meta/state.json`, and no new dependency is added.

### Current state (verified in the tree, 2026-11-28)

- **The toggles live in memory only** — sidebar width `sidebar: Option<i64>` (`src/app/mod.rs:158`; default terminal/4 clamped to 28–40, `src/ui/sidebar.rs:39-42`; written by `src/app/layout.rs:92` and `:197`); manual sidebar visibility (`src/app/mod.rs:160`, reset by `Visibility::observe` on an 80-column crossing or a document open or close, `src/layout.rs:88-95`); reader wrap (`src/app/mod.rs:136`, default `false` `:279`, flipped at `src/app/capture.rs:120`); diff split, wide context, and one file (`src/app/changes.rs:43-45`, defaults `:51`, toggled `:404-434`); dotfiles (`src/explorer/tree.rs:56`, default `true` `:69`, flipped in `src/app/tree.rs:18-22`, reset to `true` by `-` building a new tree `:120`); finder matching (`src/app/mod.rs:189`, start value from `find_mode`, `:371`; `Ctrl+f` flips it, `src/app/find.rs:172-178`, unsaved per `docs/configuration.md:112`).
- **No state file exists** — no `XDG_STATE_HOME`, `.local/state`, or disk cache in `src/`. Config I/O is `config_folder`, `read_config` (1 MiB cap), and `save_atomic` (`src/services/mod.rs:34-87`); `meta/config.schema.json` is the only other file fmc writes in its config folder (`rules.md`, "Writes").
- **Records to change** — `architecture.md:771` ("Nothing is saved across runs (RI-9)"), `:878` ("the owner chose D3 (a) … saving it needs its own decision"); `docs/configuration.md:27` (the file keeps no UI state).
- **Scouter keeps nothing** — Scouter 434d000f holds wrap (`src/app/app.ts:73`) and the sidebar width (`src/explorer/sidebar-resizer.ts:10`) in memory and writes no state (`docs/decision/0001-read-only-and-minimal.md`). Remembering state is a deviation.
- **luadoc** — `src/ui/syntax/queries/lua/injections.scm` leaves the `---` rule out: the luadoc grammar rejects the comment's leading dash and OpenTUI 0.9.9 does not evaluate `#offset!`. The Lua grammar has a `comment_content` node (tree-sitter-lua 0.5.0 `node-types.json`), and `CAPTURE_NAMES` already styles `comment.documentation` (`src/ui/syntax/mod.rs:212`).
- **eex** — connorlay/tree-sitter-eex has no licence (plan-00018 O4). The heex grammar is registered (`src/reader/language.rs:30,49`).
- **RI-2** — its status says "the rest is RI-17" (`roadmap-00001-post-parity.md:20,52`).

### Decisions

- **D1 (owner, 2026-11-28): fmc remembers four groups of UI state** — the sidebar layout, reader wrap, the diff view (split, wide context, all files), and dotfiles with the finder's matching. Fold state is per diff and stays unsaved.
- **D2 (owner, 2026-11-28): the state lives in `<config folder>/meta/state.json`**, beside the schema, global (not per project). Rejected: `$XDG_STATE_HOME/fmc/state.json`; keys in `settings.json`, which a toggle would rewrite and re-sort.
- **D3 (owner, 2026-11-28): written about a second after the last change**, atomically on a worker (`save_atomic`), so a burst of toggles is one write and a crash loses at most that second. Rejected: on quit only.
- **D4 (owner, 2026-11-28): the sidebar's width is remembered, its manual visibility is not.** The manual choice already resets on every document open or close and on an 80-column crossing (`src/layout.rs:88-95`), so a saved choice would be undone by the first open. Rejected: saving the visibility too — it would show for one screen and then reset.
- **D5 (owner, 2026-11-28): the state wins over `settings.json` for the finder's matching; the setting seeds a fresh state.** `find_mode` in `settings.json` is the start value when `state.json` has none; a `Ctrl+f` flip is saved to the state; changing `find_mode` on the Settings page also writes the state, so the page's value is what the next run starts with. Rejected: settings always wins — a `Ctrl+f` flip would never survive a restart, which is what the owner asked for.
- **D6 (owner, 2026-11-28): dotfiles survive `-`.** A new tree after `-` takes the remembered value instead of resetting to shown.
- **D7 (agent, 2026-11-28): the file is fmc's, read forgivingly.** One JSON object with `sidebar_width` (integer), `wrap`, `diff_split`, `diff_wide`, `diff_all_files`, `dotfiles` (booleans), `find_mode` (`substring`/`fuzzy`). A missing file, a file that does not parse, or a bad value means the default for that key, with no notice; the next write replaces the file. Unknown keys are dropped. No schema file: the owner does not edit it. A failed write is a footer notice, at most once per run.
- **D8 (owner, 2026-11-28): Lua documentation comments get the doc colour, no injection.** `queries/lua/highlights.scm` captures a `(comment)` whose text starts with `---` as `comment.documentation`, and the `@tag` words in it (`@param`, `@return`, `@class`, `@field`, `@type`, …) as a keyword-style capture through `#match?` on `comment_content`, if OpenTUI can paint a sub-span that way; otherwise the whole line takes the doc colour. Rejected: patching the vendored luadoc grammar (needs the tree-sitter CLI and diverges from upstream); waiting for `#offset!`.
- **D9 (owner, 2026-11-28): `.eex` and `.leex` use the heex grammar.** `EXTENSIONS` maps both to `heex`; the tags and HTML are highlighted, non-HTML text stays plain. Rejected: asking the author for a licence; dropping eex.
- **D10 (2026-11-28): the writes rule grows by one file.** `rules.md` ("Writes") and `AGENTS.md` add `meta/state.json` to fmc's own files; `architecture.md` records the deviation with the Scouter baseline (v0.3.4, `434d000f`), and `docs/parity.md` gets a line.

### Execution rules

- **Execution waits for the owner's go-ahead**; D4 to D6 are the owner's, D7 the agent's.
- **Every commit ends with** `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Comments state their reason, never a plan or doc (`AGENTS.md`).
- **The deviation's records land with the behaviour** (phase 2 and phase 3 commits), as `rules.md` requires.
- **Tests use temporary config folders**; no test touches the real `~/.config/fmc`.
- **Branch:** `feat/ui-state-and-grammar-rest`, from `main`.

### Phases

1. **The state file** — `src/state.rs` (pure: `State` with the D7 keys, `parse(text) -> State`, `json(&State) -> String`) and `services/state.rs` (`path`, `load`, `save` through `save_atomic`); the debounce on the app side (one pending write, reset by each change, fired on a worker after about a second). Tests: parse of every key, a bad value per key, a non-object, unknown keys dropped; load of a missing file; save then load round-trips; a read-only folder gives one notice. Validation: `cargo test state`. _Commit: `feat(state): read and write meta/state.json`._
2. **The toggles** — start values from the state (sidebar width, wrap, the three diff options, dotfiles, matching per D5); each toggle schedules a write; `-` keeps dotfiles (D6); the Settings page's `find_mode` writes the state too. App tests through `TestApp`: each toggle survives a simulated restart (state written, a new app loaded from it); a burst of toggles writes once; the sidebar's manual visibility is not saved (D4). Records: `rules.md` "Writes", `AGENTS.md`, `architecture.md` (the deviation, and `:771`, `:878` updated), `docs/parity.md`, `docs/configuration.md` (`:27` and a "Remembered state" section), `CHANGELOG.md`. Validation: `cargo test`; a live run in a new Rex tab: toggle each, quit, start, see each kept. _Commit: `feat(state): remember the layout, wrap, diff view, dotfiles, and finder mode`._
3. **Lua documentation comments and eex** — D8 in `queries/lua/highlights.scm` (verify the sub-span; fall back as D8 says and record which landed); D9 in `src/reader/language.rs`. Tests: a Lua sample's `---@param` line takes `comment.documentation` (and the tag capture if it landed); `x.eex` and `x.leex` resolve to `heex`. Records: `architecture.md` (the luadoc and eex parts of the grammar records), `CHANGELOG.md`, `docs/configuration.md` (filetypes). Validation: `cargo test syntax`, `cargo test language`. _Commit: `feat(syntax): Lua doc comments and eex templates`._
4. **Docs sync, agent docs sync, ledger close, handover** — roadmap RI-2, RI-9, RI-17 to `landed`, with the Status table rows; `shape.md` (`state.rs`, `services/state.rs`); plan archived with its index row; a handover. Validation: `rumdl fmt --check --quiet .`. _Commit: `docs: RI-2, RI-9, and RI-17 landed`._
5. **Final validation and live run** — the matrix; a live run in a new Rex tab: each remembered toggle survives a restart in a temporary config folder; a `---@param` line and an `.eex` file at 120 columns. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl fmt --check --quiet .` | No drift |
| Unit | `cargo test state`, `cargo test syntax`, `cargo test language` | Parse, round-trip, bad values; captures; extensions |
| App | `cargo test` (`TestApp`) | Each toggle survives a restart; one write per burst; visibility not saved |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Full suite | `just verify` | All green |
| Live run | Rex tab, temporary `XDG_CONFIG_HOME` | Toggles kept across a restart; Lua doc line and `.eex` coloured |
| E2E | n/a beyond the live run: no CLI change | — |

### Out of scope

- Per-project state, the last opened folder or file, fold state, and the theme mode (already a setting).
- A full luadoc parse inside Lua comments (waits for `#offset!` in OpenTUI).
- An eex grammar of its own.
