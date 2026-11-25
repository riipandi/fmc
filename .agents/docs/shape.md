# Shape

updated: 2026-11-26T03:19:42+07:00

The repository's shape: the layout, the toolchain, and the commands. Read once; return when a path or a command is needed.

## Layout

| Path | Role |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | The package `fmc`: one crate, edition 2024. The one dependency is `opentui_rust` 0.2.3. |
| `src/main.rs` | The entry point. Today a hello-world buffer demo: it draws into an `OptimizedBuffer` and prints; it never enters a terminal. |
| `.agents/docs/`, `.agents/skills/` | Agent docs and project skills (`.claude/skills` links to the skills). |
| `.claude/` | Claude Code settings and the format hook. |
| `target/` | Build output, git-ignored. |

No workspace, no `docs/` (human docs), no `justfile`, no server, no database. Phase 1 is a read-only terminal app (`rules.md`, "Parity with Scouter").

## Toolchain

- Cargo with rustc, rustfmt, and clippy from rustup. Edition 2024 (`Cargo.toml:3`). No `rust-toolchain.toml`: the toolchain is whatever rustup provides.
- Formatting: rustfmt with its defaults (no `rustfmt.toml`). The Claude Code `PostToolUse` hook (`.claude/hooks/format.sh`) formats each edited `.rs` file with `rustfmt --edition 2024`, and each edited `.md` file with `rumdl fmt`.
- Tools: `jq` (the hook needs it), `rumdl` (optional: the Markdown branch of the hook skips it when missing), Rex (the `rex` skill: live runs of fmc and of Scouter). Check that a tool is installed before relying on it; do not install one unasked.

## Commands

Run from the repository root.

| Gate | Command | Notes |
| --- | --- | --- |
| Format | `cargo fmt --check` | Reports drift; `cargo fmt` writes the fix. |
| Lint | `cargo clippy --all-targets -- -D warnings` | Warnings are errors. |
| Test | `cargo test` | Unit and integration tests. |
| Build | `cargo build` | Debug build; `cargo build --release` for the release profile. |
| Run | `cargo run` | Runs the binary. Once fmc draws a terminal UI, drive it through Rex or the `run` skill, not through a pipe. |

A change claims done only when format, lint, test, and build pass (`rust-verify-harden`). A change to what is drawn or how input is handled also gets a live run.
