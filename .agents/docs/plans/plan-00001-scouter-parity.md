---
status: in-progress
created: 2026-11-26T04:09:25+07:00
updated: 2026-11-26T12:03:08+07:00
phases:
    - id: 1
      title: "W0: licenses, toolchain, formatting, license gate"
      status: done
      note: "cargo-about 0.9.3 installed; deny passes on the current tree"
    - id: 2
      title: "W0: native library build script and cache"
      status: done
      note: "first build 1 min 16 s, cached rerun 0.02 s, 0 FFmpeg symbols; repo-local .cache symlink because cargo [env] cannot expand $HOME"
    - id: 3
      title: "W0: engine swap spike (opentui =0.9.9)"
      status: done
      note: "release 12 MB, 13.9 s; otool -L system libs only; Rex capture shows rendered Markdown and highlighted Rust; rust-version 1.90 confirmed"
    - id: 4
      title: "W0: parity reference and demo repository"
      status: done
      note: "demo-repo.sh: two runs give identical git status and hashes (ab4f5d2, d7da111). Scouter on core 0.9.9 vs 0.9.0, README rendered at 187x60, top and bottom: text and HTML captures byte-identical (tables included). W0 complete."
    - id: 5
      title: "W1: crate layout and command line"
      status: done
      note: "--help byte-identical to Scouter (name swapped, --upgrade dropped as out of scope); -h/-v matched as exact tokens before clap; 12 tests"
    - id: 6
      title: "W1: theme palettes and detection"
      status: done
      note: "tint values match Scouter output; DESIGN.md hex checked by a test; polling via opentui refresh_palette, stops when a refresh goes unanswered or the terminal announces; live run shows dark text #d4d8de"
    - id: 7
      title: "W1: key table and pending-key parser"
      status: done
      note: "parser in Scouter onKeyPress order (prompts read keys first, in the app); g alone goes to the top in Scouter, so gg is two Top steps; 12 key tests incl. every documented key bound and docs/keymaps.md in step"
    - id: 8
      title: "W1: app loop, chrome, welcome, help"
      status: done
      note: "80x24 text captures equal Scouter (core 0.9.9) for welcome, ?, help closed, Tab, Esc outside the tree rows (W2) and the wordmark/version; colors equal except tree colors; larger tabs could not be forced (Rex opens 80x24 tabs; stty cannot grow them), a 190x61 run matched by eye; name substitution in the wordmark awaits owner confirmation"
    - id: 9
      title: "W2: tree model and filesystem service"
      status: done
      note: "core tree (explorer/tree.rs) takes listings as values; sort via icu_collator 2.4 root collation, checked against Bun Intl.Collator order; services/fs.rs lists with noise, folders first, loop refusal; 59 tests in verify, 16 new"
    - id: 10
      title: "W2: tree view, git marks, sidebar resize"
      status: done
      note: "80x24 live captures against Scouter 0.9.9 (same keys, same demo repository): 11 text states of the tree, 10 scroll states of an overflowing list, the file-target, root `-`, page, edge, and Backspace keys, the loop refusal on a link, and 4 HTML colour states. The sidebar columns match in every state; the reader differs only in the approved wordmark and version. Mouse (edge drag, click, wheel) is an owner check. Enter on a file and `/` are W3 and W4 no-ops. services/git.rs (private index copy with a Drop guard) was built here for the marks; phase 18 extends it"
    - id: 11
      title: "W3: text loading and code view"
      status: done
      note: "Node's I/O wording in one place (services/fs.rs describe), phase 10 listings on it; loader (reader/document.rs: 1 MiB, NUL and UTF-8 checks, BOM, tabs, controls) and the code view with Scouter's gutter (minWidth 3, paddingRight 2); the vendored grammars cascade with core_captures; Core's queries for JavaScript, TypeScript, TSX, JSX, Markdown, and Zig with the four pinned crates (architecture.md; Lua predicates translated, rust.md). Ctrl+q quits and a plain q does not; a file opened from the tree keeps list focus; a file refused at startup keeps its folder (architecture.md approved deviations). Live at 80x24 against Scouter 0.9.9: text and colours identical for Rust, HTML, CSS, JSON, YAML, TOML, C, Java, Bash, Go, Python, the JavaScript family, plain text, empty, controls, a file over 1 MiB (text), and the Enter and Tab flow, apart from the approved footer quit label, focus, and wordmark. Later: scrollbar and pan (14), Markdown rendering (12), binary and image (13)"
    - id: 12
      title: "W3: Markdown rendered and source"
      status: done
      note: "Rendered view is OpenTUI Markdown with the palette styles, scope styles on, Core's grey for table and rule borders (no conceal scope in Scouter); m toggles the source with line numbers and resets to rendered per file; / and { } refuse with Scouter's footer text; the viewer scrollbar (one column, only while the text overflows) and the clip of the body. Live at 80x24 against Scouter 0.9.9 on the demo README: rendered and source text identical, colours identical, apart from the approved footer quit label. Scrolling and the paragraph motion wait for phase 14 and 15."
    - id: 13
      title: "W3: image and binary views"
      status: done
      note: "Images decode the header with imagesize 0.15 (approved; resolves at 0.15.0) and open through OpenTUI's ImageView with the auto protocol, fit contain, scaled to the reader; the footer shows WxH and the format from the native names (png, jpeg, webp, gif). Binary notice centred as Scouter draws it, focus stays in the list. Live at 80x24 against Scouter 0.9.9: the PNG and the binary at startup, and the PNG opened from the tree, identical apart from the approved footer quit label and focus. Owner check: the picture in a Kitty-capable terminal (Rex draws no pixels). Scouter's stale footer position after a binary is recorded in issue-00002, not copied."
    - id: 14
      title: "W3: reader motions, jump history, wrap, pan, copy"
      status: done
      note: "ScrollBox viewer; keys of §4.3 (j k, h l pan, space b, ctrl d u, gg G, counts, { }, zz, w) through the parser; jump history of 100 (ctrl+o, alt+i; jump.rs core); wrap kept across files; y copies the selection or the path through OpenTUI (OSC 52 and the system clipboard, copied_summary); drag selects and keeps the selection, nothing copied on release (approved, architecture.md). Live at 80x24 against Scouter 0.9.9 on the chapters fixture: 120G, G, gg, zz, { }, ctrl+d and ctrl+u, space and b, j k, w, pan, identical apart from the approved footer quit label; the jump back is identical apart from the approved focus. Open: the rendered view stops two rows short at the bottom (issue-00003). Owner check: drag selection and the wheel."
    - id: 15
      title: "W3: in-file search"
      status: done
      note: "In-file search: core reader/search.rs (whole-line matches ignoring case, wrap-around stepping, the footer text, the refresh on a shown text); app/search.rs keys (the prompt takes plain keys, Enter and arrows step, Esc keeps then clears, n N p, and / refused in the rendered view); the marks are a layer over the code, naming opentui.search.match in a buffer table that lists every capture name (rust.md). Live at 80x24 against Scouter 0.9.9 on the chapters fixture and the demo main.rs: the prompt, typing, the count (1 of 183, wrap), Enter, Shift+Enter via arrows, n, Esc keeps and clears, text and chip colours identical, including Rust syntax under a search. Owner check: none beyond the approved footer and focus differences."
    - id: 16
      title: "W4: file finder"
      status: done
      note: "/ in the tree: index walked on a worker (services/find.rs, Scouter's breadth-first order, noise and ignored skipped, 20,000 cap), core find/ with AND words, name group first, UTF-16 length then code-unit order, 200 rows and the total; finder rows replace the tree in Browser (flat rows); Enter reveals or opens with the reader focused, as Scouter; Esc returns. 16 new tests (149 in verify). Live at 80x24 against Scouter 0.9.9 on the demo repository: empty prompt, name matches, Esc, Enter on a file, Enter on a folder, typed q; text identical apart from the approved root row, reveal, and ^q label. Tab (text search) is phase 17. Empty-folder j and k panicked on HEAD (fixed). Issues 00004 (welcome clipped at 80x24) and 00005 (Markdown table widths) found, open for phases 22 and 23."

    - id: 17
      title: "W4: text search and preview"
      status: done
      note: "Tab in the finder searches the lines of the files: 2+ UTF-16 units, 150 ms debounce, 200 hits and 10 a file, 64 MiB budget (core find/text.rs with 11 tests; services/find.rs streams progress every 120 ms). Rows are name:line with the match in accent; the highlighted hit shows in its own preview view with the preview pill and a third-line scroll; Enter opens the file at the line with the query kept for n and N; Esc ends. Live at 80x24 against Scouter 0.9.9 on the chapters fixture and the demo repository: the text state, the preview (text and colours identical), Enter (line 43 at the top, search seeded 1 of 1), Tab back to names, type 2+ characters, Esc, and a deep hit (line 150); differences only the approved root row, reveal, and quit label, plus the welcome clipping (issue-00004). Found and fixed while validating: one selected row for each hit of a file (selection by row index), the open-at scroll (applied once the reader shows), and the preview sharing the reader scroll box (its own view now). Owner check: a click on a text row (picks it as Enter does)."
    - id: 18
      title: "W5: git service"
      status: done
      note: "verify 188 tests (19 services/git, 20 diff). Scouter: git.ts:8-12, :24-148, :156-198, :204-260, :303-370; patch.ts:5-206. Demo repo: Scouter 0.9.9's listChanges, readAllPatches, listMarks and fmc's Repository give the same 10 changes, 10 patch texts byte for byte, 3 marks, 3 ignored. A conflicted file lists as M in both (inventory §21). 80x24 capture: fmc at HEAD and in the tree are identical; Scouter differs only as approved, plus the welcome card gap that HEAD already has (wordmark, hint line, q row)"
    - id: 19
      title: "W5: changes screen, change list, and stacked diff body"
      status: done
      note: "verify 206 tests (18 new: diff row, list, stack). Scouter: diff-screen.ts:318-322, :379-417, :552-565; app.ts:291-315, :869-907; diff-blocks.ts:55-96, :425-521, :552-591; changes.ts:226-278; footer-hints.ts:47-79. Live 80x24 on the demo repository (/tmp/fmc-ref/out/w5-19/final, text and HTML): both key sequences match Scouter 0.9.9 cell for cell on rows 1 to 23 except the approved tree root row (steps 07, 11, b15) and the approved ^q label, which also drops `a one file` from the list-focus hint line. Empty state (clean and unborn repositories) matches. Wheel: list and body move one row per notch both ways. Found and fixed while validating: a wheel scroll undone by repaint, g and G losing the pan, the diff text colour, the stack width (gutter overflow), and a second visit reusing the first visit's blocks. Pre-existing, in HEAD too, recorded as issue-00007 (tree wheel) and issue-00008 (reader pan one column short)."
    - id: 20
      title: "W5: split, fold, context, hunks, open"
      status: done
      note: "live 80x24 on the demo repo: keys d n N ] c c x x s a a o match Scouter except the approved deviations (settled fold values, a/x landing) and the footer hint dropped by ^q (accepted); one-file header column fixed (stack reserves its bar column only while it overflows). Split at 60+ columns: owner check, not verifiable at 80x24."
    - id: 21
      title: "W5: polling refresh"
      status: done
      note: "r reloads (d9e7e46); polls: the tree every 2 s with back-off, the open file by stamp (\"is no longer on disk.\"), the changes list and patches by stamp; live 80x24 against Scouter 0.9.9: a new root file appears in both, an edit to the open file shows in both, a deleted open file shows the same notice, the changes list and body follow an edit without r, a body at its end stays at its end, mid-body holds. Index copies: 1 while the changes screen is open, 0 after quit."
    - id: 22
      title: "W6: parity audit"
      status: done
      note: "audit-00001-scouter-parity.md: 80x24 live runs against Scouter 0.9.9 on d826786 for §3 rows 1 to 16 and the §4 keys; 8 findings: F-01 welcome clipped (issue-00004), F-02 Markdown two cells narrow (issue-00005), F-03 rendered thumb and percent (issue-00003), F-04 tree wheel (issue-00007), F-05 pan (issue-00008) for phase 23; F-06 folded bar, F-07 binary footer (issue-00002), F-08 --upgrade record for the owner. 216 tests green at d826786"
    - id: 23
      title: "W6: parity fixes"
      status: pending
      note: —
    - id: 24
      title: "W6: docs sync, ledger close, handover"
      status: pending
      note: —
    - id: 25
      title: "W6: final validation"
      status: pending
      note: —
