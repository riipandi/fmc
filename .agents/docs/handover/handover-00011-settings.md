---
status: done
created: 2026-11-27T21:20:56+07:00
plan: plan-00015-settings
---

# Handover: settings, themes, and the keymap file landed; plan-00015 closed

## Closed

- plan-00015 phases 1 to 4 on `feat/settings` (not pushed): the settings write path `3f5673c`, the theme mode and the theme catalog `64c7aea`, the Settings page on `Ctrl+,` `d68af26`, and the keymap file over the defaults `1b37a2e`. Phase 5 is the docs commit, named by its message, `docs: settings, themes, and the keymap file`.
- The write path: `settings::merged` (`src/settings.rs`) sets the changed keys over the file as it is now and refuses text that is not a JSON object. `services/settings.rs` `save` writes a temp file beside the target and renames it over it; a symlink is written through to its target. No other config file is written.
- The theme catalog: 16 bundled themes from bjarneo/editor (MIT, `src/theme/catalog/LICENSE`), `theme_mode`, `dark_theme`, and `light_theme` in `settings.json`, and user themes in `themes/<id>.json`, read at start. The palette holds `Hex` values, not `Cow` (A15).
- The Settings page: `Ctrl+,` only (D12). Eight rows; Enter or Space changes a row, Left and Right step a choice, and a number opens the dialog. Each change applies and saves at once from a worker. A refused save puts the value back and the footer says why.
- The keymap file: the default keys are rows of `src/keys/bindings.rs`. `keymap.json` binds and unbinds over them; a bad entry is skipped with a notice that names its line. The `?` card and `--help` read the merged table.
- Phase 5: the records were checked against the code (the list is in the phase report); RI-9 is `landed in part`, RI-10 and RI-13 are `landed`; the index is updated; plan-00015 is archived with `status: done`. `handover-00009` and `handover-00010` are archived, since a newer `done` handover landed (the rule in `agent-docs`; `handover-00009` should have moved when `handover-00010` landed).
- Phase 5 reworded four accurate sentences that said "never writes": three about themes and the keymap now say "only reads", and one changelog line now says the settings file is no longer "read-only". The meaning is unchanged, and `grep "never writes"` finds only the released 0.1.0 entry.

## Owner checks

1. Remove the stray untracked file: `! rm /Users/ariss/Developer/github.com/riipandi/fmc/src/app/keymap_tests.rs`. It is in no commit, and no module names it (the only `keymap_tests.rs` paths are `src/keys/keymap.rs:163` and `src/services/keymap.rs:39`). This phase did not read its contents.
2. Remove the temp folders the live runs left in `/tmp`. The phase 2 and phase 3 `rm` calls were denied. These hold temp settings, a theme file, and sample files, and no real config: `! rm -rf /tmp/fmc-live-cfg.T5KVsx /tmp/fmc-live-cfg.Vk598M /tmp/fmc-live-notice.AJb7lu /tmp/fmc-live-sample.hBQT0j /tmp/fmc-live-sample.Ku9WXa /tmp/fmc-live-empty.3aDQtQ /tmp/fmc-live-before.json /tmp/fmc-live-cfg-path /tmp/fmc-live-sample-path /tmp/fmc-live-run.json /tmp/fmc-live-run2.json`. The two `run` files hold Rex window and block ids. The list comes from `ls /tmp/fmc-live-*`, since the phase reports were not in this session.
3. Check these folders before removing them. They are older than this plan (18:28 to 18:37, and 12:18, on 2026-11-27; the plan was approved at 20:05). `/tmp/fmc-live-demo` and `/tmp/fmc-live-dup` are copies of a sample project with `.git`, `node_modules`, and `target`: `! rm -rf /tmp/fmc-live-demo /tmp/fmc-live-dup /tmp/fmc-live-1-ids.txt /tmp/fmc-live-2-block.txt /tmp/fmc-live-run1.json`, once you confirm they are not needed.
4. Real-terminal checks (the Rex captures show glyphs, not colours or key delivery):
   - `Ctrl+,` opens the Settings page in Ghostty. Terminal.app may not send it, which is RI-12's open check. A terminal without the kitty keyboard protocol has no key for the page; you accepted that in D12.
   - The theme colours on a real dark and a real light terminal, with `theme_mode` on `auto`, then forced to `dark` and to `light`.
   - The 16 bundled themes: 12 fall short of 7:1 on a tint or a match pair (`SHORT` in `src/theme/theme_tests.rs:84`), and `tokyo-night-day` is also under 4.5:1 on three tints (A16).
   - Esc in the file list drops a pending cut: the footer should stop showing it (the phase 4 check; `docs/keymaps.md`, Everywhere).
