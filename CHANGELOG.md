# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A theme catalog: 16 bundled themes (bjarneo/editor, MIT), chosen with `dark_theme` and `light_theme` in `settings.json`. `scouter` is the default and Scouter's own pair.
- `theme_mode` in `settings.json` (`auto`, `dark`, or `light`) forces a mode; `auto` still follows the terminal.
- User themes: a JSON file per theme in `$XDG_CONFIG_HOME/fmc/themes/` (or `~/.config/fmc/themes/`), read at start. A file with a bundled id replaces that theme; a bad file is skipped with a footer notice.
- The Settings page on `Ctrl+,`: it lists the eight settings fmc saves, with a help line for the selected one. Enter or Space changes a value, `h` and `l` step a choice back and forward, and a number opens a dialog to type it. Esc or `q` closes the page.
- Each change on the Settings page applies at once and saves to `settings.json` at once. A save sets only its key, keeps the other keys, and is refused, with the value put back, when the file does not parse.
- Keys can be changed in `$XDG_CONFIG_HOME/fmc/keymap.json` (or `~/.config/fmc/keymap.json`), read at start: `"<context>.<key>": "<action>"` binds a key, and `null` unbinds the default. A bad entry is skipped with a footer notice that names its line. The defaults are unchanged; see [Keys](docs/keymaps.md#changing-the-keys).
- The `?` list shows the keys the file sets, and a key bound to an action with no row gets a row of its own.

### Changed

- `docs/configuration.md` no longer calls the settings file read-only: the Settings page writes it.
- The `?` key list shows the action first and the key second, groups related acts together, and lists the file acts in a Files section of their own.
- `fmc --help` no longer lists the keys. It prints the usage, the arguments, and the options, and says where the keys (`?`), the settings (`Ctrl+,`), and the docs are.
- The context menu shows each row's key at its right edge, separates its groups with a rule, and colours Delete as a destructive act.

## [0.3.0] - 2026-11-27

Many rows can be marked and copied, moved, or sent to the Trash at once, and code embedded in other languages is coloured as its own language.

### Added

- `s` marks the selected row and moves down one row, and `S` clears the marks. A marked row shows `✓`, and the footer counts the marks. `Esc` drops a copy or cut first, then the marks.
- `c`, `x`, and `v` work on every marked row. A paste of several rows runs one at a time and shows `copying 2 of 5…`; the first error stops it, and the footer says how many rows landed. The menu's Copy, Cut, and Paste say how many rows they take.
- `D` and Delete move every marked row to the Trash after one dialog that names them, up to five and then the count of the rest. The menu's Delete says how many rows it takes.
- A paste or a delete that finds an entry replaced since it was marked leaves that entry alone and stops, with a footer note.
- The code inside some files is coloured as its own language: a Svelte or Astro `<script>` and `<style>` block, an Astro frontmatter, a HEEx template, the HTML of an Elixir `~H` sigil, and the body of Elixir's `~r`, `~SQL`, `~MD`, `~PY`, and `~JS` sigils. HTML's `<script>` and `<style>` blocks are coloured too (`docs/parity.md`).
- In JavaScript, TypeScript, and Python, regex literals and the patterns of Python's `re` functions are coloured as patterns, and the JSDoc tags of a `/** */` comment are coloured as tags.

### Changed

- A Trash runs on the worker, as a paste does. A delete error now shows in the footer, and the dialog closes when the delete starts.
- A link cannot go to the Trash, because Finder refuses it. `D` on a link, or on marks that include one, says so in the footer and moves nothing.

## [0.2.0] - 2026-11-27

fmc becomes a file manager: files can be created, renamed, copied, moved, duplicated, and moved to the Trash from the file list.

### Added

- Files can be managed from the file list. `a` creates a file (a name ending in `/` makes a folder), `A` creates a folder, `r` renames, `c` copies, `x` cuts, and `v` pastes into the selected folder or beside the selected file. `D` or `Delete` moves the row to the Trash after a dialog, and Finder's Put Back restores it. `Space` and a right-click open a context menu with the same actions. A name that is taken is never overwritten.
- A context menu and a confirm dialog, drawn over the page with a dimmed background. The dialog's Cancel is the default.
- The name field of a dialog edits by word and by line, as in a macOS text field: `Option+←` and `Option+→` move by word, `Option+Backspace` and `Option+Delete` delete a word, `Cmd+←` and `Cmd+→` go to the ends of the line, and `Cmd+Backspace` deletes to its start. The Ctrl keys `Ctrl+A`, `Ctrl+E`, `Ctrl+W`, `Ctrl+U`, and `Ctrl+K` do the same (`docs/keymaps.md`, "Name field").
- `R` on a file row duplicates it: a name dialog opens with the name a paste would give, and Return copies the row into the same folder under the name typed. The context menu has a Duplicate row too. A taken name keeps the dialog open with its message.

### Changed

- fmc is no longer read-only. Its file operations are the only writes it makes, inside the folder it opened (`docs/parity.md`, "File operations").
- A pointer over a dialog button highlights it and leaves the focus where the keys put it, so Return picks the focused button.
- The welcome screen calls fmc a terminal file manager, and its closing line names the menu key: `Space opens the menu.` It said `a quiet, read-only file viewer` and `No editing. Just reading.`. `fmc --help` calls it a terminal file manager too.

### Fixed

- A folder that is swapped for a link during a copy or move is refused, not copied from the link's target. The check runs before the folder is created, so nothing outside the root is written.

## [0.1.1] - 2026-11-27

### Changed

- A Mermaid diagram in rendered Markdown sits on a panel tone across the reader's width, so it reads as one block. It follows the reader's width on a resize.

### Fixed

- A Mermaid diagram too wide for the reader shows its source in a frame, and long source lines now break between words instead of in the middle of one.
- `Ctrl+F` in the file finder changes what the footer shows before a query is typed: it names `fuzzy` while fuzzy matching is on. Before, a press on an empty prompt changed nothing on screen.

## [0.1.0] - 2026-11-27

The first release. fmc is a read-only terminal viewer for folders, code, Markdown, images, and git changes. It is a port of [Scouter](https://github.com/indrazm/scouter) v0.3.4 with the same screens and keys, except for the differences listed in `docs/parity.md` in the repository. It runs on macOS with Apple silicon.

### Added

- Open a folder with `fmc <folder>`, or a file with `fmc <file>`, which opens the file's folder with the file selected. `fmc --help` lists every key, and `fmc --version` prints the version.
- A file tree with git marks: a status letter on each changed file, a dot on each folder that holds changes, and ignored entries dimmed. Folders come first, then names in Scouter's order.
- Auto-preview: with the list focused, the file the list selects shows in the reader after a short pause. `p` turns it off or on.
- Text files with line numbers and syntax colours for Scouter's languages, and for astro, diff, elixir, heex, hcl, jsdoc, lua, regex, sql, svelte, vim, and xml. A file over 1 MiB shows its first 1 MiB.
- Reading keys: `j` and `k` scroll, `space` and `b` page, `Ctrl+d` and `Ctrl+u` move half a page, `gg` and `G` go to the top and bottom, `120G` goes to line 120, `{` and `}` move by paragraph, `zz` centres the view, and `w` wraps long lines. `Ctrl+o` and `Alt+i` go back and forward through the places you opened.
- Search in a file with `/`: the lines that contain the query are shown, ignoring case, with every match marked. `n` and `N` step through the matches.
- Markdown files open rendered: headings with their `#` markers, emphasis, links, lists, task boxes, quotes, tables, and highlighted code blocks. `m` shows the source with line numbers.
- Mermaid diagrams: a `mermaid` fence in rendered Markdown is drawn as a box diagram at the reader's width, for flowcharts, sequence, class, state, gantt, and the other diagram types the engine draws. A diagram that is too wide, or that fmc cannot draw, shows its source in a frame with the reason.
- Search in the rendered Markdown view: `/` marks every occurrence in the text as it is drawn, and `n`, `N`, and `p` step through them.
- Images (PNG, JPG, GIF, and WEBP) open scaled to fit the reader, with their size and format in the footer. Any other binary file shows a notice with its name and size.
- Git signs in the reader: inside a git repository, a strip left of the line numbers marks added lines in green, changed lines in yellow, and deleted lines in red, against `HEAD`. The rendered Markdown view marks its changed blocks the same way.
- The changes screen (`d`) lists every file git reports as changed, with its status letter, index marks, and line counts, and shows its diff. `a` stacks every file's diff, `s` shows the diff side by side at 60 columns or more, `c` folds a file, `x` shows more context lines, `n`, `N`, `]`, and `[` move between hunks, `o` opens the file at the current hunk, and `r` reloads.
- The file finder (`/`) matches names as substrings by default: every word of the query must appear in the path. `Ctrl+F` switches to fuzzy matching, where the query's letters must appear in order, and `find_mode` in the settings file sets the start. Up to 200 matches are listed. `Tab` searches the text of the files, and Enter opens a file at the matching line.
- Copying: drag across text to select it, then `y` copies the selection. With nothing selected, `y` copies the current file's path.
- Three layouts by width. From 80 columns the file list and the reader sit side by side. From 60 to 79 columns the list hides while a document is open. Below 60 columns the list and the reader are two pages, with a bottom bar of words.
- Mouse support: click to select and to focus a pane, scroll with the wheel, drag scrollbars and the list's edge, and tap the bottom bar's words in the narrow layout.
- Dark and light palettes that follow the terminal's background.
- Live refresh: the file tree, the open file, and the changes update about every two seconds, and back off when a refresh is slow.
- Dotfiles: `Ctrl+.` (or `.`) shows and hides them in the tree and the finder.
- `Ctrl+/` shows and hides the file list. `:q` quits, as in Neovim, and `Ctrl+q` quits too.
- The `?` key list, shown as tables in the reader.
- A settings file, `settings.json` in `$XDG_CONFIG_HOME/fmc` (or `~/.config/fmc`). It sets `auto_preview`, `sidebar_jump`, `reader_jump`, `git_gutter`, `file_associations`, and `find_mode`. fmc reads it when it starts and never writes it.
- `file_associations` maps file patterns to a filetype, as VS Code's `files.associations` does: `*.rs.bak` is highlighted as Rust, `.env.*` as a dotenv file, and `docs/**/*.txt` renders as Markdown.

### Changed

These differ from Scouter v0.3.4, which a Scouter user will notice. `docs/parity.md` in the repository lists every difference.

- `Ctrl+q` quits, and a plain `q` closes the open document or the key list.
- Enter opens a file and moves the focus to the reader. `l` and `→` only expand folders.
- A click anywhere in a pane focuses it, and a row in a list acts when the click is released, so a swipe over a row does nothing.
- `[` and `]` move the list by two rows and scroll the reader by five lines. Scouter uses them only on the changes screen.
- Headings in the rendered view keep their `#` markers, and line numbers stay in place when the text scrolls sideways.
- The changes screen opens on one file's diff. `a` stacks all files.
- `?` shows the key list in the reader, not as a card over the panes.

[Unreleased]: https://github.com/riipandi/fmc/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/riipandi/fmc/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/riipandi/fmc/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/riipandi/fmc/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/riipandi/fmc/releases/tag/v0.1.0
