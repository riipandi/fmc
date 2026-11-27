# Keys

Press `?` inside fmc to see this list in the reader, or run `fmc --help` for the same list in the terminal. The screens are explained in [Usage](usage.md).

## Browse

| Keys | Action |
| --- | --- |
| `j` `k` `↑` `↓` | move |
| `Enter` | open file, toggle folder |
| `l` `→` | expand a folder |
| `h` `←` `Backspace` | collapse a folder or go back |
| `/` | find a file by name; Tab: text in files; Ctrl+F: fuzzy on or off |
| `-` | open the folder above |
| `p` | auto-preview on or off |
| `[` `]` | jump the list or reader |
| `gg` `G` | top / bottom |
| `Ctrl+←` `→` | resize the list, or drag its edge |
| `space` | context menu for the row |

The file the list selects shows in the reader after a short pause, and the focus stays in the list. `p` turns that preview off or on for the session, and `auto_preview` in the settings sets the start. In the list, `[` and `]` move the selection by `sidebar_jump` rows and scroll the reader by `reader_jump` lines. On the changes screen they step through the hunks. See [Configuration](configuration.md).

## Find a file

`/` in the file list opens the finder. Its matches replace the tree until it closes.

| Keys | Action |
| --- | --- |
| type | filter by name: every word must appear in the path (or, when fuzzy is on, the whole query in order); in text search, the lines that contain the text |
| `Ctrl+F` | flip name matching between substring and fuzzy for this run (a restart returns to `find_mode`); the footer says `fuzzy` when it is on |
| `Backspace` | delete a character |
| `Tab` | switch between file names and text in files |
| `↑` `↓` | move the highlight; on a text hit the reader shows its line in a preview |
| `Enter` | open the file, at the line for a text hit; reveal the folder the query found; with no text, open or toggle the row |
| `Esc` | close the finder; the tree is back where it was |

Text search starts at two characters and lists at most 200 hits, ten from one file.

## Reader

| Keys | Action |
| --- | --- |
| `j` `k` | scroll |
| `/` | search the file; Enter next, Esc keeps it for n N |
| `n` `N` `p` | next / previous match |
| `space` `b` `^d` `^u` | page / half a page |
| `gg` `G` | top / bottom |
| `120G` | go to line 120 |
| `{` `}` | previous / next paragraph |
| `zz` | centre the match, or the middle line |
| `w` | wrap long lines |
| `m` | Markdown: rendered / source |

## Changes

| Keys | Action |
| --- | --- |
| `d` | show / hide the changes screen |
| `n` `N` `p` | next / previous hunk |
| `a` | all files / one file |
| `s` | split / unified |
| `c` | collapse / expand file diff |
| `x` | more / normal context lines |
| `r` | reload |
| `o` | open the file at the current hunk |

## Everywhere

| Keys | Action |
| --- | --- |
| `Tab` | switch panes |
| `y` | copy selection, or the current path |
| `Ctrl+o` `Alt+i` | jump back / forward through opened places |
| `Esc` | clear selection or search, or back to the file list |
| `q` | close the open document or key list; back from the changes |
| `?` | this key list; ? or Esc closes it |
| `Ctrl+/` | show / hide the sidebar |
| `Ctrl+.` `.` | show / hide dotfiles in the tree and the finder |
| `:` `q` | quit |
| `^q` `^c` | quit |

`:` opens a command line in the footer, as in Neovim. `:q`, `:q!`, `:qa`, `:qa!`, and `:quit` quit. `Esc`, or Enter on an empty line, cancels; any other command shows `Not a command`.

## Mouse

| Keys | Action |
| --- | --- |
| `Left` `click` | select file, or toggle folder |
| `Click` | focus the pane under the pointer |
| `Drag` `text` | select; y copies it |
| `Wheel` | scroll the pane under the pointer |
| `Drag` `edge` | resize the list |
| `Click` `«»` | show / hide the sidebar, from its mark |

Clicks and taps in the narrow layouts, the header chevrons, and the bottom bar's words are described in [Usage](usage.md#layouts-by-width). A click on a diff file's header folds it, and a row in the file list, the finder, or the changes list acts when the button is released, so a drag over a row does nothing. Dragging a scrollbar's thumb scrolls its pane, a click on the track jumps there, and the wheel over a bar scrolls one row.

Some keys work but are not in the lists above: `J` and `K` move five rows, Page Up and Page Down move a screen, Home and End go to the top and bottom, and `Shift+n` goes to the previous hunk on the changes screen.