5. Decisions to review, each a one-line change:
   - A16: the 12 short themes. Keep the colours and the list, change a colour, or change the threshold.
   - A19: Left and Right (`h`, `l`) step a choice back and forward; Enter and Space step forward.
   - A21: the page is modal to the mouse. Whether a click should select a row is open.
   - A26: a second key for an action joins that action's row on the card. The other reading, each added key on its own row, is open.
   - The shape decisions: A22 (the flat object `"<context>.<key>": "<action>"`), A23 (68 fixed action names), A24 (modifiers as the old arms had them), A25 (a sequence waits; the defaults have none).
   - The Settings page has no roadmap row, by your rule; RI-20 is not added. Ask before adding one.
6. `src/app/mod.rs` is 926 lines, over the 800-line rule in `rust.md`. The split waits for your decision.
7. Follow-ups found in this phase. They are recorded, not fixed:
   - `src/services/settings.rs:21-24` `load` reads `settings.json` at start with `fs::read_to_string`, with no size cap and no FIFO check. The 1 MiB cap and the FIFO refusal (A28) cover the theme and keymap reads only (`src/services/mod.rs:46-60`). A FIFO or a very large `settings.json` blocks or allocates at startup.
   - `src/settings.rs:117-119`: the `file_associations` notice echoes the pattern raw (`"{pattern}"`). The theme and keymap notices pass through `display_label`; this one does not, so a pattern with control characters reaches the footer.
   - A FIFO put in place between the check and the open can still block the startup read of a theme or keymap file (phase 4 residual, A28).
   - `services::spawn` threads are not joined, so a quit during a save can leave `.settings.json.<pid>.tmp` beside the file (phase 3 residual). The rename has not happened in that case, so the settings file keeps its old bytes.
8. Phase 6 is marked `done` in the plan, as the brief asked. The plan's test matrix still names checks that this session did not run: `security-review` on phases 1, 3, and 4, and `parity-audit` on the list, the reader, and the card (phase 4). Confirm them in phase 6, or from the phase reports.

## Decisions

The owner approved D1 to D16 on 2026-11-27, with one change: D12 binds `Ctrl+,` only (no plain `,`, no `:settings`). The agent decisions are in the plan's Decisions section (`.agents/docs/archive/plan-00015-settings.md`).

