> Point-in-time research, 2026-11-26. Source: agent research for plan-00001 (Scouter at commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`). `scouter:<path>` is relative to the Scouter repository at that commit. Other scratch paths (`<scratch>/…`) were local working copies and probes that are not kept. The tree and the locked crates win over this file.

# fmc dependency survey (phase 1: parity with Scouter 0.3.4)

Date of snapshot: 2026-11-26. Source of versions, dates, licences and MSRVs: crates.io API (`/api/v1/crates/<name>` and `/<name>/<version>`) and GitHub API for repo activity. Weights are measured, not estimated: `cargo tree -e normal,build` run in scratch projects under `<scratch>/fff-probe/` (local cargo 1.100). "Crates" counts every package in the resolved graph, including the crate itself, with default features unless a row says otherwise. Nothing was compiled, built or run; fff-search and the grammar crates were not built.

Scouter reference: `.agents/docs/research/scouter-inventory.md` and `.agents/docs/research/rendering-research.md`. Scouter source: the Scouter repository.

Decisions taken as given: opentui and opentui-sys 0.9.9, clap, git via `std::process`, polling, single crate, Mermaid deferred.

---

## 0. Recommendation at a glance

### 0.1 Phase-1 dependency list

| Crate | Version | Features | Crates added | Why |
| --- | --- | --- | --- | --- |
| `opentui` | `=0.9.9` | default (`static`) + `tree-sitter` | 27 with the feature, 3 without (`opentui`, `opentui-sys`, `libc`) | UI engine (decided). `tree-sitter` pulls tree-sitter 0.27 and the five bundled grammars (JS, TS/TSX, MD+inline, Zig, Rust). |
| `clap` | `4.6.10` | `default-features = false`, `features = ["std", "derive", "help", "usage"]` | 10 (17 with defaults) | CLI (decided). Trimming drops colour and suggestion code that Scouter's plain help does not use. |
| `anyhow` | `1.0.105` | default | 1 | Error plumbing in `main` and internal `Result`. Scouter prints `scouter: <msg>` and exits 1. |
| `imagesize` | `0.15.0` | default | 1 | W×H and format from the header only, which is what Scouter's footer needs. Avoids a full decode. |
| `icu_collator` | `2.4.0` | default (`compiled_data`) | 33 | Matches `Intl.Collator()` (CLDR root) for the tree's folder and file sort. Conditional; see 2.12. |
| `tree-sitter-json` | `=0.24.8` | default | with the set below | Scouter-vendored grammar. |
| `tree-sitter-css` | `=0.25.0` | default | | Scouter-vendored. |
| `tree-sitter-html` | `=0.23.2` | default | | Scouter-vendored. |
| `tree-sitter-go` | `=0.25.0` | default | | Scouter-vendored. |
| `tree-sitter-python` | `=0.25.0` | default | | Scouter-vendored. |
| `tree-sitter-bash` | `=0.25.1` | default | | Scouter-vendored. |
| `tree-sitter-yaml` | `=0.7.2` (Scouter: 0.7.1) | default | | Scouter-vendored. |
| `tree-sitter-toml-ng` | `=0.7.0` | default | | Scouter-vendored as `@tree-sitter-grammars/tree-sitter-toml` 0.7.0. |
| `tree-sitter-c` | `=0.24.2` (Scouter: 0.24.1) | default | | Scouter-vendored. |
| `tree-sitter-java` | `=0.23.5` | default | | Scouter-vendored. |

Set of all of the above, measured together (opentui with `tree-sitter` and the 10 grammar crates): 37 crates. Grammar crates are added via `opentui::syntax::register` or `register_lazy` (see 2.5), and their `highlights.scm` files are copied from `scouter:grammars/<name>/` with `include_str!`.

Dev-dependencies only (do not ship in the binary):

| Crate | Version | Features | Crates added | Why |
| --- | --- | --- | --- | --- |
| `insta` | `1.49.0` | default | 12 | Snapshot the frames from opentui's `testing::TestApp`. |
| `assert_cmd` | `2.2.2` | default | 12 | CLI behaviour: `-h`, `-v`, exit codes, error text. |
| `tempfile` | `3.27.0` | default | 9 | Fixture directories and throwaway git repositories. |

Declared MSRV of the set: Rust 1.90, set by opentui's `tree-sitter` feature (`tree-sitter` 0.27.2 and `tree-sitter-language` 0.1.9 both declare 1.90). Without that feature the floor is 1.88 (opentui). Set `rust-version = "1.90"` once the tree-sitter feature is on.

### 0.2 No crate needed (std or an opentui feature covers it)

| Need | Use | Source |
| --- | --- | --- |
| Git (repo root, diff, status letters, numstat, ls-files, patches) | `std::process::Command` | Scouter does the same (`src/diff/git.ts:47-77`). |
| Filesystem listing, symlinks, realpath, mtime/size stamps | `std::fs` (`read_dir`, `symlink_metadata`, `canonicalize`, `metadata`) | |
| Ignored-path marks and finder skip list | one `git ls-files --others --ignored --exclude-standard --directory -z` call | Matches Scouter exactly (`src/diff/git.ts:248-250`). |
| File-type detection by extension | opentui `syntax::filetype_for_path` (and `generated.rs` table), then an explicit allowlist matching Scouter's `languageFor` | Content sniffing (infer, tree_magic) is not Scouter's method. |
| Binary and text classification | `std::str::from_utf8` (strict), NUL byte scan in the first 1 MiB | Scouter's rule, `src/core/files.ts:172`. |
| Control-character, bidi and tab normalisation | `char` ranges in std | `src/core/files.ts:17-24`. |
| Clipboard (system and OSC 52) | `App::copy_to_clipboard` in opentui (`src/app/clipboard.rs:61`) | Replaces `pbcopy` shelling and `arboard`. |
| Terminal palette and light/dark mode | opentui `term::query_palette`, `App::palette`, `App::on_theme_mode`, `ThemeMode::from_background` | Same brightness formula as Scouter (`src/ui/theme.ts:134-141`). |
| Byte sizes (`formatBytes`) | hand-written, 8 lines | Scouter shows `N B`, `N.N KB`, `N.N MB` (`src/core/files.ts:65-71`). `bytesize` produces different text. |
| Thousands separators (`toLocaleString("en-US")`) | hand-written, 6 lines | |
| Timers and polling (2 s browse, 2 s diff, 3 s theme, 150 ms debounce) | opentui timers or `std::time::Instant` | Scouter uses `setTimeout`. |
| Plain and case-insensitive text search (200 hits, 10 per file) | `str::to_lowercase()` and `str::find` | `src/find/search.ts:144-182`. Add `memchr` only if profiling says so. |
| Recursive directory walk for the finder (20k cap, noise list) | `std::fs` breadth-first walk | `src/find/search.ts:15-47`. |
| Temporary git index copy | `std::env::temp_dir()` plus a `Drop` guard | Scouter removes it on close and on exit (`src/diff/git.ts:372-375`). `tempfile` is acceptable if more temp files appear later. |
| Home directory for `~` in the footer | `std::env::var_os("HOME")` | `src/ui/footer.ts:17`. |
| Unicode display width for truncation | `opentui::widget::painter` exposes `text_width` (`widget/painter.rs:68`); check whether app code can call it | If not, `unicode-width` 0.2.3 (1 crate). |
| Diff rendering and parsing | opentui `Diff` widget (`set_diff`, `view`, `filetype`, `line_numbers`) | Scouter feeds per-file patches to `DiffRenderable` (`src/diff/diff-blocks.ts:504-521`). |
| Markdown | opentui `Markdown` widget (`conceal`, `code_block_renderer`) | |
| Image drawing | opentui `ImageView` (`protocol` auto) | |
| Logging | none in phase 1 | Scouter has no logging. Errors go to stderr when the app exits. |
| Config files | none in phase 1 | Scouter has no config (`docs/decision/0001`). |

---

## 1. fff-search verdict

### 1.1 Facts

| Item | Finding | Source |
| --- | --- | --- |
| Latest stable | `0.11.0`, published 2026-09-21, MIT, edition 2024, no `rust-version` declared | crates.io `fff-search/0.11.0` |
| Newest version | `0.11.1-nightly.eb58401` (2026-11-23). 400 published versions, most of them nightlies | crates.io `fff-search/versions` |
| Repo | `github.com/dmtrKovalenko/fff`, MIT, 11,267 stars, pushed 2026-11-25, 94 open issues, author Dmitriy Kovalenko | GitHub API |
| Downloads | 298,945 total, 158,656 recent | crates.io |
| Default features | `ripgrep` (pulls `ignore`, `globset`). Optional: `zlob` (needs Zig 0.16 at build time, README calls it the "most optimized" build), `mimalloc-collect`, `ffi`, `definitions`, `rescan-stats` | `Cargo.toml` |
| Weight (default features) | 96 crates (normal dependencies), 111 with build dependencies | `cargo tree` in `<scratch>/fff-probe/probe-fff` |
| Native code in the build | `libgit2-sys` (vendored libgit2, C), `libz-sys` (C), `lmdb-master-sys` (LMDB, C, via `heed`) | `cargo tree` |
| Effective MSRV | 1.91, set by `notify` 9.0.0 and `notify-types` 2.3.1. `neo_frizbee` needs 1.89. 33 of 111 crates declare no MSRV | crates.io version records |
| Licences in the tree | MIT or Apache-2.0 for almost all; `option-ext` MPL-2.0; `xxhash-rust` BSL-1.0; `notify` CC0-1.0 | crates.io version records |

### 1.2 How it matches and ranks

- Matching: typo-tolerant fuzzy matching by `neo_frizbee` (a SIMD Smith-Waterman, similar to fzf). `score.rs:54` splits the query into parts and matches each one. The production typo budget is `clamp(query_len / 4, 2, 6)` (`file_picker.rs:1113`).
- Score: the frizbee base score plus case bonuses (8 and 4, `score.rs:204-205`), a filename or dirname bonus (`score.rs:491-494`), a path-distance penalty, a frecency boost from LMDB (`score.rs:712`), a query-history "combo" boost, and git signals (`score.rs` imports `is_modified_status`, and `git_recency` is a config option).
- Index: a background scan thread after `FilePicker::new_with_shared_state`. The file list lives in memory (arena and `StableVec`). A notify watcher runs unless `FilePickerOptions.watch = false`. An optional content bigram index is built if `enable_content_indexing` is set.
- Persistence: frecency and query history are LMDB databases opened by the caller (`FrecencyTracker::open(path)`, `QueryTracker::open(path)`). They are optional. If they are not opened, nothing is written to disk.
- Git: `git2` (libgit2) for status and recency, not the git CLI.
- Grep: `GrepMode::{PlainText, Regex, Fuzzy}`, casing (smart or insensitive), `page_limit`, `max_matches_per_file`, `max_file_size`, `time_budget_ms`, abort signal, before and after context (`grep/types.rs`).
- `FFFMode::{Neovim, Ai}` changes scoring and watcher behaviour (`file_picker.rs:69`).
- `fff-grep` (a sibling crate, 2 dependencies) is ripgrep-style internal plumbing (line steppers and sink helpers). It is not a searcher and is not a candidate.

### 1.3 Scouter's finder, for comparison

From `src/find/search.ts:49-88`:
- Query: lowercased, split on whitespace. Every word must be a substring of the lowercased relative path (AND).
- Order: entries whose name contains the first word come first. Within each group, shorter relative path first (`relative.length`, a UTF-16 length), then code-unit order.
- Limits: display 200 (`MAX_MATCHES`), index 20,000 entries, walk on demand and discard on close (decision 0016), git-ignored paths skipped.
- Text search: case-insensitive substring per line, 10 hits per file, 200 in total, 64 MiB scanned, 150 ms debounce, 2-character minimum.

### 1.4 Alternatives for a later fuzzy toggle

| Crate | Latest | Upkeep | Crates added | MSRV | Licence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `neo_frizbee` | 0.13.3 (2026-09-21) | repo `saghen/frizbee` pushed 2026-11-18 | 1 | 1.89 | MIT | The matcher fff-search uses. Smallest route to the same ranking core. |
| `frizbee` | 0.13.0 (2026-08-13) | same repo | not measured alone | 1.89 | MIT | Same family; 3.07M downloads. |
| `nucleo` | 0.5.0 (2024-04-02) | repo `helix-editor/nucleo` pushed 2026-06-24 | 17 | none declared | MPL-2.0 | Mature (Helix). Crate release is 2 years old. MPL is file-level copyleft. |
| `nucleo-matcher` | 0.3.1 (2024-02-20) | same repo | 3 | none declared | MPL-2.0 | Lower-level than `nucleo`. |
| `fuzzy-matcher` | 0.3.7 (2020-10-04) | stale | 3 | none declared | MIT | Not recommended. |
| `sublime_fuzzy` | 0.7.0 (2020-12-19) | stale | 1 | none declared | "non-standard" in crates.io metadata | Not recommended. |
| `fff-search` | 0.11.0 | active | 96 | 1.91 | MIT | See verdict. |

### 1.5 Verdict: (c) not now

Reasons:
1. Parity. Scouter's results are substring AND-matches in a fixed order (name group, then shorter path). Fuzzy typo-tolerant ranking admits paths that Scouter excludes (for example `mian` matches `main.rs`) and orders the rest by a different score. Phase 1 cannot use it without breaking the screen-by-screen parity audit.
2. Behaviour. fff keeps a live index with a watcher and can write LMDB frecency state. Scouter is read-only, polls, uses no watchers (decisions 0006, 0017), and walks on demand (decision 0016). fff's design runs against all three.
3. Weight. 96 crates, including three C builds (libgit2, zlib, LMDB), notify with macOS framework bindings, rayon and regex.
4. MSRV. Its effective floor is 1.91, above opentui's 1.88 and the tree-sitter floor of 1.90.
5. Churn. Hundreds of published versions, mostly nightlies, and an SDK aimed at Neovim, Node, Python and Bun bindings. Expect breaking changes.

Options (a) and (b) do not fit phase 1. Scouter has no fuzzy finder, so there is nothing to adopt yet.

Parity implication for phase 1: the finder must implement Scouter's substring AND-match and its exact ordering. In Rust, "shorter relative path" means `s.encode_utf16().count()`, and the tie-break compares UTF-16 code units. Rust's `str` `<` compares code points, which differs from UTF-16 order only for characters in U+E000 to U+FFFF. Fuzzy ranking can be a post-parity feature behind a toggle (for example a Tab-cycled mode, or a flag), using `neo_frizbee` first. Only if its grep, index or frecency are wanted later should the full `fff-search` be reconsidered, and then with the 1.91 MSRV and the C builds accepted.

---

## 2. Dependency needs

Column key: "Crates" is the number of crates added to the graph, including the crate itself. "Upkeep" is the crates.io `updated_at` date (GitHub `pushed_at` where noted). "MSRV" is the declared `rust_version` (`–` means none declared).

### 2.1 CLI parsing

| Option | Latest | Upkeep | Crates | MSRV | Licence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `clap` derive, trimmed (`std, derive, help, usage`) | 4.6.10 | 2026-11-24 | 10 | 1.85 | MIT/Apache | Recommended. |
| `clap` derive, default features | 4.6.10 | 2026-11-24 | 17 | 1.85 | MIT/Apache | Adds colour and suggestions. |
| `clap` builder API only | 4.6.10 | 2026-11-24 | not measured | 1.85 | MIT/Apache | Derive is shorter for three flags. |
| `pico-args` / `lexopt` / std `env::args` | not measured | | not measured | | | Fine for Scouter's surface (`-h`, `-v`, one path). Clap helps with generated help text. |

Recommendation: `clap` derive with the trimmed feature set. Parity notes:
- Scouter's `-v` prints `scouter 0.3.4`. Use `disable_version_flag = true` and a custom `-v`/`--version` with `ArgAction::Version`.
- Scouter's error for more than one argument is `Expected one file or directory. Use --help for usage.` (exit 1). Clap's default error text differs. Take the path as `Vec<PathBuf>` and emit Scouter's message yourself.
- Scouter's `--upgrade` belongs to its installer launcher and does not apply to fmc.
- Scouter's `-h` matches the flag anywhere in argv. Clap does not do that. This is a minor point to decide on.

### 2.2 Error handling

| Option | Latest | Upkeep | Crates | MSRV | Notes |
| --- | --- | --- | --- | --- | --- |
| `anyhow` | 1.0.105 | 2026-11-23 | 1 | 1.68 | Recommended. Messages are user-facing strings. |
| `thiserror` | 2.0.21 | 2026-09-23 | 6 | 1.77 | Add only if typed errors are needed (for example to map "binary" versus "not UTF-8" to notices without string matching). |
| `color-eyre` | 0.6.5 | 2025-05-30 | 25 | 1.65 | Reject. Installs panic and report hooks that compete with the terminal restore that opentui's `Host` performs. |
| none (hand-written enum with `Display`) | | | 0 | | Fine for about five notice types. |

Recommendation: `anyhow` only. Use `anyhow::bail!` and `anyhow!` with Scouter's exact strings. Add `thiserror` later if a typed variant becomes necessary.

### 2.3 Directory walking and ignore rules

| Option | Latest | Upkeep | Crates | MSRV | Notes |
| --- | --- | --- | --- | --- | --- |
| `std::fs` (lazy `read_dir`, BFS for the finder) | | | 0 | | Recommended. Scouter's tree is lazy and its finder is a bounded BFS. |
| `walkdir` | 2.5.1 | 2026-11-21 | 2 | none | Only if a recursive walk gets complicated. |
| `ignore` | 0.4.35 | 2026-11-22 | 13 | 1.88 | Parallel gitignore-aware walker (ripgrep). Its semantics are close to `git ls-files --exclude-standard` but not identical (nested repositories, global excludes). Scouter's parity method is git. |

Recommendation: `std::fs` for both the tree and the finder. For ignored marks and the finder skip list, call `git ls-files --others --ignored --exclude-standard --directory -z`, as Scouter does. Keep `ignore` as the fallback if the git call is too slow on very large repositories.

### 2.4 TUI runtime (decided, for completeness)

| Crate | Latest | Crates | MSRV | Notes |
| --- | --- | --- | --- | --- |
| `opentui` | 0.9.9 (2026-11-23, MIT) | 3 base (`opentui`, `opentui-sys`, `libc`) | 1.88 | Official binding. Provides `App`, `Host`, ScrollBox, Code (tree-sitter), Diff, Markdown, ImageView, clipboard, palette and theme events, `testing::TestApp`. |
| `opentui-sys` | 0.9.9 | (pulled by opentui) | 1.88 | Licence `MIT AND LGPL-2.1-or-later` (FFmpeg decode build). Needs `OPENTUI_LIB_DIR` pointing at the release native library. Ship the `LICENSE-*` files. |

Feature notes:
- `opentui/tree-sitter` pulls tree-sitter 0.27.2 (MSRV 1.90), tree-sitter-language 0.1.9, and the grammar crates for JavaScript 0.25, TypeScript 0.23 (TS and TSX), MD 0.5, Zig 1.1 and Rust 0.24. Whether `tree-sitter-md` exposes the inline grammar separately was not checked.
- `log`, `tokio` are off by default and not needed.
- Scouter pins `@opentui/core` 0.9.0. The Rust crate starts at 0.9.4 and 0.9.9 matches the in-tree TypeScript source. The version skew is already noted in `rendering-research.md`.

### 2.5 Syntax highlighting grammars

Scouter ships 16 grammars for 15 languages: five bundled with OpenTUI (JavaScript, TypeScript, TSX, Markdown with its inline grammar, Zig) and 11 vendored (JSON, CSS, HTML, Go, Rust, Python, Bash, YAML, TOML, C, Java). Rust is also bundled by opentui's `tree-sitter` feature, so fmc gets it without a separate crate.

| Grammar (Scouter filetype) | Source in Scouter | Phase-1 source for fmc | Crate and version |
| --- | --- | --- | --- |
| javascript (`.js .mjs .cjs .jsx`) | bundled in opentui | opentui `tree-sitter` feature | `tree-sitter-javascript` 0.25.0 (bundled) |
| typescript (`.ts .mts .cts`) | bundled | opentui feature | `tree-sitter-typescript` 0.23.2 (bundled, TS and TSX) |
| typescriptreact (`.tsx`) | bundled (TSX) | opentui feature | same crate |
| markdown + markdown_inline | bundled | opentui feature | `tree-sitter-md` 0.5.4 (bundled) |
| zig | bundled | opentui feature | `tree-sitter-zig` 1.1.2 (bundled) |
| rust | vendored, 0.24.0 | opentui feature | `tree-sitter-rust` 0.24.2 (bundled; Scouter's 0.24.0 is a minor drift) |
| json | vendored 0.24.8 | add crate | `tree-sitter-json` =0.24.8 |
| css | vendored 0.25.0 | add crate | `tree-sitter-css` =0.25.0 |
| html | vendored 0.23.2 | add crate | `tree-sitter-html` =0.23.2 |
| go | vendored 0.25.0 | add crate | `tree-sitter-go` =0.25.0 |
| python | vendored 0.25.0 | add crate | `tree-sitter-python` =0.25.0 |
| bash | vendored 0.25.1 | add crate | `tree-sitter-bash` =0.25.1 |
| yaml | vendored, `@tree-sitter-grammars` 0.7.1 | add crate | `tree-sitter-yaml` =0.7.2 (pin 0.7.1 if it is published; not checked) |
| toml | vendored, `@tree-sitter-grammars` 0.7.0 | add crate | `tree-sitter-toml-ng` =0.7.0 |
| c | vendored 0.24.1 | add crate | `tree-sitter-c` =0.24.2 (pin 0.24.1 if published; not checked) |
| java | vendored 0.23.5 | add crate | `tree-sitter-java` =0.23.5 |

Option comparison:

| Option | Crates | Notes |
| --- | --- | --- |
| Per-grammar crates through `opentui::syntax::register` (recommended) | opentui feature 27 + 10 grammars = 37 measured | Matches Scouter's grammar set. Native Rust, no WASM worker, no runtime file loading. |
| `register_lazy` | same | Compiles each grammar on first open. Use it so startup does not pay for all 16. |
| Bundle with `tree-sitter-highlight` directly | `tree-sitter-highlight` 0.27.2 adds 26 crates | Not needed. opentui already does highlighting and the Code widget. |
| syntect / two-face | | Different token classes and colours from Scouter (already rejected in `rendering-research.md`). |

Parity note: the highlight query text is what drives the colours. Copy Scouter's vendored `highlights.scm` files (MIT, SHA-256 in `grammars/manifest.json`) into fmc with `include_str!`, instead of using each crate's `HIGHLIGHTS_QUERY`. Then the captures match Scouter's, which is what the parity audit compares. The bundled five (JS, TS, TSX, MD, Zig, Rust) keep opentui's own queries.

Crate maintenance: the grammar crates for json, html, typescript, java, toml-ng and zig were last published in late 2024. They are stable but not actively released. css (2025-09) and go, python and yaml (2025) are newer. `tree-sitter`, `tree-sitter-md`, `tree-sitter-c`, `tree-sitter-rust` and `tree-sitter-bash` were updated in 2025 or 2026.

### 2.6 File-type detection

| Option | Crates | Verdict |
| --- | --- | --- |
| opentui `syntax::filetype_for_path` / `filetype_for_extension` (table in `generated.rs`) | 0 | Use it, with an explicit allowlist that equals Scouter's `languageFor` set. opentui's table is a superset (it also maps `.mkd`, `.mdown`, `.cc`, `.cpp`, `.rb`, and more). Without the allowlist, files Scouter shows as plain text would be highlighted or rendered as Markdown. |
| `infer` 0.22.0 (2026-07-15, MIT, MSRV 1.74) | 5 | Reject. Magic-number detection is not Scouter's method. |
| `tree_magic_mini` 3.2.2 | 10 | Reject, same reason. |
| `mime_guess` 2.0.5 (2024-06-29) | 3 | Reject. Stale, and extension-only MIME is not needed. |

Parity detail: Scouter's `.markdown` and `.md` open rendered. `.mdx` does not (`src/reader/code-view.ts:230-232`). Check the allowlist for both.

### 2.7 Image metadata

| Option | Latest | Crates | MSRV | Verdict |
| --- | --- | --- | --- | --- |
| `imagesize` | 0.15.0 (2026-07-09, MIT) | 1 | none | Recommended. Reads headers only (W×H, format). Matches Scouter's `imageInfo` use. |
| `image` (png only) | 0.25.10 (2026-03-10) | 17 | not checked | Reject for metadata. Full decode is not needed because opentui's `ImageView` decodes for display. |
| opentui native `Image::decode` + `ImageInfo` | in the crate (`native/text.rs:433`) | 0 extra | | Works, but needs a native context and a full decode (64 MiB, 16384 px limit). Use only if imagesize misses a format. |

Parity notes: Scouter's image extensions are `.png .jpg .jpeg .gif .webp` (case-insensitive). Its size limit is 10 MiB, with the exact message `Image is too large (limit: 10 MiB).` Its footer shows `W×H · format` (format as `png`, `jpg` and so on, as reported by `imageInfo`; confirm the exact strings in the parity audit).

### 2.8 Clipboard

| Option | Crates | Verdict |
| --- | --- | --- |
| opentui `App::copy_to_clipboard` (native system clipboard, then OSC 52; OSC 52 only when remote) | 0 | Recommended. Covers Scouter's `pbcopy` / `wl-copy` / `xclip` / `xsel` path and its OSC 52 path. Scouter's SSH rule maps to opentui's remote check. Verify that the SSH behaviour matches in the parity audit. |
| `arboard` 3.7.0 (2026-10-15, MIT/Apache, MSRV 1.71) | 34 | Reject. Large, and it duplicates opentui's clipboard service. |
| Shell out to `pbcopy` and friends | 0 | Scouter's way. Keep only as a fallback if the opentui path fails on a target. |
| `copypasta`, OSC 52 by hand | | Not needed. |

### 2.9 Configuration and paths

| Option | Latest | Crates | MSRV | Verdict |
| --- | --- | --- | --- | --- |
| none, `HOME` from `std::env` | | 0 | | Phase 1. Scouter has no config. |
| `dirs` 7.0.0 (2026-09-05, MIT/Apache, repo on Codeberg) | 7.0.0 | 4 | none | Later, when config or a cache dir is added. Widely used (427M downloads). |
| `etcetera` 0.11.0 (2025-10-28, MIT/Apache) | 0.11.0 | 2 | 1.87 | Alternative with XDG on every platform. |
| `directories` 6.0.0 (2025-01-12) | 6.0.0 | 4 | none | Stale; not recommended. |

Serialisation for later config: `toml` 1.2.3 (2026-11-25, MIT/Apache, MSRV 1.85, 7 crates) with `serde` 1.0.229 derive (2026-07-18, 7 crates). Not needed for phase 1.

Tree-sitter data path: Scouter's `${XDG_CACHE_HOME}/scouter` is for the WASM parser cache. Native grammars need no cache, so fmc does not need that path.

### 2.10 Logging and tracing

| Option | Latest | Crates | MSRV | Verdict |
| --- | --- | --- | --- | --- |
| none, errors to stderr on exit | | 0 | | Phase 1. Scouter prints `scouter: <msg>` and exits 1. |
| `tracing` 0.1.44 + `tracing-appender` 0.2.5 (file) + `tracing-subscriber` 0.3.23 (`env-filter`) | 0.1.44 / 0.2.5 / 0.3.23 | 22 + 16 (appender and subscriber, overlapping) | 1.65 / 1.63 / 1.65 | Later, if a debug log is wanted. Write only to a file, never stdout or stderr while the TUI runs. |
| `log` with a custom file logger | | 1 | | Smaller option if only the `log` facade is needed. |

opentui's `console` module is an in-app log buffer, not a file logger, so it does not replace this.

### 2.11 Time formatting

Scouter formats no dates. It uses `Date.now()` for throttling and `setTimeout` for polling, both replaced by `std::time::Instant`. No crate is needed for phase 1.

Later, if dates appear: `jiff` 1.0.3 (2026-11-24, Unlicense/MIT, MSRV 1.85, 2 crates) is the lighter choice. `chrono` 0.4.45 (2026-06-04, MSRV 1.62, 5 crates) is the older, heavier option.

### 2.12 Unicode width and collation

Width:

| Option | Crates | Verdict |
| --- | --- | --- |
| `opentui::widget::painter` `text_width` (`widget/painter.rs:68`) | 0 | Use it if the truncation helpers live in custom drawing. Check it is public from app code. |
| `unicode-width` 0.2.3 (2026-10-28, MIT/Apache, MSRV 1.66) | 1 | Fallback. Bun's `stringWidth` may differ from `unicode-width` on some emoji sequences (not checked); compare the footer truncation in the parity audit. |

Collation (Scouter's `Intl.Collator()` drives folders-first then name order, `src/core/files.ts:11, 50-56`):

| Option | Latest | Crates | MSRV | Parity | Verdict |
| --- | --- | --- | --- | --- | --- |
| `icu_collator` 2.4.0 (2026-10-22, Unicode-3.0, `compiled_data` default) | 2.4.0 | 33 | 1.88 | CLDR root collation, the same root Intl uses for `en-US` | Recommended for exact parity. |
| `feruca` 0.13.0 (2026-11-01, MIT, 77k downloads) | 0.13.0 | 14 | none | Unicode Collation Algorithm (DUCET). Differs from CLDR root in some punctuation and accent orders. | Lighter approximation. |
| std: compare `to_lowercase()`, then original | | 0 | | Reproduces the observed Scouter sample (`.gitignore, app.log, README.md`). Differs on punctuation (`_` versus `-`), digits and accents. | Acceptable only if the parity audit finds no mismatch. |

Observed Scouter order, screen 01: `assets, node_modules, notes, scripts, src, target, tests, .gitignore, app.log, README.md`. Plain byte order would put `README.md` before `app.log`, so byte order is ruled out.

### 2.13 Terminal theme detection

| Option | Crates | Verdict |
| --- | --- | --- |
| opentui palette query (`term::query_palette`, OSC 4 and default colours) plus `on_theme_mode` events | 0 | Recommended. Keep Scouter's behaviour: query at startup with a 500 ms timeout, re-check every 3 s, use brightness `< 0.5` for dark. |
| `termbg` 0.6.2 (2025-01-06, MIT/Apache) | 43 | Reject. Redundant with opentui and far heavier. |

### 2.14 Git

| Option | Latest | Crates | MSRV | Verdict |
| --- | --- | --- | --- | --- |
| `std::process::Command` (Scouter's method) | | 0 | | Recommended. Use the same flags (`--no-pager`, `--literal-pathspecs`, `-c core.fsmonitor=false`, `-c core.quotepath=false`, `--no-ext-diff --no-textconv --no-color -M`), env (`GIT_OPTIONAL_LOCKS=0`, `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`, private `GIT_INDEX_FILE`), 15 s timeout and output caps. |
| `git2` 0.21.0 (2026-10-06, MIT/Apache) | 0.21.0 | 12 (libgit2 C build) | none | Reject for phase 1. Diff patches still need the same output format, and Scouter's `GIT_OPTIONAL_LOCKS` and private-index tricks are CLI-specific. |
| `gix` 0.91.0 (2026-11-22, MSRV 1.95) | 0.91.0 | large | 1.95 | Reject. MSRV too high and far more API than needed. |

Timeouts in std: `Child::try_wait` in a polling loop, or a thread with `recv_timeout`. No crate needed.

### 2.15 Byte size and number formatting

| Option | Crates | Verdict |
| --- | --- | --- |
| hand-written `format_bytes` and `group_thousands` | 0 | Recommended. Scouter's rules are simple (`N B`, `N.N KB`, `N.N MB`, grouping with `,`). |
| `bytesize` 2.7.0 (2026-08-02, Apache-2.0, MSRV 1.85) | 1 | Different output (`KiB`, different rounding). Not parity. |
| `humansize` 2.1.3 (2022-12-31) | | Stale. Not recommended. |

### 2.16 Text search and in-file search

| Option | Crates | Verdict |
| --- | --- | --- |
| std: `to_lowercase()` and `find` per line | 0 | Phase 1. Scouter's rule is exactly this. Note: JS `includes` on a lowercased string and Rust `contains` agree for ordinary text. Differences occur only for special case mappings (for example `İ`), which should be handled identically by lowercasing the whole line before matching. |
| `memchr` 2.8.3 (2026-07-08, Unlicense/MIT, MSRV 1.61) | 1 or 2 | Add only if profiling shows the per-line scan is slow. `memmem` is a drop-in for `find`. |
| `grep-searcher` 0.1.18 (ripgrep) | more than 1 | Overkill for a literal case-insensitive scan. |
| `fff-search` grep | 96 | Reject for phase 1 (see section 1). |

### 2.17 Temporary directories

| Option | Crates | Verdict |
| --- | --- | --- |
| std `temp_dir()` plus `Drop` guard that removes the directory | 0 | Recommended. Scouter's cleanup is on close and on exit. |
| `tempfile` 3.27.0 (2026-03-11, MIT/Apache, MSRV 1.63) | 9 | Fine. Use it if the app needs more temp files later. |

### 2.18 File watching

| Option | Latest | Crates | MSRV | Verdict |
| --- | --- | --- | --- | --- |
| none, polling (Scouter's design) | | 0 | | Phase 1. Scouter uses no watchers (decisions 0006, 0017). |
| `notify` 9.0.0 (2026-11-21, CC0) | 9.0.0 | 10 | 1.91 | Reject for phase 1. Raises the MSRV above 1.90. |

Signal handling: not researched. opentui's `Host` handles resize and restore. Whether SIGTERM and SIGINT restore the terminal is not verified.

### 2.19 Testing

| Crate | Latest | Crates | MSRV | Use |
| --- | --- | --- | --- | --- |
| `insta` | 1.49.0 (2026-10-03, Apache-2.0) | 12 | 1.66 | Snapshot frames from opentui's `testing::TestApp` (`feed`, `press`, `click`, `drag`). Dev-dependency. |
| `assert_cmd` | 2.2.2 (2026-05-11, MIT/Apache) | 12 | 1.85 | CLI behaviour: `-h`, `-v`, exit 1 with Scouter's messages, TTY check. Dev-dependency. |
| `predicates` | 3.1.4 (2026-02-11) | comes with `assert_cmd` | 1.74 | Output assertions. |
| `tempfile` | 3.27.0 | 9 | 1.63 | Fixture trees and `git init` repositories. Dev-dependency. |
| `expectrl` | 0.9.0 (2026-05-11, MIT) | 14 | none | PTY end-to-end tests. Later, not phase 1. |
| `portable-pty` | 0.9.0 (2025-02-11, MIT) | | none | Alternative for PTY tests. Stale. |

Scouter's tests use `bun test` against the source and `tests/keymap.test.ts` checks the README table. The equivalent here is a keymap table test plus snapshot tests.

### 2.20 Other

| Need | Option | Verdict |
| --- | --- | --- |
| Hashing or content stamps | `mtime` and size from `std::fs::Metadata`, same as Scouter | No crate. |
| Path normalisation (`dunce`, `pathdiff`, `camino`) | `std::fs::canonicalize` for symlinks; `dunce` 1.0.5 (CC0/MIT/Apache) only if Windows verbatim paths appear | Not needed on macOS and Linux. |
| Serialising state | none | Not needed. |
| Mermaid | deferred (Scouter does not render it) | Not in phase 1. |

---

## 3. Later milestones (notes only, no research beyond crates.io metadata)

| Need | Options | Notes |
| --- | --- | --- |
| SQLite viewer | `rusqlite` 0.43.0 (2026-11-15, MIT, MSRV 1.89, 14 crates with `bundled`, 3 native) | Recommended. Synchronous and fits a TUI. Use `bundled` for a self-contained binary. |
| SQLite viewer (async) | `sqlx` 0.9.1 (2026-11-18, MSRV 1.94) | Heavier, async runtime, and needs a higher MSRV. Not recommended for a synchronous viewer. |
| DuckDB viewer | `duckdb` 1.20001.0 (2026-11-20, MIT, MSRV 1.88) | 103 crates and a long C++ build with `bundled`. Consider as an optional feature. |
| File-manager write ops: move, copy, rename | `std::fs` plus `walkdir` 2.5.1 for recursive copy | Recommended. `fs_extra` 1.3.0 is last published 2023-02, so it is stale. |
| Delete to the system trash | `trash` 5.4.0 (2026-11-17, MIT, MSRV 1.87, 7 crates) | Recommended for a "move to trash" action. |

---

## 4. Not verified and caveats

- Nothing was compiled. The dependency counts are from resolution only (`cargo tree`), with no build scripts run.
- Declared MSRVs come from crates.io version records. Some crates declare none (for example the grammar crates and `fff-search` itself). The 1.90 floor is the highest declared MSRV among the recommended set, and is set by tree-sitter 0.27.2.
- The `highlights.scm` files from the grammar crates were not compared with Scouter's vendored copies. The plan is to copy Scouter's files.
- Exact C and YAML pins (`tree-sitter-c` 0.24.1, `tree-sitter-yaml` 0.7.1) were not checked for publication on crates.io.
- `icu_collator` output was not compared with Bun's `Intl.Collator()` on the Scouter fixture. The comparison belongs in the parity audit.
- opentui's `painter::text_width` is public in the source; whether app code can reach it was not checked against the crate's public module path.
- opentui's clipboard SSH behaviour was read from the source, not tested over SSH.
- The fff-search "git-modified" score factor was inferred from imports in `score.rs`, not from a read of the full scoring function.
- `frizbee` 0.13.0 and `neo_frizbee` 0.13.3 come from the same repo (`saghen/frizbee`) per crates.io. The exact relationship between them was not checked.
- fff-search and `nucleo` licences were taken from crates.io metadata and the GitHub licence field, not from the LICENSE files.
