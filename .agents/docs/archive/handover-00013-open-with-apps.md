---
status: done
created: 2026-11-28T15:35:00+07:00
plan: plan-00017-open-with-apps
---

# Handover: the open actions landed; plan-00017 closed

## Closed

- plan-00017 on `feat/open-with-apps` (from `docs/plans-ri-14-17`, not `main`). Commits: `12fdaa1` (the launch service: `open` argv, the JXA app list, `capture` moved to `services/mod.rs`), `52639c3` (`o`, the binary notice's key line, the menu's Open row), `7adc043` (`editor` in settings, `e`, `E`, the menu rows), `8b57fda` (`open_with` in settings, `O`, the picker, the menu's scrolling, D8 and D9), and the docs commit (phase 5, with this file). Phase 6 made no commit.
- RI-14, RI-15, and RI-16 are `landed` in `roadmap-00001-post-parity.md`.
- Behavior: `o` opens the row or the shown file with its default app, and a folder in Finder. `O` opens a picker: the apps `open_with` names for the file's pattern, then the apps macOS lists (the default marked), then Finder (always last). `e` opens the browsed folder in `editor`, or in Finder when it is empty. `E` shows the row in Finder. The Space menu has Open, Open With…, and Show in Finder on a row, and Open Folder in Editor on empty space. Every context menu scrolls when it is longer than its window.
- Where the build differs from the plan: (a) the architecture record is two records, not one: the owner's D8 and D9 came as an amendment, and `docs/parity.md` names each; (b) the footer for `o` is `Opened <name>.`, not `Opened <name> in <app>.`, because `open` does not say which app it used; (c) the Open With row and the picker are on folders too (D9 amends O4's "files only"); (d) the menu scrolls on every context menu, not only the picker (O5 says "the menu gains scrolling"); (e) an `open_with` item that is not a non-empty string is skipped alone, and a value that is not a list skips its pattern (the plan does not say which); (f) the spawn and empty-output errors of `capture` name the program (`git is not installed or could not be started.`, `Command output could not be read.`), so git's wording of those two changed; (g) the list of apps is read on a worker when `O` is pressed, and the picker opens when it lands.
- The file list's key `E` and the rows `e`, `E`, and `O` are in `docs/keymaps.md`, and the tie tests pass.

## Live run

Run in new tabs of this session (`rex run -s "$REX_BLOCK" --focus=false`), on a sample folder with `Gringotts.md` and `Pensieve.bin`, and an empty folder. Every tab was closed.

- The binary notice shows `Press o to open it in the default app.` under its hint.
- The Space menu on a file row lists Open (`o`), Open With… (`O`), and Show in Finder (`E`), then the write rows. On the empty folder it lists Open Folder in Editor (`e`) first.
- `O` on `Gringotts.md` drew the picker: `Markdown Preview (default)`, the apps macOS lists, and Finder at the end after a rule. Down scrolled the window to Finder, and Esc closed it with no launch (the footer showed no `Opened` line).
- The key page lists `o`, `O`, `e`, and `E` in the Files and Reader sections.
- At 44 by 26 (`stty` set before fmc started), the picker drew as the compact bottom sheet with sixteen rows. In the capture the sheet's last row and bottom edge sat under the footer. A measured frame at 44 by 24 fits (rows 5 to 22, footer on 23). The two do not agree, so this is an open check (owner check 2).
- Not run: `o`, `e`, `E`, or a picked app, since each opens a real app or Finder window. The unit tests stand in for the launcher.

## Tests

- `just verify` on the final tree: 816 run, 816 passed, 1 skipped (the ignored test); licenses ok. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean. `rumdl fmt --check --quiet .` clean.
- `cargo test`: lib 345, binary 462 (1 ignored), CLI 9, all passed. The new tests: `services/launch_tests.rs` (7: argv per `How`, a dash path after `--`, the listing's filter and order, an empty listing, the script's shape, a hostile file name, a missing file refused by `open`); `app/open_tests.rs` (19: the targets, the picker's rows and rule placement, the Finder row with an empty or failing list, the folder case, scrolling at 120 and 40 columns, Esc); `settings_tests.rs` (`editor`, `open_with`, the schema); `overlay_tests.rs` (the window); the menu tests moved for the new rows.

## Owner checks

1. A real launch, which no test or Rex tab does: `o` on a PDF (the default app opens it, and the focus is sensible), `e` with `"editor": "Zed"` in `settings.json` (Zed opens the folder), `E` on a file (Finder reveals it, selected), and `O`, then an app (the app opens the file). Why: each one opens a real app or window.
2. The compact picker on a real terminal at about 44 by 24: the bottom sheet's last row (Zed) and its edge may sit under the footer, as the Rex capture showed. The measured frame fits, so the capture may be an artifact of the Rex block's size. Why: only a real terminal shows the real size.
3. The footer wording for `o`: `Opened Gringotts.md.` names no app. Keep it, or name the default app (a listing before the launch, which costs about 0.1 s)? Why: the copy is a product choice the plan did not settle.
4. The amended rules (D8, D9): `O` on folders, the Finder row always last, and an `open_with` item that is not a string skipped alone. Confirm they match what you meant.
5. The `docs/usage.md` and `docs/configuration.md` text for the open actions, since the owner reads them as the product's words.

## Not in this work

- An untracked `.agents/docs/plans/plan-00019-footer-root-branch.md` appeared in the working tree during the session. It is not part of plan-00017 and is not committed here.
- Push: `feat/open-with-apps` is pushed with a normal push after the docs commit; no PR is open.
