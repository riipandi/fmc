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
- Reader motions: `j` and `k` scroll, `h` and `l` pan, `space` and `b` page, `Ctrl+d` and `Ctrl+u` move half a page, `gg` and `G` go to the top and bottom, `120G` goes to line 120, `{` and `}` move by paragraph, `zz` centres the view, and `w` wraps long lines. `Ctrl+o` and `Alt+i` go back and forward through the places opened. A scrollbar shows beside text taller than the reader.
- Images (PNG, JPG, GIF, WEBP) open scaled to fit the reader, with their size and format in the footer. A binary file shows Scouter's notice with its name and size, and focus stays in the file list.
- Markdown files open rendered: headings, emphasis, links, lists, task boxes, quotes, tables, and highlighted code blocks. `m` shows the source with line numbers, and `/` and `{ }` say which key shows the source while the document is rendered.
- Product and design records (`PRODUCT.md`, `DESIGN.md`) and the plan for parity with Scouter.

### Changed

- The file tree has no root row: the folder's files and folders start at the top, with no indent. `h` on a top-level row does nothing; `-` still opens the folder above.
- Enter on a file opens it in the reader and keeps focus in the file list; Tab, or a click on the reader, moves focus there.
- Quit is `Ctrl+q` (or `Ctrl+c`); a plain `q` no longer quits.
- In-file search: `/` opens the prompt, and typing shows the lines that contain the query, ignoring case, with every match marked. Enter and `↓` go to the next match, Shift+Enter and `↑` to the previous, `n` and `N` step on, and Esc keeps the matches for `n` and `N` before it clears them.
- Dragging across text selects it and keeps the selection on screen. `y` copies the selection and `Esc` clears it; Scouter copies as soon as the mouse is released.
- File errors read as Node's messages, such as `ENOENT: no such file or directory, scandir '<path>'`.

[Unreleased]: https://github.com/riipandi/fmc/commits/main