---

## Port Scouter to Rust — fmc reaches 100% read-only parity with Scouter on macOS aarch64

### Objective

The owner asked for a Rust port of Scouter on OpenTUI with 100% visual and functional parity as the first milestone. After this plan, fmc opens a folder or a file, browses the tree, reads text, Markdown, images, and binaries, finds files and text, and shows git changes as stacked diffs, the same way Scouter does, except for the two approved deviations in `architecture.md`. The plan stays read-only: no write to the user's files, and every post-parity feature stays in `roadmap-00001-post-parity.md`.

### Current state (verified in the tree, 2026-11-26)

- **The crate is a hello world.** `src/main.rs` draws into an `OptimizedBuffer` and prints; it never enters a terminal. `Cargo.toml` declares only `opentui_rust = "0.2.3"`.
- **No tooling files.** No `rustfmt.toml`, `rust-toolchain.toml`, `.cargo/config.toml`, `deny.toml`, license files, or `docs/` (checked with `ls -A` at the root). `README.md`, `PRODUCT.md`, and `DESIGN.md` are written with this plan.
- **Toolchain.** `rustc 1.100.0 (2026-11-09)`, `stable-aarch64-apple-darwin`.
- **Decisions.** All 15 records in `architecture.md` are dated 2026-11-26; one pending item (the exact `rust-version`) is settled by phase 3.
- **Reference.** Scouter at commit `434d000` is inventoried in `research/scouter-inventory.md`; the static build of `opentui` 0.9.9 was proven on macOS arm64 in `research/opentui-binding-risk.md` §3.

