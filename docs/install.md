# Install

fmc runs on macOS with Apple silicon. Pick one of the three ways below.

## Homebrew

```sh
brew install riipandi/tap/fmc
```

The formula `fmc` comes from the `riipandi/homebrew-tap` tap. `brew upgrade fmc` installs a newer release.

## Release archive

Every release has a `fmc-<version>-aarch64-apple-darwin.tar.gz` archive and a `.sha256` file beside it, on the [releases page](https://github.com/riipandi/fmc/releases).

```sh
shasum -a 256 -c fmc-<version>-aarch64-apple-darwin.tar.gz.sha256
tar -xzf fmc-<version>-aarch64-apple-darwin.tar.gz
mv fmc-<version>-aarch64-apple-darwin/fmc ~/.local/bin/
```

The check prints `OK` when the archive is intact. The archive unpacks into a folder of the same name. It holds the `fmc` binary, the README, the changelog, the licenses, and `THIRD_PARTY_LICENSES.html`, the licenses of the code fmc includes. Move `fmc` into a folder on your `PATH`, such as `~/.local/bin` if that folder is on yours, then run `fmc --version` to check.

## Build from source

See [Development](development.md#first-build). You need Rust 1.92 or newer and Zig 0.17.1.

## Font and terminal

- A [Nerd Font](https://www.nerdfonts.com/) in your terminal draws the file icons. fmc uses its glyphs, so the icons need one.
- fmc asks the terminal for the Kitty keyboard protocol, which lets it receive keys such as `Ctrl+.` and `Ctrl+/`. Terminal.app cannot send `Ctrl+.`, so use `.` there to show or hide dotfiles.