- A1: the write is atomic: a temp file beside the target (`create_new`), `sync_all`, then a rename. `fs::write` and `tempfile` were rejected.
- A2: a save re-reads the file and sets only the changed keys. Text that does not parse is refused and kept.
- A3: a save writes the keys sorted, since `serde_json` has no `preserve_order` here.
- A4: the bundled catalog is JSON in `src/theme/catalog/`, embedded with `include_str!`, with bjarneo's `LICENSE` beside it.
- A5: palette fields as `Cow`. Amended by A15.
- A6: the page draws its rows like the tree, not with OpenTUI's `Select`.
- A7: the settings are `theme_mode`, `dark_theme`, and `light_theme`. A bad id or mode keeps the default and adds a notice.
- A8: the key table is data. The defaults reproduce the old match arms, and `keys/tests.rs` passes unchanged.
- A9: the bjarneo role mapping: 18 roles map to fmc's tokens, 11 values are derived by fixed mixes, and 6 roles are read and checked but not mapped. The plan names the role `fun`; the code and the bundled files use `fn`.
- A10: the card reads the merged table, and a user binding shows on its action's row.
- A11: a bad keymap entry is skipped with a notice that names its line. `null` unbinds. A user `g` sequence makes `g` wait.
- A12: every live run uses a temporary `XDG_CONFIG_HOME`.
- A13: the id `scouter` is Scouter's pair: `DARK` in a dark slot, `LIGHT` in a light slot.
- A14: a forced mode that differs from the terminal uses the palette's fallback background.
- A15: palette fields are `Hex`, a seven-byte `Copy` value (amends A5).
- A16: 12 bundled themes fall short of 7:1; the pairs are listed (open, owner check 4).
- A17: a theme's id is its file name, and a file named `scouter.json` is refused.
- A18: the footer shows the notices in order: settings, themes, keymap, then the palette choice (extended by A27).
- A19: Left and Right (`h`, `l`) step back and forward; Enter and Space step forward (open, owner check 5).
- A20: one save is in flight. A change made during a save waits as one batch, and a refused batch takes back itself and the queue.
- A21: the page is modal to the mouse (open, owner check 5).
- A22: the keymap file is one flat object: `"<context>.<key>": "<action>"` to bind, `null` to unbind.
- A23: action names are the variants in snake_case, 68 names, listed in `docs/keymaps.md`.
- A24: modifiers match as the old arms did. A differential run over 945,024 cases showed no difference; the run was a scratch test outside the tree.
- A25: a sequence waits (`More`), and the defaults have none.
- A26: a card row lists the merged keys of its actions (open, owner check 5).
- A27: the notice order is settings, themes, keymap, then the palette choice. `--help` reads the file and prints its notices to stderr.
- A28: config reads are capped at 1 MiB, and a FIFO is refused by its metadata before it opens. The residuals are in owner check 7.
- A29: `tests/cli.rs` sets `XDG_CONFIG_HOME` to a folder that does not exist.

## Validation

- `KACHE=false just verify` in phase 5 (after the docs edits): exit 0; 757 tests run, 757 passed, 1 skipped (the `#[ignore]` timing report at `src/ui/markdown_blocks_tests.rs:330`); licences ok. The test count is to be confirmed in phase 6.
- `KACHE=false just format-check` (rustfmt and rumdl): exit 0.
- `cargo build`: exit 0.
- The roadmap: 19 status lines and 19 table rows. RI-9 is `landed in part`, and RI-10 and RI-13 are `landed` in both the table and the entry.
- No live run in phase 5; it changes no screen.

## Where things live

- The plan: `.agents/docs/archive/plan-00015-settings.md` (done).
- The records in `.agents/docs/architecture.md`: "Keys are dispatched from one key table through a pure pending-key parser" (the correction), "The default keys are one table, and keymap.json merges over it", "Approved deviation: the keys can be changed in keymap.json", "A bad keymap entry is skipped with a notice that names its line", "The `?` card and `--help` show the merged keys", "Decision: a read-only `settings.json` with its keys" (superseded in part), "fmc writes its settings.json, atomically, and never over text it cannot read", "Approved deviation: the Settings page edits settings.json on Ctrl+,", and "Approved deviation: fmc offers a theme catalog, chosen in settings.json".
- The rules: `.agents/docs/rules.md` ("Writes", "Licenses") and `AGENTS.md` ("Writes are bounded").
- The human docs: `docs/configuration.md`, `docs/keymaps.md` ("Changing the keys"), `docs/usage.md` ("Settings"), `docs/parity.md`, `DESIGN.md` ("Catalog", "Settings page"), and `CHANGELOG.md` `[Unreleased]`.
- The roadmap: `.agents/docs/roadmap/roadmap-00001-post-parity.md`, RI-9, RI-10, and RI-13.

## Suggested skills for the next session

- `rust-verify-harden` on `src/services/settings.rs` and `src/services/mod.rs` (the follow-ups in owner check 7).
- `security-review` on the settings save and the keymap read (the plan's phases 1, 3, and 4).
- `parity-audit` on the `?` card and the list keys (the plan's phase 4 row).
- `agent-docs` for the closing sweep.

## Trap that bit this session

An unquoted `--include=*.md` in zsh fails with "no matches found". Quote the glob in `grep`.