### Decisions

The decisions are the records in `architecture.md` (all 2026-11-26): engine, video-off native library, license, platform, crate layout, toolchain pins, clap, git subprocess with polling, ICU sort, parity reference on core 0.9.9, the 16 grammars, the finder, the key table and parser, and the two approved deviations. This plan adds two execution decisions:

- **D1 (2026-11-26): Scouter's evidence is cited per phase from `research/scouter-inventory.md` sections, and re-verified against `scouter:` source before the port.** The inventory is a snapshot; the source at commit `434d000` wins. Rejected: porting from the inventory alone — it has known drift (§21) and untriggered paths (§19.3).
- **D2 (2026-11-26): mouse behavior is validated by unit tests on hit-testing and by an owner check, not by Rex.** Rex cannot inject mouse events (`research/scouter-inventory.md` §4.6). Rejected: skipping mouse parity — Scouter enables the mouse (`scouter:src/index.ts:54`).

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| Decisions and deviations | `architecture.md` | The 15 records; a new deviation is a new record |
| Dependency allowlist, license policy | `rules.md`, "Dependencies", "Licenses" | Approved crates and versions; the `opentui-sys` license exception |
| Core purity, file size, test placement | `rust.md`, "Layout rules" | Binding from phase 5 |
| Commands, tools, native library path | `shape.md` | Updated in phases 1 to 4 |
| Human docs | `README.md`, `PRODUCT.md`, `DESIGN.md`, `docs/{development,cli,keymaps}.md` | Each phase syncs what it changes |

