# Shape

updated: 2026-11-26T04:34:54+07:00

The repository's shape: the layout, the toolchain, and the commands. Read once; return when a path or a command is needed.

## Layout

| Path | Role |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | The package `fmc`: one crate, edition 2024. Dependencies are listed in `rules.md` ("Dependencies"); the engine is `opentui` =0.9.9. |
| `src/lib.rs` | The pure core (no I/O, no `opentui`): `keys/` (key list for `--help` and `?`, the `Key` and `Action` types, and the pending-key `Parser` in Scouter's dispatch order; tests tie the list to the parser and to `docs/keymaps.md`), `explorer/`, `reader/`, `find/`, `diff/`, `text.rs` (one-line display text), `theme.rs` (palettes, brightness, tint, `ThemeWatch`; tests check them against `DESIGN.md`). |
| `src/main.rs`, `src/args.rs` | The binary: command line (`args.rs`, tests in `args_tests.rs`), then the app. Binary-only modules: `app/` (loop and dispatch; `app/theme.rs` reads the terminal background), `services/` (I/O workers), `ui/` (views). |
| `tests/cli.rs` | End-to-end command-line tests; the ones that need a terminal run fmc under `script -q /dev/null`. |
| `LICENSE-MIT`, `LICENSE-APACHE` | fmc is `MIT OR Apache-2.0` (`Cargo.toml`). |
| `rust-toolchain.toml`, `rustfmt.toml` | Toolchain pin and formatting rules. |
| `deny.toml`, `about.toml`, `about.hbs` | License gate (`cargo deny`) and third-party notices (`cargo about`). |
| `justfile`, `scripts/task-*.just` | Task shortcuts (`just` lists them): development (`native`, `run`, `build`, `start`, `demo`, `deps`), quality (`format`, `format-check`, `check`, `lint`, `test`, `licenses`, `notices`, `verify`), utility (`cleanup`). Build, run, and quality tasks run `native` first. |
| `scripts/opentui-native.sh`, `.cargo/config.toml` | Builds the OpenTUI native library with video off into `~/.cache/fmc/opentui-native/<version>` and links it at `.cache/opentui-native/<version>` (git-ignored); `.cargo/config.toml` points `OPENTUI_LIB_DIR` there. Run it before the first build and after an `opentui` version bump (both files name the version). |
| `scripts/demo-repo.sh <dir>` | Builds the deterministic demo git repository for live runs of fmc and Scouter (same hashes and `git status` every run); replaces `<dir>` only when it holds the script's marker `.git/fmc-demo-repo` or is empty. |
| `docs/` | Human docs: `development.md` (prerequisites, first build, checks, demo repository, parity reference). |
| `CHANGELOG.md` | Keep a Changelog; user-visible changes go under `[Unreleased]`. |
| `README.md`, `PRODUCT.md`, `DESIGN.md` | Human docs: overview and usage, product purpose and principles, the design system (palettes, layout, components). |
| `.agents/docs/`, `.agents/skills/` | Agent docs and project skills (`.claude/skills` links to the skills). |
| `.claude/` | Claude Code settings and the format hook. |
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
