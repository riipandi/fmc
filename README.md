# fmc

A fast, read-only file viewer for the terminal, written in Rust on OpenTUI.

fmc browses a folder, reads code, Markdown, and images, finds files and text, and shows your git changes as diffs. Today it is a port of [Scouter](https://github.com/indrazm/scouter), with the same screens and keys, except for the [differences listed here](docs/parity.md). Later it grows into a file manager.

> **Status:** early. The port is in progress and fmc does not run as a viewer yet. It targets macOS on Apple silicon first.

## Usage

```sh
fmc            # open the current folder
fmc <folder>   # open a folder
fmc <file>     # open a file, with its folder in the tree
fmc --help     # every key
```

Press `?` inside fmc for every key, and `Ctrl+q` or `:q` to quit. A [Nerd Font](https://www.nerdfonts.com/) shows the file icons.

With the file list focused, the file it selects shows in the reader. Enter opens a file and moves to the reader. `p` turns that preview off or on. `[` and `]` jump; their steps and the preview's start are set in `settings.json` (see [docs/configuration.md](docs/configuration.md)).

Inside a git repository, a strip left of the line numbers marks the lines that differ from `HEAD`: green for an added line, yellow for a changed one, and a red `▁` or `▔` where lines were removed. `git_gutter` in `settings.json` turns it off.

A ```` ```mermaid ```` fence in a Markdown file is drawn as a box diagram in the rendered view. A diagram too wide for the reader, or one fmc cannot draw, shows its source in a frame with the reason. `m` shows the Markdown source.

From 60 to 79 columns the file list hides while a file is open: a `›` at the start of the reader's title shows it, and a `‹` at the start of the list's title hides it. `Ctrl+/` does both too. Below 60 columns the list and the reader are two pages: a file opens full screen, and the header's `‹` or `q` goes back. The bottom bar has the page's keys as words, and each word is a tap. At 80 columns and wider a `«` or `»` on the border shows and hides the list. `Ctrl+.` shows or hides dotfiles.

## Build

You need Rust and Zig 0.17.1. See [docs/development.md](docs/development.md).

```sh
scripts/opentui-native.sh
cargo build --release
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
