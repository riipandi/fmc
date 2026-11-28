---
status: in-progress
created: 2026-11-28T18:50:00+07:00
updated: 2026-11-28T18:55:32+07:00
phases:
    - id: 1
      title: The hex layout, the string match, and the lossy text in the core
      status: pending
      note: —
    - id: 2
      title: Block reads and the string search on workers
      status: pending
      note: —
    - id: 3
      title: The hex view, its keys, the jump, and the notice's choices
      status: pending
      note: —
    - id: 4
      title: String search in the hex view
      status: pending
      note: —
    - id: 5
      title: Text mode from the notice and the refusals
      status: pending
      note: —
    - id: 6
      title: Docs sync, agent docs sync, ledger close, handover
      status: pending
      note: —
    - id: 7
      title: Final validation, live run, PR
      status: pending
      note: —
---

## Open a file fmc refuses as text or in a hex view — fast at any size, read-only

### Objective

On 2026-11-28 the owner asked that a file fmc cannot show (a binary, or text it refuses) open as plain text or in a hex view, chosen from the notice by key or click, with `o` (default app) kept, and that the hex view can search for a string. After this plan the notice offers three choices; the hex view shows offset, hex bytes, and an ASCII column for a file of any size, reading only the visible window on a worker, so open, scroll, a jump to an offset, and the end stay instant on several GiB; a string search runs over the whole file on a worker. Nothing is written: this is a viewer (`rules.md`, "Writes").

### Current state (verified in the tree, 2026-11-28)

- **The refusal.** `src/reader/document.rs:47-51` makes a NUL in the first MiB `Refusal::Binary`, invalid UTF-8 `Refusal::NotUtf8`; `src/services/fs.rs:83-94` maps `Binary` to `Opened::Binary { size }` and the rest to a footer error. A FIFO or device is refused before its open (`:45-47`); a large or unreadable image is refused at `:98-120`.
- **The notice.** `src/ui/binary.rs:9-13,56-64`: title, name and size, hint, `Press o to open it in the default app.`; shown by `src/app/reader.rs:215-224` `show_binary`, focus on the list (reader in compact).
- **No windowed reader.** Every read is whole up to a cap (`fs.rs:86-88`, 1 MiB; images 10 MiB). The reader's `ScrollBox` holds the whole text (`src/ui/viewer.rs:17`).
- **A virtual list exists.** The tree paints only visible rows and drives `src/ui/scrollbar.rs:45-47` with `usize` totals.
- **The search.** The reader's search ignores case and runs over the shown text on the UI thread (`src/reader/search.rs:1-3`, `src/app/search.rs`); its prompt is `m.search` with `editing`, which the key capture routes (`src/app/capture.rs`).
- **Research.** `.agents/docs/research/hex-view-research.md`: `read_at` with a 64 KiB block cache costs 0.03 to 0.07 ms to open and read the last page of 2 GiB, 2 µs per page scroll, 7.5 µs per random jump; `memmap2` is 5 µs faster per jump and SIGBUSes on a shrunk file (measured); a hand formatter is 11x faster than `pretty-hex`; `Painter::cells` draws a screen in one record; a std scan of 2 GiB takes about 0.5 s.
- **Scouter.** No hex or text fallback: `scouter:src/core/files.ts:172,183`, `scouter:src/app/app.ts:329-341,411-419`, `scouter:src/reader/binary-view.ts:60-69` (v0.3.4, `434d000f`). This plan is a parity deviation.

### Decisions

