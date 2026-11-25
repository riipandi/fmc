# Architecture

updated: 2026-11-26T05:49:00+07:00

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

An approved parity deviation is a record too. It names the Scouter behavior (with its `file:line` or its decision record), the fmc behavior, and the owner's approval with its date. A difference with no record is a defect.

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

Context: Scouter highlights JavaScript, TypeScript, TSX, Markdown (with inline), and Zig through OpenTUI's bundled grammars, plus JSON, CSS, HTML, Go, Rust, Python, Bash, YAML, TOML, C, and Java from `scouter:grammars/manifest.json`. The owner's wider Neovim set waits.

Decision: OpenTUI's bundled grammars plus one crate per remaining language, pinned exactly (`rules.md`, "Dependencies"), with Scouter's `highlights.scm` files loaded through `include_str!`.

Consequences:

- (positive) Every file is coloured as Scouter colours it.
- (negative) Grammar crates are native C builds; compile time grows.

Alternatives considered:

- Scouter's set plus the Neovim set — rejected for phase 1: a file Scouter shows plain would be coloured, a deviation; parked in the roadmap.

### The finder matches like Scouter: substring AND words, no fuzzy ranking

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

Context: in Scouter, opening a file inside a collapsed nested folder (finder, `o` from the changes screen, or a startup file argument) expands the grandparent but not the parent, so the file has no row and the selection falls to the root (`scouter:src/explorer/file-tree.ts:76-81`, reproduced live). The same happens for a folder picked in the finder.

Decision: fmc expands every ancestor and selects the revealed row. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The revealed file is visible and selected.
- (negative) A deliberate difference from Scouter's screen in that state.

Alternatives considered:

- Reproduce the bug — rejected by the owner.

### Approved deviation: the `?` card lists the Mouse section

Context: `--help` prints a Mouse section; the `?` card omits it (`scouter:src/ui/help.ts:55`).

Decision: the `?` card shows the Mouse section too, drawn from the same key table. Approved by the owner on 2026-11-26.

Consequences:

- (positive) `?` and `--help` agree.
- (negative) The card is taller than Scouter's; at small heights it scrolls or clips the same way Scouter's card does with its other sections.

Alternatives considered:

- Reproduce the omission — rejected by the owner.

### Approved deviation: the product name is fmc wherever Scouter says scouter

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

Context: Scouter quits on `q` or `Ctrl+c`, also from inside the help overlay (`scouter:src/core/keys.ts:7-9`, `src/app/app.ts:740-744`).

Decision: `Ctrl+q` (shown `^q`) and `Ctrl+c` quit; plain `q` does nothing at the global level. `--help`, the `?` card, the footer hints, the welcome hint card, and `docs/keymaps.md` say `^q`. Approved by the owner on 2026-11-26.

Consequences:

- (positive) No quit on a stray key; `q` is free for file-manager actions later.
- (negative) Scouter users' muscle memory for `q` stops working.

Alternatives considered:

- `Ctrl+q` in addition to `q` — rejected by the owner.

### Approved deviation: Enter on a file keeps focus in the sidebar

Context: in browse mode, Enter, `l`, or `→` on a file opens it and moves focus to the reader (`scouter:src/app/app.ts:425`, `finishShowing`); a binary file already keeps focus on the list (`:417`).

Decision: opening a file from the tree keeps focus on the list, so `j`/`k` keep browsing; `Tab` or a click on the reader moves focus there. The finder and the changes screen keep Scouter's behaviour. Approved by the owner on 2026-11-26.

Consequences:

- (positive) Browsing file after file needs no `Tab` back.
- (negative) Reading a long file needs one extra `Tab` before scrolling keys apply to it.

Alternatives considered:

- Enter previews, a second Enter on the same file focuses the reader — rejected by the owner.

### Approved deviation: a file refused at startup still shows its folder

Context: Scouter reads a startup file before it reveals it (`scouter:src/app/app.ts:330-335`), so a refused file (not UTF-8) leaves the sidebar empty under `Files`.

Decision: fmc lists the parent folder and selects the file, then shows the same footer and reader notice as Scouter. Approved by the owner on 2026-11-26.

Consequences:

- (positive) The user can browse away from the refused file at once.
- (negative) The startup capture differs from Scouter's in the sidebar.

Alternatives considered:

- Reproduce the empty sidebar — rejected by the owner: it reads as a defect.

## Pending decisions

- `rust-version` 1.90 was confirmed by `cargo +1.90 check` in `plan-00001` phase 3 and is recorded in `rust.md`.