### Execution rules

- **Atomic commit per phase.** The owner's go-ahead for a wave authorizes one commit per phase in it, with the message the phase names. Waves group phases; a wave runs only on the owner's go-ahead.
- **Approved dependencies only** (`rules.md`, "Dependencies"); each is added in the phase that first uses it, and the phase confirms the version resolves. Anything else stops and asks.
- **Read-only.** No feature writes to the user's files. The only writes are fmc's own temp files (the private git index copy, phase 18), removed by a `Drop` guard.
- **Scouter first.** Before a phase ports a behavior, re-read the cited `scouter:` source; a difference from the inventory is fixed in `research/scouter-inventory.md` in the same commit.
- **Per-phase validation runs before the commit**: the matrix rows the phase touches, always format, lint, test, and build.
- **Live runs.** A phase that changes the screen captures fmc and Scouter in Rex on the demo repository (phase 4) at 80x24 and 160x48, the same keys on both, and diffs the text captures. Every difference is fixed, or recorded as an issue for phase 23.
- **Docs sync per phase.** The phase that lands a user-visible behavior updates `docs/` and `README.md`; the phase that lands a tool, path, or rule updates the agent doc `rules.md` names.

### Phases

#### W0 — Foundation

1. **Licenses, toolchain, formatting, license gate** — `LICENSE-MIT`, `LICENSE-APACHE`; `Cargo.toml` `license = "MIT OR Apache-2.0"`, `description`, `repository`; `rustfmt.toml` (`max_width = 120`, `use_small_heuristics = "Max"`); `rust-toolchain.toml` pinning `1.100.0` with `clippy` and `rustfmt`; `deny.toml` with the license allowlist from `rules.md`; `about.toml` and template for `THIRD_PARTY_LICENSES`; `shape.md` (toolchain, tools). Validation: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, `cargo deny check licenses` pass on the current tree. _Commit: `chore: add licenses, toolchain pin, rustfmt and license gate`._
2. **Native library build script and cache** — `scripts/opentui-native.sh <version>`: checks `zig version` is 0.17.1, fetches the OpenTUI source at tag `v<version>`, runs `zig build -Doptimize=ReleaseFast -Dvideo=false`, installs `libopentui.a` and headers into `~/.cache/fmc/opentui-native/<version>/`, and is a no-op when the cache exists; `.cargo/config.toml` `[env] OPENTUI_LIB_DIR`; `docs/development.md` (prerequisites: Rust via the toolchain file, zig 0.17.1, git, a Nerd Font; first build); `shape.md`. Validation: the script runs twice (second run under 1 s); `nm` on the archive shows 0 `avcodec`/`avformat` symbols. _Commit: `chore: build the opentui native library without video`._
3. **Engine swap spike** — replace `opentui_rust` with `opentui` and `opentui-sys` `=0.9.9` (`tree-sitter` feature); `src/main.rs` renders one Markdown widget and one Rust `Code` widget, quits on `q`; set `rust-version` to the lowest version that builds (start at 1.90) and record it in `rust.md`; `rules.md` "Library source" drops `opentui_rust`. Validation: `cargo build --release`; `otool -L target/release/fmc` lists only system libraries; a Rex capture shows highlighted Rust and rendered Markdown; `cargo deny check licenses` passes with the documented exception. Undo: `git revert` of this commit restores `opentui_rust`; if the spike fails, stop and report before any later phase. _Commit: `feat: switch the engine to opentui 0.9.9`._
4. **Parity reference and demo repository** — `scripts/demo-repo.sh <dir>` builds the demo git repository from `research/scouter-inventory.md` §19.1 (Markdown with every construct and a `mermaid` fence, rs/ts/py/go files, nested folders, `.gitignore`, staged, unstaged, untracked, deleted, renamed, binary, image), with Harry Potter and Dan Brown names; `docs/development.md` gains "Run the parity reference": clone Scouter at `434d000` into a scratch directory, bump `@opentui/core` to 0.9.9, run it with `bun --config=/dev/null --no-env-file src/index.ts`. Validation: the script is idempotent (`git -C <dir> status --short` gives the same output twice); Scouter on core 0.9.9 opens the demo repository in Rex, and the capture is stored in the phase's notes. _Commit: `chore: add the demo repository script and parity reference steps`._