- **D1 (2026-11-28): positioned reads, not a memory map.** `std::os::unix::fs::FileExt::read_at`, one `services::spawn` job per 64 KiB block (open, read, answer; the open costs tens of µs, and a job per block needs no long-lived worker to stop); the UI caches up to 16 blocks and asks for the blocks around the viewport. A short read is the new end, so a file that shrinks shows its new end instead of crashing. Cost: a block not yet cached paints blank for a frame. Rejected: `memmap2` — kills fmc with SIGBUS when the file shrinks under it (research §3), needs `unsafe` and a new dependency, for 5 µs per jump. Rejected: reads on the UI thread — `rust.md` forbids file I/O there, and a network volume can stall seconds.
- **D2 (2026-11-28): a hand-written formatter.** It builds `PaintCell`s with colours directly; 1.8 µs per screen. Rejected: `pretty-hex` (returns a string, no byte classes, 11x slower); `encoding_rs` and `chardetng` (legacy encodings and guessing nobody asked for; 5 and 3 extra crates).
- **D3 (2026-11-28): the hex view is a `Custom` component outside the reader's `ScrollBox`**, painted with `Painter::cells`, with its own top offset and the bar from `ui/scrollbar.rs`. Rejected: rows in the `ScrollBox` — 134 M rows for 2 GiB.
- **D4 (owner, 2026-11-28, O1 option 1): `t` opens the shown file as text, `X` in the hex view.** Both in the `browse` and `reader` contexts (free in both, and in `everywhere`), so they work from the notice whatever the focus; inside either view they switch to the other. The notice draws them as clickable buttons beside `o`. Rejected: one cycling key (a hidden order).
- **D5 (owner, 2026-11-28, O2 option 1): binary and not-UTF-8 files get the choices.** A not-UTF-8 file shows the notice with Scouter's refusal as its title (`Only UTF-8 text files are supported.`) instead of a footer error. Images past 10 MiB or unreadable keep their footer error; FIFOs and devices stay refused. Rejected: images too (beyond the ask); binary only (leaves Latin-1 text unreadable).
- **D6 (owner, 2026-11-28, O3 option 1): bytes per row follow the pane: 32, 16, 8, or 4, the widest that fits.** Groups of 8 split by one space, an offset of 8 hex digits (more past 4 GiB), and the byte classes coloured from the palette: NUL `ignored`, printable ASCII `text`, ASCII whitespace `added`, other ASCII controls `accent_soft`, non-ASCII `modified`; the offset `dim`. The ASCII column shows printable bytes and `.` for the rest. Rejected: a fixed 16 with a pan (half the screen off at 80 columns).
- **D7 (owner, 2026-11-28, O4 option 1): text mode shows invalid UTF-8 as `�` and NUL and C0 controls as control pictures (`␀`, `␛`, U+2400 to U+241F, `␡` for DEL)**, tabs as four spaces, CR and CRLF as line ends; C1 and bidi controls stay `�` (`text::display_text`). If OpenTUI measures a control picture two cells wide, they fall back to `�`. Rejected: all `�` (NUL and ESC look the same).
- **D8 (owner, 2026-11-28, O5 option 1): text mode is the existing reader on the first 1 MiB, decoded lossily.** Search, wrap, selection, `y`, line numbers, and the `first 1 MiB` badge work as for any text; a `lossy` badge says the bytes were changed; the hex view covers the whole file. Rejected: a windowed text view (no search, wrap, or line numbers in its first version, and a second text renderer to keep).
- **D9 (owner, 2026-11-28, O6 option 1 plus the owner's ask for string search): the hex view jumps and searches.** `:` takes an offset (`:0x1f40`, `:8000`, `:50%`), `g` and `G` go to the start and the end. `/` searches the whole file for a string (its UTF-8 bytes, ASCII case ignored, as the reader's search ignores case), from the view's top on Enter, on a worker that stops when a newer search starts or the view closes; `n` and `N` find the next and previous hit, wrapping at the ends. The hit is marked in both columns and scrolled into view; the footer says `searching…`, then the hit's offset or `no matches`. No hit count: counting a 2 GiB file is a full scan per keystroke. Rejected: hex-byte queries (`/ 7f 45`) — the owner asked for strings; a later plan can add them.
- **D10 (2026-11-28): no new dependency.** std covers reads, formatting, lossy decoding, and the search.

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The deviation | `architecture.md`, new "Approved deviation: a refused file opens as text or hex"; `docs/parity.md` line | Scouter's notice and refusals (§6 of the research), fmc's choices, the owner's approval and date |
| D1 to D3 | `architecture.md`, a decision record "A huge file is read in blocks on workers" | the numbers and the rejected mmap |
| The block contract | `rust.md`, "Contracts and traps" | the block cache, a short read is the end, a late block for another file is dropped, a search stops on its flag |
| Keys | `src/keys/bindings.rs`, `docs/keymaps.md` | the new rows |
| The screen | `docs/usage.md`, `DESIGN.md` (hex colours), `CHANGELOG.md` | what users see |
| Layout | `shape.md` | the new modules |

### Execution rules

- **Atomic commit per phase**, the message named in each phase; the owner's go-ahead (2026-11-28, "kerjakan berurutan") covers the phases in order. Commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Read-only.** No file is opened for writing; the workers open with `File::open` only.
- **No new dependency.**
- **The other agent's files are untouchable:** `src/ui/markdown*`, `src/reader/images.rs`, and the plan-00022 ledger.
- **New code goes in new modules** (`src/app/mod.rs` is already 956 lines; the limit is 800): `src/reader/hex.rs`, `src/services/window.rs`, `src/ui/hex.rs`, `src/app/hex.rs`, each with a sibling tests file.
- **Per-phase validation** runs before each commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; `just verify` before phases 6 and 7.
- **Live runs** in a new tab of this Rex session (`rex run -s "$REX_BLOCK" --focus=false -- …`), never a new session; the tab closes after.

### Phases

