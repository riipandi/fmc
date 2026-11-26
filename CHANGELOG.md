# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Command line: `fmc [file-or-directory]`, `--help` with every key, and `--version`, with the same checks and messages as Scouter.
- Dark and light palettes that follow the terminal's background, checked at start, every 3 seconds, and when the terminal reports a change.
- The two panes with their headers, the two-row footer with hints that fit the width, the welcome screen, and the `?` key list, which also lists the mouse.
- The file tree: folders expand and collapse (`l` `h` Enter, a click), `-` opens the folder above, and a file argument opens its folder with the file selected. Folders come first, then names in Scouter's collation order.
- Git marks in the tree: status letters, a dot on folders with changes, and ignored entries dimmed. A scrollbar appears on long lists.
- Sidebar width: `Ctrl+←` and `→` change it by two columns, and dragging its edge does too.
- Project workspace: agent instructions, agent docs, and project skills.
- Text files open in the reader with a line-number gutter, syntax colours for Scouter's languages, and the footer's line count, language, and scroll position. A file over 1 MiB shows its first 1 MiB.
- Reader motions: `j` and `k` scroll, `h` and `l` pan, `space` and `b` page, `Ctrl+d` and `Ctrl+u` move half a page, `gg` and `G` go to the top and bottom, `120G` goes to line 120, `{` and `}` move by paragraph, `zz` centres the view, and `w` wraps long lines. `Ctrl+o` and `Alt+i` go back and forward through the places opened. A scrollbar shows beside text taller than the reader.
- Images (PNG, JPG, GIF, WEBP) open scaled to fit the reader, with their size and format in the footer. A binary file shows Scouter's notice with its name and size, and focus stays in the file list.
- Markdown files open rendered: headings, emphasis, links, lists, task boxes, quotes, tables, and highlighted code blocks. `m` shows the source with line numbers, and `/` and `{ }` say which key shows the source while the document is rendered.
- Product and design records (`PRODUCT.md`, `DESIGN.md`) and the plan for parity with Scouter.
- The file finder: `/` in the file list finds a file or folder by name. Every word must appear in its path, the matches come in Scouter's order, and up to 200 are listed with the total. Enter opens a file or reveals a folder, and Esc returns the tree to where it was.
- Text search in the finder: `Tab` searches the lines of the files, from two characters, with up to 200 hits. The highlighted hit shows in the reader as a preview, and Enter opens the file at its line with the search kept for `n` and `N`.
- The changes screen: `d` lists the files git reports as changed, with their status letters, index marks, and counts. The stacked diff below shows every file under its own header, with the counts, a stat bar, and the file's lines. Moving the list selection scrolls the diff to that file, `Enter` moves to the diff, `Tab` switches between them, and `w` wraps the lines. `d` or `q` goes back to browsing. Split, folds, context, hunks, and opening a file from the diff come next.

### Changed

- `?` shows the key list as tables in the reader instead of a card over the panes. `?` or `Esc` puts the document back, with its scroll position and search.
- The file tree has no root row: the folder's files and folders start at the top, with no indent. `h` on a top-level row does nothing; `-` still opens the folder above.
- Enter on a file opens it in the reader and keeps focus in the file list; Tab, or a click on the reader, moves focus there.
- Quit is `Ctrl+q` (or `Ctrl+c`); a plain `q` no longer quits. It closes the open document, and the welcome screen returns, or it closes the key list.
- In-file search: `/` opens the prompt, and typing shows the lines that contain the query, ignoring case, with every match marked. Enter and `↓` go to the next match, Shift+Enter and `↑` to the previous, `n` and `N` step on, and Esc keeps the matches for `n` and `N` before it clears them.
- Dragging across text selects it and keeps the selection on screen. `y` copies the selection and `Esc` clears it; Scouter copies as soon as the mouse is released.
- File errors read as Node's messages, such as `ENOENT: no such file or directory, scandir '<path>'`.

### Fixed

- `j` and `k` in a folder with nothing to show no longer crash the app.

[Unreleased]: https://github.com/riipandi/fmc/commits/main
