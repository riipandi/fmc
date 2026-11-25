# Shape

updated: 2026-11-26T04:20:50+07:00

The repository's shape: the layout, the toolchain, and the commands. Read once; return when a path or a command is needed.

## Layout

| Path | Role |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | The package `fmc`: one crate, edition 2024. The one dependency is `opentui_rust` 0.2.3. |
| `src/main.rs` | The entry point. Today a hello-world buffer demo: it draws into an `OptimizedBuffer` and prints; it never enters a terminal. |
| `LICENSE-MIT`, `LICENSE-APACHE` | fmc is `MIT OR Apache-2.0` (`Cargo.toml`). |
| `rust-toolchain.toml`, `rustfmt.toml` | Toolchain pin and formatting rules. |
| `deny.toml`, `about.toml`, `about.hbs` | License gate (`cargo deny`) and third-party notices (`cargo about`). |
| `scripts/opentui-native.sh`, `.cargo/config.toml` | Builds the OpenTUI native library with video off into `~/.cache/fmc/opentui-native/<version>` and links it at `.cache/opentui-native/<version>` (git-ignored); `.cargo/config.toml` points `OPENTUI_LIB_DIR` there. Run it before the first build and after an `opentui` version bump (both files name the version). |
| `docs/` | Human docs: `development.md` (prerequisites, first build, checks). |
| `CHANGELOG.md` | Keep a Changelog; user-visible changes go under `[Unreleased]`. |
| `README.md`, `PRODUCT.md`, `DESIGN.md` | Human docs: overview and usage, product purpose and principles, the design system (palettes, layout, components). |
| `.agents/docs/`, `.agents/skills/` | Agent docs and project skills (`.claude/skills` links to the skills). |
| `.claude/` | Claude Code settings and the format hook. |
| `target/` | Build output, git-ignored. |

No workspace, no `justfile`, no server, no database. Phase 1 is a read-only terminal app (`rules.md`, "Parity with Scouter").

## Toolchain

- Rust 1.100.0 pinned in `rust-toolchain.toml` with clippy and rustfmt; rustup installs it on first use. Edition 2024.
- Formatting: `rustfmt.toml` (`max_width = 120`, `use_small_heuristics = "Max"`). The Claude Code `PostToolUse` hook (`.claude/hooks/format.sh`) formats each edited `.rs` file with `rustfmt --edition 2024`, and each edited `.md` file with `rumdl fmt`.
- Tools: `jq` (the hook needs it), `rumdl` (optional: the Markdown branch of the hook skips it when missing), Rex (the `rex` skill: live runs of fmc and of Scouter), zig 0.17.1 (the native library build), `cargo-deny` and `cargo-about` (`cargo install cargo-deny`, `cargo install cargo-about --locked --features cli`). Check that a tool is installed before relying on it; do not install one unasked.

## Commands

Run from the repository root.

| Gate | Command | Notes |
| --- | --- | --- |
| Format | `cargo fmt --check` | Reports drift; `cargo fmt` writes the fix. |
| Lint | `cargo clippy --all-targets -- -D warnings` | Warnings are errors. |
| Test | `cargo test` | Unit and integration tests. |
| Build | `cargo build` | Debug build; `cargo build --release` for the release profile. |
| Licenses | `cargo deny check licenses` | Allowlist from `rules.md` ("Licenses"); the `opentui` exception lives in `deny.toml`. |
| Notices | `cargo about generate about.hbs -o target/THIRD_PARTY_LICENSES.html` | Ships with a release binary. |
| Run | `cargo run` | Runs the binary. Once fmc draws a terminal UI, drive it through Rex or the `run` skill, not through a pipe. |

A change claims done only when format, lint, test, and build pass (`rust-verify-harden`). A change to what is drawn or how input is handled also gets a live run.
