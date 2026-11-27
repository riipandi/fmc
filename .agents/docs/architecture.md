# Architecture

updated: 2026-11-27T21:30:00+07:00

The architecture and the reasoning behind it. The layout lives in `shape.md`; the binding rules derived from these decisions live in `rules.md` and `rust.md`. This file keeps the why, so a settled decision is not re-litigated.

## Decision records

One record per decision. Format:

```text
### <The decision, stated as the state of the world>

Context: <the forces: the constraint, the problem, the Scouter behavior it touches. One short paragraph.>

Decision: <what was picked, and the one reason that survived the alternatives.>

Consequences:

- (positive) <the benefit, named concretely>
- (negative) <the cost, the trade-off, or the capability given up>

Alternatives considered:

- <alternative> — rejected: <the reason, specific to this tree>
```

An approved parity deviation is a record too. It names the Scouter baseline (version and commit), the Scouter behavior (with its `file:line` or its decision record), the fmc behavior, and the owner's approval with its date. Every deviation also gets a line in `docs/parity.md`. A difference with no record is a defect.

All records below were decided by the owner on 2026-11-26 during the planning of `plan-00001-scouter-parity.md`. Research behind them: `.agents/docs/research/`.

### The UI engine is the official OpenTUI Rust binding, `opentui` and `opentui-sys` pinned at =0.9.9 with the `tree-sitter` feature

Context: Scouter draws everything with `@opentui/core` renderables: `MarkdownRenderable`, `CodeRenderable` with tree-sitter, `DiffRenderable`, `ImageRenderable`, `ScrollBoxRenderable`, flex layout (`research/scouter-inventory.md` §0, §16). `opentui_rust` 0.2.3 is a low-level renderer with no event loop, layout, scroll, Markdown, tree-sitter, or diff view (`research/rendering-research.md`).

Decision: depend on the upstream binding (`packages/rust` of the OpenTUI repository), which drives the same native engine as Scouter and ships the same widgets. It is the shortest path to parity.

Consequences:

- (positive) Markdown, Code, Diff, ScrollBox, and Image come from the same engine Scouter uses, so rendering differences shrink to the 0.9.0 → 0.9.9 skew.
- (negative) A native static library per platform, FFI surface (about 600 `unsafe` mentions, callbacks wrapped in `catch_unwind`), a crate with 4 releases in 3 days and one reverse dependency: pin exactly and budget each upgrade (`research/opentui-binding-risk.md` §5).
- (negative) MSRV rises to the binding's floor (1.88) and the `tree-sitter` crate's (1.90).

Alternatives considered:

- `opentui_rust` 0.2.3 — rejected: the event loop, layout, scroll, Markdown renderer, tree-sitter highlighting, and diff view would all be rebuilt by hand, and their output would still differ from Scouter's.
- Eternal130/opentui-rust — rejected: git-only, pinned to OpenTUI 0.5.14, far behind Scouter's 0.9.0.

### The native library is built from source without video, one path for dev and release

Context: the release `libopentui.a` of v0.9.9 embeds FFmpeg 9.0.2 (LGPL-2.1). Statically linked into a distributed binary, it brings notice and relinking obligations. Scouter never plays video. `zig build -Dvideo=false` swaps FFmpeg for `src/video-shim.c` (upstream `packages/native/build.zig:391`, `:619`); the probe found 0 FFmpeg symbols and a 12.3 MB binary versus 14.6 MB (`research/opentui-binding-risk.md` §3).

Decision: a script builds the native library with zig 0.17.1 and `-Dvideo=false`, once per OpenTUI version, into `~/.cache/fmc/opentui-native/<version>/`; `OPENTUI_LIB_DIR` points there through `.cargo/config.toml`. Dev and release use the same library.

Consequences:

- (positive) No LGPL code in the binary; the dev binary and the release binary are identical.
- (negative) zig 0.17.1 is a required tool; the first build of a version takes about 4 minutes.
- (negative) The crate metadata still says `MIT AND LGPL-2.1-or-later`, so `cargo-deny` carries a documented exception for `opentui` and `opentui-sys`.

Alternatives considered:

- The release zip with LGPL notices — rejected: relinking obligations for a static single-file binary, for a feature fmc does not use.
- Release zip for dev, own build for release — rejected: two paths to keep working and a dev binary that differs from the shipped one, to save 4 minutes once per version.

### fmc is licensed `MIT OR Apache-2.0`

Context: the owner wanted the safest license that keeps the build simple. Phase-1 dependencies are MIT, Apache-2.0, or Unicode-3.0 (`icu_collator`); the video-off native library leaves no LGPL code.

Decision: dual license `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`, `license` in `Cargo.toml`). `cargo-deny` enforces an allowlist (`rules.md`, "Licenses"); `cargo-about` generates the third-party notices shipped with a release.

Consequences:

- (positive) The Rust ecosystem default: an explicit patent grant through Apache-2.0, GPLv2 compatibility through MIT.
- (negative) Two license files and two tools (`cargo-deny`, `cargo-about`, installed with `cargo install`, not crate dependencies).

Alternatives considered:

- MIT only — rejected: no explicit patent grant, for no saving.

### The first and only platform in phase 1 is macOS aarch64

Context: the probe built and ran a static binary on macOS arm64 (and Linux x86_64 musl); Intel macOS, Linux glibc, and Windows are unverified, and upstream only compile-checks most targets (`research/opentui-binding-risk.md` §5).

Decision: phase 1 targets `aarch64-apple-darwin` only. Other platforms are parked in `roadmap-00001-post-parity.md`.

Consequences:

- (positive) One native library, one CI target, no cross-compilation.
- (negative) Scouter's Linux clipboard helpers (`wl-copy`, `xclip`, `xsel`) are ported but not exercised.

Alternatives considered:

- macOS x86_64 and aarch64 — rejected by the owner on 2026-11-26 (a correction of an earlier pick): x86_64 is unverified.

### fmc is a single crate split into a pure core, I/O services, and the UI, with Scouter's areas inside

Context: Scouter's `src/` splits into `app`, `core`, `diff`, `explorer`, `find`, `reader`, `ui` (`research/scouter-inventory.md`), and mixes I/O into its UI modules. The modal editor in `research/editor-study.md` keeps a core with no I/O and no window code, workers that send events back, and an app layer that carries out actions; that split makes the logic testable without a terminal. The owner chose this shape on 2026-11-26.

Decision: one crate with this layout:

- `src/lib.rs` is the pure core, with no I/O and no `opentui`: `keys/` (the key table and the pending-key parser), `explorer/` (tree model, ICU sort), `reader/` (motions, jump history, in-file search), `find/` (finder matching and order), `diff/` (patch parsing, change list, marks), `theme.rs` (palettes, brightness, tint).
- `src/services/{git,fs,poll}.rs` are I/O workers. They run off the UI thread and send `Event`s back. No file I/O or process spawn happens on the UI thread.
- `src/app/` holds the `opentui` loop and dispatch: a keymap returns `Action`s, and the app carries them out.
- `src/ui/` holds the views, one per screen or component.
- `src/main.rs` starts the app; `src/args.rs` holds the `clap` definition.

The binding rules (core purity, file size, test placement) live in `rust.md`, "Layout rules".

Consequences:

- (positive) Every behavior Scouter defines in logic (sort, match, motion, patch split, key parse) is a unit test with no terminal and no git.
- (positive) Each core module still maps to one Scouter directory to diff against.
- (negative) An event and action layer between workers, core, and UI that Scouter does not have; a port of a Scouter function may split across core and service.

Alternatives considered:

- Modules that mirror Scouter's `src/` one to one — rejected by the owner: it keeps Scouter's mix of I/O and UI, which is harder to test.
- A Cargo workspace from the start — rejected: crate boundaries before any code exists, and overhead without a second consumer.

### Formatting and toolchain are pinned in the repository

Context: the owner wants a fixed toolchain and a wider line. The current stable on the dev machine is 1.100.0 (2026-11-09); the MSRV floor from the dependencies is 1.90.

Decision: `rustfmt.toml` with `max_width = 120` and `use_small_heuristics = "Max"`; `rust-toolchain.toml` pins the current stable channel version with `clippy` and `rustfmt`. Both land in `plan-00001` phase 1.

Consequences:

- (positive) Every machine and CI formats and lints the same way.
- (negative) A toolchain bump is a commit.

Alternatives considered:

- rustfmt defaults and whatever rustup provides — rejected by the owner.

### The command line is parsed with `clap`, reproducing Scouter's surface

Context: Scouter takes `-h/--help`, `-v/--version`, and one optional path, with exact error strings and exit codes (`research/scouter-inventory.md` §1).

Decision: `clap` 4 with `derive` and no default features (`rules.md`, "Dependencies"). Help text, version string, and errors are configured to match Scouter's output.

Consequences:

- (positive) Declarative flags; a later file-manager CLI grows on the same parser.
- (negative) clap's own help and error formats must be overridden to match Scouter's text; a test pins them.

Alternatives considered:

- Hand-written `std::env::args` parsing — rejected by the owner.

### Git is read by running `git` as a subprocess, with polling for changes

Context: Scouter runs `git` through `Bun.spawn` (`scouter:src/diff/git.ts:54`) with `--no-ext-diff --no-textconv -M` against a private index copy, and polls every 2 s with back-off (`research/scouter-inventory.md` §11).

Decision: `std::process::Command` with Scouter's arguments and environment, and the same polling. fmc also passes `--no-optional-locks`, so polling never takes the user's index lock. That flag changes no output and nothing on screen, so it is not a parity deviation; if it ever becomes observable, it needs a deviation record.

Consequences:

- (positive) Identical semantics to Scouter, no C dependency, no `git2`/`gix` build cost.
- (negative) Requires `git` on the host, as Scouter does.

Alternatives considered:

- `git2` or `gix` — rejected: different semantics from the `git` CLI that Scouter's output depends on, and a heavy build.
- `notify`-based watching — rejected: Scouter polls; `notify` 9 raises the MSRV to 1.91.

### The file tree is sorted with `icu_collator`

Context: Scouter sorts with `Intl.Collator` (CLDR root order, `research/scouter-inventory.md` §9.2). Only ICU reproduces it; lowercase comparison in std matches the sampled order but differs on punctuation (`research/dependency-survey.md`).

Decision: `icu_collator` 2.4 with compiled data (`rules.md`, "Dependencies").

Consequences:

- (positive) Exact sort parity.
- (negative) 33 crates for one sort.

Alternatives considered:

- std lowercase comparison — rejected: known punctuation mismatches, against the 100% parity target.

### The parity reference is Scouter run locally with `@opentui/core` bumped to 0.9.9

Context: Scouter pins `@opentui/core` 0.9.0; fmc's engine is 0.9.9. `Code` and `Diff` are unchanged between them; `Markdown` (+44/−18) and `TextTable` (+120/−2) changed (`research/opentui-binding-risk.md` §4).

Decision: parity is measured against Scouter at commit `434d000` with its `@opentui/core` dependency bumped to 0.9.9 in a scratch checkout outside the repository. Scouter's behavior is the reference; the engine is the same on both sides.

Consequences:

- (positive) An engine difference never masquerades as a port defect.
- (negative) The reference is a modified Scouter; a behavior that only appears under 0.9.0 is not tracked.

Alternatives considered:

- Unmodified Scouter on 0.9.0 — rejected: Markdown table differences would be unavoidable and recorded as permanent deviations.

### Phase 1 ships exactly Scouter's 16 tree-sitter grammars

Superseded for the owner's twelve extra grammars by "Approved deviation: the owner's extra grammars are highlighted" (plan-00006 O2, 2026-11-27). Scouter's set is unchanged.

Context: Scouter highlights JavaScript, TypeScript, TSX, Markdown (with inline), and Zig through OpenTUI's bundled grammars, plus JSON, CSS, HTML, Go, Rust, Python, Bash, YAML, TOML, C, and Java from `scouter:grammars/manifest.json`. The owner's wider Neovim set waits.

Decision: OpenTUI's bundled grammars plus one crate per remaining language, pinned exactly (`rules.md`, "Dependencies"), with Scouter's `highlights.scm` files loaded through `include_str!`.

Consequences:

- (positive) Every file is coloured as Scouter colours it.
- (negative) Grammar crates are native C builds; compile time grows.

Alternatives considered:

- Scouter's set plus the Neovim set — rejected for phase 1: a file Scouter shows plain would be coloured, a deviation; parked in the roadmap.

### The finder matches like Scouter: substring AND words, no fuzzy ranking

Superseded in part by "Approved deviation: the finder can match fuzzily, from `find_mode` and `Ctrl+F`" (plan-00006 O3, 2026-11-27). Scouter's substring match stays the default.

Context: Scouter's finder is a substring AND match with a fixed order (`scouter:src/find/search.ts:56-87`). `fff-search` is typo-tolerant fuzzy ranking with frecency, 96 crates, three C builds, and an effective MSRV of 1.91 (`research/dependency-survey.md`).

