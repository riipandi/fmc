# Development

fmc builds on macOS with Apple silicon. Other platforms come later.

## Prerequisites

- Rust. `rustup` installs the pinned version from `rust-toolchain.toml` the first time you run `cargo`.
- Zig 0.17.1 (`brew install zig`). fmc uses it once per OpenTUI version to build the native library.
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

The script builds the library without video support, so the fmc binary contains no FFmpeg code. The first run takes a few minutes. It keeps the result in `~/.cache/fmc/opentui-native/<version>`, so later runs and other checkouts finish at once.

## Checks

Run these before you send a change, or run `just verify`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo deny check licenses
```

`cargo deny` and `cargo about` are separate tools:

```sh
cargo install cargo-deny
cargo install cargo-about --locked --features cli
```

## Demo repository

The demo repository is a small git repository with every kind of file and change fmc shows: Markdown, code in several languages, an image, a binary file, staged, unstaged, untracked, deleted, and renamed files, ignored folders, a very large log, a very wide line, text that is not UTF-8, and symbolic links (one of them a loop). Build it with:

```sh
scripts/demo-repo.sh /tmp/fmc-demo
```

Run it again at any time to get the same repository back. The script only replaces a folder it made itself, or an empty one.

## Run the parity reference

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

Open fmc on the same folder in a terminal of the same size, press the same keys, and compare the two screens.
