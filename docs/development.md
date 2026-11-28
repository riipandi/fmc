# Development

fmc builds on macOS with Apple silicon. Other platforms come later.

## Prerequisites

- Rust 1.92 or newer. `rustup` installs the pinned toolchain from `rust-toolchain.toml` the first time you run `cargo`.
- Zig 0.17.1, exactly. fmc uses it once per OpenTUI version to build the native library, and the build stops on any other version. `brew install zig` may not give that version, so check with `zig version`.
- [kache](https://github.com/kunobi-ninja/kache) (`cargo install kache --locked`), the compiler cache the just tasks use. Optional with `KACHE=false`.
- Git.
- [just](https://just.systems) (`brew install just`), for the task shortcuts. Optional: every task is a plain command you can run yourself.
- A terminal with a Nerd Font, for the file icons.

## First build

Build the OpenTUI native library, then build fmc:

```sh
scripts/opentui-native.sh
cargo build
```

With just, `just build` does both. `just` alone lists every task.

To put fmc on your `PATH`, run `just install`. It builds a release binary and copies it to `~/.local/bin/fmc`. `just install debug` installs a debug build as `fmc-debug`. Set `INSTALL_DIR` to use another folder.

The native library is built without video support, so the fmc binary contains no FFmpeg code. The first run takes a few minutes. The result is kept in `~/.cache/fmc/opentui-native/<version>`, so later runs and other checkouts finish at once.

## Build cache

The just tasks run cargo through kache, which keeps compiled crates and reuses them across builds, checkouts, and `cargo clean`. If kache is missing, a task stops and says how to install it.

To run plain cargo, set `KACHE` to `false`, `0`, or `no`:

```sh
KACHE=false just verify
```

`kache stats` shows the hit rate and the time saved. Plain `cargo` commands follow your own Cargo setup; `kache init` makes them use kache too.

## Checks

Run `just verify` before you send a change. It runs these, and it checks the Markdown format with `rumdl` when that tool is installed:

```sh
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check licenses
```

Run `cargo build` too before a commit that changes code.

`cargo deny` and `cargo about` are separate tools:

```sh
cargo install cargo-deny
cargo install cargo-about --locked --features cli
```

CI runs for every push to `main`, every pull request, and on demand. It first looks at which files changed, then runs only the checks they need:

- A change to the Rust code, the build files, the just tasks, `DESIGN.md`, or `docs/keymaps.md` runs `just verify` on macOS (Apple silicon), with kache's store kept in the GitHub Actions cache. The tests read those two Markdown files.
- A change to any Markdown file checks its format with rumdl, on Linux.
- A change to anything else, such as `.claude/` or the release workflow, runs no check.

The `ci` job reports one result for the whole run. A manual run, or a push with no earlier commit to compare against, runs every check.

## Demo repository

The demo repository is a small git repository with every kind of file and change fmc shows: Markdown, code in several languages, an image, a binary file, staged, unstaged, untracked, deleted, and renamed files, ignored folders, a very large log, a very wide line, text that is not UTF-8, and symbolic links (one of them a loop). Build it with:

```sh
scripts/demo-repo.sh /tmp/fmc-demo
```

Run it again at any time to get the same repository back. The script only replaces a folder it made itself, or an empty one.

## Parity reference

fmc copies Scouter's screens and keys. To compare the two, run Scouter on the same OpenTUI version as fmc (0.9.9). You need [Bun](https://bun.sh).

```sh
git clone https://github.com/indrazm/scouter /tmp/scouter
cd /tmp/scouter
git checkout 434d000f0f934fb04fced8b1bc1d0e673c6ed4fe
bun install
bun add @opentui/core@0.9.9 --exact
cd /tmp/fmc-demo
bun --config=/dev/null --no-env-file /tmp/scouter/src/index.ts .
```

Open fmc on the same folder in a terminal of the same size, press the same keys, and compare the two screens. [Differences from Scouter](parity.md) lists what should differ.

## Releasing

A release is a version number, a changelog section, and a tag. The tag starts the `release` workflow, which builds the macOS archive and publishes the GitHub Release.

1. Set the new version in `version` under `[package]` in `Cargo.toml`, for example `0.1.1`. Run `cargo build` so `Cargo.lock` takes the same version.
2. In `CHANGELOG.md`, move the entries under `## [Unreleased]` into a new section, `## [0.1.1] - YYYY-MM-DD`. Keep an empty `## [Unreleased]` above it, and update the compare and tag links at the bottom. The workflow uses that section as the release notes, so it must make sense on its own.
3. Run `just verify`, then commit `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md`.
4. Merge the change into `main`.
5. On `main`, create the tag and push it: `git tag v0.1.1` and `git push origin v0.1.1`.
6. The `release` workflow stops unless the tag matches the version in `Cargo.toml` and the changelog has a `## [0.1.1]` section. When it passes, it builds `fmc-0.1.1-aarch64-apple-darwin.tar.gz` and its `.sha256` file, and creates the GitHub Release with both and the notes.
7. Update the Homebrew formula in `riipandi/homebrew-tap`, file `Formula/fmc.rb`: its `version`, its `url` (the new archive), and its `sha256` (the value in the `.sha256` file). The release does not do this yet. Automation needs a `HOMEBREW_TAP_TOKEN` secret in this repository, which is not set up.
8. Install from the tap on a clean Mac, or from the archive, and run `fmc --version`.