Decision: implement Scouter's matcher and order with std.

Consequences:

- (positive) Same results in the same order; no dependency.
- (negative) No fuzzy matching until the post-parity milestone.

Alternatives considered:

- `fff-search` as a dependency — rejected: different result set and order, heavy tree.
- `neo_frizbee` or `nucleo-matcher` — deferred to the roadmap as a fuzzy toggle.

### Keys are dispatched from one key table through a pure pending-key parser

Context: Scouter keeps a keymap for `?` and `--help` (`scouter:src/core/keymap.ts:3-55`) but writes footer hints by hand (`scouter:src/ui/footer-hints.ts`), which let the help card drift from the real keys (`research/scouter-inventory.md` §4.7). The modal editor studied in `research/editor-study.md` uses a pure parser that returns `More`, `Bad`, or `Done` over pending keys.

Decision: one key table drives dispatch, the `?` card, and `--help`; a pure parser over pending keys (counts included) returns `More | Bad | Done`, tested with key-sequence tables. The keys and dispatch order are Scouter's (`research/scouter-inventory.md` §4).

Consequences:

- (positive) Help and dispatch cannot drift; key handling is unit-testable without a terminal.
- (negative) Footer hints stay per context as in Scouter, so they keep their own table.

Alternatives considered:

- A Neovim-style grammar — rejected: `Tab`, `?`, `q`, `w`, `d`, `y`, `m` mean other things in Scouter (`research/editor-study.md` §4); parked as an alternate keymap in the roadmap.

### Approved deviation: revealing a file expands its immediate parent folder

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: in Scouter, opening a file inside a collapsed nested folder (finder, `o` from the changes screen, or a startup file argument) expands the grandparent but not the parent, so the file has no row and the selection falls to the root (`scouter:src/explorer/file-tree.ts:76-81`, reproduced live). The same happens for a folder picked in the finder.

Decision: fmc expands every ancestor and selects the revealed row. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The revealed file is visible and selected.
- (negative) A deliberate difference from Scouter's screen in that state.

Alternatives considered:

- Reproduce the bug — rejected by the owner.

### Approved deviation: the `?` card lists the Mouse section (superseded)

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: `--help` prints a Mouse section; the `?` card omits it (`scouter:src/ui/help.ts:55`).

Superseded on 2026-11-26 by "`?` opens the key list as a table in the reader"; the Mouse section lives in that page.

Decision: the `?` card shows the Mouse section too, drawn from the same key table. Approved by the owner on 2026-11-26.

Consequences:

- (positive) `?` and `--help` agree.
- (negative) The card is taller than Scouter's; at small heights it scrolls or clips the same way Scouter's card does with its other sections.

Alternatives considered:

- Reproduce the omission — rejected by the owner.

### Approved deviation: the product name is fmc wherever Scouter says scouter

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter names itself on screen: the welcome wordmark (block letters `SCOUTER` and its version), `--help`, `-v`, and error messages (`scouter:src/ui/welcome.ts`, `scouter:src/index.ts`).

Decision: fmc shows its own name and version in the same places: the wordmark spells `FMC` in Scouter's block-letter style and colours, `-v` prints `fmc <version>`, and help and error text say `fmc`. Layout, colours, and wording around the name stay Scouter's. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The binary never claims to be another product.
- (negative) Text captures differ from Scouter's on the name and version; a parity diff masks those spans.

Alternatives considered:

- Show `SCOUTER` until parity closes — rejected by the owner: it ships a wrong name for the whole milestone.
- A new fmc wordmark — rejected: a larger visual deviation with no design yet.

### The file tree is a list fmc draws from a scroll position, not OpenTUI's ScrollBox

Context: Scouter's tree is a `ScrollBoxRenderable` of row boxes with a custom half-block thumb (`scouter:src/ui/styled.ts:28-51`) and a native scroll-into-view (`scrollChildIntoView`). The row's selection background and the thumb must not overlap, and the folder dot sits at the row's right edge, beside the thumb only while the list overflows (`research/scouter-inventory.md` §9.4, §19.2).

Decision: fmc keeps the selection and the scroll position in the core (`explorer/browser.rs`), draws the visible rows and the thumb itself (`ui/tree.rs`), and keeps the selection at the nearest edge as OpenTUI's scroll-into-view does. Captures at 80x24 match Scouter's text and colours for every tree state in plan-00001 W2 phase 10 (`research/` evidence in the plan note).

Consequences:

- (positive) Scroll, selection, and hit-testing are pure and unit-tested; the thumb matches Scouter's glyphs.
- (positive) A click maps a screen row to a list row by arithmetic, so it is testable without a mouse.
- (negative) fmc keeps its own copy of the scroll rules; a change in OpenTUI's scroll-into-view will not reach the tree on its own.
- (negative) The wheel scrolls one row per tick. Scouter's wheel is OpenTUI's default, not verified (`research/scouter-inventory.md` §4.6); the owner checks it with a mouse.

Alternatives considered:

- OpenTUI `ScrollBox` with a row widget per entry — rejected: its scrollbar glyphs differ from Scouter's, and hit-testing, selection, and the thumb would then be checked only in the terminal.

### JavaScript, TypeScript, TSX, JSX, Markdown, and Zig use Core's highlight queries

Context: Scouter colours these with the queries `@opentui/core` ships (`assets/{javascript,typescript,typescriptreact,markdown,markdown_inline,zig}/highlights.scm`); the Rust binding bundles the grammar crates' own queries, which name some captures differently (TypeScript property: plain text in Scouter, function colour in fmc; `issue-00001-w3-reader-parity-gaps.md`).

Decision: fmc depends directly on `tree-sitter-javascript`, `tree-sitter-typescript`, `tree-sitter-md`, and `tree-sitter-zig` at the exact versions `opentui` 0.9.9 links, and registers Core's copied queries (MIT notice beside them) for those languages. Chosen by the owner on 2026-11-26.

Consequences:

- (positive) Colours match Scouter cell for cell; no extra C build, the crates are already in the graph.
- (negative) Four direct dependencies whose versions must move with every `opentui` bump.

Alternatives considered:

- Keep the binding's bundled queries and record a colour deviation — rejected by the owner: a visible difference on six languages.

### Approved deviation: `Ctrl+q` quits, plain `q` does not

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter quits on `q` or `Ctrl+c`, also from inside the help overlay (`scouter:src/core/keys.ts:7-9`, `src/app/app.ts:740-744`).

Decision: `Ctrl+q` (shown `^q`) and `Ctrl+c` quit; plain `q` does nothing at the global level. `--help`, the `?` card, the footer hints, the welcome hint card, and `docs/keymaps.md` say `^q`. Approved by the owner on 2026-11-26.

Consequences:

- (positive) No quit on a stray key; `q` is free for file-manager actions later.
- (negative) Scouter users' muscle memory for `q` stops working.

Alternatives considered:

- `Ctrl+q` in addition to `q` — rejected by the owner.

### Approved deviation: Enter on a file keeps focus in the sidebar

Superseded on 2026-11-27 by "Auto-preview in the sidebar" below: Enter on a file moves the focus to the reader again, as in Scouter, and `l`/`→` no longer open a file.

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: in browse mode, Enter, `l`, or `→` on a file opens it and moves focus to the reader (`scouter:src/app/app.ts:425`, `finishShowing`); a binary file already keeps focus on the list (`:417`).

Decision: opening a file from the tree keeps focus on the list, so `j`/`k` keep browsing; `Tab` or a click on the reader moves focus there. The finder and the changes screen keep Scouter's behaviour. Approved by the owner on 2026-11-26.

Consequences:

- (positive) Browsing file after file needs no `Tab` back.
- (negative) Reading a long file needs one extra `Tab` before scrolling keys apply to it.

Alternatives considered:

- Enter previews, a second Enter on the same file focuses the reader — rejected by the owner.

### Approved deviation: a file refused at startup still shows its folder

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter reads a startup file before it reveals it (`scouter:src/app/app.ts:330-335`), so a refused file (not UTF-8) leaves the sidebar empty under `Files`.

Decision: fmc lists the parent folder and selects the file, then shows the same footer and reader notice as Scouter. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The user can browse away from the refused file at once.
- (negative) The startup capture differs from Scouter's in the sidebar.

Alternatives considered:

- Reproduce the empty sidebar — rejected by the owner: it reads as a defect.

### Approved deviation: dragging selects text, `y` copies it

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: in Scouter a left drag across text selects it, copies it on release (OSC 52 plus the local clipboard), and clears the selection; `y` copies the active selection, else the current path (`scouter:src/app/app.ts:801-830`, `docs/decision/0033-...`).

Decision: a drag selects and the selection stays on screen; nothing is copied on release. `y` copies the selection, shows Scouter's `Copied N characters.` / `Copied N lines.` notice, and clears the selection; with no selection `y` copies the path as Scouter does. `Esc` clears the selection. Approved by the owner on 2026-11-26.

Consequences:

- (positive) Selecting text no longer overwrites the clipboard by accident; copying works like a yank in Neovim.
- (negative) Copying a drag needs one more key than in Scouter.

Alternatives considered:

- Keep copy on release — rejected by the owner.
- Add a keyboard visual mode (`v`, `V`) now — not asked for; it would be a separate post-parity item.

### Approved deviation: the tree has no root row

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter draws the browsed folder as row 0 of the tree, with a `▾`/`▸` marker, and it collapses like any folder; its children sit one level in (`scouter:src/explorer/explorer.ts:258-293`, `src/explorer/file-tree.ts`). The sidebar title already shows the root folder's name (`scouter:src/app/chrome.ts:29-32`).

Decision: fmc drops the root row. The root's children are the top level, with no depth guide and no extra indent; the name stays in the sidebar title. `h`/`←`/`Backspace` on a top-level row does nothing; `-` still makes the parent folder the root; a selection that loses its row falls back to the first row. Approved by the owner on 2026-11-26.

Consequences:

- (positive) One level less indentation on every row; no way to collapse the whole tree by accident.
- (negative) Every tree capture is one row shorter and one level shallower than Scouter's.

Alternatives considered:

- Keep the root row without a marker, not collapsible — rejected by the owner.

### Approved deviation: `?` opens the key list as a table in the reader

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's `?` draws a centred help card over the panes and consumes the next key (`scouter:src/ui/help.ts`, `src/app/app.ts:746-750`).

Decision: `?` opens a "Keys" page in the reader pane: tables of `Key | Action` with the key table's own sections (Scouter's card groups: Browse, Reader, Changes, Everywhere, Mouse), generated from the key table, so `--help` and the page share one source and `--help` stays Scouter's text and drawn with the Markdown table renderer. `?` moves focus to the reader so the reader keys scroll the page; `?` or `Esc` restores the previous document, its scroll position, and the previous focus. Search follows rendered Markdown (Scouter refuses `/` there, `scouter:src/app/app.ts:999-1005`): `/` works in the key page's rendered view too, as in any rendered Markdown file (approved deviation, plan-00010, 2026-11-27). This replaces the card, and with it the earlier deviation that added the Mouse section to the card. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The full list is readable, scrollable, and searchable; it always matches the dispatch table.
- (negative) The help no longer floats over the current view; the reader's document is swapped out while it is open.

Alternatives considered:

- Keep Scouter's card and add the table under another key — rejected by the owner.

### Approved deviation: `q` closes the open document or screen

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: in Scouter `q` quits from anywhere (`scouter:src/core/keys.ts:7-9`). fmc moved quitting to `Ctrl+q` (record above), which left `q` unbound. Scouter has no key that closes a document: the reader shows the welcome screen only before the first open.

Decision: with focus in the tree or the reader, `q` closes the open document and shows the welcome screen; focus moves to the tree and the file's row stays selected. With no document open, `q` does nothing. On the Keys page, `q` acts as `?`/`Esc` and returns to the previous document. On the Changes screen, `q` goes back to browsing, as `d` does (lands with plan-00001 W5). In a prompt (finder, search), `q` is typed as text; `Esc` closes it, as in Scouter. Approved by the owner on 2026-11-26.

Consequences:

- (positive) A Neovim-like way to put a buffer away; the welcome hints come back.
- (negative) `q` means different things on different screens; muscle memory from Scouter (`q` quits) now closes instead.

Alternatives considered:

- `q` returns to the previously opened document — rejected by the owner.
- `q` only with reader focus — rejected by the owner.

### The context menu and the dialog are fmc views over OpenTUI's absolute views, with a catcher and a scrim

Context: OpenTUI 0.9.9 has no menu, dialog, or modal widget (plan-00012, "Current state"). Its console overlay is an absolute view at `z_index(i32::MAX)` (`opentui-0.9.9/src/console.rs:450`). `Select` cannot disable a row, and a press on a row that is not selected only moves the selection (`opentui-0.9.9/src/widget/select.rs`, the mouse handler). A dialog must dim the page under it, which `Painter::color_matrix` does (`opentui-0.9.9/src/widget/painter.rs:207`).