#### W1 — App shell

5. **Crate layout and command line** — create `src/lib.rs`, the empty core modules, `src/services/`, `src/app/`, `src/ui/`, `src/args.rs` per `architecture.md`; `clap` reproduces `-h/--help` (usage plus the key list with the Mouse section), `-v/--version` (`fmc <version>`), one optional path, the TTY check, and Scouter's errors and exit code 1 (`research/scouter-inventory.md` §1); `docs/cli.md`; `README.md` usage. Validation: `assert_cmd` tests for every row of §1 (help, version, two paths, not a TTY, socket path, missing path); `cargo test`. _Commit: `feat(cli): add the command line and crate layout`._
6. **Theme palettes and detection** — `theme.rs`: both palettes with every token in `research/scouter-inventory.md` §14.1, brightness, 2.5% tint, fallback background; the app side queries the terminal background, listens for theme changes, and polls every 3 s (§14.3). Validation: unit tests for brightness, tint, and a contrast check per palette (as `scouter:tests/theme.test.ts`); every hex in `DESIGN.md` matches `theme.rs` (a test reads both). _Commit: `feat(theme): add the dark and light palettes and detection`._
7. **Key table and pending-key parser** — `keys/`: one table with every key in §4.1 to §4.6 (context, key, action, help label, hint), the dispatch order of §4, and the parser returning `More | Bad | Done` with counts; `--help` and the `?` card read the table. Validation: table tests for each key sequence in §4 (including `gg`, `zz`, `120G`, count reset, `Esc i` as `Alt+i`); a test that every `?` entry and every `--help` line comes from the table; `docs/keymaps.md` generated from or checked against the table. _Commit: `feat(keys): add the key table and the pending-key parser`._
8. **App loop, chrome, welcome, help** — the `opentui` app loop on the alternate screen with the mouse on; two rounded panels with the focus border, headers, badges, error notice, the 2-row footer with pills, location, facts, and fitted hints (§2); the welcome screen (§3 row 1); the help card with the Mouse section (approved deviation); `Tab`, `Esc`, `q`, `Ctrl+c`; terminal restore on exit and on panic (recorded in `rust.md`). Validation: Rex captures of welcome and help at both sizes match Scouter's except the Mouse section; a panic test restores the terminal; owner check of the mouse pointer on the resize edge. _Commit: `feat(app): add the app loop, panels, footer, welcome and help`._

