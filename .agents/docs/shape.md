# Shape

updated: 2026-11-26T06:56:18+07:00

The repository's shape: the layout, the toolchain, and the commands. Read once; return when a path or a command is needed.

## Layout

| Path | Role |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | The package `fmc`: one crate, edition 2024. Dependencies are listed in `rules.md` ("Dependencies"); the engine is `opentui` =0.9.9, and the vendored grammar crates are pinned exactly beside it. |
| `src/lib.rs` | The pure core (no I/O, no `opentui`): `keys/` (key list for `--help` and `?`, the `Key` and `Action` types, and the pending-key `Parser` in Scouter's dispatch order; tests tie the list to the parser and to `docs/keymaps.md`), `explorer/` (the tree model: `tree.rs` rows, expansion, reveal; `browser.rs` selection, scroll, hit-test; `sort.rs` noise and collation), `reader/` (`document.rs`: the 1 MiB cap, the NUL and UTF-8 checks, display text; `language.rs`: the highlighted filetypes; `markdown.rs`: the rendered and source views and their refusals; `motion.rs`: paragraphs, line tops, and paging amounts; `jump.rs`: the jump history; `search.rs`: in-file search (whole-line hits, the hit under the cursor, the footer's text); `image.rs`: the image extensions, size cap, and messages; `binary.rs`: the size text; `position.rs`: the footer's scroll label), `find/`, `diff/` (`marks.rs`: change kinds, `--name-status` parsing, the tree's marks), `text.rs` (one-line display text), `theme.rs` (palettes, brightness, tint, `ThemeWatch`; tests check them against `DESIGN.md`). |
| `src/main.rs`, `src/args.rs` | The binary: command line (`args.rs`, tests in `args_tests.rs`), then the app. Binary-only modules: `app/` (loop and dispatch; `app/keys.rs` names OpenTUI keys as Core does, `app/theme.rs` reads the terminal background), `ui/` (`styled.rs` segments and fitting, `header.rs`, `footer.rs` and `hints.rs`, `help.rs`, `welcome.rs`, `sidebar.rs` width), `services/` (I/O workers, each on its own thread, sending `Event`s: `fs.rs` lists, opens, and reads entries (Node's error wording in `describe`), `git.rs` reads the tree's marks from a private index copy, `mod.rs` holds `Event` and `spawn`), `app/tree.rs` (tree keys and events), `app/reader.rs` (the shown document, its header and footer facts), `app/scroll.rs` (the reader's motions, `y`, and the jumps), `app/search.rs` (the search prompt and its keys), `ui/reader.rs` (the code view with its line-number gutter), `ui/markdown.rs` (the rendered view), `ui/picture.rs` (the image view), `ui/binary.rs` (the binary notice), `ui/scrollbar.rs` (the viewer's bar), `ui/viewer.rs` (the scrolling pane around the text),  `ui/syntax/` (the grammars Scouter vendors and the style table from the palette; `queries/<filetype>/` holds each `highlights.scm` with its `LICENSE`), `ui/tree.rs` (rows, scrollbar thumb, hit-test line), `ui/` (views). |
| `tests/cli.rs` | End-to-end command-line tests; the ones that need a terminal run fmc under `script -q /dev/null`. |
| `LICENSE-MIT`, `LICENSE-APACHE` | fmc is `MIT OR Apache-2.0` (`Cargo.toml`). |
| `rust-toolchain.toml`, `rustfmt.toml` | Toolchain pin and formatting rules. |
| `deny.toml`, `about.toml`, `about.hbs` | License gate (`cargo deny`) and third-party notices (`cargo about`). |
| `justfile`, `scripts/task-*.just` | Task shortcuts (`just` lists them): development (`native`, `run`, `build`, `start`, `install`, `demo`, `deps`), quality (`format`, `format-check`, `check`, `lint`, `test`, `licenses`, `notices`, `verify`), utility (`cleanup`). Build, run, and quality tasks run `native` first. |
| `scripts/opentui-native.sh`, `.cargo/config.toml` | Builds the OpenTUI native library with video off into `~/.cache/fmc/opentui-native/<version>` and links it at `.cache/opentui-native/<version>` (git-ignored); `.cargo/config.toml` points `OPENTUI_LIB_DIR` there. Run it before the first build and after an `opentui` version bump (both files name the version). |
| `scripts/demo-repo.sh <dir>` | Builds the deterministic demo git repository for live runs of fmc and Scouter (same hashes and `git status` every run); replaces `<dir>` only when it holds the script's marker `.git/fmc-demo-repo` or is empty. |
| `docs/` | Human docs: `development.md` (prerequisites, first build, checks, demo repository, parity reference), `parity.md` (every approved difference from Scouter, with the baseline). |
| `CHANGELOG.md` | Keep a Changelog; user-visible changes go under `[Unreleased]`. |
| `README.md`, `PRODUCT.md`, `DESIGN.md` | Human docs: overview and usage, product purpose and principles, the design system (palettes, layout, components). |
| `.agents/docs/`, `.agents/skills/` | Agent docs and project skills (`.claude/skills` links to the skills). |
| `.claude/` | Claude Code settings, the format hook, and the project agents (`agents/executor.md`, `agents/scout.md`). |
| `target/` | Build output, git-ignored. |

No workspace, no `justfile`, no server, no database. Phase 1 is a read-only terminal app (`rules.md`, "Parity with Scouter").

## Toolchain

- Rust 1.100.0 pinned in `rust-toolchain.toml` with clippy and rustfmt; rustup installs it on first use. Edition 2024.
- Formatting: `rustfmt.toml` (`max_width = 120`, `use_small_heuristics = "Max"`). The Claude Code `PostToolUse` hook (`.claude/hooks/format.sh`) formats each edited `.rs` file with `rustfmt --edition 2024`, and each edited `.md` file with `rumdl fmt`.
- Tools: `jq` (the hook needs it), `rumdl` (optional: the Markdown branch of the hook skips it when missing), `just` (task shortcuts), Rex (the `rex` skill: live runs of fmc and of Scouter), zig 0.17.1 (the native library build), `cargo-deny` and `cargo-about` (`cargo install cargo-deny`, `cargo install cargo-about --locked --features cli`). Check that a tool is installed before relying on it; do not install one unasked.

## Commands

`just verify` runs format-check (rustfmt and rumdl), check, lint, test, and licenses in one go.

Run from the repository root.

| Gate | Command | Notes |
| --- | --- | --- |
| Format | `cargo fmt --check` | Reports drift; `cargo fmt` writes the fix. |
| Lint | `cargo clippy --all-targets -- -D warnings` | Warnings are errors. |
| Test | `cargo test` | Unit and integration tests. |
| Build | `cargo build` | Debug build; `cargo build --release` for the release profile. |
| Licenses | `cargo deny check licenses` | Allowlist from `rules.md` ("Licenses"); the `opentui-sys` exception lives in `deny.toml`. |
| Notices | `cargo about generate about.hbs -o target/THIRD_PARTY_LICENSES.html` | Ships with a release binary. |
| Run | `cargo run` | Runs the binary. Once fmc draws a terminal UI, drive it through Rex or the `run` skill, not through a pipe. |

A change claims done only when format, lint, test, and build pass (`rust-verify-harden`). A change to what is drawn or how input is handled also gets a live run.
