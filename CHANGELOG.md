# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Syntax colours for the owner's extra languages: astro, diff, elixir, heex, hcl (with `.hcl`, `.tf`, and `.tfvars`), jsdoc, lua, regex, sql, svelte, vim, and xml. Code blocks in rendered Markdown and `file_associations` accept these names.
- Search in the rendered Markdown view: `/` searches the text the view draws, so `**Harry** Potter` matches `Harry Potter`; every occurrence is marked, the current one in its own colour, and `n`, `N`, and `p` step through them, with the footer's `k of N`. Scouter refuses `/` there.
- Command line: `fmc [file-or-directory]`, `--help` with every key, and `--version`, with the same checks and messages as Scouter.
- Dark and light palettes that follow the terminal's background, checked at start, every 3 seconds, and when the terminal reports a change.
- The two panes with their headers, the two-row footer with hints that fit the width, the welcome screen, and the `?` key list, which also lists the mouse.
- `]` and `[` jump: the list moves by `sidebar_jump` rows, and the reader scrolls by `reader_jump` lines, both from the settings file.
- `:q` quits, as in Neovim: `:` opens a command line in the footer, and `:q!`, `:qa`, `:qa!`, and `:quit` work too.
- Auto-preview: with the list focused, the file it selects shows in the reader after a short pause. `p` turns it off or on.
- The file tree: `l` and `→` expand a folder, `h` and `←` collapse it, Enter opens a file (with the focus in the reader) or toggles a folder, a click selects a row, and `-` opens the folder above, and a file argument opens its folder with the file selected. Folders come first, then names in Scouter's collation order.
- Git marks in the tree: status letters, a dot on folders with changes, and ignored entries dimmed. A scrollbar appears on long lists.
- Git signs in the rendered Markdown view: each block with a changed line gets a bar beside it, and a removed line shows a `▁` under the block above, so changes read in the rendered text too. `git_gutter` turns them off with the reader's strip.
- Git signs in the reader: inside a git repository, a strip left of the line numbers marks added lines (green), changed lines (yellow), and deletions (red), against `HEAD` as the changes screen does. The `git_gutter` setting in the settings file turns it off.
- Sidebar width: `Ctrl+←` and `→` change it by two columns, and dragging its edge does too.
- Sidebar visibility: below 80 columns the sidebar hides while a document is open, and the welcome screen keeps it. `Ctrl+/`, or the `«` or `»` on the border, shows and hides it.
- Dotfiles: `Ctrl+.` or `.` shows and hides them in the tree and the finder. They are shown by default, and the noise names stay hidden.
- Phone layout below 80 columns: a tap on the reader's header row shows the hidden file list. Below 60 columns the list covers the screen as a drawer and closes when you open a file. Footer hints can be tapped, a `Copy` hint appears while text is selected, and finder rows show the file name before its folder. From 60 to 74 columns the welcome drops its key card, since the reader beside the list is narrower than the card.
- Project workspace: agent instructions, agent docs, and project skills.
- The just tasks build through the kache compiler cache; `KACHE=false` turns it off. CI runs `just verify` on macOS for pushes and pull requests.
- A read-only settings file, `settings.json` under `XDG_CONFIG_HOME/fmc` (or `~/.config/fmc`), with the keys `auto_preview`, `sidebar_jump`, `reader_jump`, `git_gutter`, and `file_associations` (`docs/configuration.md`).
- File associations in the settings file: `file_associations` maps file patterns to a filetype, as VS Code's `files.associations` does. `*.rs.bak` highlights as Rust, `.env.*` as an env file, and `docs/**/*.txt` renders as Markdown with `m`. The reader, the finder's preview, and the changes screen all use it.
- Text files open in the reader with a line-number gutter, syntax colours for Scouter's languages, and the footer's line count, language, and scroll position. A file over 1 MiB shows its first 1 MiB.
- Reader motions: `j` and `k` scroll, `h` and `l` pan, `space` and `b` page, `Ctrl+d` and `Ctrl+u` move half a page, `gg` and `G` go to the top and bottom, `120G` goes to line 120, `{` and `}` move by paragraph, `zz` centres the view, and `w` wraps long lines. `Ctrl+o` and `Alt+i` go back and forward through the places opened. A scrollbar shows beside text taller than the reader.
- Images (PNG, JPG, GIF, WEBP) open scaled to fit the reader, with their size and format in the footer. A binary file shows a notice with its name and size, that fmc only displays UTF-8 text and images, and focus stays in the file list.
- Markdown files open rendered: headings, emphasis, links, lists, task boxes, quotes, tables, and highlighted code blocks. `m` shows the source with line numbers, and `/` and `{ }` say which key shows the source while the document is rendered.
- Product and design records (`PRODUCT.md`, `DESIGN.md`) and the plan for parity with Scouter.
- The file finder: `/` in the file list finds a file or folder by name. Every word must appear in its path, the matches come in Scouter's order, and up to 200 are listed with the total. Enter opens a file or reveals a folder, and Esc returns the tree to where it was.
- Text search in the finder: `Tab` searches the lines of the files, from two characters, with up to 200 hits. The highlighted hit shows in the reader as a preview, and Enter opens the file at its line with the search kept for `n` and `N`.
- The changes screen: `d` lists the files git reports as changed, with their status letters, index marks, and counts. The stacked diff below shows every file under its own header, with the counts, a stat bar, and the file's lines. Moving the list selection scrolls the diff to that file, `Enter` moves to the diff, `Tab` switches between them, and `w` wraps the lines. `d` or `q` goes back to browsing.
- The file tree, the open file, and the changes poll every 2 seconds (backing off to 30 seconds when a refresh is slow): a new or removed file appears, an open file re-reads when it changes, and a file that is gone says so. The changes list and diff follow edits without `r`.
- `r` reloads the changes while the screen shows: the list keeps its selection, and the diff is read again.
- The diff's keys: `s` shows side by side at 60 columns or more, `a` switches between one file and all files, `c` folds a file, `x` shows 12 lines of context, `n` `N` `]` `[` `p` move between hunks, and `o` opens the file at the hunk in the reader. A click on a file's header folds it.
- Mermaid fences in rendered Markdown are drawn as box diagrams at the reader's width: flowchart, sequence, class, state, gantt, and the other types the engine draws. A diagram too wide for the reader, or one fmc cannot draw, shows its source in a frame with the reason, and a resize draws it again at the new width.

