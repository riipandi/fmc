---
status: done
created: 2026-11-27T03:45:00+07:00
plan: plan-00003-browse-ux
---

# Handover: browse UX landed; plan-00003 closed

## Closed

- plan-00003 phases 1 to 7. Commits, oldest first: `cd1e413` (phase 1: tree arrows stay out of the reader, `Ctrl+/` focuses the shown sidebar), `e7a3580` (phase 2: `settings.json`, with `serde_json` only), `e78db07` (phase 3: Enter opens and focuses, `l`/`→` only expand, a first click only focuses), `7d5e593` (phase 4: auto-preview, `p`), `3474efa` (phase 5: `[` and `]`), `d788b83` (phase 6: `:q`), `8b8a88c` (a refactor that moves the mouse wiring out of `app/mod.rs` to keep it under 800 lines), `f64bbd9` (an issue for the finder row), and the phase 7 closing commit.
- Tests: 246 at the start, 281 at the end (`just verify`). `cargo clippy --all-targets -- -D warnings`, `cargo build --release`, and `cargo deny check` pass.

## Where things live

- Key capture: `src/app/capture.rs`. A key the parser handles stops there, so the reader's scroll box never sees it (`rust.md`, "Keys are read in capture").
- Auto-preview: `src/app/preview.rs`. A list move starts a 100 ms timer, and a read returns as `Event::Selected` (`rust.md`, "A preview is a timer, a generation, and its own event").
- Settings: `src/settings.rs` (pure parse) and `src/services/settings.rs` (path and one read in `main.rs`, before the UI). Docs: `docs/configuration.md`.
- Command line: `src/app/command.rs`. The `:` prompt uses `Prompt::Command` in the footer.
- Mouse wiring: `src/app/mouse.rs`.

## Open for the owner

- The 160-column footer test pins Scouter's footer. Plan phase 5 asked for a `[ ]` footer hint, and it was not added (open question below).
- `serde`'s derive is not used. The settings record lists `serde` (derive) and `serde_json` as approved crates; only `serde_json` is in `Cargo.toml`.
- `issue-00012` (the finder row that reads `..gitignore` in a wide tab) is open. It does not reproduce at 80x24, nor headless at 160 columns.

## Owner checks

- Live runs at 80x24 in new tabs, all closed: arrow on a folder leaves the reader unchanged; `Ctrl+/` focuses the list; a bad `settings.json` shows its notice in the footer; Enter opens and focuses; the first click only focuses; holding `j` previews the last file; `p` toggles the preview; `[` and `]` move the list two rows and the reader five lines; `:` shows the prompt, `Not a command` shows, Esc cancels, `:q` exits.
- Not checked live: the footer at a width other than 80 (the finder row issue), and the mark click, which still shows or hides the sidebar without moving the focus.

## Open questions

1. Footer hint for `[ ]`. Options: (1) leave the footer as Scouter's, and the key card shows `[ ]` (current); (2) add `[ ] jump` at the end of the list and reader hints, and update the 160-column test and `parity.md`. Recommended: (1) unless the owner wants the hint visible.
2. Does a mark click that shows the sidebar also focus the list, as `Ctrl+/` does? The record says only `Ctrl+/`, so it does not. Recommended: keep it that way.
3. Confirm that a click on a file previews it, not only selects it. Phase 4 makes it preview, as `architecture.md` says.
