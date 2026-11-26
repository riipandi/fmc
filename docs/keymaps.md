# Keys

Press `?` inside fmc to see this list as tables in the reader, or run `fmc --help`.

## Browse

| Keys | Action |
| --- | --- |
| `j` `k` `↑` `↓` | move |
| `Enter` | open file, toggle folder |
| `l` `→` | expand a folder |
| `h` `←` `Backspace` | collapse a folder or go back |
| `/` | find a file by name; Tab: text in files |
| `-` | open the folder above |
| `gg` `G` | top / bottom |
| `Ctrl+←` `→` | resize the list, or drag its edge |

## Find a file

`/` in the file list opens the finder. Its matches take the place of the tree until it closes.

| Keys | Action |
| --- | --- |
| type | filter by name: every word must appear in the path; in text search, the lines that contain the text |
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
| `^q` `^c` | quit |

## Mouse

| Keys | Action |
| --- | --- |
| `Left` `click` | select file, or toggle folder |
| `Click` | focus the pane under the pointer |
| `Drag` `text` | select; y copies it |
| `Wheel` | scroll the pane under the pointer |
| `Drag` `edge` | resize the list |
| `Click` `«»` | show / hide the sidebar, from its mark |

A click on the file list while another pane has the focus only moves the focus there; the next click selects the row. A click selects a file but does not open it: Enter does.

Some keys work but are not in the list: `J` and `K` move five rows, Page Up and Page Down move a screen, Home and End go to the top and bottom, `[` and `]` step through hunks, and `Shift+n` goes to the previous hunk on the changes screen. Dragging a scrollbar's thumb scrolls its pane, a click on the track jumps to that row, and the wheel over a bar scrolls one row.
