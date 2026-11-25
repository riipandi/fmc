# Development

fmc builds on macOS with Apple silicon. Other platforms come later.

## Prerequisites

- Rust. `rustup` installs the pinned version from `rust-toolchain.toml` the first time you run `cargo`.
- Zig 0.17.1 (`brew install zig`). fmc uses it once per OpenTUI version to build the native library.
- Git.
- A terminal with a Nerd Font, for the file icons.

## First build

Build the OpenTUI native library, then build fmc:

```sh
scripts/opentui-native.sh
cargo build
```

The script builds the library without video support, so the fmc binary contains no FFmpeg code. The first run takes a few minutes. It keeps the result in `~/.cache/fmc/opentui-native/<version>`, so later runs and other checkouts finish at once.

## Checks

Run these before you send a change:

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
