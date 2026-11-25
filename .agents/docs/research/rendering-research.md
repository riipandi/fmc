> Point-in-time research, 2026-11-26. Source: agent research for plan-00001 (Scouter at commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`). `scouter:<path>` is relative to the Scouter repository at that commit. Other scratch paths (`<scratch>/…`) were local working copies and probes that are not kept. The tree and the locked crates win over this file.

# Rendering research: Markdown, Mermaid, code highlighting, diffs in a Rust OpenTUI app

Date: 2026-11-26. Scope: research only. Nothing was compiled or run; claims come from source reading and crates.io / GitHub metadata snapshots taken on that date.

Sources and abbreviations:
- `CRATE` = `~/.cargo/registry/src/<index>/opentui_rust-0.2.3` (locked 0.2.3). Its `src/` is byte-identical to the repo clone `<scratch>/opentui_rust` (diff -rq clean); `Cargo.toml.orig` matches the repo `Cargo.toml`. So the repo HEAD and the locked source agree.
- `UP` = `<scratch>/opentui` (upstream TypeScript OpenTUI, `@opentui/*` 0.9.9 in-tree; `packages/rust` is the official Rust binding, workspace version 0.9.9).
- `SC` = `scouter:` (uses `@opentui/core` 0.9.0 per `package.json:18` and the installed `node_modules/@opentui/core/package.json`).

## 0. Headline findings

1. **opentui_rust 0.2.3 is a low-level renderer, not a UI toolkit.** It has buffers, cells, ANSI output, a diffing renderer, text rope/views/editor, a key/mouse parser, and hand-written highlighters for six languages. It has no event loop, widgets, layout, scrolling, images, Markdown, tree-sitter, diff view, or Mermaid. Its `FEATURE_PARITY.md` "100%" refers to the Zig core spec (`FEATURE_PARITY.md:8`, "verified against EXISTING_OPENTUI_STRUCTURE.md"), not to the TypeScript component layer that Scouter uses.
2. **Upstream OpenTUI now has an official Rust binding that the brief did not list.** Crate `opentui` 0.9.9 (plus `opentui-sys` 0.9.9) was published to crates.io on 2026-11-23 (versions 0.9.4 to 0.9.9 exist; `anomalyco-opentui` was the interim name for 0.9.4 and 0.9.5). It drives the same native Zig engine as the TypeScript package, through Yoga layout, and ships App/event loop, ScrollBox, Input, Textarea, Select, TabSelect, Slider, Table, Code, LineNumbers, Diff, Markdown, TerminalView, ImageView, VideoView (`packages/rust/crates/opentui/src/lib.rs:1-19`). This is the closest route to visual parity.
3. **Scouter does not use Mermaid.** A case-insensitive search for "mermaid" across `SC` (source, docs, tests, package files, excluding node_modules) returns nothing. Mermaid is therefore not a parity requirement. Upstream TS also has no built-in Mermaid renderer; it exposes a per-fence hook (`createMarkdownCodeBlockRenderer`, `UP/packages/core/src/renderables/Markdown.ts:232`), and the Rust binding exposes the same hook (`widget/markdown.rs:631`).
4. **Version skew.** Scouter is on `@opentui/core` 0.9.0. The Rust crates start at 0.9.4 and the in-tree upstream is 0.9.9. Pin `=0.9.9` (matching upstream) and budget for small API/behavior drift, or bump Scouter first.
5. **Scouter's own plumbing is small.** It shells out to `git` (`SC/src/diff/git.ts:54`, `Bun.spawn(["git", "-C", ...])`), polls for diff changes instead of watching files (`SC/src/diff/diff-screen.ts:214-222`, `pollMs`), and matches files by lowercase substring with AND-ed words (`SC/src/find/search.ts:56-87`, no fuzzy library). None of these needs a Rust crate beyond what the standard library and `std::process` provide.

## 1. What opentui_rust 0.2.3 provides (verified in locked source)

Dependencies (`CRATE/Cargo.toml:268-286`): bitflags, libc, ropey, unicode-bidi, unicode-normalization, unicode-segmentation, unicode-width. No layout, async, or parser crates. `rust-version = "1.85"` (`Cargo.toml:14`).

Present:
- Renderer: `Renderer::new` enters alt screen, hides cursor, enables mouse, and optionally queries capabilities (`src/renderer/mod.rs:233-250`); double buffer and diff (`present`, `:499`); threaded renderer (`src/renderer/threaded.rs`); debug overlay (`:315`); layers (`render_to_layer`, `:430`); hit grid (`register_hit_area`, `:673`; `src/renderer/hitgrid.rs`).
- Terminal: raw mode with RAII guard (`src/terminal/raw.rs:69-73`), `terminal_size` via TIOCGWINSZ (`raw.rs:88`), cursor styles, sync output, title, OSC 12 cursor color (`src/terminal/mod.rs:153-310`).
- Capability detection: DA1 gives sixel and kitty-graphics flags (`src/terminal/mod.rs:110-119`; `src/terminal/capabilities.rs:42,52`); XTVERSION and pixel/kitty queries are sent (`src/terminal/queries.rs`).
- Input: `Event` enum with Key, Mouse, Resize, FocusGained/Lost, Paste (`src/input/event.rs:8-21`); ANSI parser (`src/input/parser.rs`), SGR and X10 mouse, bracketed paste, focus. Resize is parsed only from the CSI `8;h;w t` report (`parser.rs:223`, `:464-476`).
- Color: f32 RGBA, Porter-Duff `blend_over` (`src/color.rs:220`); opacity stack (`src/buffer/opacity.rs`); scissor stack (`src/buffer/scissor.rs`).
- Buffer drawing: text, box styles (`src/buffer/drawing.rs`), pixel/supersample and grayscale buffers (`src/buffer/pixel.rs`).
- Unicode: `WidthMethod` WcWidth or Unicode (`src/unicode/width.rs:10-16`); grapheme segmentation; bidi module (`src/unicode/bidi.rs`).
- Hyperlinks: OSC 8 start and end with URL escaping (`src/ansi/mod.rs:399-403`; `src/ansi/output.rs:217-238`); `Style::link` (`src/style.rs:384`); `LinkPool` (`src/link.rs`).
- Text: `TextBuffer` on ropey (`src/text/buffer.rs`, `src/text/rope.rs`); `TextBufferView` with `WrapMode` None/Char/Word (`src/text/view.rs:29-37`), selection, scroll, line info; `EditBuffer` with undo/redo (`src/text/edit.rs`); `EditorView` with line-number gutter and visual cursor movement (`src/text/editor.rs`).
- Highlighting (hand-written, not tree-sitter): `Tokenizer` trait (`src/highlight/tokenizer.rs:41`), `TokenizerRegistry::with_builtins` (`:159`) registers JavaScript, TypeScript, JSON, Markdown, Python, Rust, TOML (`src/highlight/languages/*.rs`). `HighlightedBuffer` (`src/highlight/highlighted_buffer.rs:14`) and `Theme` (`src/highlight/theme.rs`).
- Threading and examples: `examples/` (11 numbered examples plus editor, hello, dashboard). `src/bin/demo_showcase.rs` (10k lines) is a demo app with its own input pump, toasts, palette, panels and animation clock (`demo_showcase.rs:834`). It is not library API.

Absent (checked by grep over `src/`, excluding the demo binary):
- Event loop: "OpenTUI doesn't include an event loop" (`README.md:250`, limitations `:657`). Caller must poll stdin and call `present()`. `src/input` has no stdin reader; `raw.rs:73` only builds a guard.
- Widgets: no Box/Text/ScrollBox/Input/Select/Tabs/Textarea/Slider/Table component types (README `:658`, "No widgets").
- Layout: no flex or Yoga (README `:659`, "You calculate positions yourself").
- Resize: no SIGWINCH handling; `ResizeEvent` is constructed only by the parser (`parser.rs:223`) and tests. README `:690` tells you to listen for SIGWINCH yourself.
- Kitty keyboard protocol: detected in queries/capabilities only; no key parsing for it (grep for kitty in `src/input` is empty).
- Images and video: no encoder for kitty graphics or sixel; README `:662` says "Text-only: No image protocols (sixel, kitty graphics) yet".
- Markdown rendering: absent. `highlight/languages/markdown.rs` is a tokenizer for Markdown source text, not a renderer.
- Code/Diff/Markdown widgets, tree-sitter, Mermaid: absent.
- Animation/easing module: absent from `src/` (only in the demo binary).
- Post-processing effects: absent (only opacity stacks).

Notes that could mislead:
- README says "Nightly Rust required" (`README.md:660`), but the manifest says `rust-version = "1.85"` and edition 2024 is stable from 1.85. Treat the README line as stale.
- `FEATURE_PARITY.md` says 78/78 features complete. That list covers Zig-core spec items (buffer, renderer, terminal, input, unicode). It does not cover the TypeScript component layer.

## 2. Upstream TypeScript OpenTUI and the official Rust binding

### Markdown
- Upstream: present. Parser is native Zig: "one streaming CommonMark 0.31.2 and GFM parser" (`UP/packages/native/src/markdown.zig:1-3`), with inline and table modules (`markdown-inline.zig`, `markdown-tables.zig`). The TS `MarkdownRenderable` (`UP/packages/core/src/renderables/Markdown.ts`) writes paragraphs and headings to text nodes and uses `CodeRenderable` for fences; code fences are highlighted via `treeSitterClient` (`Markdown.ts:18`, `:168`). Per-language fence renderers go through `createMarkdownCodeBlockRenderer` (`Markdown.ts:232`). Math extension is supported (`Markdown.ts:130`).
- Rust binding: present, same native engine. `widget/markdown.rs` uses `crate::native::{MarkdownBlock, MarkdownDoc, ...}`; `MarkdownExtension::Math` (`widget/markdown.rs:205-210`); `code_block_renderer` (`:631`); conceal, streaming, table options (`:553`, `:591`, `:609`). Design doc: `packages/rust/crates/opentui/docs/markdown.md` ("Markdown parity with Core").
- opentui_rust: absent (only the source tokenizer at `src/highlight/languages/markdown.rs`).

### Code (syntax-highlighted code view)
- Upstream: present. `CodeRenderable` (`UP/packages/core/src/renderables/Code.ts`) takes a `treeSitterClient` and `SimpleHighlight` spans, converted to styled chunks by `lib/tree-sitter-styled-text.ts` (`Code.ts:9-11`). Parsers are web-tree-sitter WASM with nvim-treesitter-style `highlights.scm` (`UP/packages/core/src/lib/tree-sitter/default-parsers.ts`). Bundled filetypes: javascript, typescript, tsx, markdown, markdown_inline, zig. Other grammars are added at runtime with `addFiletypeParser` (Scouter does this at `SC/src/reader/grammars.ts:16-25`, for 11 languages in `SC/grammars/manifest.json`).
- Rust binding: present. `Code` widget (`widget/code.rs`, `set_code`, `filetype`, `syntax_theme`, line numbers, selection, scroll). Highlighting needs the `tree-sitter` cargo feature, which bundles tree-sitter-javascript 0.25, tree-sitter-typescript 0.23, tree-sitter-md 0.5, tree-sitter-zig 1.1 and tree-sitter-rust 0.24 (`packages/rust/crates/opentui/Cargo.toml`, feature `tree-sitter`; `src/syntax/mod.rs:1-20`). Others are added with `syntax::register(name, Grammar::new(LANGUAGE, HIGHLIGHTS_QUERY))` (`src/syntax/mod.rs:1-20`). Caveat in the docs: capture names follow each grammar's own query, while Core uses nvim-treesitter's query names; `SyntaxTheme::from_theme` maps both.
- opentui_rust: partial at best. Six hand-written tokenizers, no tree-sitter, no widget that consumes them except `HighlightedBuffer` on a text buffer. Not usable for parity.

### Diff
- Upstream: present. `DiffRenderable` (`UP/packages/core/src/renderables/Diff.ts`) parses the patch with the npm `diff` package (`Diff.ts:7` import, `:202` `parsePatch(this._diff)`), then renders each side through `CodeRenderable` plus `LineNumberRenderable` (`Diff.ts:3-4`). Syntax comes from the same tree-sitter client. Scouter uses it through `DiffRenderable` (`SC/src/diff/diff-blocks.ts:3`).
- Rust binding: present. `widget/diff.rs` has its own unified-diff parser (`:114-144`, "Parses the first file of a unified diff"), `set_diff`, `view(DiffView)` unified/split, `filetype`, `line_numbers`, `gutter`, `colors`, and `code_views`.
- opentui_rust: absent. No diff parser, no diff view.
- Diff computation: Scouter does not compute diffs; it gets git's output (`SC/src/diff/git.ts:54`). Upstream expects a patch string too.

### Mermaid
- Upstream: absent. No Mermaid code in `UP/packages/core` except tests that exercise the code-block hook (`UP/packages/core/src/renderables/__tests__/Markdown.test.ts:1755-1779`, `:5710-5721`). Grep of all of `UP/packages`: no other mermaid hits outside these tests, a docs mention, and the Rust docs.
- Rust binding: absent as a renderer. Example hook for ` ```mermaid ` at `widget/markdown.rs:1755` and the docs `widget/markdown.rs:627`.
- opentui_rust: absent.
- Scouter: absent (zero references).

### Images
- Upstream: present. `Image.ts` with protocol `auto|kitty|sixel|blocks` (`UP/packages/core/src/types.ts:114`; `Image.ts:19-26` resolves the protocol and falls back from sixel to blocks when resolution is unknown). Scouter uses `ImageRenderable` (`SC/src/reader/image-view.ts:19`).
- Rust binding: present. `ImageView` with Kitty graphics, Sixel or half blocks (`widget/image.rs:29`); `VideoView` also exists (`lib.rs:14-15`), backed by a native decode-only FFmpeg build.
- opentui_rust: absent (capability flags only).

### Layout, scroll, input, select, tabs
- Upstream: yoga-based flex (`UP/packages/core/src/yoga.ts`); ScrollBox, Input, Textarea, Select, TabSelect, Slider, Table, ASCIIFont etc. (`UP/packages/core/src/renderables/`). Scouter uses ScrollBox 28 times and TextRenderable 57 times.
- Rust binding: Yoga-backed `layout.rs` (`Length`, `FlexDirection`, `Edge`, `Gutter`), `widget/scroll.rs`, `input.rs`, `select.rs`, `tabs.rs`, `textarea.rs`, `slider.rs`, `table.rs`, `text.rs`, `view.rs`.
- opentui_rust: absent.

### Event loop and app
- Upstream: `createCliRenderer` (Scouter `SC/src/index.ts:46`).
- Rust binding: `App::new(AppOptions)` and `App::run()`, handlers `on_key`, timers, animation, messages from other threads (`lib.rs:6-9`; `app/mod.rs:453`). `Host` handles setup, input, resizes, suspension and restoration (`lib.rs:39`).
- opentui_rust: absent. You write the loop (README `:250`).

### Clipboard, hyperlinks, animation
- Rust binding has `app/clipboard.rs` (OSC 52 copy and read). Scouter's own clipboard uses `Bun.spawn` (`SC/src/core/clipboard.ts:28`), so no parity gap there.
- Hyperlinks: present in opentui_rust (see section 1). The Rust binding's hyperlink API was not inspected.
- Animation: Rust binding has `animation.rs` (timelines, easing, loops; `lib.rs:9`). opentui_rust has none outside the demo.

## 3. Capability matrix

"Scouter uses" is from `SC/src` imports. Upstream = `@opentui/core` in `UP`, 0.9.9. Official Rust = crate `opentui` 0.9.9. opentui_rust = 0.2.3 locked source.

| Capability | Scouter uses | Upstream TS 0.9.9 | opentui_rust 0.2.3 | Official Rust `opentui` 0.9.9 | Rust crate fallback (if you stay on opentui_rust) |
|---|---|---|---|---|---|
| Double-buffered diff renderer, alt screen | yes | present (native Zig) | present (`renderer/mod.rs:233,499`) | present (native) | none needed |
| Event loop / app runtime | yes | present (`createCliRenderer`) | absent (README:250) | present (`app/mod.rs:453`, `App::run`) | crossterm / tokio |
| Keyboard, mouse, paste, focus | yes | present | present, no kitty keyboard parse (`input/parser.rs`) | present (`input.rs`) | crossterm |
| Resize | yes | present | partial: CSI-t parse only (`parser.rs:223`), no SIGWINCH | present (host handles resizes, `lib.rs:39`) | signal-hook / crossterm |
| Flex layout | yes | present (yoga, `yoga.ts`) | absent (README:659) | present (Yoga, `layout.rs`) | taffy (via tuika, or Eternal130 port) |
| ScrollBox | yes (28 uses) | present | absent | present (`widget/scroll.rs`) | ratatui scroll widgets (different look) |
| Text / StyledText / Box | yes | present | partial: TextBuffer/View only (`text/`) | present (`widget/text.rs`, `view.rs`) | none |
| Input, Textarea, Select, Tabs | no (not imported) | present | partial: EditBuffer/EditorView (`text/edit.rs`, `editor.rs`), no widgets | present | tui-input / edtui (no parity) |
| Syntax highlighting (tree-sitter) | yes (Code, Markdown, Diff) | present (web-tree-sitter WASM) | absent (6 hand-written tokenizers, `highlight/languages/`) | present (`tree-sitter` feature, `syntax/mod.rs`) | tree-sitter-highlight 0.27 + grammars |
| Code view with line numbers | yes | present (`Code.ts`, `LineNumberRenderable.ts`) | partial: EditorView gutter only | present (`widget/code.rs`, `line_numbers.rs`) | build yourself |
| Markdown rendering (CommonMark + GFM) | yes (`MarkdownRenderable`, `code-view.ts:132`) | present (native Zig, `markdown.zig`) | absent | present (native, `widget/markdown.rs`) | pulldown-cmark 0.13 or comrak 1.2 + own renderer (large) |
| Markdown code-block hook | yes (indirect) | present (`createMarkdownCodeBlockRenderer`) | absent | present (`code_block_renderer`, `markdown.rs:631`) | n/a |
| Diff view (unified/split, line numbers) | yes (`diff-blocks.ts:3`) | present (`Diff.ts`, npm diff parsePatch) | absent | present (`widget/diff.rs`, own parser) | similar 3.2 / imara-diff 0.2 only to compute diffs |
| Git invocation | yes (`diff/git.ts:54`) | n/a (app code) | n/a | n/a | std::process::Command |
| Mermaid | no | absent (hook only) | absent | absent (hook only, `markdown.rs:631`) | mermaid-text 0.58 (flowchart only) |
| Images (kitty / sixel / blocks) | yes (`image-view.ts`) | present (`Image.ts`, `types.ts:114`) | absent (flags only, `capabilities.rs:42,52`) | present (`widget/image.rs:29`) | ratatui-image 11.1 (requires ratatui) |
| Video | no | present | absent | present (`VideoView`, `lib.rs:15`) | n/a |
| Hyperlinks (OSC 8) | not verified | present | present (`ansi/mod.rs:399`, `output.rs:217`) | not verified | n/a |
| Unicode width / graphemes | yes | present | present (`unicode/width.rs:10`) | present (native) | unicode-width 0.2 |
| Alpha blending, scissor, opacity | not verified | present | present (`color.rs:220`, `buffer/scissor.rs`, `opacity.rs`) | present (native) | n/a |
| Animation / easing | not verified | present (`animation/`) | absent | present (`animation.rs`) | n/a |
| Post-processing effects | not verified | present | absent | present (`post/`) | n/a |
| Clipboard (OSC 52) | no (Scouter uses external commands) | present | absent | present (`app/clipboard.rs`) | arboard / OSC 52 by hand |
| Tests without a terminal | not verified | present | absent (demo has a PTY harness) | present (`testing.rs`, `app/headless.rs`) | n/a |

Partial-vs-absent summary for the four gaps in the brief:
- Markdown: upstream present; opentui_rust absent; official Rust present.
- Code highlighting: upstream present (tree-sitter WASM); opentui_rust absent (hand-written tokenizers, six languages, no widget); official Rust present (tree-sitter feature).
- Diff: upstream present (npm diff plus DiffRenderable); opentui_rust absent; official Rust present (own parser).
- Mermaid: absent everywhere, and not used by Scouter.

## 4. Rust crate options per gap (crates.io snapshot 2026-11-26)

| Gap | Crate | Latest | Last updated | Notes | Fit |
|---|---|---|---|---|---|
| Markdown parsing | pulldown-cmark | 0.13.6 | 2026-10-30 | CommonMark, GFM tables and task lists; very widely used | Fit only if building your own renderer |
| | comrak | 1.2.0 | 2026-11-23 | CommonMark 0.31.2 and GFM, spec badges 652/652 and 670/670 | Same as above, plus richer AST |
| | markdown-rs | 1.0.0 | 2025-04-23 | 100% CommonMark, GFM, math, mdast AST; no release in ~19 months | Less active |
| Syntax highlighting | tree-sitter | 0.27.2 | 2026-11-07 | Parser runtime | Matches upstream (tree-sitter) |
| | tree-sitter-highlight | 0.27.2 | 2026-11-07 | Highlight query runner | Used by the Rust binding path |
| | tree-sitter-{javascript 0.25.0, typescript 0.23.2, md 0.5.4, zig 1.1.2, rust 0.24.2} | | | Bundled by official `tree-sitter` feature | Exact set Core uses |
| | tree-sitter-{go 0.25.0, json 0.24.8, css 0.25.0, html 0.23.2, bash 0.25.1, yaml 0.7.2, toml-ng 0.7.0, c 0.24.2, java 0.23.5, python 0.25.0} | | | Scouter's 11 extra languages; `syntax::register` | Need checking against Scouter's pinned `.wasm` grammars |
| | syntect | 5.3.0 | 2025-09-27 | Sublime/TextMate regex grammars | Different token classes and colors than upstream |
| | two-face | 0.5.2+bat-0.26.1 | 2026-08-07 | Extra syntect syntaxes and themes | Same as syntect |
| | giallo | 0.6.0 | 2026-11-04 | VS Code-compatible TextMate highlighting; 32k downloads | Young; no hook into a Code widget |
| | arborium | 2.18.2 | 2026-08-28 | Tree-sitter bundle with HTML and WASM output | Overlaps tree-sitter; extra layer |
| Diff | similar | 3.2.1 | 2026-10-24 | Text diff algorithms | Only if computing diffs (Scouter does not) |
| | imara-diff | 0.2.0 | 2025-06-14 | Fast histogram/Myers diff | Same |
| Git | git2 | 0.21.0 | 2026-10-06 | libgit2 binding, C dependency | Overkill for `git diff` |
| | gix | 0.91.0 | 2026-11-22 | Pure-Rust gitoxide; large API surface | Overkill |
| | (std::process) | | | Scouter's approach (`SC/src/diff/git.ts:54`) | Recommended |
| File watching | notify | 9.0.0 | 2026-11-21 | inotify/FSEvents/kqueue | Scouter polls, so not needed for parity |
| Fuzzy match | nucleo | 0.5.0 | 2024-04-02 | Helix's matcher; stale | Scouter does substring AND-ed words |
| | nucleo-matcher | 0.3.1 | 2024-02-20 | Same, lower level | Same |
| | frizbee | 0.13.0 | 2026-08-13 | SIMD fuzzy matcher, small community | Different ranking |
| Mermaid to terminal | mermaid-text | 0.58.0 | 2026-11-05 | Unicode box-drawing, pure Rust, MIT, Rust 1.92; flowchart/graph only | Best Rust fit if ever needed |
| | merman-ascii | 0.11.0 | 2026-11-21 | ASCII and Unicode for flowchart and sequence; MIT or Apache-2.0; Rust 1.95 | Raises MSRV |
| | mermansi | 0.1.6 | 2026-08-04 | Pure Rust terminal Mermaid renderer and CLI | Less established |
| | mermaid-rs-renderer | 0.3.1 | 2026-07-06 | Pure-Rust SVG renderer, 23 diagram types; output is SVG | Needs rasterizing (resvg) before ImageView |
| | termaid | 0.1.0 | 2026-04-01 | Flowchart, state, sequence; 53 downloads | Too new |
| Image protocols | ratatui-image | 11.1.0 | 2026-10-18 | Requires ratatui | Not usable with opentui |
| | viuer | 0.11.0 | 2025-12-09 | Simple image printing | Not usable for parity |
| Layout (if you build your own) | taffy | via tuika 0.14.0 (2026-11-24) | | Flexbox TUI framework | Different engine |

Non-Rust Mermaid-to-ASCII references, for reference only: `beautiful-mermaid` (TypeScript, `renderMermaidASCII`, 6 diagram types, ported from Go `mermaid-ascii`, pushed 2026-05-06) and `AlexanderGrooff/mermaid-ascii` (Go, 1630 stars, pushed 2026-11-17). Both need a runtime that the Rust app does not otherwise carry.

## 5. Recommendations per gap

Overall: move to the official Rust binding `opentui` = 0.9.9 (plus `opentui-sys` = 0.9.9 with the matching native library from the GitHub release `opentui-native-v0.9.9-<platform>.zip`, pointed to by `OPENTUI_LIB_DIR`; `packages/rust/crates/opentui-sys/build.rs`). It is the only option that uses the same rendering engine and the same component logic as Scouter, so parity comes from reuse rather than reimplementation. Keep opentui_rust only if the goal is a bespoke renderer, not parity.

Gap by gap:

1. **Terminal, event loop, input, resize.** Use the official `App`/`Host` (`app/mod.rs`, `host.rs`). Rejected: opentui_rust plus crossterm (you would write the loop, resize handling and widget layer yourself, and still lack parity); ratatui (different renderer, so cell-level parity is lost).

2. **Layout and widgets (ScrollBox, Input, Select, Tabs, Textarea, Text, Box).** Use the official widgets on Yoga (`layout.rs`, `widget/*.rs`). Rejected: tuika 0.14 (flexbox framework, different engine and API); ratatui widgets (different visuals).

3. **Markdown.** Use the official `Markdown` widget. It runs the same native CommonMark 0.31.2 + GFM engine as Core, and Math and code-block hooks are present (`widget/markdown.rs:205-210`, `:631`). Rejected: pulldown-cmark or comrak plus a custom renderer (rebuilds block layout and table logic and will not match Core's output); termimad (own styling, not parity).

4. **Code highlighting.** Use the official `Code` widget with the `tree-sitter` feature (`syntax/mod.rs`). Register Scouter's 11 extra languages with `syntax::register` and the matching `tree-sitter-*` crates. Check that the query files match what Scouter ships in `SC/grammars/*/highlights.scm`; capture names may differ. Rejected: syntect 5.3 plus two-face (regex grammars, token classes and colors differ from Core's tree-sitter output, so pixel parity fails); giallo 0.6 (accurate to VS Code, but no integration point in the Code widget and only 32k downloads, so it would need a custom widget); arborium (extra layer over tree-sitter for no gain here).

5. **Diff view.** Use the official `Diff` widget with `set_diff(git_output)` (`widget/diff.rs:114`, `:495`). Keep git invocation in app code, as Scouter does (`SC/src/diff/git.ts:54`). Rejected: similar or imara-diff (you do not need to compute diffs; git already does, and Scouter does not compute them either); building a renderer on top of Code (rebuilds the split and unified layouts).

6. **Git.** Shell out to `git` with `std::process::Command`, mirroring `SC/src/diff/git.ts:54` and its read-only options. Rejected: git2 0.21 (libgit2 C build, and still needs the same patch rendering); gix 0.91 (large API for a `git diff` you can run as a child process).

7. **Images.** Use `ImageView` (`widget/image.rs:29`). Rejected: ratatui-image 11.1 and viuer 0.11 (both require ratatui or are not tied to the opentui surface).

8. **Mermaid.** Not required: Scouter has no Mermaid. If it becomes a requirement, render fenced `mermaid` blocks via `Markdown::code_block_renderer("mermaid", ...)` (`widget/markdown.rs:631`). Best Rust fit for terminal output: mermaid-text 0.58 (flowchart and graph only, box drawing, no image protocol). For sequence or more types: merman-ascii 0.11 (raises MSRV to 1.95). Rejected: mermaid-rs-renderer 0.3 (SVG output needs resvg rasterization, then ImageView; heavy and only sensible if pictures are wanted); calling beautiful-mermaid or Go mermaid-ascii as a subprocess (adds a Node or Go runtime).

9. **File watching.** Keep Scouter's polling (`SC/src/diff/diff-screen.ts:214-222`) using an app timer. Rejected: notify 9.0.0 (kernel-level watching is more moving parts than Scouter needs; use it only if polling proves too slow).

10. **Fuzzy and file search.** Port Scouter's substring AND-ed word matcher (`SC/src/find/search.ts:56-87`) directly, with no crate. Rejected: nucleo 0.5 (last release 2024-04, and its ranking differs from Scouter's); frizbee 0.13 (different ranking; add later only if Scouter adds fuzzy matching).

## 6. Alternative Rust ports (brief check)

- **Eternal130/opentui-rust** (GitHub only, not on crates.io, 0 stars, pushed 2026-11-20). "API-level Rust port of OpenTUI 0.5.14", split into `opentui-core` (taffy flexbox, renderables for Box, Text, Code, Diff, Markdown, Image, Textarea, ScrollBox, LineNumber, TextTable, EmbeddedTerminal, Scrollbar), `opentui-keymap` and `opentui-solid`. Differential harness against the real npm `@opentui` 0.5.14 claims byte-identical frames on 48 scenarios. It would make parity easier than opentui_rust because the widgets exist, but: (a) it is pinned to 0.5.14, four minor versions behind Scouter's 0.9.0 and five behind the official 0.9.9; (b) it is a re-implementation, so its parity is only as good as its harness; (c) it is not published, so you would vendor it by git. Worth a look only if the official binding is unacceptable, for example if the native library distribution cannot be used.
- **spence/opentui-core-rs** (GitHub only, pushed 2026-04-10, 1 star). FFI bindings to the old sst/opentui Zig core, needs Zig 0.15.2 to build, its `src/` listing shows buffer, color, edit_buffer, editor_view, event and syntax_style modules, with no widgets, Markdown, Code or Diff (partial listing, not read in full). Not materially better than opentui_rust.

## 7. Risks and open items

- **Native library distribution.** The official crate links a prebuilt native library (`opentui-sys/build.rs`): release zip per platform, `OPENTUI_LIB_DIR` set by hand, static `libopentui.a` by default. You need a CI step to fetch the right zip, and musl or Windows targets have extra constraints (`build.rs` asserts on crt-static musl binaries).
- **Licensing.** The crate is `MIT AND LGPL-2.1-or-later` because the native library statically includes a decode-only FFmpeg build (`opentui-sys` README). If you ship binaries, ship `LICENSE-FFMPEG`, `FFMPEG-SOURCE.md` and the other `LICENSE-*` files from the release archive.
- **Version skew.** Scouter is on 0.9.0; the earliest Rust crate is 0.9.4 and the TS source in `UP` is 0.9.9. No Rust 0.9.0 exists. Expect small API and rendering differences; verify with Scouter's test suite (`SC/tests`) after porting.
- **MSRV.** Official crate needs Rust 1.88 (opentui_rust needs 1.85). merman-ascii would raise it to 1.95.
- **Tree-sitter parity.** Scouter's upstream path uses web-tree-sitter WASM plus nvim-treesitter-style queries. The Rust path uses native grammar crates with their own queries unless you copy the `.scm` files. Token classes and colors can differ slightly; compare the same file in both apps before signing off.
- **opentui_rust gaps that would block parity** if you stay on it: no event loop, widgets, layout, scroll, images, Markdown, tree-sitter, diff or resize signal (SIGWINCH). Building these means rebuilding most of the official binding.
- **Not verified.** I did not compile or run either crate, did not inspect the official hyperlink, animation, post-processing or clipboard APIs in depth, did not diff `@opentui/core` 0.9.0 against 0.9.9, and did not check that Scouter's 11 grammar `.wasm` files match the tree-sitter-* crate versions listed above. crates.io data is a snapshot from 2026-11-26.
- **Scope note.** The brief asked for opentui_rust 0.2.3 as the target. I treated it as the baseline to compare against, which the findings above show it cannot meet on its own.
