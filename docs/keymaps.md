# Keys

Press `?` inside fmc to see this list in the reader. The screens are explained in [Usage](usage.md).

## Browse

| Action | Keys |
| --- | --- |
| move | `j` `k` `↑` `↓` |
| top / bottom | `gg` `G` |
| jump the list or reader | `[` `]` |
| open file, toggle folder | `Enter` |
| expand a folder | `l` `→` |
| collapse a folder or go back | `h` `←` `Backspace` |
| open the folder above | `-` |
| find a file by name; Tab: text in files; Ctrl+F: fuzzy on or off | `/` |
| auto-preview on or off | `p` |
| resize the list, or drag its edge | `Ctrl+←` `→` |

The file the list selects shows in the reader after a short pause, and the focus stays in the list. `p` turns that preview off or on for the session, and `auto_preview` in the settings sets the start. In the list, `[` and `]` move the selection by `sidebar_jump` rows and scroll the reader by `reader_jump` lines. On the changes screen they step through the hunks. See [Configuration](configuration.md).

## Files

The file acts work on the selected row, or on every marked row when some are marked.

| Action | Keys |
| --- | --- |
| context menu for the row | `space` |
| new file; a name ending in / makes a folder | `a` |
| new folder | `A` |
| rename | `r` |
| duplicate the row under a new name | `R` |
| copy the marked rows, or the row | `c` |
| cut the marked rows, or the row; a paste moves them | `x` |
| paste into the folder, or beside the file | `v` |
| move the marked rows, or the row, to the Trash | `D` `Delete` |
| mark the row and move down | `s` |
| clear the marks | `S` |

## Find a file

`/` in the file list opens the finder. Its matches replace the tree until it closes.

| Action | Keys |
| --- | --- |
| filter by name: every word must appear in the path (or, when fuzzy is on, the whole query in order); in text search, the lines that contain the text | type |
| flip name matching between substring and fuzzy for this run (a restart returns to `find_mode`); the footer says `fuzzy` when it is on | `Ctrl+F` |
| delete a character | `Backspace` |
| switch between file names and text in files | `Tab` |
| move the highlight; on a text hit the reader shows its line in a preview | `↑` `↓` |
| open the file, at the line for a text hit; reveal the folder the query found; with no text, open or toggle the row | `Enter` |
| close the finder; the tree is back where it was | `Esc` |

Text search starts at two characters and lists at most 200 hits, ten from one file.

## Name field

The name field of a dialog (a new file or folder, a rename) holds one line. Its caret moves and its text is edited by word and by line. These keys are not in the `?` list.

| Action | Keys |
| --- | --- |
| insert at the caret | type |
| delete the character before the caret | `Backspace` |
| move the caret by word | `Option+←` `Option+→` `Option+B` `Option+F` `Ctrl+←` `Ctrl+→` |
| delete the word before the caret | `Option+Backspace` `Ctrl+W` |
| delete the word after the caret | `Option+Delete` `Option+D` |
| move the caret to the start or the end of the line | `Cmd+←` `Cmd+→` `Ctrl+A` `Ctrl+E` |
| delete to the start of the line | `Cmd+Backspace` `Ctrl+U` |
| delete to the end of the line | `Ctrl+K` |
| move the focus between the field and the buttons | `←` `→` `Tab` |
| confirm; cancel | `Enter` `Esc` |

The Option keys need the terminal to send Option as Alt. The Cmd keys need the terminal to pass Cmd on to fmc; a terminal that does neither sends what its own keymap maps the chord to, such as `Ctrl+A` for `Cmd+←`. The bare arrows move the focus between the field and the buttons, not the caret, so the caret moves with the keys above.

## Reader

| Action | Keys |
| --- | --- |
| scroll | `j` `k` |
| page / half a page | `space` `b` `^d` `^u` |
| top / bottom | `gg` `G` |
| go to line 120 | `120G` |
| previous / next paragraph | `{` `}` |
| search the file; Enter next, Esc keeps it for n N | `/` |
| next / previous match | `n` `N` `p` |
| centre the match, or the middle line | `zz` |
| wrap long lines | `w` |
| Markdown: rendered / source | `m` |

## Changes

| Action | Keys |
| --- | --- |
| show / hide the changes screen | `d` |
| next / previous hunk | `n` `N` `p` |
| open the file at the current hunk | `o` |
| all files / one file | `a` |
| split / unified | `s` |
| collapse / expand file diff | `c` |
| more / normal context lines | `x` |
| reload | `r` |

## Everywhere

| Action | Keys |
| --- | --- |
| switch panes | `Tab` |
| jump back / forward through opened places | `Ctrl+o` `Alt+i` |
| copy selection, or the current path | `y` |
| clear selection or search, or back to the file list; in the list, drop a copy or cut, then the marks | `Esc` |
| close the open document or key list; back from the changes | `q` |
| this key list; ? or Esc closes it | `?` |
| show / hide the sidebar | `Ctrl+/` |
| show / hide dotfiles in the tree and the finder | `Ctrl+.` `.` |
| quit | `:` `q` |
| quit | `^q` `^c` |
| select file, or toggle folder | `Left` `click` |
| focus the pane under the pointer | `Click` |
| scroll the pane under the pointer | `Wheel` |
| select; y copies it | `Drag` `text` |
| context menu for the row | `Right` `click` |
| select a context-menu row; highlight a dialog button | `Hover` |
| resize the list | `Drag` `edge` |
| show / hide the sidebar, from its mark | `Click` `«»` |

`:` opens a command line in the footer, as in Neovim. `:q`, `:q!`, `:qa`, `:qa!`, and `:quit` quit. `Esc`, or Enter on an empty line, cancels; any other command shows `Not a command`.

## Mouse

| Action | Keys |
| --- | --- |
| select file, or toggle folder | `Left` `click` |
| focus the pane under the pointer | `Click` |
| scroll the pane under the pointer | `Wheel` |
| select; y copies it | `Drag` `text` |
| context menu for the row | `Right` `click` |
| select a context-menu row; highlight a dialog button | `Hover` |
| resize the list | `Drag` `edge` |
| show / hide the sidebar, from its mark | `Click` `«»` |

Clicks and taps in the narrow layouts, the header chevrons, and the bottom bar's words are described in [Usage](usage.md#layouts-by-width). A click on a diff file's header folds it, and a row in the file list, the finder, or the changes list acts when the button is released, so a drag over a row does nothing. Dragging a scrollbar's thumb scrolls its pane, a click on the track jumps there, and the wheel over a bar scrolls one row.

Some keys work but are not in the lists above: `J` and `K` move five rows, Page Up and Page Down move a screen, Home and End go to the top and bottom, and `Shift+n` goes to the previous hunk on the changes screen.
