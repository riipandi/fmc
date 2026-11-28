# Hex and text view research

updated: 2026-11-28T18:48:00+07:00

Point-in-time research, 2026-11-28, for plan-00023 (a file fmc refuses opens as text or in a hex view). Agent audience; evidence as `file:line` in the tree, in the locked `opentui` 0.9.9 (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opentui-0.9.9/`), and in Scouter v0.3.4 (`434d000f`, the scratch copy at `/tmp/scouter-434d000f`). Measurements ran on this Mac (Apple M2 Pro, 16 GiB, APFS SSD at 94 % full) with a scratch crate in a fresh `mktemp -d`, release profile; the crate is not kept.

## 1. Verdict

- **Read with `read_at` (std `FileExt`) through a small block cache, on a worker.** Every measured operation is microseconds: open plus the last page in 0.03 to 0.07 ms, a page scroll in 2 µs, a random jump (a cache miss each) in 7.5 µs. Memory mapping is about 5 µs faster per jump, which no one can see, and a file that shrinks under a map kills fmc with SIGBUS (measured, exit 138). No new dependency.
- **Hand-write the hex formatter.** It is about 20 lines, formats a 60 x 16 screen in 1.8 µs (11x faster than `pretty-hex`, which also cannot colour bytes by class or hand back cells), and fmc needs cells with colours, not a string.
- **Text mode: std `String::from_utf8_lossy` on the visible window, plus fmc's own control-byte mapping.** `encoding_rs` and `chardetng` would add six crates for a guess the owner did not ask for.
- **Draw with a `Custom` component and `Painter::cells`**: only the visible rows, one record per frame, the bar from `ui/scrollbar.rs` sized by the whole file (rows = size / bytes per row; 2 GiB at 16 bytes is 134 M rows, fine in `usize` and `f64`).

## 2. Today in fmc

- **The binary test.** `src/reader/document.rs:47-51` `decode`: a NUL in the first 1 MiB (`MAX_BYTES`, `:7`) is `Refusal::Binary`; invalid UTF-8 is `Refusal::NotUtf8` (`:54`). `src/services/fs.rs:83-94` `read` reads `MAX_BYTES + 1` bytes and maps `Binary` to `Opened::Binary { size }`, any other refusal to `Err(message)`.
- **The other refusals.** `src/services/fs.rs:45-47`: a FIFO, a device, or a socket is `"Only regular text files can be viewed."` before any open (its open could block). `:98-120` `read_image`: an image past 10 MiB is `image::TOO_LARGE`, a header it cannot parse is `image::UNREADABLE`. Each `Err` goes to the footer as an error (`src/app/tree.rs:255-260`, `src/app/preview.rs:73`), and the reader keeps what it showed.
- **The notice.** `src/ui/binary.rs:9-13` holds the hint and `Press o to open it in the default app.`; `paint` (`:56-64`) draws title, name and size, hint, open line. `src/app/reader.rs:215-224` `show_binary` keeps focus on the list (the compact tier moves it to the reader). The notice's view is shown at `src/app/mod.rs:775`.
- **The keys while the notice shows.** Focus is the list, so the `browse` context applies (`src/keys/bindings.rs:326-366, 387-394`); in compact it is `reader` (`:368-408`). Letters free in both contexts and in `everywhere` (`:282-305`): `f`, `i`, `t`, `u`, `z` is the parser's (`keys/parser.rs`), and uppercase `C`, `F`, `I`, `M`, `T`, `U`, `V`, `W`, `X`, `Y`, `Z`. `x` is `CutFile` in browse; `h` is `Collapse` in browse and `Pan` in reader; `b` is `Page(-1)` in reader.
- **No frame-time I/O rule.** `.agents/docs/rust.md` "Layout rules": no file I/O on the UI thread; workers in `src/services/` answer with `Event`s (`services::spawn`).
- **Virtual lists already exist.** The tree paints only its visible rows from a scroll position (`src/ui/tree.rs`, `thumb` at `:189-210`, O(height)); `src/ui/scrollbar.rs:45-47` `sync` takes `total`, `viewport`, `top` as `usize`, so it is not tied to text.

## 3. Reading a window of a huge file

Bench: a 2 GiB sparse file (`set_len`) and a 2 GiB generated file (xorshift bytes with a `\n` or NUL every 61 bytes, written in 4.0 s). A screen is 60 rows of 16 bytes (960 bytes). The pread cache holds 8 blocks of 64 KiB, LRU. "Scroll" is 1000 consecutive pages from the middle; "jumps" are 1000 random 16-byte-aligned offsets. Each step includes formatting the screen. Two runs, second shown.

| Method | File | Open + last page | 1000 page scroll | 1000 random jumps |
| --- | --- | --- | --- | --- |
| `read_at` + cache | sparse | 0.043 ms | 1.98 ms (16 misses) | 7.58 ms (1013 misses) |
| `read_at` + cache, `F_NOCACHE` | sparse | 0.065 ms | 1.95 ms | 7.53 ms |
| `memmap2::Mmap` | sparse | 0.079 ms | 1.93 ms | 2.87 ms |
| `read_at` + cache | generated | 0.034 ms | 1.95 ms (16 misses) | 7.60 ms (1013 misses) |
| `read_at` + cache, `F_NOCACHE` | generated | 0.059 ms | 1.99 ms | 7.30 ms |
| `memmap2::Mmap` | generated | 0.071 ms | 1.94 ms | 2.84 ms |

- Per step: a page scroll costs about 2 µs either way; a jump costs 7.5 µs (pread of 64 KiB) against 2.9 µs (mmap). A frame is about 16 ms.
- **Cold cache not measured.** The generated file was just written, so its pages sat in the unified buffer cache, and `F_NOCACHE` only stops new caching; it does not evict. A true cold read needs `sudo purge` (owner check). An SSD random 64 KiB read is in the order of 0.1 ms, still far under a frame; a network or spun-down disk can take seconds, which is why the read belongs on a worker anyway.
- **Shrink under the reader.** A file truncated from 1 MiB to 4 KiB after the open: `read_at` at 512 KiB returns 0 bytes (a short read the view can show as the new end). The same touch through an `Mmap` killed the process with SIGBUS (exit status 138). `memmap2::Mmap::map` is `unsafe` for this reason (its docs: the file must not be modified or truncated while mapped). A log rotated or a download being rewritten while fmc shows it would crash fmc; there is no safe Rust recovery short of a SIGBUS handler.
- **Full scans.** A newline count over 2 GiB with std (`iter().filter(..).count()`, 8 MiB reads) took 508 ms (warm) and 497 ms (`F_NOCACHE`): 25.9 M lines. Any feature that needs the whole file (a line count for text mode, a search) is a worker job of about half a second per 2 GiB at best, with progress, never a frame.

## 4. Crates

| Crate | Version | Licence | Maintenance | Normal deps | Covered by std? |
| --- | --- | --- | --- | --- | --- |
| `memmap2` | 0.9.12 | MIT OR Apache-2.0 | updated 2026-10-21, 454 M downloads | `libc` (already in `Cargo.lock` 0.2.191) | `read_at` covers the need; see §3 for why mmap loses |
| `pretty-hex` | 0.4.3 | MIT | updated 2026-11-24 | 0 | yes: a 20-line formatter is 11x faster and gives cells with colours |
| `hexyl` | 0.17.0 | MIT/Apache-2.0 | a CLI; reference only | — | its layout (offset, two groups of 8, ASCII panel, colour by byte class: NUL, ASCII printable, ASCII whitespace, other ASCII, non-ASCII) is a design reference, not code to take |
| `encoding_rs` | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause | updated 2026-10-30 | 5 (`cfg-if`, `core_detect`, `multiversion_no_op`, `scopeguard`, `simdutf8`) | for UTF-8, yes (`from_utf8_lossy`); it adds legacy encodings (Latin-1, Shift_JIS, UTF-16) |
| `chardetng` | 1.0.0 | Apache-2.0 OR MIT | updated 2026-03-30 | 3 (`cfg-if`, `encoding_rs`, `memchr`) | no; it guesses the legacy encoding of non-UTF-8 text |
| `memchr` | 2.8.3 | Unlicense OR MIT | in `Cargo.lock` already (transitive) | 0 | std's counting loop measured 0.5 s per 2 GiB; a direct dependency would need approval |

Format of one 60 x 16 screen, 10 000 runs: hand-written 1.8 µs, `pretty_hex::config_hex` 20.6 µs. `String::from_utf8_lossy` on 1 MiB of random bytes: 6.3 ms (worst case: most bytes replaced); a visible window is a few KiB, so about 20 µs.

Licence note: `encoding_rs`'s BSD-3-Clause part and `memchr`'s `Unlicense OR MIT` both pass the allowlist (`rules.md`, "Licenses") only through the allowed side; no new crate is recommended, so the point is moot.

## 5. Drawing a virtual view in OpenTUI 0.9.9

- **`Custom` + `Component`** (`opentui-0.9.9/src/widget/custom.rs:39-67`): `paint(&self, &mut Painter)`, plus `mouse` with the local cell (`:55-57`) and `key`. fmc already uses it (`src/ui/markdown.rs:106,121`, `src/app/overlay.rs:29`).
- **`Painter::cells(x, y, width, &[PaintCell])`** (`opentui-0.9.9/src/widget/painter.rs:154-159`) records a whole grid in one record; its doc says to prefer it over a loop of `cell` and to keep the `PaintCell`s between frames. `PaintCell::new(char, Style)` (`src/native/record.rs:99-104`). `Painter::text` (`:109-114`) draws one styled run per call; a hex row with colour classes would be up to 33 runs, about 2000 calls per screen, so `cells` is the better fit.
- **Scrolling.** The hex view must not live in the reader's `ScrollBox` (`src/ui/viewer.rs:17`): a 2 GiB file would be a 134 M-row content box. Like the tree, it keeps its own `top` row, paints rows `top..top+height`, and drives `ui/scrollbar.rs` with `total = ceil(size / bytes_per_row)`. The wheel comes through `Component::mouse` or the pane's mouse handler, as the tree's does (`src/app/mouse.rs:39-40`).
- **Only the visible window is held.** The view needs `height x bytes_per_row` bytes (under 4 KiB at 60 x 64); the cache holds a few 64 KiB blocks around it.

## 6. Scouter

Scouter v0.3.4 has no hex or text fallback. Its read loop throws `BinaryFileError` at the first NUL in the first 1 MiB (`scouter:src/core/files.ts:6,172`, `BinaryFileError` at `:59-61`), and non-UTF-8 text throws `Only UTF-8 text files are supported.` (`:10,178-184`, `TextDecoder` with `fatal: true`). `openPath` shows the binary notice on `BinaryFileError` and rethrows anything else to the footer (`scouter:src/app/app.ts:329-341`); `showBinary` sets the footer's `binary` facts and focuses the list (`:411-419`). The notice is three lines: title, name and size, `Scouter only displays UTF-8 text and images.` (`scouter:src/reader/binary-view.ts:60-69`). A hex or text view is therefore a parity deviation: it needs an `architecture.md` record and a `docs/parity.md` line naming v0.3.4 (`434d000f`), as plan-00017's `o` line did.

## 7. Recommendation

1. A pure core module (`src/reader/hex.rs`): row layout by width, byte classes, the formatter into cells, the text-mode mapping of one window, offset parsing. Unit-tested without a terminal.
2. A service (`src/services/window.rs`): one worker per open file that owns the `File` and answers block requests with `read_at`; a short read is the new end. The UI keeps up to 16 blocks of 64 KiB (1 MiB) and asks for the blocks around the viewport ahead of a scroll. A missing block paints as blank rows for at most a frame on a warm cache.
3. A view (`src/ui/hex.rs`): `Custom` painted with `Painter::cells`, the scrollbar from `ui/scrollbar.rs`.
4. Text mode reads the same windows. Scrolling by rows of text needs line starts; a row count needs a full scan (0.5 s per 2 GiB on a worker). The cheap shape is to scroll text by byte offset (the bar by bytes, a jump snapped back to the previous line start), so open and the end stay instant at any size.
5. No new dependency.