#### W2 — Explorer

9. **Tree model and filesystem service** — core tree model: lazy folder listing, the noise list, folders first then `icu_collator` order (§9.1, §9.2), expansion rules, reveal that expands every ancestor (approved deviation), `-` to make the parent the root; `services/fs.rs` lists folders off the UI thread. Validation: unit tests with `tempfile` trees for sort (including punctuation and case), noise, symlink loop (`Cannot expand a symbolic-link loop.`), reveal of a file three folders deep. _Commit: `feat(explorer): add the tree model and the folder listing service`._
10. **Tree view, git marks, sidebar resize** — row anatomy, Nerd Font icons and colours, depth guides, ignored rows dimmed from one `git ls-files --others --ignored --exclude-standard --directory -z` (§9.3, §9.4); status letters and folder dots for the browsed repository only; keys of §4.2; click to open or toggle; sidebar width rule, `Ctrl+←/→`, and edge drag (§2). Validation: Rex captures of the tree at both sizes, expanded and collapsed, match Scouter; hit-test unit tests for click and drag; owner mouse check. _Commit: `feat(explorer): add the file tree view, git marks and sidebar resize`._

#### W3 — Reader

11. **Text loading and code view** — `services/fs.rs` reads with the 1 MiB cap, UTF-8 and NUL checks, tabs as 4 spaces, control characters as `�` (§22); the code view with the line-number gutter; OpenTUI's bundled grammars plus the 10 approved grammar crates with Scouter's `highlights.scm` through `include_str!`; the syntax style from the palette (§8). Validation: unit tests for the loader limits; a Rex capture per language in the demo repository compared with Scouter's (HTML capture for colours on at least Rust, TypeScript, HTML, and Markdown source). _Commit: `feat(reader): add the text loader and the highlighted code view`._
12. **Markdown rendered and source** — `.md` opens rendered (§6): headings, emphasis, OSC 8 links, inline code, fences highlighted by info string, lists, task boxes, quotes, rules, tables; `m` toggles source; a `mermaid` fence stays plain text; search and `{ }` refused with Scouter's hints. Validation: Rex captures of the demo README rendered and source match Scouter on core 0.9.9. _Commit: `feat(reader): render Markdown and toggle its source`._
13. **Image and binary views** — images (PNG, JPG, JPEG, GIF, WEBP) scaled to fit with `W×H · format` from `imagesize`; the binary notice screen with focus kept in the sidebar (§3 rows 6 and 7). Validation: Rex captures of the binary notice match; an owner check of the image in a Kitty-capable terminal; the footer position after an image then a binary is correct (Scouter's stale `All` is a defect to report, not copy, unless the owner says otherwise in phase 23). _Commit: `feat(reader): add the image and binary views`._
14. **Reader motions, jump history, wrap, pan, copy** — the keys of §4.3: scroll, pages, half pages, `gg`, `G`, counts, `{ }`, `zz`; jump history of 100 with `Ctrl+o` and `Alt+i`; wrap `w` kept across files; pan by 4; `y` copies the selection or the path through OSC 52 and the host helper, skipped over SSH; drag-to-copy. Validation: unit tests for motions and history in the core; Rex captures of `120G`, `{`, `zz`, wrap, and pan match Scouter; an owner check of drag-to-copy. _Commit: `feat(reader): add motions, jump history, wrap, pan and copy`._
15. **In-file search** — `/` prompt, live jump, `Enter`, `↓`, `Shift+Enter`, `↑`, `n`, `N`, `p`, the three-step `Esc`, amber chips, `N of M` (§4.3, §3 row 11). Validation: core tests for match finding and wrap-around; Rex captures of search states match Scouter's screens 29 to 33. _Commit: `feat(reader): add in-file search`._

#### W4 — Finder

16. **File finder** — `/` in the sidebar: a 20 000-entry index built off the UI thread, AND words over names, Scouter's order (name group, then shorter path in UTF-16 units, then code-unit order), result rows with paths, prompt states (§3 row 9, §10.1); opening reveals the file. Validation: core tests on order with a fixed index; Rex captures of screens 08, 09, 12 match. _Commit: `feat(find): add the file finder`._
17. **Text search and preview** — `Tab` in the finder: 2+ characters, 150 ms debounce, 200 hits, 10 per file, 64 MiB budget, status strings, the preview with the `preview` badge and third-line scroll (§3 row 10, §10.1). Validation: core tests for limits and debounce; Rex captures of screens 10 and 80 match. _Commit: `feat(find): add text search with preview`._

#### W5 — Git changes

18. **Git service** — `services/git.rs`: `rev-parse`, the private index copy with a `Drop` guard, `diff --name-status/--numstat -z --no-ext-diff --no-textconv -M` against `HEAD` or the empty tree, `--no-optional-locks`, untracked synthesis, the per-repository queue, the 2 s racy window, the 1000-change cap, the 1 MiB and 4 MiB limits (§11.1, §11.4). Validation: integration tests on `tempfile` repositories for each change kind (M, A, D, R, T, U, ?), an empty repository, and a non-repository; the user's `.git/index` is byte-identical before and after. _Commit: `feat(git): add the git reader service`._
19. **Changes screen, change list, and stacked diff body** — `d` enters and leaves (with `Loading changes…` and cancel), the `Changes N` header, list rows with letters and marks, the empty state, keys of §4.4 for the list; the stacked diff body: one `Diff` widget per file with band headers, 5-cell stat bars, shared gutter width, notices, no `@@` lines (§11.3) (moved here from phase 20 by the owner on 2026-11-26). Validation: core tests for the stat bar; Rex captures of screens 35, 36, and the empty state match. _Commit: `feat(diff): add the changes screen, the change list, and the stacked diff`._
20. **Split, fold, context, hunks, open** — `s` split at 60+ columns, `a` one file, `c` fold, `x` context 3 or 12, `n`/`]` and `N`/`p`/`[` hunks across files, `o` opens in the reader, click on a header folds. Validation: core tests for the hunk index; Rex captures of screens 37 to 45, 75, 77 match. _Commit: `feat(diff): add split, fold, context, and hunk navigation to the diff`._
21. **Polling refresh** — `services/poll.rs`: browse polling every 2 s with back-off, re-listing expanded folders, refreshing marks, re-reading the open file by `mtime:size` stamp, the `no longer on disk` notice, paused on the changes screen; diff polling with per-file stamps (§11.7, §12). Validation: an integration test edits a file and sees one refresh event; a Rex run edits a demo file and captures screen 67's state. _Commit: `feat(services): add polling refresh for the tree, reader and changes`._

#### W6 — Parity and close

22. **Parity audit** — run `parity-audit` over every screen in `research/scouter-inventory.md` §3 and every key table; report in `.agents/docs/audit/audit-00001-scouter-parity.md`. Validation: every row of §3 and §4 has a verdict with evidence. _Commit: `docs(audit): record the scouter parity audit`._
23. **Parity fixes** — before this phase runs, the plan gains one sub-phase per finding group (23a, 23b, …) with its own validation and commit; a finding the owner accepts as a deviation becomes an `architecture.md` record instead. Validation: re-run `parity-audit` on each fixed screen. _Commit: one per sub-phase, `fix(<area>): <the finding>`._
24. **Docs sync, ledger close, handover** — `README.md`, `PRODUCT.md`, `DESIGN.md`, `docs/` checked against the binary; agent docs synced; this plan archived; `index.md` rows moved; handover written. Validation: every key in `docs/keymaps.md` exists in the key table; `grep` finds no dangling ledger reference. _Commit: `docs: close the scouter parity plan`._
25. **Final validation** — the full matrix over the tree as it stands. Validation: the matrix green end to end. _Commit: none if green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check` | No drift |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Unit and integration | `cargo test` | All green; core tests need no terminal and no git |
| Build | `cargo build` and `cargo build --release` | Builds; `otool -L` lists only system libraries |
| Licenses | `cargo deny check licenses` | Only the documented `opentui-sys` exception |
| Live run | Rex, fmc and Scouter (core 0.9.9) on the demo repository, 80x24 and 160x48, same keys | Text captures identical except approved deviations |
| Mouse | hit-test unit tests and an owner check | Click, drag-resize, drag-copy, header fold behave as §4.6 |
| Parity | `parity-audit` per screen | No open difference, or each one recorded |
| E2E | n/a: no E2E harness beyond the CLI tests; the live run covers the screen | — |

### Explicitly out of scope

- **Everything in `roadmap-00001-post-parity.md`**: Mermaid rendering, the Neovim grammar set, fuzzy finding, file-manager write operations, the SQLite and DuckDB viewer, more platforms, an alternate Vim keymap, `notify`-based watching.
- **Scouter's installer and `--upgrade`** — they belong to Scouter's Bun launcher; fmc ships as one binary. Packaging is a later plan.
- **Scouter defects other than the two approved deviations** — reported in the audit, fixed only on the owner's word.