Decision: the menu is a bordered absolute view at `z_index(i32::MAX)` with fmc's own rows (`ui/menu.rs`). The dialog is the same kind of view with a title, a wrapped message, an `Input`, an error line, and buttons (`ui/dialog.rs`). A catcher view at `i32::MAX - 1` covers the screen beneath both. It takes every mouse event, and a press closes a menu. A dialog adds a scrim, a `Custom` component that dims the page with `ColorMatrix::gain(0.4)`. While an overlay is open, `app/capture.rs` gives every key to it before the parser, except Ctrl+q and Ctrl+c. Placement is pure: a menu sits at its anchor cell, flips above it or moves left to stay on screen, and in the compact tier is a bottom sheet across the width above the bottom bar. A dialog is centred, 56 columns at most, and spans the width less a column each side in the compact tier. The menu's own rows are the one reason to draw them rather than use `Select`: a disabled row and a single press that picks are what a context menu needs. A debug build opens a dialog at start for a live capture, `FMC_OVERLAY=confirm` or `name` (`app/overlay.rs`, `debug_open`), as `FMC_PANIC_AFTER_START` does. The file keys open the same dialogs and menu (`app/write.rs`). A pointer over a menu row selects it, as a key does; a pointer over a dialog button only highlights it (`Dialog.hovered`, `app/overlay.rs` `hover_button`), and the focus stays where the keys put it, so Return picks the focused button (the owner's decision on 2026-11-27).

Consequences:

- (positive) Placement at the edges and tiers, disabled-row skipping, and key swallowing are unit tests with no terminal (`ui/menu_tests.rs`, `ui/dialog_tests.rs`, `overlay_tests.rs`, `app/overlay_tests.rs`).
- (positive) Both palettes use the existing tokens. The dim is the engine's own colour transform, so it matches the page's colours cell for cell.
- (negative) fmc owns the placement, the click-away, and the focus: closing an overlay blurs the focus, or a key the parser leaves would type into a dialog's field. An upstream change to OpenTUI's overlay behaviour will not reach these views.
- (negative) A menu holds 16 rows and a dialog 3 buttons; both are `shortcut:` comments in the views.

Alternatives considered:

- OpenTUI `Select` as the menu — rejected: it has no disabled rows, and a click on an unselected row only selects it.
- The finder's footer prompt for every dialog — rejected: it fits a name, not a confirmation that lists paths, nor a floating menu (plan-00012, D1).
- Waiting for an upstream modal widget — rejected: none is in the locked source.

### The file-manager operations are the one write path, bounded to the browsed root

Context: the read-only rule (`AGENTS.md`, `rules.md`) held every write shut until the owner opened the file-manager phase. The owner approved plan-00012 on 2026-11-27, which opens it for five operations and a copy and cut, from the tree (its D2 and D3).

Decision (plan-00012 D10): the read-only rule is amended, not removed. Create, rename, copy, move, and delete (to the Trash) are allowed inside the folder fmc opened, from the tree and its context menu. Any other write still needs the owner: saving a file, a write outside the browsed root, and fmc's own settings, which stay read-only (the record "a read-only `settings.json`").

Consequences:

- (positive) The rule names exactly what may write, so a new write can be checked against this record and `rules.md` ("Writes").
- (negative) fmc now writes the user's files, so a defect in `services/write.rs` can lose data. The rules in `rules.md` and the tests in temporary folders carry that load.

Alternatives considered:

- Keep the phase closed and ship the file list alone: rejected, the owner asked for the operations (RI-4).
- One general write API that the finder and the changes screen also call: rejected, the tree only (plan-00012 D2, out of scope).

### Every write checks its path before it acts, and never overwrites

Context: a link inside the browsed root can point outside it, so a write through a link would escape the root (plan-00012 D6). std has no no-clobber rename and no `O_NOFOLLOW`; both need libc.

Decision (plan-00012 D5, D6): each operation runs the same checks before its one system call: the name is one segment, both folders lie inside the root after resolving, the source exists, a folder does not go into itself, and the destination is vacant. A link is operated on as the link. A file copy opens the source and compares the opened file's device and inode with the `lstat` taken before the open (`services/write.rs`, `open_seen`), so a link swapped in after the `lstat` is refused before a byte is read; this needs no libc constant. A folder copy makes the same check before it creates the destination or reads a child (`copy_seen`). A cross-volume move takes the source's device and inode before the copy and refuses to remove the source when they changed (`Error::Changed`), leaving the copy in place.

Consequences:

- (positive) Each refusal and each swap has a test in a temporary folder (`services/write_tests.rs`), with the fixture names of the sample-data rule.
- (negative) The races that remain are each a `shortcut:` comment in `services/write.rs`. The check and the create, rename, or move: std cannot refuse an existing target atomically. The folder copy: its identity is checked before its children are read, so a folder swapped for a link before the check is refused (LOW, from the 2026-11-27 `security-review`). A swap in the gap between that check and `read_dir` still copies the outside folder (the upgrade reads the checked descriptor, `fdopendir`, a libc call). The check and the Trash call. Each needs a process running as the user, which can already write there.
- (negative) A cross-volume move refused after its copy leaves the copy at the new place and the original where it was.

Alternatives considered:

- A Replace, Keep both, or Cancel dialog for a taken name: rejected for the first cut (D5); more surface, and it can follow. The owner kept it deferred on 2026-11-27: a paste into a folder that has the name keeps the automatic `copy` names (`services/write.rs`, `free_name`).
- `O_NOFOLLOW` through a libc constant: rejected, it needs a new dependency or a per-OS constant that the identity check does not need.

### The clipboard is fmc's own, and the footer names what it holds

Context: plan-00012 D2 and D3 put copy, cut, and paste on the tree, and D8 runs a paste off the frame. A system clipboard for files (Finder's copy and paste between apps) is out of scope.

Decision (owner, 2026-11-27: the footer wording is approved; plan-00014 D11 amends the one-path clause): the clipboard holds the paths of the entries `c` or `x` took, each with its device and inode at that moment, and whether they are a cut, and nothing else; a run leaves an entry alone when its device and inode have changed since, and says so in the footer (the security review of phase 3, finding 1); it lives in the model and is lost when fmc exits. The footer says `Copied <name>. Paste it with v.` or `Cut <name>. Paste it with v.` for one entry, and `Copied 3 items. Paste them with v.` for several. A paste runs as a bulk run on the worker (the record below): a second `v` while one runs says `Still working. Try again when it finishes.`, and the footer says `copying` or `moving` while it runs. A cut row is dimmed in the tree until it is pasted or the cut is dropped; `Esc` in the list drops a copy or a cut (the second `x` was the other option).

Consequences:

- (positive) A copy and a cut are one key each in the tree, with no mode to remember.
- (negative) The clipboard does not survive a restart and does not reach other apps.

Alternatives considered:

- `y` and `p`, as yazi uses: rejected, `y` copies the path here, and `p` toggles the preview (plan-00012 D3).

### Marks are a set of paths in the browser, and Esc drops them after the clipboard

Context: plan-00012 D7 left multi-select out: the tree has one cursor, and each write acts on one row. A mark kept by row index would name another entry once a folder collapses or a listing changes (plan-00014, "Current state").

Decision (owner, 2026-11-27: `s` marks and moves down, `S` clears; plan-00014 D1 to D3, D9, D10, D14): the browser holds the marked paths in a `BTreeSet` (`explorer/browser.rs`). A listing that drops an entry drops its mark, and a new root starts with none. The tree draws `✓` in the bar's cell and the footer counts the marks; the finder and the changes screen draw and count none. Esc in the list drops the clipboard first, then the marks.

Consequences:

- (positive) The set follows the entries: scrolling, folding, and a relisting keep the marks that still exist, and a renamed or trashed entry leaves its mark.
- (negative) A mark under a collapsed folder is not drawn, but the footer counts it, and the confirm of an action names it (plan-00014, D6). A hidden dotfile keeps its mark for the same reason.
- (negative) Marks are not saved: they are lost on exit, as the clipboard is.

Alternatives considered:

- A mark on a row index: rejected, a collapse moves the rows under it.
- A background tint for a mark: rejected, the selection already uses that tint.
- Ctrl+click or Cmd+click: rejected, terminals do not send them; Shift+click is the terminal's own selection (the owner, 2026-11-27).
- Space to mark: rejected, Space opens the menu (plan-00012, D3).

### A bulk run goes to the worker one item at a time, and stops at its first error

Context: plan-00012 D8 ran one paste on the worker. Several marked entries (plan-00014) need a run that the footer can count, that stops at the first error, and that does not race two writes in one folder.

Decision (plan-00014 D7, D12, D13): a run is a queue in the model (`app/write.rs`, `Bulk`). Each item is one job on the worker (`services::spawn`), and it answers with `Event::Written`; the UI starts the next item when the answer arrives, so the footer says `copying 2 of 5…`. The first error stops the run and says how many landed (`Copied 2 of 5. <error>`), and the items the run did not reach stay where they are. A run's marks clear only when every item landed. Copy, move, and duplicate use the queue, and a single entry is a queue of one.

Consequences:

- (positive) One write is in flight at a time, so two items never race each other, and a second write waits (`Still working`).
- (positive) No new thread or channel: the worker and the event path that a paste already used carry each item.
- (negative) A run pays one round trip through the event loop per item. A run of many small entries is slower than one bulk call would be.
- (negative) No cancel key is offered: a run ends at its next answer or at its last item.

Alternatives considered:

- One worker job for the whole run: rejected, the footer could not count the items without a stream, and the stop at the error would live on the worker, away from the tree's relisting.
- Items in parallel: rejected, two writes into one folder race, and the footer cannot count them.

## Pending decisions

- `rust-version` 1.90 was confirmed by `cargo +1.90 check` in `plan-00001` phase 3. Superseded by the owner's decision O4 of `plan-00006` (1.92, for `mermaid-text`; see "Decision: Mermaid is drawn by `mermaid-text`"); `rust.md` records the floor.

Note (2026-11-26, plan-00001 W5): a knock-on of the `Ctrl+q` record above. The footer hint `^q quit` is one column wider than Scouter's `q quit`, so Scouter's own fit (`scouter:src/ui/footer.ts` `fitHints`, width minus 2) drops one more hint at 80 columns; on the changes screen that is `a one file`. Accepted as a consequence of the approved key change; the key list always shows every key.

### Deleted entries go to the Trash through Finder, not through a direct move

Context: plan-00012 D4 makes a delete recoverable, so Finder's Put Back is the undo. The `trash` crate (5.4.0) moves a macOS entry in one of two ways: its default `DeleteMethod::Finder` runs `osascript` with `tell application "Finder" to delete { POSIX file "…" }`, and `NsFileManager` calls `trashItemAtURL` (`trash-5.4.0/src/macos/mod.rs:13-22`, `:45-47`, `:85-110`). The crate's table says only Finder gives Put Back; `NsFileManager` is faster, makes no sound, and asks for no permission.

Decision: keep the default, Finder through `osascript`. Put Back is the reason D4 chose the Trash, and the plan's undo for delete depends on it. The crate passes the path as an argument with no shell and escapes `"` and `\` (`trash-5.4.0/src/macos/mod.rs:184-192`), so a file name cannot become script text. A non-UTF-8 name is refused before the call, because the crate percent-encodes such a path and names a different entry (`services/write.rs`, `Error::NotUtf8`).

Consequences:

- (positive) Finder's Put Back restores a deleted entry, as D4 and the confirm dialog promise.
- (negative) Each delete runs a subprocess, `osascript`, and waits for Finder.
- (negative) The first delete from a terminal asks macOS for Automation permission to control Finder. A refusal fails the delete with the dialog's error line, and nothing moves.
- (negative) The delete path depends on Finder being available; a Mac without it cannot trash.

Amended (plan-00014, D12 and D15): the Trash runs in the bulk queue on the worker, one entry per job, as copy and move do. A single delete is a run of one. The method stays Finder through `osascript`. Each entry's device and inode are checked before the first call, and again on the worker before each one (a changed entry is skipped and stops the run, with the footer naming it). Errors go to the footer, not to the dialog, which has closed by then.

Links (plan-00014, D16; a live check on 2026-11-27): Finder refuses to delete a symbolic link, dangling or not, with error -5000 (`osascript -e 'tell application "Finder" to delete { POSIX file "…/link" }'`), while a plain file in the same folder goes to the Trash. So fmc refuses a link before its confirm and moves nothing. This amends plan-00012 D6, which said a link is trashed as the link. The owner's options: (1) `NsFileManager` for links only, which the crate's table says gives no Put Back (not checked here); (2) keep the refusal, the current state; (3) another route to the Trash for a link. The owner picks; until then the refusal stands.

The checks before Finder and the Finder call are separate steps. The shortcut in `services/write.rs` (`trash_with`) records that a same-user process can swap an entry or a parent in between (the security review of phase 4, findings 1 and 2).

Alternatives considered:

- `NsFileManager` (`DeleteMethod::NsFileManager`) — rejected: no Put Back, so the delete is not undoable from Finder.
- `std::fs::remove_*` — rejected: not recoverable (D4).
- Calling `osascript` directly — rejected: the crate already does it with the escaping, and a second copy would be ours to keep right.

### Approved deviation: the diff chrome shows settled values after a layout change

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: after `c` folds or unfolds a file in the stacked diff, Scouter updates the footer (hunk index, percentage) and the scrollbar thumb before the layout settles, so they keep the pre-fold values until the next scroll (`scouter:src/app/diff-screen.ts:268-275`, `src/app/diff-blocks.ts:242-252`). Live at 80x24 on the demo repo: Scouter `2/8 · 37%`, fmc `3/8 · 41%`; after one scroll both show `3/8 · 41%`.

Decision: fmc updates the footer and the thumb from the settled layout right after the fold. Approved by the owner on 2026-11-26. Widened by the owner the same day: whenever Scouter's chrome (footer counts, percentage, thumb) lags a layout change on the changes screen until the next scroll, fmc shows the settled values instead, without a new question. First case after folds: `r` reload, where Scouter shows `9/9 hunks · Top` and fmc `1/9 hunks · Top` with identical bodies.

Consequences:

- (positive) The footer and thumb always describe what is on screen.
- (negative) A capture taken right after `c` differs from Scouter's until the next scroll.

Alternatives considered:

- Reproduce the lag — rejected by the owner: it is an ordering accident, not a designed behaviour.

### Approved deviation: `a` and `x` keep the selected file at the top

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: after `a` (one file or all files) or `x` (context size), Scouter's `jumpAfterLayout` intends to put the selected file at the top, but the scroll lands on the file's pre-change row (live: `a` lands at Top with `8/8`; `x` with lumos selected lands at README's end, 14%).

Decision: fmc puts the selected file's header at the top of the body after `a` or `x`, as `jumpAfterLayout` intends. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The file you were reading stays in view after a layout change.
- (negative) The scroll position after `a` or `x` differs from Scouter's.

Alternatives considered:

- Reproduce Scouter's landing row — rejected by the owner.

### Approved deviation: a binary file's footer shows no stale position

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's binary view does not update the footer position and `setDocument` does not clear it, so a binary opened after another document keeps that document's position (`scouter:src/app/app.ts:411-419`, `showBinary`). Live at 80x24: Scouter `4.0 KB · binary · All` for `assets/seal.bin` opened after `assets/marauder-map.png`; fmc `4.0 KB · binary` (audit-00001 F-07, issue-00002).

Decision: fmc shows no position for a binary file. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The footer describes only what is on screen.
- (negative) A binary footer differs from Scouter's after another document was open.

Alternatives considered:

- Copy the stale position — rejected by the owner.

### Approved deviation: no `--upgrade` flag

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's `--help` lists `scouter --upgrade`, which runs its installer (`scouter:src/index.ts:11`, handled at `:26-28`). plan-00001 lists the installer and `--upgrade` as out of scope; fmc installs with `just install` (audit-00001 F-08, issue-00010).

Decision: fmc has no `--upgrade` flag and its `--help` omits the line. Approved by the owner on 2026-11-26.

Consequences:

- (positive) No self-updating installer or release channel to maintain during parity.
- (negative) Users update by rebuilding or reinstalling.

Alternatives considered:

- Implement `--upgrade` — rejected by the owner: it needs an installer and a release channel.

### Approved deviation: the changes screen opens in one-file mode

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's changes screen starts with every changed file stacked in one body (`scouter:src/diff/diff-screen.ts:82`, `scope: DiffScope = "all"`); `a` toggles to one file.

Decision: fmc opens the changes screen showing only the selected file's diff; `a` toggles to all files and back, as in Scouter. Approved by the owner on 2026-11-26, after the parity evaluation.

Consequences:

- (positive) The diff body shows one file at a time by default, which is easier to read in a large change set.
- (negative) The first frame of the changes screen differs from Scouter's; Scouter users press `a` once more.

Alternatives considered:

- Keep Scouter's all-files default — rejected by the owner.

### Approved deviation: a click focuses the pane under the pointer

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: in Scouter only rows take a click: a tree row (`scouter:src/explorer/explorer.ts:190`), a change row (`src/diff/changes.ts:188`), and a diff header (`src/diff/diff-blocks.ts:468`). A click elsewhere in a pane, for example in the reader text, does not move the focus; `Tab` does.

Decision: a left click anywhere in a pane (the tree, the reader, the changes list, the diff body), including its scrollbar, focuses that pane. A click on a row keeps its action (open, toggle, select, fold) as before; in the reader a press still starts a drag-select. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The mouse can move the focus without `Tab`.
- (negative) A click in the reader now changes the focus and the footer hints, where Scouter changes nothing.

Alternatives considered:

- Keep Scouter's row-only clicks — rejected by the owner.

### Approved deviation: the sidebar hides below 80 columns, and a border mark or `Ctrl+/` shows or hides it

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter always draws the sidebar. Its width is a quarter of the terminal within 28 to 40 columns, constrained so the reader keeps at least 28 (`scouter:src/explorer/sidebar-resizer.ts:48-53`), and no key or click hides it. `Ctrl+/` is unbound there; Scouter binds `/` only (`scouter:src/app/app.ts:931,965`). A live probe in Rex at 80x24 logged that `Ctrl+/` reaches fmc as `Char('/')` with Ctrl under the Kitty keyboard protocol that OpenTUI enables (`opentui-0.9.9/src/host.rs:147`), and as `Char('_')` with Ctrl from the legacy byte `0x1F` (`opentui-0.9.9/src/input/core_tests.rs:202`).

Decision: below 80 columns the sidebar hides on its own, and the reader takes the full width. At 80 columns or more the layout is Scouter's, so the 80x24 captures do not change. `Ctrl+/`, or a click on the mark on the border, flips the current state: `«` on the sidebar's border while it is shown, `»` on the reader's left border while it is hidden. A manual choice holds until the width crosses 80 again, and a crossing resets to the automatic state. When the focused sidebar hides, the focus moves to the reader. Nothing is saved across runs (RI-9). Approved by the owner on 2026-11-26. Amended by the owner on 2026-11-27: while no document is open (the welcome screen), the sidebar stays shown at any width, and the 80-column rule applies once a document opens; `q` closing the document brings the list back. The mark sits on the top border row (`«` left of the sidebar's top-right corner; `»` at the reader's top-left corner). While hidden: the footer shows `^/ sidebar` and drops `Tab files`; `Tab`, `Esc`, and `Ctrl+←`/`Ctrl+→` neither focus nor resize the sidebar. `Ctrl+_` is bound with `Ctrl+/`, because the legacy encoding (Terminal.app) sends `0x1F`, which OpenTUI reports as `Ctrl+_`.

Consequences:

- (positive) A narrow terminal gets a full-width reader, and 80x24 keeps Scouter's layout.
- (negative) Below 80 columns the tree is out of sight until it is toggled back, where Scouter always shows it.

Alternatives considered:

- Hide below 60 columns — rejected by the owner: a cramped reader from 60 to 79.
- Hide below 100 columns — rejected by the owner: it hides the sidebar at 80x24 and changes every parity capture.
- The sidebar title as the click target — rejected by the owner: hard to see, and nothing to click once the sidebar is hidden.

### Approved deviation: `Ctrl+.` or `.` shows and hides dotfiles, which are shown by default

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter lists every entry except its noise names (`.git`, `.hg`, `.svn`, `.DS_Store`, `.AppleDouble`, `Thumbs.db`, `desktop.ini`; `scouter:src/core/files.ts:34`), so dotfiles are shown, and it has no key that hides them. `.` and `Ctrl+.` are unbound in Scouter. A live probe in Rex logged `Ctrl+.` as `Char('.')` with Ctrl, which only the Kitty keyboard protocol can carry, and a plain `.` as `Char('.')` with no modifiers. Terminal.app cannot send `Ctrl+.`.

Decision: `Ctrl+.` and `.` show or hide dotfiles in the tree and in the finder's index. Dotfiles are shown by default, and the noise names stay hidden in both states. `.` is the fallback for `Ctrl+.`, which reaches fmc only from terminals with the Kitty keyboard protocol. Nothing is saved across runs (RI-9). Approved by the owner on 2026-11-26.

Consequences:

- (positive) Dotfiles are one key away in every terminal, and the default matches Scouter.
- (negative) `.` is a second key for the same toggle, and it is a key Scouter leaves free.

Alternatives considered:

- `Ctrl+.` alone — rejected by the owner: Terminal.app cannot send it.
- Dotfiles hidden by default — rejected by the owner: Scouter shows them.

### Approved deviation: auto-preview in the sidebar, and Enter alone opens

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: in Scouter a file shows in the reader only when it is opened: Enter, `l`, or `→` on a file opens it and focuses the reader (`scouter:src/app/app.ts:425`), and a click on a tree row opens it (`src/explorer/explorer.ts:190`). Moving the selection with `j`/`k` changes nothing in the reader. fmc's earlier record "Enter on a file keeps focus in the sidebar" is superseded by this one.

Decision (owner, 2026-11-27):

- Auto-preview: while the sidebar is focused, a file that becomes selected (by `j`/`k`, the arrows, `[`/`]`, `g`/`G`, or a click) shows in the reader after a short debounce, without moving the focus. A folder leaves the reader as it is. On by default; `p` in the sidebar toggles it for the session, and `auto_preview` in `settings.json` sets the default.
- Enter on a file opens it and moves the focus to the reader, as in Scouter. Enter on a folder toggles it.
- `l`/`→` expand a folder and do nothing on a file; `h`/`←` collapse. Only Enter opens.
- A click in the sidebar while another pane is focused only focuses the sidebar: the selection and the reader do not change. Once the sidebar is focused, a click selects the row (and previews a file, or toggles a folder). This amends "a click focuses the pane under the pointer", whose row actions now need the sidebar focused first.
- `Ctrl+/` or a click on the `»` mark that shows the sidebar also focuses it (the mark click added by the owner on 2026-11-27).

Consequences:

- (positive) Browsing shows each file without a key press; reading a file is one Enter away; a stray click on the list never replaces the document being read.
- (negative) Moving through a folder of large files reads each one (bounded by the debounce); a click on the list from the reader needs a second click to act.

Alternatives considered:

- Enter keeps the focus in the sidebar (the superseded record) — rejected by the owner.
- A first click that also moves the selection — rejected by the owner.

### Approved deviation: `[` and `]` jump in the sidebar and the reader

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: in Scouter `[` and `]` step hunks on the changes screen (`scouter:src/app/app.ts`, diff keys) and do nothing while browsing.

Decision: while browsing, `]` moves the sidebar selection down by `sidebar_jump` rows (default 2) and `[` up; in the reader `]` scrolls down by `reader_jump` lines (default 5) and `[` up. Both values come from `settings.json`. On the changes screen `[`/`]` keep stepping hunks. Approved by the owner on 2026-11-27.

Consequences:

- (positive) A middle step between one row and a page.
- (negative) `[`/`]` mean different things on the two screens.

### Decision: a read-only `settings.json` with its keys

Context: auto-preview and the `[`/`]` steps must be configurable (owner, 2026-11-27); RI-9 planned `~/.config/fmc/settings.json`.

Decision: fmc reads `~/.config/fmc/settings.json` (under `$XDG_CONFIG_HOME/fmc/` when set) once at start. Keys: `auto_preview` (bool, default `true`), `sidebar_jump` (positive integer, default 2), `reader_jump` (positive integer, default 5). A missing file means the defaults. A file that fails to parse, or a key with a wrong type or a value below 1, is not applied: fmc keeps the defaults for the bad keys and shows a notice once. Unknown keys are ignored. fmc never writes the file in this step; saving UI state and the theme override stay in RI-9. Crate: `serde_json` only (the parse reads a `Value`; `serde` derive, also approved, is not needed). Approved by the owner on 2026-11-27.

### Approved deviation: `:q` quits, as in Neovim

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter quits with `q` or `Ctrl+c`; fmc moved quitting to `Ctrl+q` (record above). `:` is unbound in both.

Decision: `:` outside a prompt opens a one-line command prompt in the footer, Neovim style. `q`, `q!`, `qa`, `qa!`, and `quit` quit on Enter; `Esc` or an empty Enter cancels; anything else shows `Not a command: <text>`. No other commands in this step. `Ctrl+q` and `Ctrl+c` still quit. Approved by the owner on 2026-11-27 (the prompt shape is the agent's pick, open to change).

Consequences:

- (positive) Neovim muscle memory quits fmc.
- (negative) A new prompt mode; `:` is no longer free for another binding.

### Approved deviation: a phone layout below 80 columns

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: the owner runs fmc in Rex on a phone (narrow terminal, touch: a tap is a left click, a swipe is the wheel, keys like Esc and Ctrl are hard to type). The phone audit (`audit/audit-00002-mobile.md`, proposals P1 to P8) found a one-cell `»` target, a 12–17 cell sidebar below 56 columns, keyboard-only actions, cut names, and a clipped welcome. Scouter has no narrow-terminal layout.

Decision (owner, 2026-11-27). Everything here applies only below 80 columns, so the layout at 80 columns and wider stays Scouter's (plus the earlier records), except P7:

- P1: while the sidebar is hidden, a tap anywhere on the reader's header row shows it (and focuses it, as the `»` mark does). The `»` mark stays.
- P3: below 60 columns the shown sidebar is a full-width drawer over the reader. Opening a file (Enter or a tap on a file) closes the drawer and shows the reader; `Ctrl+/` or a tap on the header row toggles it. This amends "the sidebar hides below 80 columns" for widths below 60. Amended by the owner on 2026-11-27: below 60 columns the welcome screen shows full width with the list hidden (overriding, for these widths only, the rule that the welcome keeps the sidebar); a header tap opens the list as the drawer and a second tap hides it. While the drawer covers the reader, auto-preview does not run (the reader is not visible, and a preview would close the drawer); a tap or Enter on a file opens it and closes the drawer. The duplicate `[ ]` row in the key list is removed (agent decision: a bug in fmc's own key list, not a Scouter screen).
- P2: footer hints are tappable: a tap runs the hint's action (`Esc close`, `? help`, `/ find`, `Tab`, `d changes`, `q close`, …). `Ctrl+q` stays keyboard-only.
- P4: rows that show a path (finder results, change list) show the file name first and the folder dimmed after it.
- P6: footer hints are fitted in a fixed priority order per screen, so the hint that leaves the current state (`Esc`, `^/ sidebar`, `q`) stays.
- P5: below about 50 columns the welcome drops its card and wraps its lines instead of cutting them.
- P8: while a text selection exists, the footer shows a tappable `Copy` hint that does what `y` does.

Consequences:

- (positive) fmc is usable by touch on a phone; no action needs a hard-to-type key except quitting.
- (negative) Narrow layouts have their own rules and tests; the footer gains click handling.

Amended by the owner on 2026-11-27 (screenshot from a phone): the one-cell `»` mark is not accessible by touch. Below 80 columns, while the list is hidden, the reader's header shows a button to the left of its title, drawn as a filled box like a web menu button (`≡`); a tap shows the list and focuses it. While the drawer is open, its header shows the same box with `✕` to the left of its title; a tap hides it. The `»` mark is not drawn below 80 columns. The tappable `^/ sidebar` footer hint stays as a second target near the thumb. The welcome text names the button while the list is hidden (`Tap ≡ to browse files.`), and its key and reading lines break at the ` · ` instead of mid-phrase. At 80 columns and wider the marks stay as they are: `«` left of the sidebar's top-right corner, `»` on the reader's top-left corner.

Implementation notes (agent, 2026-11-27, under the amendment above; each is a sub-decision, not a new deviation). The box is the palette's pill pair, `accent` behind and `accent_ink` on it (bold), not the `accent_soft` behind `text` the amendment suggested: `text` (`#d4d8de`) on `accent_soft` (`#c7d2fe`) is near-invisible in the dark palette, and `accent_soft` (`#4338ca`) behind `text` (`#1f2328`) is dark on dark in the light one. The menu glyph is `≡` (U+2261), not `☰` (U+2630): opentui's Unicode width method measures `☰` two cells wide (`text_width`), which would widen the box; `≡` and `✕` (U+2715) measure one cell. The close box shows below 80 columns while the list shows and can hide, so a drawer below 60 and a list beside a document from 60 to 79; the welcome's list at 60 to 79 cannot hide, so it has no box. The marks `«` and `»` are not drawn below 80, which also covers `«` that the amendment does not name, since the boxes are the touch targets. The welcome's closing line breaks at its ` · ` below 80 columns, and the separator is dropped at the break.

### Approved deviation: a diff header folds on release, not on press

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter folds a diff file on a press on its header (`scouter:src/diff/diff-blocks.ts:468`). On a phone a swipe that starts on a header folds it.

Decision: a header folds when the button is released over the same header without the pointer having moved; a drag or swipe that starts on a header does not fold. Applies at every width. Approved by the owner on 2026-11-27 (audit-00002 P7).

Consequences:

- (positive) Scrolling by touch no longer folds files by accident.
- (negative) A click folds on release instead of on press, a small timing difference from Scouter.

### Approved deviation: the line-number gutter stays put when the reader pans

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's code view wraps the text in a `LineNumberRenderable` (`scouter:src/reader/code-view.ts:121`) inside the reader's `ScrollBoxRenderable` with `scrollX: true` (`src/app/reader-pane.ts:31-48`), so a horizontal pan (`h`/`l`, a sideways wheel) moves the line numbers out of view with the text. fmc matched that.

Decision: the gutter is sticky: a horizontal pan moves only the text, and the line numbers stay in their column, as in common editors. Vertical scrolling is unchanged. Applies to every view with a line-number gutter (the code view, Markdown source, the finder preview if it shows numbers). Approved by the owner on 2026-11-27.

Consequences:

- (positive) The line number of every visible row stays readable while panning long lines.
- (negative) The pan range no longer includes the gutter; a capture after `l` differs from Scouter's.

### Approved deviation: the reader shows git change signs in a strip left of the line numbers

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's code view draws line numbers only. Its gutter is one `LineNumberRenderable` with no signs, and the file makes no git call (`scouter:src/reader/code-view.ts:121-131`; the plan cited 99-108, which holds the fields). Scouter shows a change only on the changes screen (`src/diff/`), so the open file's changes are not visible in place.

Decision: the owner approved (2026-11-27, plan-00008, O1 to O3) a one-column strip left of the line numbers in the text view. A `▎` in `added` (green) marks an added line, a `▎` in `modified` (yellow) a changed line, a `▁` in `removed` (red) the line above a deletion, and a `▔` on line 1 a deletion above it. The base is the changes screen's (HEAD, staged and unstaged together), so the strip and `d` agree. `git_gutter` (bool, default `true`) in `settings.json` turns it off. Inside a repository the strip shows for every text file, with or without changes. Outside a repository, or with the setting off, the reader is as Scouter draws it.

Agent decisions within that approval (`plan-00008`, D1 to D7):

- D1: the strip is reserved by a blank sign on line 1 (`ui/reader.rs` `draw_signs`), so the numbers do not move as signs come and go. "Inside a repository" is the tree's marks (`app/tree.rs`, `m.in_repo`).
- D2: the classes come from a `-U0` patch (`src/diff/signs.rs`). The first new lines of a hunk are modified, as many as the old side had, and the rest are added; a hunk with no new lines marks the line above, or line 1 with `▔`; an untracked file is all added. A renamed file diffs from its old path, found by the listing's rename detection, so a pure rename has no signs, as `d` shows it (`services/git.rs` `file_signs`).
- D3: the text view only. The finder's preview and rendered Markdown, images, binaries, and the changes screen draw no signs. Auto-preview's reader does.
- D4 (refined): the sign job runs when a text is shown and on each browse tick (2 s to 30 s), not only when the file reloads or the change stamp moves. A commit that keeps some of the file's edits leaves its mark at `M`, so the marks cannot show that the signs moved (checked live: after a partial commit only the uncommitted line kept its sign). The job is a `-U0` diff of one file plus an untracked check: about 45 ms on fmc's repository, against about 60 ms for the marks job the same tick runs, and it runs on a worker. An answer is applied only to the showing it was asked for: `Shown::Text.serial` counts showings, and a reload keeps its showing, so the next tick brings the reloaded file's signs (`app/gutter.rs` `apply`). The plan keyed the answer by a reload generation; the showing serial covers the same case with one counter.
- D5: a git failure, or a file outside a repository, clears the signs with no notice.
- D6 (accepted): `LineSign::before` adds one column to the gutter's width (`opentui-0.9.9/src/widget/line_numbers.rs:95-106`). Live at 70 columns, the first digit of line 1 moved from column 3 to column 4 inside a repository.
- D7: every tier shows the strip. Live: 49 columns (compact), 70 (medium), and 100 (wide).

Consequences:

- (positive) A change shows where it is read, and against the base the changes screen uses.
- (negative) A text file in a repository is one column narrower in its text, so a capture taken there differs from Scouter's by one column. Outside a repository and with the setting off, the captures are byte-identical to v0.3.4 for the 80 by 24 case checked live.
- (negative) Each open text costs one more git job per browse tick, about 45 ms on a worker.

Alternatives considered:

- Signs from the changes listing, with each file's patch read once per listing — rejected: an unchanged file has no entry, and the listing stops at 1000 changes.
- Refresh only on a reload or a change in the marks — rejected by D4: a partial commit moves the signs and leaves the marks as they were.
- A strip only while the file has changes — rejected by D1: the numbers would move as the file is edited.

### Decision: three layout tiers, and a phone-native compact tier

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context (2026-11-27): on a real phone (about 49 columns) touch navigation did not work, and the drawer, the filled `≡`/`✕` boxes glued to the title, the two-row footer, the panel borders, and three overlapping ways to toggle the list made the UI clunky (owner screenshots). The owner delegated the redesign ("go wild, no confirmation needed; document every decision and why"). This record supersedes, below 60 columns, the drawer, the header-row tap, the menu and close boxes, the full-width welcome, and the tappable footer hints of "a phone layout below 80 columns" and its amendments; the parts not named here stay.

Decision (agent, on the owner's delegation, 2026-11-27):

1. **Three tiers by terminal width:** compact below 60, medium 60–79, wide 80 and over. Why: each range has a different binding constraint (finger and thumb below 60, column budget at 60–79, Scouter parity at 80+); one set of rules stretched across all three is what made the phone UI clunky.
2. **Wide (80+) stays Scouter plus the earlier records.** Why: parity is the product's base; the owner's desktop complaints were about behaviour already fixed (auto-preview, Enter), not layout.
3. **Compact is a navigation stack, not a drawer.** Two full-screen pages: Files and Reader (plus the existing Changes, Keys, and Find pages). Opening a file pushes Reader; Back returns to Files with the selection and scroll kept. Why: this is how every phone file browser works (iOS Files, Android Files); a drawer over a reader only makes sense when the reader is visible beside it, which it is not at 40–59 columns.
4. **Compact starts on Files, with no welcome card.** Why: on a phone the first useful screen is the list; the welcome card was telling the user to tap something to get to it.
5. **Compact drops the panel border and the header rule.** Chrome becomes one top bar and one bottom bar (2 rows instead of 6, 0 border columns instead of 2). Why: at 49x40 the border, the header rule, and the two-row footer took 6 of 40 rows and 2 of 49 columns; content is what a phone screen is for. Superseded in part by plan-00007 (owner, 2026-11-27): compact draws the border and the two-row header too (the record below).
6. **Top bar:** left, a text back button `‹ Files` on Reader-like pages (the folder name on Files); centre-left, the page title (the file name); right, the existing quiet tag (`files`, `read-only`). Plain text in the accent colour with a space after it, no filled box. Why: the filled box glued to the title read as a rendering glitch; a text back button is the platform idiom and needs no glyph-width tricks. Superseded in part by plan-00007: the back text is a lone `‹`, and the page name is dropped.
7. **Bottom bar:** one row of equal-width segments, each a word, no key glyphs: Files page `Find · Changes · Keys`; Reader page `Back · Search · Keys`; Changes page `Back · All/One · Split`; a prompt (find, search, `:`) replaces the bar while open, with `Cancel` as the first segment. Why: equal segments make every target at least 12 columns wide at 40 columns, thumbs live at the bottom, and words beat `^/` and `?` glyphs for people who are not using a keyboard. The mode pill and the path line go; the top bar carries the location.
8. **Touch rules in every tier:** a row acts on release when the pointer did not move (as the fold rule already does), so a swipe never opens or toggles; a tap on a file opens it (compact: pushes Reader; medium and wide: as today); swipes scroll the pane under them; the whole row is the target. Why: a finger lands, drifts, and lifts; acting on press turns every scroll attempt into an accidental open.
9. **Compact turns auto-preview off.** Why: the reader is not on screen while the list is; a preview there is wasted work.
10. **Medium (60–79) keeps the sidebar-plus-reader split with auto-hide once a document opens (plan-00002 D1), but the toggle becomes a text chevron in the header:** `‹` at the start of the list's title hides it, `›` at the start of the reader's title shows it, each followed by a space. The filled boxes and the border marks are not drawn there. Why: the same idea as the back button, at a size that fits a column budget instead of a thumb.
11. **Keys keep working everywhere** (`Ctrl+/`, `q`, `Esc`, `Tab`, `?`, `/`, `:q`), and in compact `q`/`Esc` mean Back. Why: Rex on a phone has a key bar; touch and keys must reach the same actions.
12. **Touch delivery is verified, not assumed.** A raw-input probe ran in a tab on the phone to see what Rex's phone client sends for a tap, a swipe, and a long-press; whatever it shows decides the event handling (for example, a tap that arrives only as a press without a release). Why: the owner reported that touch did nothing, and SGR clicks from the Mac never showed it.

Consequences:

- (positive) A phone gets a familiar two-page browser with large targets and almost all of its rows for content; the desktop is unchanged at 80+.
- (negative) Compact is a third layout to test; the welcome card and the path line are not shown below 60 columns.

Agent decisions (plan-00005, taken under the owner's delegation; each is reversible and listed here with its reason):

- Phase 1, `DRAWER` is renamed `COMPACT`: agent decision: the constant is the compact edge in the new design, and the drawer it named goes in phase 2 — why: one name per edge, and a name for a removed feature would mislead.
- Phase 1, the row click reuses the header fold's tracker, renamed `Click`: agent decision: one rule for every click (a press and a release at one cell, no drag between) — why: a second tracker would drift from the fold rule and need its own tests.
- Phase 1, the press still focuses the list at once; only the row's action waits for the release: agent decision: focus feedback stays immediate, and the release is what a swipe must not trigger — why: record 8 names the action, not the focus.
- Phase 2, the page stack follows the screens at each paint instead of being pushed at each open: agent decision: Find, Changes, and Keys open and close with their screens, and Reader follows the focus in the reader — why: a file, a key page, a finder, and a change can each open from several places, and one reconciliation keeps the stack in step with the state that draws it. `Pages` keeps each page in place while it is open.
- Phase 2, the compact tier sets the focus from the top page: agent decision: the reader and the key page take the focus, and the list pages hold it with the list — why: the keys and the focus already mean "the reader is on screen" (the keys scroll the reader only while it holds the focus, and move the list only while the list holds it), so a page and its focus cannot disagree.
- Phase 2, `q` and `Esc` go back a page in compact, and `q` on the list page keeps its wide meaning (it closes the open document): agent decision: Back keeps the document open, so the next tap on a file or the next `Enter` shows it again; the list page has nothing to go back to — why: record 11 names `q` and `Esc` as Back, and closing the document there would lose the place without a reason.
- Phase 2, a binary file opens the reader page in compact: agent decision: the compact tier moves the focus to the reader for a binary's notice, where Scouter keeps it on the list — why: a tap that showed nothing would read as a broken tap, and the notice is the page's only content. Medium and wide keep Scouter's focus.
- Phase 2, the startup file opens the reader page as a tapped file does: agent decision: the owner's "start on Files" applies to a start with no file argument; a file named on the command line opens as Scouter opens it — why: the user asked for that file by name.
- Phase 2, Tab and `Ctrl+/` switch between the list and the reader in compact: agent decision: `Tab` already moves the focus between them, and `Ctrl+/` (the sidebar key) takes the same step, so both keys keep working (record 11) — why: a second meaning for `Ctrl+/` in compact is simpler than a hidden key that does nothing.
- Phase 2, the welcome screen's narrow wrap, its menu prompt, and `NARROW_WELCOME` are removed: agent decision: the welcome is not drawn below 60 columns, and the menu prompt only named the drawer — why: a deleted screen's text is dead code, and the medium tier's welcome always shows its list.
- Phase 2, the welcome card at 60 to 79 columns wraps its text beside the 28-column list: an open observation, not changed in this phase: the card's lines are cut or wrapped in the medium tier, as before the plan — why: the medium tier is phase 4's; the owner decides whether the card drops below a width.
- Phase 3, the bottom bar's Changes labels show the scope on screen: `All` while every file shows, `One` while one file shows, and `a` toggles it: agent decision: the label names the state, as the footer's pill does, and the tap runs the key — why: D3 names `All/One` without saying which half is shown, and the state is what a reader needs to read the diff.
- Phase 3, `Copy` takes the Reader bar's middle cell while text is selected: agent decision: the bar keeps P8's tappable copy on a phone, and the search key is one tap away again when the selection is cleared — why: a selection cannot be copied on a phone without a tap, and the plan's bar has no other slot for it.
- Phase 3, a prompt's bar is `Cancel` in the first cell and then the prompt's own text: agent decision: D3 says `Cancel`, then the prompt, so the text stays in the bar where a thumb reads it, and the cell's tap presses `Esc` — why: a segment per prompt key would hide the query the user is typing.
- Phase 3, the back text is followed by two columns, not one: agent decision: one column ran `‹ Files` into the title (`‹ Files README.md`) — why: a gap that reads as a separator, not as one phrase. The spec said a space; this is a visible change of width, and the owner can set it back.
- Phase 3, the back text names the page Back returns to, so the key page over a document reads `‹ Reader`: agent decision: Back goes to that page, and the label says where — why: a fixed `‹ Files` would name a page Back does not reach. Superseded by plan-00007: the back chevron names no page; Back still goes to the page under the top.
- Phase 3, the reader's position (`Top`, `37%`) and the file's path leave the compact footer: agent decision: record 7 removes the path line, and the bar has no room for the position beside its words; the scrollbar still shows where the reader is — why: a second line of facts would bring back the two-row footer the record removes.
- Phase 3, the finder's bar in compact is one row with no rule, as the pane headers are: agent decision: the finder's bar takes the header's place (plan-00005, record 5), so its rows start on the same row as the list — why: the bar's second row was pushing the list below the list top the hit test assumes. Superseded by plan-00007: the finder bar is two rows with its hairline in every tier.
- Phase 4, the chevron and the compact back text share the header's lead cell, and each header has one press handler that acts by tier: agent decision: one cell holds one action at a time, so the handler asks the tier (Back in compact, a toggle in medium) rather than two handlers fighting for the press — why: a second handler on the same cell would stop or miss the press depending on the order of the two.
- Phase 4, a chevron shows only where its action works: `‹` on the list's header while a document is open and the list shows, `›` on the reader's header while the list is hidden, and none on the welcome screen — agent decision: a control that does nothing is a defect, and the welcome's list cannot hide (plan-00002, D7) — why: the chevron must read as a control the user can use.
- Phase 4, a tap on the header row no longer toggles the list at any width below 80 (plan-00004, P1 and P3 retired by record 10): agent decision: the chevron is the toggle, so a tap on a title does nothing — why: the owner's report named three ways to toggle the list as the clunky part, and a title tap that hides the list is one of them.
- Phase 4, the `«` and `»` marks are not drawn below 80 columns, as plan-00004 had them: no change in this phase — why: the plan's removal was already true, and the marks are the wide tier's controls only.
- Phase 5, the probe log (747 chunks, 1087 mouse events, 2026-11-27) shows the phone's touch as press and release pairs for a tap (43 taps, each a press and its release at one cell, none unpaired) and as wheel codes 64 and 65 for a swipe (five runs, 1001 events, no drag and no motion events, no long press): agent decision: the handler needs no change, because the release rule and the wheel scroll already match those bytes, and the commit pins them with tests built from the logged bytes (`app/mouse_tests.rs`) — why: a change to a handler that works would only add a second path to keep right. The owner checks the medium tier by hand (see the report).

Agent decisions after plan-00005 (2026-11-27, owner's delegation), answering the executor's open questions:

- The two-space gap after the back text stays: one space ran `‹ Files` into the title.
- The Changes bar shows the scope on screen (`All`/`One`); `Copy` takes the reader bar's middle cell while text is selected; the compact bars show no reader position (the scrollbar does). Each keeps the bar to words that describe what the user sees.
- Phase 5 keeps its `fix(input)` subject; history is not rewritten.
- The medium-tier chevron's tap target widens to the chevron, the gap, and the title text. Why: a one-cell target fails a finger in landscape, and the title is the next obvious thing to tap.
- The focus-first click rule ("a click in the sidebar while another pane is focused only focuses it") applies only in the wide tier. In compact and medium, the first tap on a row acts. Why: on touch there is no hover or visible focus to aim with, so a tap that does nothing reads as broken; that rule is the likely cause of "touch does nothing" on the phone, since the probe showed taps arrive as clean press-release pairs.
- The welcome card is dropped when the reader beside the list is narrower than the card, leaving the wordmark, the tagline, and the prompt. Why: at 60–79 columns the card wrapped into a ragged block.

Follow-ups to the layout tiers (2026-11-27, the same delegation), decisions the executor took:

- The medium chevron's target is the lead text and the title beside it, and the title's box runs from the chevron to the badges: a press on its blank end toggles the list too. Why: the framework hit-tests the box, not the words, and a box whose edge the finger must find is the trap the chevron had.
- Below 80 columns the first press on a row acts (tree, finder, changes); at 80 and wider it only focuses the list, as the owner's rule says. In compact the page sets the focus on every paint, so the focus-first branch is reached only in medium and wide. Why: tests in compact set the focus by hand, so they pin the handler, not a reachable state.
- The welcome card goes in the medium tier when the reader beside the list is narrower than the card. The test is the reader's inner width (its width less its border) against the card's 45 columns, which the widest hint, `changes in a git repository`, sets. With the default sidebar the card goes below 75 columns. Why: at 60 to 74 the card cut into a ragged block. The wide tier keeps the card as it was, so a sidebar dragged wide at 80 still clips it: an open question for the owner, not decided here.

### Decision: compact has the medium tier's border and two-row header

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context (2026-11-27): the owner's screenshots at about 50 columns put compact next to medium. Compact had no panel border and a one-row header, so it read as a different app. Record 5 removed both to free six rows and two columns; the owner asked for both tiers to look the same.

Decision (owner, 2026-11-27, plan-00007):

1. Compact pages draw the medium tier's rounded panel border. In compact the visible page always holds the focus, so its border is the accent colour.
2. The pane headers and the finder bar are two rows with the hairline in every tier. The `compact` switch on `Header` and `Find` is removed.
3. The compact back chevron is a lone `‹`, styled and spaced as medium's (accent, two columns after it). A press on it or on the title beside it goes Back. The page name is dropped, so `‹ Files` is `‹`. Files has no lead: nothing is underneath it.
4. The bottom bar is unchanged. Its words are the taps, and `Back` names the action.

Why: the tiers differ by their width budget, not by their look, and one frame and one header make the phone read as the medium screen. The page name costs six columns the path needs at 40 to 59 columns, and the bottom bar already says `Back`.

Consequences:

- (positive) Compact and medium share one frame, one header, and one hairline; the whole title box is a back target, not a single glyph.
- (negative) Compact loses two columns and three rows (the top border, the hairline, the bottom border), so the list has five rows fewer than the screen. A long title is cut sooner: at 40 columns the live run shows `hogwarts-00-the-maraude` with `read-only` kept, and the cut has no ellipsis.
- (negative) The chevron no longer names the page Back returns to. The Keys page sits over Files, so Back goes to Files as before, but the header no longer says so.

Alternatives considered:

- Keep the one-row header and draw only the border: rejected, because the owner's screenshots asked for the header too, and the one-row header is what made compact look unlike medium.
- Keep `‹ Files` beside the chevron: rejected, because at 40 to 59 columns the name takes the columns the path needs, and the bottom bar already names Back.
- Move the bottom bar inside the frame: rejected for now (plan-00007, D4). Its words are the phone's tap targets, and the owner did not ask for a change there.

Supersedes: the border and header items of record 5, the `‹ Files` back text of record 6, and the finder-bar note of plan-00005 phase 3.

Agent decisions in plan-00007 (each reversible, with its reason):

- The compact lead and the medium chevron share one drawing, the `PaneHeader` lead (`‹` and two columns): agent decision: one lead for both tiers — why: the owner asked the tiers to match, and the lead is the part they compare.
- The lead and the title run one press handler, `app/layout.rs` `header_press`: it runs Back in compact and the chevron toggle in medium: agent decision: the tier decides the action, not the cell — why: the title box is the back target, and two handlers on one box would fight for the press.
- The list starts on `LIST_TOP` (row 3) in every tier, and `list_top` is removed: agent decision: one offset, since the border and the header are the same in both tiers — why: the row maps then need no compact case to keep right.
- A press on the compact title goes Back, where plan-00005 pinned that it did nothing: agent decision: the whole title box is the target (plan-00007, D3) — why: the chevron alone is a one-column target for a finger.
- The row tests tap the first and last visible rows of the tree, the finder, and the change list at 40 and 59 columns, and check the row that opens is the one drawn there. The diff header's fold is checked live, not in a unit test, because the diff body needs its patches from the worker — agent decision: the diff's row maps follow the laid-out rectangles, so a live tap is the proof — why: the unit harness does not run the worker.

### Approved deviation: search runs in the rendered Markdown view

Context: Scouter refuses `/` in a rendered Markdown file: `startSearch` shows "Press m to search this Markdown file's source." and returns (`scouter:src/app/app.ts:999-1005`). The code view's search matches source lines, but the rendered view shows the text without its markers (`**Harry** Potter` reads `Harry Potter`), so a source match cannot be marked where the reader draws it. Owner objective, plan-00010 (approved 2026-11-27): search in the rendered view, with the matches marked in place and counted in the footer.

Decision: `/` opens the prompt in the rendered view, `n`, `N`, and `p` step, Esc keeps then clears, as in the code view, and the footer reads `/query  k of N` (the code view's text; plan-00010 D1 said `3/12`, refined here). The search reads the text the view draws: each paragraph, heading, and table cell, and each code block's text. A match is an occurrence, not a line, counted in document order, ignoring case (no smart case: the code view ignores case always, `reader/search.rs`). Every match is marked in the match colours; the current one takes `target-bg` (D3). A fence in a highlighted language is marked through a code-block renderer that keeps the default look when no mark is set (plan-00010 phase 1: no differing cell). A fence in another language, an HTML block, and an indented block are counted and reached but not marked (D5). A mermaid diagram has no searchable text. A jump scrolls the block of the current match into view, and inside a block taller than the view to an estimated row: the match's byte offset over the block's text, times the block's rows (D4), because OpenTUI exposes no byte's row. Esc clears the search and its marks, and so does `m`, which switches the view (D8). A shown text that changes keeps the query, with its hits counted again (the code view's rule, D9).

Consequences:

- (positive) A search finds what the reader shows, and the marks sit on the text the reader draws; the key page and any Markdown file are searchable.
- (positive) The hook and renderers need no per-block identity: they count their calls in document order, and an offscreen index of each block's text names the block of every count (`ui/markdown_search.rs`).
- (negative) A search step rebuilds the document: about 3 ms in release on DESIGN.md (the measurement of plan-00010 phase 1); a 1 MiB document is not measured.
- (negative) An estimated row can land a few rows off inside a tall block, such as a long list.
- (negative) A block that is not drawn (a plain fence, HTML) is counted without a mark.

Alternatives considered:

- Search the source and map matches to blocks — rejected: marker and entity text differs from what is drawn, so the marks cannot be placed by source offsets.
- Keep Scouter's refusal — rejected by the owner's objective.
- Smart case, as in the usual editors — rejected: it would make the rendered view disagree with the code view on the same query.
- Exact rows inside a block — rejected: OpenTUI 0.9.9 exposes a block's rows, not a byte's row (`opentui-0.9.9/src/widget/markdown.rs:758`).

### Approved deviation: the rendered view shows git change signs beside its blocks

Context: plan-00008 draws the git signs in the text view's strip and keeps them out of the rendered view (its D3). Scouter's reader has no git signs (`scouter:src/reader/code-view.ts:99-108`). plan-00010 asks for the signs beside the rendered blocks, so a changed paragraph reads as changed where the reader draws it (owner, 2026-11-27).

Decision: each top-level block takes the signs of its source lines (the split of `ui/markdown_blocks.rs`, marked by `reader/rendered_signs.rs`). A block with an added or changed line gets a bar in its strongest colour (`modified` over `added`) for its rows, in a column over the viewer's left edge, where the rendered text's padding is. A deletion is a `▁` in `removed` on the row under the block above it, and a deletion above line 1 is a `▔` over the first block. A sign on a blank line between blocks belongs to no block, so an added blank line draws nothing. The column is absolutely positioned, so the text does not move, and each mark is placed at its block's document row less the scroll at each paint. The signs are plan-00008's (the same base, the same refresh), and `git_gutter` turns them off (plan-00010 D7).

Consequences:

- (positive) A changed paragraph, heading, or list is marked where the reader draws it, and a deletion sits under the block that lost its line.
- (negative) A bar covers the whole block, not the changed line; a deletion inside a block is drawn under the block.
- (negative) A blank line added between blocks is not marked.
- (negative) The column overlays the viewer's first column; a pointer on it does not reach the scroll box (not measured).

Alternatives considered:

- A per-line gutter beside the viewer, as the code view has — rejected: a gutter slot takes its width from the text, which would move the rendered text. The padding is the column the owner asked for (plan-00010 D6).
- Per-line signs inside a block — rejected: OpenTUI draws a block's rows, not its source lines (`opentui-0.9.9/src/widget/markdown.rs:758`), so the block is the unit.

### Approved deviation: a Mermaid fence in rendered Markdown is drawn as a diagram

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter has no Mermaid. A ```` ```mermaid ```` fence in rendered Markdown shows as an ordinary code block with its body as plain text (`research/scouter-inventory.md` §7; no Mermaid code in the repo, `bun.lock`, or `@opentui/core`). The OpenTUI binding has a code-block renderer for a fence language (`opentui-0.9.9/src/widget/markdown.rs:631`), so a diagram is a renderer, not a change to the widget. The roadmap names the feature (RI-1, `roadmap-00001-post-parity.md`).

Decision: the owner approved (2026-11-27, `plan-00006`, O1 and O4) that a `mermaid` fence in the rendered view (`m`) is drawn as a box diagram at the reader's width. Its box lines and arrows take `border`, its labels `text`, and a fallback's reason `muted` (D3). The engine does not mark edge labels apart from node labels, so edge labels take `text`, not `muted`: the simplification D3 allows. The raw code view keeps the fence as highlighted text, and `.mmd` files stay plain (D1), because the renderer belongs to the rendered view.

Consequences:

- (positive) Flowcharts, sequence, class, state, and gantt diagrams read in the preview without leaving fmc.
- (negative) Every Mermaid fence in the rendered view differs from Scouter's plain text. A diagram that does not fit, or that the engine cannot draw, shows its source in a frame, with the reason (see the fallback record below). Edge labels are not muted.

The owner asked on 2026-11-27 for a background on the render area, and said to drop it if it caused problems on resize or at narrow widths. It sits on the palette's `fallback-background` (`#131519` dark, `#f6f8fa` light), over the box only: the widget fills its measured rows up to the last glyph of the widest row, so the engine's trailing padding is not filled. Amended the same day, again at the owner's request (a screenshot of two diagrams on box-wide fills): the fill now spans the reader's text width, and the widget measures the whole bounded width it is given, so diagrams read as full-width blocks; the reader's two padding cells stay clear. It does not move on a resize (`ui/mermaid.rs` `paint` and `measure`; `app/mermaid_tests.rs`, `a_diagram_sits_on_the_panel_tone_at_each_width_and_follows_a_resize`). Checked live at 120, 70, and 49 columns, and across a resize from 120 to 70, in dark: the fill followed the measured width each time, and a framed fallback at 49 columns carries it too. Rejected: `row-active-bg` (the active row's tone reads as a selection), and a new palette token (one fill would need its own `DESIGN.md` entry and theme checks).

Alternatives considered:

- Scouter's plain code block — rejected by the owner: the diagram is the feature the roadmap asked for.
- The raw code view draws the diagram too — rejected: a second renderer on a surface nobody asked for (D1).

### Decision: Mermaid is drawn by `mermaid-text` with a strict column budget

Context: two engines were spiked (`research/mermaid-spike.md`, plan-00006 phase 1). A port of Grok's `mermaid.rs` (Apache-2.0, 5,699 lines; five diagram kinds; a fallback with no reason text; no gantt) and `mermaid-text` 0.58.0 (MIT; about 18 diagram kinds; typed errors; a strict width budget). A fallback needs a reason for each fence, and a diagram wider than the reader must become a fallback, not a clipped drawing (D2).

Decision: `mermaid-text` 0.58.0 with `RenderOptions::max_width_strict` (`src/reader/mermaid.rs`). Its error variants give the reason text, and the strict budget turns "too wide" into an error (`mermaid-text` `lib.rs:180-185`, `539-560`). The owner approved it on 2026-11-27 (O4), with the MSRV raised from 1.90 to 1.92 and the crates in `rules.md` ("Approved for plan-00006 phase 2").

Consequences:

- (positive) One dependency covers the gantt sample and 18 diagram kinds, and the reason comes from the engine's own error text.
- (negative) MSRV 1.92. The lockfile gains packages for other targets, listed in `rules.md`. The engine's quirks are accepted by the owner: class members can be reordered (`+String passcode` is drawn `+passcode String`), and some malformed lines draw without an error (an unclosed bracket in a flowchart still draws its text in a box).
- (negative) The engine is not idempotent at its own width: the gantt sample is 96 columns when drawn within 100, and rendering at 96 falls back. The widget's paint keeps the measured rows for this reason (`rust.md`, "A Mermaid diagram learns its width from layout").

Alternatives considered:

- The Grok port — rejected: a 5,700-line module to keep, five diagram kinds, and no reason text for the fallback.
- `mermaid-rs-renderer` — rejected: it rasterises SVG, and fmc is a text terminal app (RI-1).
- `mermaid-text` with a soft width hint — rejected: a diagram can be wider than the reader, and nothing would catch it.

### Decision: a Mermaid diagram is measured by layout, so a resize measures it again

Context: the renderer runs when a block is built or changes, and when streaming ends (`opentui-0.9.9/src/widget/markdown.rs:928-931`, `1013`, `1359-1375`). `CodeBlock` carries no width (`markdown.rs:157-175`), and a window resize does not rebuild the document, so a renderer that reads the width when it runs keeps the old width. Rebuilding the document on each resize would also reset its scroll.

Decision: the renderer returns a measured `Custom` widget (`Custom::measured`, `opentui-0.9.9/src/widget/custom.rs:155-171`). Layout offers it the reader's width through `Component::measure`, and a resize measures it again. Proof: the TestApp test `a_resize_measures_the_diagram_again` (`app/mermaid_tests.rs`), and a live run, where a 100 to 60 column resize with the document open changed the frame's reason from `the reader has 65` to `the reader has 53` (2026-11-27). Chosen over re-showing the document on resize (`MarkdownView::show`), which rebuilds it.

Consequences:

- (positive) The reader's width reaches the diagram with no resize hook and no shared width cell, and the document keeps its scroll position.
- (negative) `measure` runs inside native layout, so it must not change other widgets. A render happens once per width: about 0.6 ms for the largest sample at 60 columns (spike §7).

Alternatives considered:

- Re-show the document from the resize path — rejected: it rebuilds the whole document and resets its scroll.
- A shared width cell read when the renderer runs — rejected: the renderer does not run on a resize, so an open document would keep the old width.

### Decision: a diagram wider than the reader falls back to its source in a frame

Context: rendered Markdown wraps and has no horizontal pan, so a diagram wider than the reader cannot scroll sideways. Grok's Mermaid falls back to the source in the same way (`xai-grok-markdown/src/mermaid.rs`, the spike's source). Plan-00006 D2 records the rule.

Decision: a diagram that is too wide for the reader (the strict budget) or that the engine cannot draw shows its source in a frame (`│`, `─`), with the source lines wrapped inside it and one reason row in `muted`: `too wide: <N> columns, the reader has <M>`, `unsupported diagram: <keyword>`, or `parse error: <message>`. The reason is cut to the width with `…`. Under five columns there is no room for the frame (`reader/mermaid.rs` `framed`). An empty fence is an ordinary code block. Owner approved (2026-11-27, D2).

Consequences:

- (positive) The source stays readable at every width, and the reason says why the diagram is not drawn. A phone shows a frame, not a broken drawing.
- (negative) A wide flowchart is a frame at phone widths. Its reason gives the width the diagram needs; `m` shows the source as plain text, without the frame.

Alternatives considered:

- Clip the diagram or pan it sideways — rejected: rendered Markdown has no horizontal pan, and a clipped diagram loses its arrows.
- Scale the diagram down to fit — rejected: the engine already compacts within its budget, and a diagram that still does not fit is a fallback.

### Approved deviation: a change-row click keeps the focus on the list

Context: in Scouter, a click on a change row selects it and focuses the diff body (`ChangeList` `onMouseDown` → `openSelected`, Scouter v0.3.4 `434d000f`). The owner asked, on 2026-11-27, for the changes list to behave like the file tree's sidebar: a click there selects and previews but leaves the focus on the list ("Auto-preview in the sidebar").

Decision: in the medium and wide tiers, a click on a change row selects it and shows its file in the body (the one-file body reads it, and the stacked body jumps to it), and the focus stays on the list. Enter, `l`, and `→` still focus the body. In compact the list and the body are separate pages, so a tap opens the body, as Enter does and as a tap on a file in the tree does (`src/app/changes.rs` `click_row`).

Consequences:

- (positive) Clicking through the list keeps `j`/`k` on the list, and the two sidebars act the same.
- (negative) Reaching the body takes Enter, Tab, or a click in the body.

Alternatives considered:

- Keep Scouter's focus move — rejected by the owner.

### Approved deviation: rendered Markdown headings keep their `#` marker

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's preview is a `MarkdownRenderable` with no `conceal` option (`scouter:src/reader/code-view.ts:132-145`), so the widget's default hides every Markdown marker, a heading's `#` run included. A terminal cannot enlarge a heading, so the level reads from colour and weight alone, and `#`, `##`, and `###` look alike (`research/scouter-inventory.md` §7, `index.bun.js:10673`).

Decision (owner, 2026-11-27, `plan-00011`): in the rendered view (`m`), each heading keeps its ATX marker (`#` to `######`, with its space), drawn in the heading's own style. The colours of the heading and the other markers stay as they are.

- D1: only headings keep a marker. Emphasis, code, and link markers stay concealed.
- D2: the marker is the ATX run for the heading's level plus one space. A setext heading (`Title` over `===` or `---`) shows the marker of its level, and its underline row is no longer drawn, so every heading reads the same way.
- D3: the marker is written into the source before the widget reads it (the next record).
- D4: copying from the rendered view includes the marker, because the marker is on screen.
- D5: the source view (`m` again) is unchanged.

Consequences:

- (positive) The heading levels read in a terminal without a larger font.
- (negative) Every heading row is wider by its marker. The setext underline row is gone, where the engine draws it as text today. The `?` key page is rendered Markdown, so its headings show markers too. A copy from a heading includes its marker.

Alternatives considered:

- `conceal(false)` on the widget — rejected: it shows every marker, `**`, backticks, and link brackets included, which D1 forbids.
- Markers only through the inline hook — rejected: the hook cannot tell the levels apart (the next record).

### Decision: heading markers are a source rewrite, not an inline hook

Context: the plan asked for a check first. The inline hook (`opentui-0.9.9/src/widget/markdown.rs:213-223`, `InlineContext`) receives the block's inline text as `source`, which has no `#` run: `# Hogwarts` gives `source="Hogwarts"` (a TestApp probe, 2026-11-27, through `MarkdownView` with the dark palette). The span style does not give the level either: the app's theme sets `markup.heading.1` to `.6` to the same bold function style (`src/ui/syntax/mod.rs:97-99`), and the probe shows the same foreground and attributes for levels 1 to 6. So the hook cannot tell `##` from `###`.

Decision: a pure source rewrite in the core crate (`src/reader/headings.rs`, `with_markers`), called from `MarkdownView::show` before `set_markdown`. Each ATX heading keeps its real run, which the engine conceals, and the same run is written again as HTML entities (`&#35;`) at the start of its text. A setext heading takes the same treatment, and its underline line becomes empty. Fenced code, indented code, and HTML blocks are left as they are. Every source line stays a line, so the line mapping that plan-00010 relies on holds. The entities matter: a backslash escape (`\#`) takes a different colour (`#a7c7a1` in the dark palette), while `&#35;` takes the heading's colour (the TestApp test `each_heading_level_shows_its_marker_in_the_heading_colour`).

Consequences:

- (positive) No engine change and no new dependency. The rewrite is pure and unit-tested (`headings_tests.rs`, 21 cases), and the TestApp test checks the colour on screen.
- (negative) The rewrite is a line scanner that follows CommonMark's block rules only as far as it needs. Known limits, all leaving the heading unmarked rather than marking code: a multi-line setext heading, a setext heading inside a list or a quote, and a line that only opens an HTML tag (start condition 7).

Alternatives considered:

- The inline hook with the level from `source` or the style — rejected: neither carries the level (above).
- Parsing with `tree-sitter-md` — rejected: it needs the `tree-sitter` crate in the core, which `rules.md` does not approve.
- Backslash escapes (`\#\#`) — rejected: they take the escape colour, so the marker would not be in the heading's style.

### Approved deviation: `file_associations` in `settings.json` picks a file's filetype

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's `languageFor` reads only the extension, through OpenTUI's `pathToFiletype` (`scouter:src/reader/code-view.ts:31-34`, with `supportedLanguages` at `:20-29`), and its Markdown view is gated on the same result (`isMarkdown`, `:230-232`). A file such as `vault.rs.bak` or `louvre.env.example` is shown plain, and OpenTUI has no `dotenv` filetype (`plan-00009-file-associations.md`, the owner's request of 2026-11-27: "a setting like VS Code").

Decision (owner, 2026-11-27, with the go-ahead of plan-00009):

- D1. The key `file_associations` is an object of pattern to filetype name, shaped as VS Code's `files.associations`.
- D2. A pattern with no `/` matches the file name; a pattern with `/` matches the path from the folder fmc opened. Syntax: `*`, `**`, `?`, and `{a,b}`. Case follows the filesystem (ignored on macOS and Windows). The matcher is hand-written (`reader/glob.rs`) rather than `globset`: a small function with tests, and no dependency to approve.
- D3. Values are fmc filetype names, plus `shellscript` (bash), `jsonc` (json), and `dotenv` and `env` (bash, which reads `KEY=value` and `#` lines). The core resolves them, so no OpenTUI alias is registered. `plaintext` shows plain text. An unknown value, or a bad pattern, is skipped with a notice that names it.
- D4. An association beats OpenTUI's lookup. Among matches, a path pattern beats a name pattern, then the longer pattern wins, then the pattern that sorts first: `serde_json` keeps no file order (no `preserve_order`), so "the last one wins" is not defined.
- D5. `plaintext` turns highlighting off even where OpenTUI would highlight the file.
- D6. No built-in association for env files: the owner's file decides.
- D7. An association to `markdown` also makes `m` render the file (`reader/markdown.rs` `is_markdown` reads the same language).

One lookup serves every filetype use: `ui/syntax/mod.rs` `language_for(associations, opened, path)`. The reader and the finder's preview pass the path. The changes screen passes the repository root joined with the change's path, since git's paths are repository-relative; a path pattern is matched from the folder fmc opened, so a file outside it matches by name only.

Consequences:

- (positive) A file is highlighted by the name the owner uses for it, in every view that shows it, and `m` follows the same rule.
- (positive) The matcher and the table are pure and unit-tested (`glob_tests.rs`, `associations_tests.rs`); the reader's behaviour has TestApp tests (`app/associations_tests.rs`).
- (negative) A setting is read once at start, so an edit needs a restart, as every key does.
- (negative) A folder opened through a symlink matches path patterns in the reader but only by name on the changes screen, because git reports the real path.

Alternatives considered:

- `globset` — rejected: a new dependency needs the owner's approval, and the matcher is about 150 lines with tests.
- "The last matching entry wins" — rejected: the parsed object has no order, so the rule would not be defined.
- A once-set global for the table — rejected: tests would share it and could not set their own associations; the model carries the table instead.
- Built-in env-file defaults — rejected by the owner's D6 choice; the file decides.

### Approved deviation: the owner's extra grammars are highlighted

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter colours its own sixteen grammars, and a file of any other filetype shows plain: `languageFor` checks `supportedLanguages` (`scouter:src/reader/code-view.ts:31-34`, the set at `:20-29`, with `extraLanguages` from its grammar manifest). The owner's Neovim set (RI-2, `roadmap-00001-post-parity.md`) names nineteen filetypes. Twelve have a crate that loads on the locked `tree-sitter` 0.27.2: diff, elixir, heex, hcl, jsdoc, lua, regex, sql, svelte, xml, astro, and vim. The other seven (json5, just, luadoc, luap, query, vimdoc, eex) have no crate that loads: six pin another `tree-sitter` (0.20 or 0.25), and eex has none published (`research/grammar-load-spike.md`, section 5).

Decision (owner, 2026-11-27, O2 of `plan-00006`: every crate that loads): fmc highlights those twelve filetypes. Each grammar crate is pinned exactly in `Cargo.toml`, and its highlight query and LICENSE sit under `src/ui/syntax/queries/<filetype>/`. HCL ships no query; its query is adapted from nvim-treesitter's (Apache-2.0, a header names it; D7). `.heex`, `.hcl`, `.tf`, and `.tfvars` map to heex and hcl before OpenTUI's lookup (`reader/language.rs`, `EXTENSIONS`), because OpenTUI's table has no entry for them. A capture a query uses is renamed to a Core-style name the palette styles (`keyword.repeat` for Lua's `repeat`, `variable.member` for SQL's `field`, and so on), or drawn plain when the palette styles none (`error`). Injections are not registered, so code inside a file is not coloured by its own grammar; a Markdown fence and `file_associations` reach these filetypes by name.

Consequences:

- (positive) A file in these languages is coloured in every view, as Scouter's are. `file_associations` and Markdown fences accept the names, and `m` renders a `markdown` association.
- (negative) A file in these languages is coloured where Scouter shows it plain. Code inside one of them, such as Svelte's `<script>` body, is not highlighted: the spike's Svelte sample gave three captures (`research/grammar-load-spike.md`, section 3).
- (negative) Twelve more C grammars. A clean debug build takes about 4 s more CPU on this machine, and the lockfile gains twelve packages with no transitive additions (`research/grammar-load-spike.md`, section 4).

Alternatives considered:

- Scouter's sixteen only: rejected by the owner (O2).
- Register the injections now: deferred. Each injection query needs its own review against the palette, and the spike measured no injected case. The follow-up is in the spike report.
- Wait for the nine missing grammars: not taken. The twelve load and compile their queries today; the seven are parked in the roadmap.

### Approved deviation: the finder can match fuzzily, from `find_mode` and `Ctrl+F`

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's finder matches a name by substring: every word of the query must appear in the lowercase path, in a fixed order (`scouter:src/find/search.ts:56-87`). RI-3 (`roadmap-00001-post-parity.md`) asked for a fuzzy option, and the owner chose how it starts and how it is switched (O3, plan-00006, 2026-11-27).

Decision (owner, O3; D4 to D6 of the plan): `find_mode` in `settings.json` is `"substring"`, the default and Scouter's match, or `"fuzzy"`. `Ctrl+F` in the finder flips it for the rest of the run and runs the open query again. Fuzzy is `neo_frizbee` 0.13.3 (D4), with the whole query as one needle, spaces included (D5). Case is ignored, typos are off (`max_typos` 0), and the rows rank by the match's score, then by Scouter's order on a tie (the shorter path, then UTF-16 code units; `find/mod.rs` `fuzzy`). The matched characters take `accent_soft` in bold, the colour the text hits use for their match (`ui/tree.rs` `marked_text`). The footer's status puts the mode first, `fuzzy, 11 matches`, so a narrow sidebar still shows it; the first live run had the count first and cut the mode. Owner confirmed the scope on 2026-11-27: a flip lasts for the run, a restart returns to `find_mode`, and fmc never writes `settings.json`. A per-run flip is the only switch that keeps `find_mode` the one saved value.

With an empty query the hint carries the mode too (`fuzzy, type to find a file, Tab to search inside files`): a Ctrl+F on an empty prompt changed the match and nothing drawn, which the owner read as "Ctrl+F does not work" (2026-11-27). Raw bytes were not the cause: `0x06` and `\e[102;5u` both reach the toggle (`app/find_tests.rs`). A value that is not `"substring"` or `"fuzzy"` keeps substring and shows a notice (D6). Text search, under `Tab`, is unchanged.

Consequences:

- (positive) Scouter's match and order stay the default, so nothing changes for a user who does not opt in.
- (negative) With fuzzy on, the rows and their order differ from Scouter's. Recorded in `docs/parity.md` (Finder).
- (negative) A sidebar narrower than the status cuts the count (at 26 columns, `fuzzy, 11 matc`). The mode leads, so it survives; the count is the part that is cut.
- (negative) The matcher is a dependency with its own `unsafe` SIMD code. The owner approved it as D4 named it.

Alternatives considered:

- `nucleo-matcher`: rejected; MPL-2.0 is not in the allowlist (`deny.toml`, `rules.md`).
- `fff-search`: rejected; a large tree, LMDB frecency, and a different result set (`research/dependency-survey.md`).
- `frizbee`: the same code under its old name; `neo_frizbee` is the newer release (D4).
- A setting alone, or a key alone: rejected by O3, which asks for both.

### Approved deviation: the welcome screen names fmc a file manager

Scouter baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

Context: Scouter's welcome says it is a read-only viewer: the tagline `a quiet, read-only file viewer` (`scouter:src/ui/welcome.ts:46`) and the closing line `No editing. Just reading.` (`scouter:src/ui/welcome.ts:80`). fmc now creates, renames, copies, moves, and deletes files (the record "The file-manager operations are the one write path"), so both lines are false for fmc.

Decision (owner, 2026-11-27): the tagline reads `a quiet terminal file manager`, and the closing line reads `Space opens the menu.`, which names the key that opens the menu (`Space`, plan-00012). The stacked welcome below 80 columns uses the same two parts. The rest of the welcome stays Scouter's: the wordmark, the prompt, the key card, and the layout. At 80 columns Scouter's closing line is cut after `Just readin` (`research/scouter-inventory.md`, section 18); the new line is shorter and fits whole. The `--help` banner says `a minimal terminal file manager` for the same reason.

Consequences:

- (positive) The welcome matches what fmc does, and the menu key is named where a new user looks first.
- (negative) A text capture of the welcome differs from Scouter's on these two lines; a parity diff masks them.
- (negative) At 80 columns the closing line is still cut; a shorter line would fit, and the owner has not chosen one.

Alternatives considered:

- Keep Scouter's text: rejected by the owner, since it says fmc does not write.
- `Space: file menu.`: not taken; `Space opens the menu.` also fits at 80 columns and reads as a sentence.
