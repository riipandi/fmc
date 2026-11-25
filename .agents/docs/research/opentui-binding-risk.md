> Point-in-time research, 2026-11-26. Source: agent research for plan-00001 (Scouter at commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`). `scouter:<path>` is relative to the Scouter repository at that commit. Other scratch paths (`<scratch>/…`) were local working copies and probes that are not kept. The tree and the locked crates win over this file.

# OpenTUI official Rust binding: static-link feasibility and risk assessment

Date: 2026-11-26. Scope: crates `opentui` 0.9.9 and `opentui-sys` 0.9.9 (crates.io), upstream repo tag `v0.9.9` at `<scratch>/opentui` (`packages/rust`, `packages/native`, `packages/core`). Host: macOS arm64 (Darwin 27.2), rustc 1.100.0, zig 0.17.1 (`/opt/homebrew/bin/zig`). Nothing in the fmc repo was touched. All probe files live under `<scratch>/opentui-probe`.

## 1. Verdict

**Yes, with conditions.** A single statically linked fmc binary is achievable on:

- macOS arm64: verified. `otool -L` shows only `libSystem`, `libc++`, `libobjc`, `libiconv` and Apple frameworks. No OpenTUI or FFmpeg dylib.
- Linux x86_64 with musl: verified. The binary is a static ELF (no PT_INTERP, no PT_DYNAMIC) and runs in an Alpine amd64 container.

Not verified: Windows, Linux glibc (gnu), Linux arm64, Intel macOS. Treat them as "compiles at best" until tested.

Conditions:

1. **Use the default `static` feature.** Keep `opentui = { version = "=0.9.9", features = ["tree-sitter"] }` with default features. Turning off `static` produces a dynamic binary that does not start unless the dylib is found. Verified: `dyld: Library not loaded: @rpath/libopentui.dylib ... Reason: no LC_RPATH's found`, exit 134 (section 3.3). Dependents get no rpath from the `-sys` build script.
2. **Pin exact versions** `=0.9.9` for both crates, and fetch the native archive of the same tag: `opentui-native-v0.9.9-<platform>.zip` from `anomalyco/opentui` release `v0.9.9`. Point `OPENTUI_LIB_DIR` at its extracted directory. `opentui-sys/build.rs` checks only that the archive's `opentui.h` matches the header the bindings were generated from (`build.rs`, `generated_header_hash` and the `fnv1a64` assert). It does not verify the binary. Pin the archive SHA-256 in CI. Values observed on 2026-11-26:
   - `opentui-native-v0.9.9-darwin-arm64.zip` sha256 `c576d7ac40dfa0d9a50343c338da0936e5f89cf4c3f1132f6a830a5873c382b5`
   - `opentui-native-v0.9.9-linux-x64-musl.zip` sha256 `054458b635cce0b6275895d6afbb8e831ac9f032631a0199328248ddb596145a`
3. **Licensing decides the video question, not the build.** The release `libopentui.a` statically includes a decode-only FFmpeg 9.0.2 build under LGPL-2.1-or-later (`FFMPEG-SOURCE.md`; `opentui-sys/README.md` "Licenses"). Verified: the release static binary contains 565 `avcodec`/`avformat`/`sws_`/`swr_` symbols. Choose one:
   - (a) Ship the FFmpeg notices (`LICENSE-FFMPEG`, `FFMPEG-SOURCE.md`, the other `LICENSE-*` files) and satisfy LGPL-2.1 for a static link. That means letting users relink with a modified FFmpeg, which the upstream notes describe, and is a question for counsel.
   - (b) Build your own native library with video off: `zig build -Doptimize=ReleaseFast -Dvideo=false` in `packages/native` (documented in `opentui-sys/README.md`). Verified: the resulting binary has 0 FFmpeg symbols. You then own a native build step for every release. This variant is not the one upstream ships, so its regressions are yours to catch.
4. **Enable the `tree-sitter` feature.** Without it, `Code`, `Diff` and Markdown code blocks show plain text (`src/syntax/mod.rs`, module doc: "Without the feature, widgets show plain text"). The feature compiles Core's grammars (JavaScript, TypeScript, TSX, Markdown, Zig) and Rust into the binary via `cc`, so no grammar files are needed at runtime. Scouter's extra 11 grammars must be added as Cargo crates with `syntax::register` (see section 5).
5. **Platform scope.** Start with macOS arm64 and Linux x86_64 musl only. For the musl build, see the linker notes in section 3.2.

## 2. Build mechanics (file:line evidence)

### 2.1 `opentui-sys`

- `crates/opentui-sys/Cargo.toml`: `links = "opentui"`, `default = ["static"]`, `static = []`. License `MIT AND LGPL-2.1-or-later`.
- `crates/opentui-sys/build.rs`:
  - `TARGETS` table (lines 6-17): 10 Rust triples. Linux gnu x86_64 and arm64, Linux musl x86_64 and arm64, macOS arm64 and x86_64, Windows MSVC and GNU x86_64, MSVC and GNULLVM arm64. Each maps to an archive directory and release platform name.
  - `build.rs:52`: `static_link = CARGO_FEATURE_STATIC && (msvc || !windows)`. Static is the default. On Windows GNU, the DLL is linked either way.
  - `build.rs:59`: crt-static assert. A crt-static Unix executable cannot load the shared library, so the build fails for shared + musl + crt-static.
  - Static archive: `libopentui.a` copied into `OUT_DIR` (`build.rs:134`), then `cargo:rustc-link-lib=static=opentui` (`:136`). macOS adds the frameworks AppKit, Foundation, ImageIO, CoreFoundation, CoreAudio, AudioToolbox, VideoToolbox, CoreMedia, CoreVideo, CoreServices, QuartzCore (`:147-161`), plus `c++` and `objc` (`:162`). Linux gnu adds `m`, `dl`, `pthread` (`:168`). Musl adds nothing.
  - Dynamic path (`build.rs:174-182`): `cargo:rustc-link-lib=dylib=opentui`, and `-Wl,-rpath,<dir>` through `cargo:rustc-link-arg`, which applies only to this crate's own targets. Dependent binaries therefore get no LC_RPATH.
  - `DOCS_RS` short-circuits the build, so docs.rs needs no native library.
- Native artifacts: `opentui.h` plus `libopentui.a` (static) or `libopentui.dylib` / `libopentui.so` / `opentui.dll` (shared). The header's hash must match `src/generated.rs`.

### 2.2 `opentui`

- `crates/opentui/Cargo.toml`: `static = ["opentui-sys/static"]`. Optional features: `tokio`, `log`, and `tree-sitter` (bundles `tree-sitter` 0.27, `tree-sitter-javascript` 0.25, `-md` 0.5, `-rust` 0.24, `-typescript` 0.23, `-zig` 1.1). `rust-version = "1.88"` from the workspace. Edition 2024.
- `packages/rust/rust-toolchain.toml` pins toolchain `1.99.1` for CI. `packages/rust/Cargo.toml` sets `unsafe_op_in_unsafe_fn = "deny"`.

### 2.3 Native library (`packages/native`)

- `build.zig.zon`: `minimum_zig_version = "0.17.1"`. Local zig is 0.17.1, so it matches.
- `build.zig` lines 1011-1030 (`shared` at 1011, `static` at 1024): builds a `shared` (dynamic) and a `static` (`libopentui.a`, or `opentui-static.lib` on Windows) library from the same module. Option `-Dvideo=false` (line 619) builds without FFmpeg. `-Drequire-video` (line 618) fails when the FFmpeg prefix is missing.
- Targets (`build.zig:14-22`): `x86_64-linux-gnu.2.17`, `aarch64-linux-gnu.2.17`, musl x86_64/aarch64, `x86_64-macos.13.0`, `aarch64-macos.13.0`, Windows GNU x86_64/aarch64. Minimum macOS 13.0.
- Vendored Zig dependencies: `src/vendor/zig-deps.tar.gz` (5.2 MB), extracted by `scripts/prepare-zig-deps.sh`. No download was needed for the build I ran.
- Video: FFmpeg is built by `scripts/build-ffmpeg.ts` (bun), cached under `.cache/ffmpeg`. Without a prefix, the build warns and builds without video (build.zig lines 390-417). Not exercised.
- Native features: Yoga (C++), ghostty-vt (lazy dep), uucode, libwebp, lcms2, miniaudio (audio), wuffs, stb. There is no tree-sitter in the native library (`grep -rln tree_sitter packages/native/src` is empty). Grammars come from the Rust side.
- Static vs dynamic: `static` is built for every target. Whether a given archive can be linked into a static binary depends on the target (see musl notes).

### 2.4 Release archives (GitHub, tag `v0.9.9`, published 2026-11-23)

Native archives: `darwin-arm64` (14.4 MB), `darwin-x64` (15.3 MB), `linux-x64` (16.8 MB), `linux-arm64` (16.1 MB), `linux-x64-musl` (16.8 MB), `linux-arm64-musl` (16.1 MB), `windows-x64` (14.7 MB), `windows-arm64` (13.6 MB). Also symbol archives and examples.

Contents of `darwin-arm64` (checked with `unzip -l`): `libopentui.a` 36.3 MB, `libopentui.dylib` 6.9 MB, `opentui.h`, `FFMPEG-SOURCE.md`, `LICENSE-FFMPEG`, `LICENSE-LIBWEBP`, `LICENSE-WUFFS`, `LICENSE-LLVM`, `LICENSE-GHOSTTY`, `LICENSE-LCMS2`, `LICENSE-STB`, `LICENSE-GDTOA`, `LICENSE-ZLIB`, `LICENSE-ZIG`, `LICENSE-YOGA`, `LICENSE-MINGW-W64`, `AUTHORS-LIBWEBP`, `PATENTS-LIBWEBP`, `LICENSE`, `README.md`.

Linux musl: `libopentui.a` 57.4 MB and `libopentui.so` 8.9 MB, same license files.

## 3. Probe (run in `<scratch>/opentui-probe`)

### 3.1 Steps

1. `gh release download v0.9.9 -R anomalyco/opentui -p 'opentui-native-v0.9.9-darwin-arm64.zip'` into `dl/darwin-arm64/`. Unzipped to `native/darwin-arm64/`.
2. `cargo new --bin app` in `opentui-probe/`. Cargo.toml dependency: `opentui = { version = "=0.9.9", features = ["tree-sitter"] }`.
3. `src/main.rs`: a `Markdown` widget (heading, bold, inline code, a fenced ```` ```rust ```` block, a table, a list) and a `Code` widget (`filetype("rust")`, `line_numbers(true)`) with a 6-line Rust snippet. `App::new(AppOptions::default())`, `app.on_key` quits on `q`, `app.run()`. The API follows `examples/showcase.rs:219` and `:281`.
4. Static build: `OPENTUI_LIB_DIR=.../native/darwin-arm64 cargo build --release`. Result: finished in 14.0 s wall (fresh target dir, 426% CPU). Log: `build-static.log`.
5. `otool -L` and size checks (section 3.2).
6. Dynamic variant `app-dyn`: same code, `default-features = false`. Built in 15.4 s. Launched without a library path: crash (exit 134, dyld error above). Relaunched with `DYLD_LIBRARY_PATH` set: renders.
7. Rex: `rex skill show` for the CLI; `rex whoami --json` for the session (`session:01a2144a-042e-7635-acec-be63f815a6d2`); `REX_BLOCK` pane untouched. Each run was a new tab created in that session with `rex window new <label> -s <session> --focus=false --shell none -c <dir> --json -- <binary>`. Captured with `rex capture -b <block> --trim` (text and `--format=html`). Quit with `rex send -b <block> q`. Each tab closed itself after quit (no `--keep-open`). `rex window ls` confirmed no leftover probe tabs.
8. Zig source build: copied `packages/native` to `opentui-probe/native-src`, ran `sh scripts/prepare-zig-deps.sh`, then `zig build -Doptimize=ReleaseFast -Dvideo=false --summary all`. Wall time 3 min 46 s (344 s user), exit 0. Artifacts in `native-src/lib/aarch64-macos/`. The header is byte-identical to the release `opentui.h`.
9. Static build against the source-built library (`OPENTUI_LIB_DIR=native-src/lib/aarch64-macos`): finished in 12.2 s, same header check passes.
10. Musl build (`x86_64-unknown-linux-musl`) against `opentui-native-v0.9.9-linux-x64-musl.zip`. Setup: `zig cc`/`zig ar` as CC/AR/linker through two small wrapper scripts (`tools/musl-cc.sh` drops cc-rs's `--target=` flag; `tools/musl-ar.sh`). The first link failed with duplicate `_start` from rustc's self-contained crt objects, so the build used `RUSTFLAGS="-C link-self-contained=no"`. Result: finished in 12.5 s, static ELF.
11. Musl binary run: `docker run --platform linux/amd64 alpine:3.20`, binary mounted read-only, run under a pty with `timeout 4`. Output: `ldd` says "Not a valid dynamic program"; the app renders `Probe` and `inline code`, sends terminal capability queries, exits with rc=0.

### 3.2 Results

| Artifact | Size | Linking | `otool -L` / ELF check |
|---|---|---|---|
| `app` (static, release lib, tree-sitter on) | 14.6 MB | static `libopentui.a` | system only: libiconv, AppKit, Foundation, ImageIO, CoreFoundation, CoreAudio, AudioToolbox, VideoToolbox, CoreMedia, CoreVideo, CoreServices, QuartzCore, CoreGraphics, libc++, libobjc, libSystem |
| `app` (static, source lib, `-Dvideo=false`) | 12.3 MB | static | same list; 0 FFmpeg symbols (`nm | grep -c 'avcodec\|avformat\|sws_\|swr_'` = 0) |
| `app-dyn` (dynamic) | 9.5 MB + 6.9 MB dylib to ship | dynamic `@rpath/libopentui.dylib` | needs rpath at run time; crashes without it |
| `app` (musl, static, release lib) | 17.3 MB | static | `file`: "statically linked"; no PT_INTERP, no PT_DYNAMIC; runs in Alpine amd64 |

Note: the framework list in the macOS binary is a consequence of `build.rs` linking those frameworks unconditionally for `apple-darwin` (`build.rs:147-161`), including VideoToolbox and CoreMedia even in the no-video build. Apple system frameworks are dynamic on macOS whatever you do; "static" here means no third-party dylib.

Captures (in `<scratch>/out/probe-evidence/`):
- `static-screen.txt` / `.html`: Markdown (heading, bold, code fence, table, list) and Code with line numbers. The HTML capture has multiple distinct syntax colors (for example `rgb(199, 146, 234)` on keywords), so tree-sitter highlighting ran.
- `dyn-screen.txt`: same content; the table is narrower because the tab geometry differs (not investigated).
- `novideo-static-screen.txt` / `.html`: same content from the no-video build.

### 3.3 Findings from the probe

- The official setup works first try on macOS arm64 once `OPENTUI_LIB_DIR` points at the extracted release directory. The `-sys` header check passes.
- The release static archive carries FFmpeg; a source build with `-Dvideo=false` does not, and the app still renders identically.
- Dynamic linking is a trap for an app that expects a single file. Setting `DYLD_LIBRARY_PATH` or adding an rpath is needed.
- Musl static works, but a rustc/zig linker mismatch needs `-C link-self-contained=no` on this host. A real Linux CI with musl-gcc is untested and may not need it.

## 4. Crate and project facts (verified)

- crates.io, `opentui` versions: 0.9.6 (2026-11-20), 0.9.7 (2026-11-20), 0.9.8 (2026-11-23), 0.9.9 (2026-11-23). All MIT, `rust_version` 1.88. Total downloads 665; recent 265; one reverse dependency (crates.io API). `opentui-sys` 0.9.9: 930 downloads.
- Note: `opentui` on crates.io starts at 0.9.6. `packages/rust/README.md` says it was published under that name from the first release after 0.9.5. The prior research note said 0.9.4 to 0.9.9 exist under `opentui`; that was wrong. `opentui-sys/README.md` says 0.9.4 and 0.9.5 were published as `anomalyco-opentui-sys`.
- docs.rs: `opentui` 0.9.9 builds page shows "All builds succeeded" (rustc 1.102.0-nightly, 2026-11-22). `opentui-sys` 0.9.9 also has a successful build. docs.rs builds use `DOCS_RS` and need no native library.
- GitHub `anomalyco/opentui`: 13,921 stars, 828 forks, pushed 2026-11-25, MIT. 58 open issues in total; the one matching "rust" in the title is #1315 (testRender React act warning). Not a Rust-binding issue.
- Release cadence of the whole project: v0.9.0 (2026-11-14), v0.9.1 and v0.9.2 (11-16), v0.9.3 (11-18), v0.9.4 and v0.9.5 (11-19), v0.9.6 and v0.9.7 (11-20), v0.9.8 and v0.9.9 (11-23). Ten releases in nine days.
- Rust binding churn between tags `v0.9.6` and `v0.9.9`: 10 commits touch `packages/rust`. Examples: `rust: share one syntax theme across widgets (#2190)`, `rust: draw unhighlighted code in its text color (#2180)`, `rust, native: align scroll follow and zero-width edits with Core (#2185)`, `native: one default selection color for Core and Rust (#2198)`.
- Unsafe surface (grep count): `crates/opentui/src` 403 lines mentioning `unsafe`; `crates/opentui-sys/src` 206; 293 `ot_` declarations in `opentui-sys/src/generated.rs`.
- Panics across FFI: native callbacks are guarded. `crates/opentui/src/native/session.rs:594` wraps the writer callback in `catch_unwind`. `crates/opentui/src/native/measure.rs:73` wraps the measurer and resumes the stored payload after the native call returns (comment at `measure.rs:71`: a panic must not unwind through native frames). `crates/opentui/src/host.rs:237-238` takes and replaces the panic hook; I did not trace what the replacement does.
- Terminal restore on exit: `Drop` impls in `host.rs:1020` and `term/unix.rs:92` (not exercised by a crash test).

## 5. Parity drift from Scouter's `@opentui/core` 0.9.0

Compared `v0.9.0` and `v0.9.9` in `<scratch>/opentui` (`git diff --stat`):

- `packages/core/src/renderables/Code.ts` and `Diff.ts`: unchanged (0 files changed). These two drive Scouter's code view and diff view directly.
- `packages/core/src/renderables/Markdown.ts`: +44 / -18. Adds `taskMarkers` option (default `☐`/`☑`, same glyphs as before), changes how list items and blocks are shown again after a change (`show`/`refresh`).
- `packages/core/src/renderables/TextTable.ts`: +120 / -2. Markdown tables are built from `TextTableRenderable` (6 references in `Markdown.ts`). Adds width-method following and selection pinning.
- `packages/core/src/renderables/Textarea.ts`: +4 / -2 (not used by Scouter).
- `packages/native/src/markdown.zig`: 3-line change in inline cut handling (line 2960 area). Affects partial-token styling at a cut.
- `packages/core/src/zig.ts`: +47 / -10 (new edit-buffer options, selection colors, width-method setter).

Scouter uses `CodeRenderable` with `conceal: false`, `drawUnstyledText: true`, `wrapMode: "none"` and a `treeSitterClient` (`scouter:src/reader/code-view.ts:109-137`). It uses `MarkdownRenderable` in preview with the same client. Whether the Rust `Code`/`Markdown` widgets match those options (`drawUnstyledText` has no obvious Rust analogue) was not checked. The Rust `Code` widget does have `fg`, `line_numbers`, `selectable`, `wrap`; the Rust Markdown widget has `conceal`, `streaming`, `table_options`, `code_block_renderer`.

Recommendation: either bump Scouter to `@opentui/core` 0.9.9 first and measure the delta, or run the parity audit on the Rust port against Scouter 0.9.0 output.

## 6. Grammar supply

- The `tree-sitter` feature bundles Core's five grammars as Cargo crates and compiles them through `cc` at build time.
- Scouter's 11 extra languages (`scouter:grammars/manifest.json`, e.g. `tree-sitter-json` 0.24.8, `tree-sitter-css` 0.25.0, `tree-sitter-html` 0.23.2, `tree-sitter-go` 0.25.0, `tree-sitter-rust` 0.24.x) are shipped as WASM plus `highlights.scm` with pinned SHA-256s. The Rust route uses the same package names as crates, plus `syntax::register(filetype, Grammar::new(LANGUAGE, HIGHLIGHTS_QUERY))` (`src/syntax/highlight.rs:51,60,260`).
- Not verified: that the crate versions match Scouter's pinned WASM byte-for-byte, and that query captures produce the same colors. Capture names follow each grammar's own query (see `syntax/mod.rs` module docs).

## 7. Risks (severity: H/M/L; probability of being hit)

| # | Risk | Sev | Evidence | Mitigation |
|---|---|---|---|---|
| R1 | LGPL-2.1 FFmpeg is statically linked into the default binary; distribution obligations apply (relink, notices, source offer) | H if closed-source distribution | section 2.4; `nm app-static-release-bin \| grep -c 'avcodec\|avformat\|sws_\|swr_'` = 565 | Build without video (verified: 0 FFmpeg symbols) and own that build; or ship notices and satisfy LGPL relinking. Get counsel's view on the choice. |
| R2 | API and behavior churn: 4 `opentui` releases between 2026-11-20 and 11-23 (0.9.6 to 0.9.9), 10 commits to `packages/rust` in that range, very low downstream use (665 downloads, 1 reverse dep) | H | sections 4 and 5 | Exact pin; vendor the crate source in-tree or in a fork; keep the probe app as a smoke test; budget an upgrade step per release. |
| R3 | Version skew against Scouter 0.9.0: Markdown tables (TextTable), Markdown task markers, Textarea changes | M | section 5 | Bump Scouter first or diff frames against Scouter 0.9.0 before claiming parity. |
| R4 | Binary-only native supply chain: `build.rs` checks the header, not the library's provenance; 14 to 17 MB zips per platform fetched from GitHub | M | build.rs assert list; section 2.4 | Pin archive SHA-256 in CI; or build the native library from source with zig 0.17.1 (verified on this Mac, 3 min 46 s, vendored deps). |
| R5 | FFI unsafe surface (about 600 lines mentioning `unsafe` across the two crates); panic path depends on `catch_unwind` at native callbacks | M | section 4 | Keep `panic = "unwind"`; run the crate's own tests (`sh test.sh` in `packages/rust`) in CI; review FFI changes on every bump. |
| R6 | Platform coverage: Windows static path uses a MinGW-built archive and extra system libs; only compile-checked upstream (`scripts/check-targets.sh`) | H for Windows, L for macOS | `build.rs` msvc branch; `scripts/check-targets.sh` header comment | Limit to macOS arm64 and Linux musl until a Windows CI job links and runs. |
| R7 | Linux glibc static is not supported by this setup; musl is the only static Linux path (needs a linker setup on non-Linux hosts) | M | `build.rs` crt-static assert; my musl linker workaround | Build musl on a Linux CI runner with musl-tools; verify with `file` and a container run. |
| R8 | Dynamic-linking trap: dependent binaries get no rpath | L (only if someone turns `static` off) | probe `app-dyn` crash, exit 134 | Keep default features. |
| R9 | Tree-sitter grammars compiled in: parity of highlight queries and colors vs Scouter's WASM grammars is unverified | M | section 6 | Compare the same files in both apps; copy `highlights.scm` from Scouter's grammar set where they differ. |
| R10 | Video and audio native code linked but unused (miniaudio, CoreAudio, VideoToolbox frameworks) | L | `build.rs` framework list; `build.zig` audio deps | Accept, or drop with a custom `-Dvideo=false` build (audio stays). |
| R11 | Declared MSRV 1.88; CI pins 1.99.1 | L | `Cargo.toml` `rust-version`; `packages/rust/rust-toolchain.toml` | Test with 1.88 before MSRV is promised. Not done here. |
| R12 | Binary size: 14.6 MB static release, 12.3 MB no-video; 36 MB `.a` on disk | L | section 3.2 | Accept, or strip. |
| R13 | Build-time C toolchain: tree-sitter grammars need a C compiler per target | L | `cc` build of `tree-sitter` crates in probe | Standard CI images have one; musl needs musl-gcc or zig cc. |
| R14 | Visual parity with Scouter not measured; the probe checks only that content renders | M | section 3 | Run the parity-audit against Scouter with the same content. |

## 8. Recommendation

1. Proceed with a spike on macOS arm64 and Linux x86_64 musl only, using `=0.9.9` pins, default `static`, and `tree-sitter` on.
2. Decide the FFmpeg question before the spike becomes a product. If fmc is closed-source or distributed as a binary, prefer a custom native build with `-Dvideo=false`, and record that build in the repo as a reproducible step (zig 0.17.1, `scripts/prepare-zig-deps.sh`, `zig build -Doptimize=ReleaseFast -Dvideo=false`). If video is never needed, this removes the LGPL obligation entirely.
3. Pin archive hashes in CI, and gate each bump on the probe app plus fmc's own tests.
4. Compare fmc's Markdown, Code and Diff output with Scouter (`parity-audit`) before calling it parity. Expect TextTable and Markdown table differences.
5. Defer Windows.

## 9. Not checked

- Windows (MSVC and GNU), Linux glibc, Linux arm64, Intel macOS: not built or run.
- The Rust binding under MSRV 1.88 (declared, not tested).
- Musl build on a real Linux host with musl-gcc (I used zig cc plus `link-self-contained=no`).
- Scouter's grammar WASM vs crate parity; visual diff against Scouter.
- Terminal restore after a panic (only read the code).
- `opentui-ssh` crate (not part of the requested scope).
