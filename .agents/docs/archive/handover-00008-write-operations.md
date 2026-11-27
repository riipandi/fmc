---
status: done
created: 2026-11-27T17:28:56+07:00
plan: plan-00012-write-operations
---

# Handover: write operations landed; plan-00012 closed

## Closed

- plan-00012 phases 1 to 5 and the fix: the write layer `047a58b`, the menu and dialog `17aefe0`, create, rename, and delete to the Trash `46f13a0`, copy, cut, and paste `738501b`, the fix `872fd21` (a replaced source is not trashed; a dialog's hover highlights a button and leaves the focus), and the docs commit `docs: the file-manager phase and its write operations`.
- The file-manager operations are fmc's only writes: create, rename, copy, move, and delete to the Trash, from the tree and its menu, inside the browsed root. The read-only rule is amended (`architecture.md`, "The file-manager operations are the one write path"; `rules.md`, "Writes").
- Dependency: `trash` `=5.4.0` (plan-00012 D4), recorded in `rules.md`, "Dependencies", and in `Cargo.toml`.
- Security: `security-review` on `src/services/write.rs` before the fix. One LOW finding (a folder swapped for a link after its `lstat` is read through the link) is a `shortcut:` in `copy_entry`, with its upgrade named, and it is recorded in `architecture.md`, "Every write checks its path before it acts". The fix for the replaced source and the swapped file is in `872fd21`.
- Roadmap: RI-4 landed; RI-19 (multi-select and bulk operations, plan-00012 D7) parked.

## Owner checks

These need a real terminal or the owner's hand. The agent could not judge them.

1. **The name cursor.** Press `a` and check that the terminal cursor shows in the name field. Press Tab and check that it leaves the field. Test in Ghostty and in Terminal.app.
2. **The hover.** Hover a button in the delete confirm: its label should turn accent-coloured with no fill, and Return should still pick Cancel. Then hover a context-menu row: it should select.
3. **Right-click.** A right-click on a row should open the menu at the pointer. Check in Ghostty and in Terminal.app; the `Space` key is the fallback.
4. **Automation permission.** The first delete asks macOS to let fmc control Finder. Grant it, delete a test file, and check that Finder's Put Back restores it.
5. **A taken name.** Rename or create onto an existing name: a notice should show, and the dialog should stay open with the typed name.
6. **Decide: a Replace, Keep both, or Cancel dialog (D5 deferred).** A paste into a taken folder takes `copy`, `copy 2`, and so on. Is that enough for now?
7. **Decide: RI-19 (multi-select).** Parked. Should it be the next file-manager work?
8. **Decide: the directory race (LOW).** The `shortcut:` in `copy_entry` can be fixed now, by opening the folder and comparing its identity before reading. It needs a process running as the user to exploit.

## Decisions the agent took (cheap to undo; each has its record)

- `copy_then_remove` takes the copy step as a parameter, so a test can replace the source during the copy. The same test seam as `trash_with`.
- The dialog's button hover is accent text with no fill, so it is not mistaken for the focus. The owner's decision was highlight only; the style is the agent's choice (`ui/dialog.rs`). Owner check 2 covers it.
- The archived plan names its phase 5 commit by message, not by hash: the commit's hash is its own.
- Done handovers archive when a newer one lands (`agent-docs`): `handover-00005`, `00006`, and `00007` moved to `archive/`. `handover-00001` stays, it is partial.

## Validation

- `KACHE=false just verify` at each phase: 629 tests passed, 1 skipped, after the fix. `cargo build` passes.
- The write layer's unit tests run in temporary folders. The Trash is a test stand-in.
- Phase 6 (the final `KACHE=false just verify`, a Rex run at 120x40 and 45x30, and `rumdl`) runs after the docs commit. Its result is in the session report, not in this ledger.

## Where things live

- `src/services/write.rs` (the write layer), `src/services/write_tests.rs`.
- `src/app/write.rs` (the tree's operations and their dialogs), `src/app/write_tests.rs`.
- `src/overlay.rs` (the menu's and the dialog's model), `src/app/overlay.rs` (the views and their clicks), `src/ui/menu.rs`, `src/ui/dialog.rs`.
- `docs/usage.md` ("Managing files"), `docs/keymaps.md`, `docs/parity.md` ("File operations").
- `architecture.md` records: the file-manager path, the write checks, the clipboard, and the Trash.
