# Usage

A tour of the screens, in the order you meet them. Every key is listed in [Keys](keymaps.md).

## Open a folder or a file

```sh
fmc            # the current folder
fmc <folder>   # a folder, as the root of the file list
fmc <file>     # a file: its folder opens, with the file selected
```

fmc opens with the file list on the left and the reader on the right. `?` shows every key, and `Ctrl+q` or `:q` quits.

## The panes

The file list shows the folder as a tree. Each file has a git mark when git reports a change: a status letter, and a dot on a folder that holds changes. Ignored entries are dimmed. A scrollbar appears on long lists and on long text.

`Tab` moves focus between the two panes, and a click focuses the pane under the pointer. `Ctrl+←` and `Ctrl+→` change the list's width two columns at a time, and dragging its edge does the same.

The footer names the pane and the file's line count, language, and position, then lists the keys that work here.

fmc follows the terminal's background and switches between a dark and a light palette when the background changes. The tree, the open file, and the changes refresh by themselves, about every two seconds, so a file that an editor or an agent changes shows up without a reload.

## Preview

With the file list focused, the file it selects shows in the reader after a short pause. The focus stays in the list, so you can move through a folder and read as you go. Selecting a folder leaves the reader as it is.

Enter opens the file and moves focus to the reader. `p` turns the preview off or on, and `auto_preview` in the [settings](configuration.md) sets where it starts. Preview is off in the narrow layout below 60 columns.

## Text

Text files show line numbers and syntax colours. `j` and `k` scroll, `gg` and `G` go to the top and bottom, `120G` goes to line 120, and `[` and `]` scroll by the `reader_jump` setting. `w` wraps long lines. `Ctrl+o` and `Alt+i` go back and forward through the places you opened.

A file over 1 MiB shows its first 1 MiB.

`/` searches the file. `Enter` goes to the next match, `n` and `N` step through the matches, and `Esc` ends the prompt and keeps them.

## Images and binary files

PNG, JPG, GIF, and WEBP files open scaled to fit the reader, with their size and format in the footer. Any other binary file shows a short notice with its name and size. fmc displays UTF-8 text and images only.

## Markdown

Markdown files open rendered: headings, emphasis, links, lists, task boxes, quotes, tables, and highlighted code blocks. Headings keep their `#` markers in the heading's colour, so the level shows without a bigger font. `m` switches to the source, with line numbers, and back.

A `mermaid` code fence is drawn as a box diagram at the reader's width. A diagram too wide for the reader, or one fmc cannot draw, shows its source in a frame with the reason.

`/` searches the rendered text as you read it, so `**Harry** Potter` matches `Harry Potter`. Every match is marked and the current one has its own colour. `n` and `N` step through them.

## Changes

`d` opens the changes screen: every file git reports as changed, with a status letter (`M` modified, `A` added, `D` deleted, `R` renamed, `T` type change, `U` conflict, `?` untracked), its index marks, and its line counts. The diff of the selected file shows on the right.

- `a` stacks every file's diff, and `a` again goes back to one file.
- `s` shows the diff side by side when the reader is 60 columns or wider.
- `c` folds a file, and `x` shows more context lines.
- `n`, `N`, `]`, and `[` move between hunks, and `o` opens the file at the current hunk in the reader.
- `r` reloads the list and the diff.

`d` or `q` returns to browsing.

## Git signs

Inside a git repository, the reader marks the lines that differ from `HEAD`. A strip left of the line numbers shows an added line in green and a changed line in yellow. A red `▁` or `▔` marks where lines were removed. The rendered Markdown view marks its changed blocks in the same way, beside each block. `git_gutter` in the settings turns both off.

## Find a file

`/` in the file list opens the finder. Its matches replace the tree until you press `Esc`, and the tree returns where it was.

