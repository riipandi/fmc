# fmc

A fast, read-only file viewer for the terminal, written in Rust on OpenTUI.

fmc browses a folder, reads code, Markdown, and images, finds files and text, and shows your git changes as diffs. Today it is a port of [Scouter](https://github.com/indrazm/scouter), with the same screens and keys. Later it grows into a file manager.

> **Status:** early. The port is in progress and fmc does not run as a viewer yet. It targets macOS on Apple silicon first.

## Usage

```sh
fmc            # open the current folder
fmc <folder>   # open a folder
fmc <file>     # open a file, with its folder in the tree
fmc --help     # every key
```

Press `?` inside fmc for every key, and `q` to quit. A [Nerd Font](https://www.nerdfonts.com/) shows the file icons.

## Build

Build instructions arrive with the first build tooling.

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
