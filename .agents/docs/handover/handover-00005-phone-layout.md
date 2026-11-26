---
status: done
created: 2026-11-27T02:49:32+07:00
plan: plan-00004-phone-layout
---

# Handover: phone layout landed; plan-00004 closed

## Closed

- plan-00004 phases 1 to 5, oldest first: `f46692d` (fold on release, P7; with a rumdl fix in `docs/parity.md`, since the gate was red at HEAD), `f4f46bd` (header tap, P1, and the drawer below 60, P3), `14b7078` (footer taps, priority, and the Copy hint, P2, P6, P8), `355957e` (finder rows name first, P4), `18b623f` (the duplicate `[ ]` row removed), `6be0b8e` (the welcome below 60 starts with its list hidden; auto-preview waits while the drawer covers the reader), `5ec24e4` (the narrow welcome below 50, P5), and the phase 5 docs commit.
- Tests: 283 at the start, 307 at the end (`just verify`).
- The welcome decision (drawer or full width below 60) is recorded in `architecture.md` by `cc7ae66`.
- Live matrix on the final build at 40x30, 50x40, 59x30, and 80x24, in tabs that were closed after each run: the welcome with its list hidden or shown, the drawer, a file opened from it, the keys page, the changes screen, and the finder rows.

## Where things live

- Sidebar and drawer rules: `src/layout.rs` (`Visibility::shown`, `covers`, `DRAWER`, `BREAKPOINT`, `NARROW_WELCOME`) and `src/app/layout.rs` (the header taps, the drawer, the mark).
- A tap on a file under the drawer opens it through the Enter path: `src/app/mouse.rs`.
- Auto-preview waits while the drawer covers the reader: `src/app/preview.rs` `wanted`.
- Footer: `src/ui/footer.rs` (`hint_row`, `hint_cells`, `key_at`), `src/ui/hints.rs` (`fit_hints`, `phone_rank`). A tap presses the hint's key through `src/app/capture.rs` `press_key` and `src/app/keys.rs` `hint_presses`.
- Welcome: `src/ui/welcome.rs` (`Welcome::fit`).
- Finder rows: `src/ui/tree.rs` `row_segments`.
- Docs: `docs/keymaps.md`, `docs/parity.md`, `README.md`, `CHANGELOG.md` (`[Unreleased]`), and `.agents/docs/rust.md`.

## Open for the owner

- The footer's priority order is the agent's pick (`ui/hints.rs` `phone_rank`). Confirm it, or name the order you want.
- Hints with no tap: `h l` (pan), `{ }` (paragraph), `^←→` (resize), and the prompt's `type`. A tap sends a hint's first key only.
- Under the drawer below 60 columns, the reader is hidden while the finder holds the list, so text-search snippets show only after Enter.
- The short-height rule of audit P5 (drop the welcome's closing line under 24 rows) is not implemented.

## Owner checks (real touch on a phone)

- Tap the reader's header row to show the list, and the list's header row to hide it.
- From the welcome at 40 columns: tap the header to open the list, `j` and `k` move without opening a file, and a tap on a file opens it and closes the drawer.
- Footer taps: `? help`, `/ find`, `Esc cancel`, and `^/ sidebar` act; `^q quit` does not quit.
- Drag-select text, then tap `Copy`.
- Tap a diff header to fold the file; a swipe that starts on a header does not.
- Finder rows show the file name before its folder.