1. **The hex layout, the string match, and the lossy text in the core** — `src/reader/hex.rs`: bytes per row for a width (D6), the byte classes, a row's cells (offset, hex groups, ASCII, the hit), the row count for a size, the offset parse for a jump (D9), the block cache (16 x 64 KiB, LRU, the blocks a window needs), the case-ignoring string match forward and back in a buffer (D9); `document::decode_lossy` (D7, D8). Validation: `cargo test hex document` covers widths 30, 50, 80, 120, 200; the last short row; offsets past 4 GiB; every byte class; the cache's eviction and a short last block; a match back and forward; the lossy mapping of NUL, ESC, DEL, invalid UTF-8, a cut character. _Commit: `feat(reader): hex rows, a byte search, and lossy text in the core`._
2. **Block reads and the string search on workers** — `src/services/window.rs`: `read_block` (open, `read_at` until 64 KiB or the end), `search` (1 MiB chunks with an overlap of the query's length less one, forward or back from an offset, wrapping, stopping on a flag); `Event::Block` and `Event::HexFound`. Validation: tests on temp files read the first, middle, and last block; a file truncated after the open gives a short block; a search finds a hit across a chunk edge, wraps, goes back, and stops on the flag; `grep` finds no write call in the module. _Commit: `feat(services): read blocks and search a file on workers`._
3. **The hex view, its keys, the jump, and the notice's choices** — `src/ui/hex.rs` (`Custom`, `Painter::cells`), `Shown::Hex` in `src/app/reader.rs`, `src/app/hex.rs` (open, scroll by row, page, half page, `g`, `G`, `[` `]`, the wheel, the bar drag, `:` offset jump, a resize keeps the top byte, blocks requested around the view, a late block of another file dropped), the notice's buttons and `t`/`X` (D4), the header badge `hex`, the footer's size and position. Validation: `TestApp` tests for the keys, the click, the jump, a block landing; live run at 80x24 and 160x48 on a 2 GiB sparse file and on `/bin/ls`: open, `G`, a jump, a page scroll without a visible stall. _Commit: `feat(reader): a hex view for files fmc cannot show as text`._
4. **String search in the hex view** — `/`, Enter, `n`, `N`, Esc through the reader's search prompt; the worker's answer moves the view and marks the hit; a newer search or leaving the view stops the old one. Validation: `TestApp` tests: a hit past the first block, a wrap, `N` back, no match, Esc; live run on the 2 GiB generated file. _Commit: `feat(reader): search a string in the hex view`._
5. **Text mode from the notice and the refusals** — `t` reads the first 1 MiB lossily into the reader (D7, D8) with a `lossy` badge; a not-UTF-8 file shows the notice with its own title (D5). Validation: `TestApp` tests for a Latin-1 file and a binary, `t` then `X` then `t`; live run of each. _Commit: `feat(reader): open a refused file as text`._
6. **Docs sync, agent docs sync, ledger close, handover** — every record of the table above, `index.md` rows, the plan archived, a handover. Validation: the docs' claims checked against the landed code; `just verify` green (the keymaps test reads `docs/keymaps.md`). _Commit: `docs: the hex and text views`._
7. **Final validation, live run, PR** — the matrix; push the branch; open the PR to `main`, not merged. _Commit: none if green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check` | No drift |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Unit | `cargo test hex window` | layout, classes, lossy, blocks, search, keys |
| Full suite | `cargo test`, `just verify` | All green |
| Large file | 2 GiB sparse and 2 GiB generated, release build, live | open, `G`, jump, scroll each without a visible stall; a search reports within a second |
| Shrink | truncate the shown file while the hex view is open | fmc shows the new end, no crash |
| Live run | Rex tab, 80x24 and 160x48, binary notice → hex → search → text → `o` | the choices drawn, keys and clicks work, a hit is marked |
| Parity | the deviation recorded in `architecture.md` and `docs/parity.md` | both name v0.3.4 (`434d000f`) |
| Write check | `grep -n "OpenOptions\|File::create\|write" src/services/window.rs` | no write path |
| E2E | n/a — no E2E harness beyond `tests/cli.rs`, which does not open the reader | — |

### Explicitly out of scope

- **Editing bytes** — fmc is a viewer here; writes stay the file-manager operations.
- **Encoding detection** (Shift_JIS, UTF-16, Latin-1 by name) — no dependency approved; reopen if the owner asks for it.
- **A hex view for files fmc already shows** (text, images) — the owner asked for refused files; `X` works from the text it opened with `t`, not from a file that reads as text.
- **Hex-byte queries and a hit count** — D9.
- **FIFOs, devices, sockets** — their open can block or never end; they stay refused.
- **Remembering the chosen mode per file** — not asked.
