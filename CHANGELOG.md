# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Command line: `fmc [file-or-directory]`, `--help` with every key, and `--version`, with the same checks and messages as Scouter.
- Dark and light palettes that follow the terminal's background, checked at start, every 3 seconds, and when the terminal reports a change.
- The two panes with their headers, the two-row footer with hints that fit the width, the welcome screen, and the `?` card, which also lists the mouse.
- The file tree: folders expand and collapse (`l` `h` Enter, a click), `-` opens the folder above, and a file argument opens its folder with the file selected. Folders come first, then names in Scouter's collation order.
- Git marks in the tree: status letters, a dot on folders with changes, and ignored entries dimmed. A scrollbar appears on long lists.
- Sidebar width: `Ctrl+←` and `→` change it by two columns, and dragging its edge does too.
- Project workspace: agent instructions, agent docs, and project skills.
- Text files open in the reader with a line-number gutter, syntax colours for Scouter's languages, and the footer's line count, language, and scroll position. A file over 1 MiB shows its first 1 MiB.
- Product and design records (`PRODUCT.md`, `DESIGN.md`) and the plan for parity with Scouter.

### Changed

- Enter on a file opens it in the reader and moves focus there; Tab moves between the file list and an open file.
- File errors read as Node's messages, such as `ENOENT: no such file or directory, scandir '<path>'`.

[Unreleased]: https://github.com/riipandi/fmc/commits/main
