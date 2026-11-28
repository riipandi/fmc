---
status: done
created: 2026-11-28T14:16:10+07:00
plan: plan-00016-settings-schema
---

# Handover: the settings JSON Schema landed; plan-00016 closed

## Closed

- plan-00016 on `feat/settings-schema` (not pushed). Phase 1 `1b5611a`: `settings::schema()` (draft-07), built from the Settings rows, their help lines, the defaults, and the name lists (`Matching::ALL`, `ModeChoice::ALL`, `Language::names()`, `slot_ids`). Phase 2 `4412013`: `services::save_atomic` (moved from `services/settings.rs` `save`), `services::settings::schema_path` and `ensure_schema`, the call in `src/main.rs` for `Command::Open` only, and `tests/cli.rs` for `--version` and `--help`. Phase 3 `82ab4b3`: the `$schema` link on save (O1 (b)). Phase 4: the docs commit, `docs: the settings JSON Schema`. Phase 5: the live run, below.
- Behavior: at each start that opens the UI, fmc writes `<config folder>/meta/config.schema.json` when the file is missing, unreadable, or differs from `settings::schema()`. An equal file is not rewritten. A failed write is a footer notice at each start (O2 (a)). The Settings page's save adds `"$schema": "./meta/config.schema.json"` to a file that has none, and a `$schema` the owner wrote stays. Startup never writes `settings.json`.
- The relative link was checked in the source before phase 3. The languageservice resolves a relative `$schema` against the document (`vscode-json-languageservice` `src/services/jsonSchemaService.ts:1367-1372`). VS Code's host does the same (`extensions/json-language-features/server/src/jsonServer.ts:67-70`). Zed runs `vscode-langservers-extracted` 4.10.0 (`zed-industries/zed` `crates/languages/src/json.rs:41,146`), whose `lib/json-language-server/jsonServer.js:115-118` resolves the same way. Draft-07 is in `jsonParser.ts:216`.
- Where the build differs from the plan's text: (a) the theme keys' enum lists the slot's ids (`slot_ids`: `default` and the bundled ids of that mode), not every bundled id in both slots, because A13 keeps an id in its own slot; (b) `schema_path` lives in `services/settings.rs`, not `services/mod.rs` as the plan's layout table says; (c) the tie between `parse` and `schema()` is the `KEYS` list in `settings_tests.rs`, not a constant both read, so a key added to `parse` alone is not caught; (d) the live run ran before the phase 4 commit, so its result is in the ledger without a commit of its own.

## Live run (phase 5)

Run in new tabs of this session (`rex run -s "$REX_BLOCK" --focus=false`), with a temporary `XDG_CONFIG_HOME` and a sample folder holding a note named `sophie-neveu.md`.

- First start: the file list drew. `fmc/meta/config.schema.json` was created: 3332 bytes, draft-07, ten properties, `additionalProperties` false. No `settings.json` was created. `Ctrl+q` quit, and no fmc process was left.
- Second start (a new tab, same folder): the mtime (1795850029) and the SHA-256 were unchanged.
- Not run: a start with a read-only `meta/` and no schema, to see the footer notice. The permission system denied that command, and the agent did not retry it. The unit test `a_read_only_folder_gives_a_notice_and_no_panic` covers the failed write in a temporary folder.
- Both Rex tabs were closed.

## Tests

- `just verify` after phase 3: 775 run, 775 passed, 1 skipped; licenses ok. Phase 4 changed only documentation.
- `cargo test` after phase 3: lib 333, bin 433 (1 ignored), CLI 9, all passed. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` were clean.

## Owner checks

1. Editor completion (not run here). Open a `settings.json` that has the `$schema` line in VS Code or Zed, and check that the keys and their enum values complete and that a misspelt key is flagged. Why: the source check shows the link resolves; only an editor on the owner's setup shows completion.
2. The read-only notice on a real start. Run `! mkdir -p /tmp/fmc-live-ro/fmc/meta && chmod 555 /tmp/fmc-live-ro/fmc/meta && XDG_CONFIG_HOME=/tmp/fmc-live-ro /Users/ariss/Developer/github.com/riipandi/fmc/target/debug/fmc /tmp/fmc-live-sample.uThfLG`, read the footer (it should name `meta/config.schema.json`), quit with `Ctrl+q`, then run `! chmod 755 /tmp/fmc-live-ro/fmc/meta`. Why: this is the O2 path, and the live run did not reach it.
3. Remove the temporary files from this session: `! rm -rf /tmp/fmc-live-cfg.qwx3ro /tmp/fmc-live-sample.uThfLG /tmp/fmc-live-run1.json /tmp/fmc-live-run2.json /tmp/fmc-verify-p2.log /tmp/fmc-verify-p3.log /tmp/fmc-settings_tests.bak /tmp/fmc-docs-edit1.py /tmp/fmc-docs-edit2.py /tmp/fmc-docs-edit3.py /tmp/fmc-docs-edit4.py /tmp/fmc-vjls-jsonSchemaService.ts /tmp/fmc-vjls-jsonLanguageService.ts /tmp/fmc-vjls-types.ts /tmp/fmc-vjls-jsonParser.ts /tmp/fmc-vscode-jsonServerMain.ts /tmp/fmc-vs-extensions_json-language-features_server_src_jsonServer.ts /tmp/fmc-vs-extensions_json-language-features_server_src_node_jsonServerMain.ts /tmp/fmc-zedjson`. Why: they hold a temporary config, a sample file, logs, and downloaded source copies, and no real config.
4. Push and the pull request. The branch has the four commits above and is not pushed. Say when to push `feat/settings-schema` and open the pull request to `main`.
5. The open items of `handover-00011-settings.md` still stand (archived). Its real-terminal checks, the decisions A16, A19, A21, and A26, and the split of `src/app/mod.rs` (926 lines) are unchanged. Two of its follow-ups are fixed in the tree: `services/settings.rs` `load` reads through `read_config` (the size cap and the FIFO check), and the `file_associations` notice passes through `display_label` (`src/settings.rs:194`). The FIFO race and the temp file left by a quit during a save remain.
