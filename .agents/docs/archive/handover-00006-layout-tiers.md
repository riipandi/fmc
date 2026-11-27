---
status: done
created: 2026-11-27T03:58:00+07:00
plan: plan-00005-layout-tiers
---

# Handover: layout tiers landed; plan-00005 closed

## Closed

- plan-00005 phases 1 to 6, oldest first: `166051c` (tiers, row actions on release), `2ade9d6` (compact pages: Files and Reader, no border, one-row headers, no welcome, no preview), `c962920` (top bar and bottom bar, the bottom bar's words and prompt bar, the back text), `d3b1943` (medium chevrons; boxes, header-row tap, and the drawer gone), `b9b7ca3` (probe bytes pinned by tests; no handler changed), and the docs commit that closes the plan.
- The design is the record "three layout tiers, and a phone-native compact tier" in `architecture.md`, items 1 to 12. The agent decisions taken under the owner's delegation are listed under it, each with its reason.
- Tests: 322 at the start, 350 at the close. `just verify`, `cargo build`, and `cargo build --release` pass.
- The touch probe log was read and not cleared. It held 1087 mouse events from 747 chunks: 43 taps, each a press and its release at one cell, and five swipe runs of wheel codes (1001 events), with no drag, motion, or long-press events and no unpaired press.

## Live matrix (final build, Rex tabs closed after each run)

- Ten steps at each width: Files, the reader after `G` and Enter, `q`, Find, Escape, Keys, `?` again, Changes, the diff, and `d`. Captures are under `/tmp/fmc-cap/w<cols>-<cols>/`, not in the repository.
- 80x24 is byte-identical to the phase 1 build (`166051c`) on all ten steps. 60x24 and 70x24 differ from it only where the chevrons and the boxes changed.
- Compact (40, 49, 59 columns): the top bar reads `‹ Files` on Reader, Keys, and Changes, the folder name on Files, and the finder's prompt on Find. The bottom bar reads `Find · Changes · Keys` on Files, `Back · Search · Keys` on the reader, `Back · One · Split` on Changes and its diff, `Back` on Keys, and `Cancel` then the prompt on Find.
- Medium (60 and 70): the reader's `›` shows the list and the list's `‹` hides it, both checked by an SGR press written to the block.
- Bottom-bar taps at 49 columns: the `Find` cell opens the finder, and the `Cancel` cell closes it.

## Where things live

- Tiers and the stack: `src/layout.rs` (`Tier`, `tier`, `Page`, `Pages`, `Visibility`) and `src/app/pages.rs` (`sync` at each paint, `back`, `switch`, `back_label`, `Model::compact`).
- Sidebar, chevrons, and the back press: `src/app/layout.rs` (`shown`, `width`, `list_chevron`, `reader_chevron`, `lay_out`, and the lead-press handler in `wire`).
- Bars: `src/ui/bars.rs` (`page_segments`, `cells`, `render`, `prompt`, `back_text`) with `src/ui/bars_tests.rs`; the footer's compact branch is `Footer::paint` and `FooterState::bar` in `src/ui/footer.rs`.
- Headers: `src/ui/header.rs` (`lead`, `on_lead_press`, `compact`) and the finder bar's `compact` in `src/ui/find.rs`.
- Clicks: `src/app/mouse.rs` (`Click`, `act_on_row`, the list and changes handlers); tests in `src/app/mouse_tests.rs`, including the logged bytes.
- Keys: `src/app/capture.rs` (the compact `q` and `Esc` arms).
- Docs: `DESIGN.md` (Tiers, with sketches), `docs/keymaps.md`, `docs/parity.md`, `README.md`, `CHANGELOG.md` (`[Unreleased]`), and `.agents/docs/rust.md`, `shape.md`, `architecture.md`.

## Open for the owner

- Agent decisions with the most visible effect, each reversible: the two columns after the back text (the spec said one), `All` or `One` for the scope on screen, `Copy` in place of `Search` while text is selected, and no reader position (`Top`, `37%`) in the compact bars. Confirm or name the change.
- Chevrons are one cell wide. A thumb may miss them; say whether the tap target should be wider.
- The welcome card at 60 to 79 columns wraps its text beside the 28-column list. This was already so before the plan and is not changed. Decide whether the card waits for a wider screen.
- Medium tier (60 to 79): a tap on a row while the reader has the focus only focuses the list, as approved for a click (`architecture.md`, "a click focuses the pane under the pointer", amended). A second tap opens the file. Confirm that reads right on a phone.
- The probe's bytes showed no long press. If a long press does something on the phone, say what it should do.

## Owner checks (real touch on a phone, Rex)

- At about 49 columns: the list is the first screen with no border; a tap on a file opens it full screen; `‹ Files` at the top goes back; the list's selection is where you left it.
- The bottom bar's words: `Find` opens the finder and shows `Cancel` in the bar; `Cancel` closes it. On the reader, `Search` opens the search, and `Keys` opens the key list.
- A swipe over the list or the reader scrolls it and opens nothing.
- At 60 to 79 columns: the `›` at the reader's title shows the list; the `‹` at the list's title hides it; a tap on a title does nothing.
- Changes at a phone width: `Back`, `One` or `All`, and `Split` work; a tap on a change opens its diff full screen.
- A notice (for example an unreadable file) appears in the bottom bar and clears by itself.

## Notes for the next session

- The probe log is still at `/tmp/fmc-probe/touch.log`; the owner's probe tab is still running, and it was not touched.
- The demo repository was reset with `scripts/demo-repo.sh /tmp/fmc-ref/demo-fmc` at the close.
- The unit suite runs with nextest; `cargo test` alone also works.