### Changed

- In the rendered Markdown view each heading keeps its `#` marker (`#`, `##`, and so on) in the heading's colour, so the levels read in a terminal. A setext heading shows the marker of its level, and its underline row is no longer drawn.
- Below 60 columns fmc is a two-page browser: the file list fills the screen, a file opens full screen, and `q` or `Esc` goes back. The panels have the same border and two-row header as the medium tier, but there is no welcome card below 60 columns, and auto-preview does not run there.
- Below 60 columns the header shows a `‹` (a tap on it or on the title goes back), the page's title, and its tag, and the bottom bar is one row of equal words: `Find`, `Changes`, and `Keys` on the file list; `Back`, `Search`, and `Keys` on the reader. Each word runs the key it names.
- A row in the file list, the finder, or the changes list opens or selects on release, so a swipe over a row does nothing.
- `?` shows the key list as tables in the reader instead of a card over the panes. `?` or `Esc` puts the document back, with its scroll position and search.
- The file tree has no root row: the folder's files and folders start at the top, with no indent. `h` on a top-level row does nothing; `-` still opens the folder above.
- Enter on a file opens it and moves focus to the reader, as in Scouter; Tab moves it back. `l` and `→` only expand a folder, and a click on a file only selects it.
- A left click anywhere in a pane focuses it: the file list, the reader, the changes list, or the diff, headers and scrollbars included. A row's own action still happens, and a press in the reader still starts a text selection. Scouter moves the focus only from a row.
- Quit is `Ctrl+q` (or `Ctrl+c`); a plain `q` no longer quits. It closes the open document, and the welcome screen returns, or it closes the key list.
- In-file search: `/` opens the prompt, and typing shows the lines that contain the query, ignoring case, with every match marked. Enter and `↓` go to the next match, Shift+Enter and `↑` to the previous, `n` and `N` step on, and Esc keeps the matches for `n` and `N` before it clears them.
- Dragging across text selects it and keeps the selection on screen. `y` copies the selection and `Esc` clears it; Scouter copies as soon as the mouse is released.
- The changes screen opens showing one file's diff, where Scouter stacks every file at first. `a` stacks all files, and `a` again goes back to one.
- File errors read as Node's messages, such as `ENOENT: no such file or directory, scandir '<path>'`.
- A diff file folds when you release the click on its header, not when you press, so a swipe that starts on a header does not fold it.
- The line numbers stay in place when the reader pans sideways: only the text moves, and a wheel over the numbers scrolls the text beside them.
- At 60 to 79 columns a tap on a chevron, the gap after it, or the title beside it shows or hides the list. Below 80 columns the first tap on a row acts, with the reader focused; at 80 columns and wider it only focuses the list.
- Below 80 columns the `»` and `«` marks are not drawn. At 60 to 79 columns a `‹` at the start of the file list's title hides the list, and a `›` at the start of the reader's title shows it. The closing line of the welcome breaks at ` · ` instead of mid-phrase.
- The minimum supported Rust version is 1.92, up from 1.90, for the Mermaid engine.

### Fixed

- `→` and `←` on a folder no longer also pan the reader, and `Ctrl+/` that shows the sidebar focuses it.
- `j` and `k` in a folder with nothing to show no longer crash the app.
- Dragging a scrollbar's thumb scrolls its pane, and a click on the track jumps there, in the file list, the reader, the changes list, and the diff. The wheel over a bar scrolls one row, as Scouter's scroll boxes do.
- The `?` key list no longer lists `[ ]` twice.
- The hidden-sidebar hint `^/ sidebar` stays in the footer at phone widths.

[Unreleased]: https://github.com/riipandi/fmc/commits/main