Typing filters by name. By default (`substring`) every word of the query must appear in the path, ignoring case, and up to 200 matches are listed with the total. `Ctrl+F` switches to fuzzy matching: the whole query is one pattern whose letters must appear in order anywhere in the path, the best matches come first, and the matched letters are highlighted. `find_mode` in the settings sets the start. The footer names `fuzzy` while it is on, including before the first letter.

`Enter` opens the file, or reveals the folder that matched. `Tab` switches to search inside the files. From two characters on, it lists the matching lines, and the highlighted line shows in the reader. `Enter` opens that file at the line, and `n` and `N` keep stepping through the matches.

## Copy

Drag across text to select it. The selection stays on screen until you press `y`, which copies it, or `Esc`, which clears it. With nothing selected, `y` copies the path of the current file.

## Managing files

The file list creates, renames, copies, moves, and deletes files. Each action works on the selected row, inside the folder fmc opened.

- `a` creates a file. A name ending in `/` creates a folder, and `A` creates a folder.
- `r` renames the row. `R` duplicates it: a name dialog opens with the name a paste would give (`sophie-neveu copy.md`), and Return copies the row into the same folder under the name you type. A copy of a folder keeps its links as links.
- `c` copies the marked rows, or the selected row when none is marked, and `x` cuts them. `v` pastes them into the selected folder, or beside the selected file. A cut row is dimmed until it is pasted or the cut is dropped with `Esc`. Copy, Cut, and Paste in the menu say how many rows they take, as `Copy 3 items`.
- `D` or `Delete` moves the marked rows, or the row when none is marked, to the Trash, after a dialog that names them (up to five, then the count of the rest). Every entry is checked before the first one moves: one that was replaced since the dialog opened stops the whole delete. A link cannot go to the Trash, since Finder refuses it: `D` on a link, or on marks that include one, says so and moves nothing. The dialog selects Cancel, so Return keeps the file. Finder's Put Back restores a trashed item. The first delete asks macOS for permission to let fmc control Finder.
- `Space`, or a right-click on a row, opens a menu with the same actions, and Duplicate. Paste is dimmed until something is copied or cut.
- `s` marks the row and moves down one row, and `S` clears every mark. A marked row shows `✓`, and the footer counts the marks. Marks stay while you scroll or fold a folder, and an entry that leaves the folder takes its mark with it. `Esc` drops a copy or cut first, then the marks. The finder and the changes screen have no marks.

fmc never overwrites. A name that is taken is refused with a notice, and the dialog stays open for another name. A paste into a folder that has the same name takes `copy`, then `copy 2`, before the extension. A copy of a folder keeps its links as links, and fmc does not follow them. A paste runs in the background, and the footer says `copying` or `moving` while it runs, with `copying 2 of 5…` for several rows. The rows go one at a time: the first error stops the paste and the footer says how many landed. A second paste is refused until the first one finishes.

There is no undo beyond the Trash. `r`, `R`, and `a` work on one row; the copy, cut, paste, and delete keys work on every marked row.

## Help

`?` shows the key list as tables in the reader, in place of the document. `?` or `Esc` brings the document back, with its scroll position and search.

## Layouts by width

fmc changes its layout with the width of the terminal.

- **80 columns and wider.** The file list and the reader sit side by side. A `«` on the list's border hides the list, and a `»` on the reader's left border shows it again. `Ctrl+/` does the same.
- **60 to 79 columns.** The list sits beside the reader and hides while a document is open. A `›` at the start of the reader's title shows the list, and a `‹` at the start of the list's title hides it. `Ctrl+/` works here too.
- **Below 60 columns.** The two panes become two pages. The file list fills the screen, and Enter opens a file full screen. The `‹` in the header goes back, and so do `q` and `Esc`. The bottom bar shows the page's keys as words, and each word is a tap: `Find`, `Changes`, and `Keys` on the list; `Back`, `Search`, and `Keys` on the reader. A tap acts when the finger lifts without moving, so a swipe never opens a file. There is no welcome screen.
