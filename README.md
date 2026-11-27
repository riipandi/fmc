# fmc

A fast, read-only file viewer for the terminal. fmc browses a folder, reads code, Markdown, and images, finds files and text, and shows what git reports as changed. It is a Rust port of [Scouter](https://github.com/indrazm/scouter) with the same screens and keys, except for [the differences](docs/parity.md). It runs on macOS with Apple silicon.

## Install

```sh
brew install riipandi/tap/fmc
```

The release archive and building from source are in [docs/install.md](docs/install.md).

## Quick start

```sh
fmc            # open the current folder
fmc <path>     # open a folder, or a file (its folder opens too)
```

Five keys cover most of it:

| Key | Does |
| --- | --- |
| `?` | Shows every key, in the reader |
| `/` | Finds a file by name |
| `Enter` | Opens the selected file, and moves to the reader |
| `Tab` | Switches between the file list and the reader |
| `d` | Shows what git reports as changed |

Quit with `Ctrl+q` or `:q`.

## Documentation

Start at [docs/index.md](docs/index.md). It lists each page and what it answers: install, usage, keys, configuration, the command line, the differences from Scouter, and development.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
