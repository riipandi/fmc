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
| `p` | auto-preview on or off |
| `[` `]` | jump the list or reader |
| `gg` `G` | top / bottom |
| `Ctrl+←` `→` | resize the list, or drag its edge |

With the list focused, the file it selects shows in the reader after a short pause, without moving the focus (auto-preview). A folder selection leaves the reader as it is. `p` turns auto-preview off or on for this session, and `auto_preview` in the settings file sets the start (`configuration.md`). `]` and `[` move the list by `sidebar_jump` rows, and scroll the reader by `reader_jump` lines; on the changes screen they step through the hunks.

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
| `:` `q` | quit |
| `^q` `^c` | quit |

`:` opens a command line in the footer, as in Neovim. `:q`, `:q!`, `:qa`, `:qa!`, and `:quit` quit; Esc or an empty Enter cancels; any other command shows `Not a command`.

## Mouse

| Keys | Action |
| --- | --- |
| `Left` `click` | select file, or toggle folder |
| `Click` | focus the pane under the pointer |
| `Drag` `text` | select; y copies it |
| `Wheel` | scroll the pane under the pointer |
| `Drag` `edge` | resize the list |
| `Click` `«»` | show / hide the sidebar, from its mark |
| `Click` header | fold / unfold the file, on release; a drag does not |
| `Click` `≡` box, below 80 columns | show the hidden file list and focus it |
| `Click` any cell of the reader header, below 80 columns | show the hidden file list and focus it |
| `Click` `✕` box, below 80 columns | hide the file list |
| `Click` any cell of the list header, below 60 columns | hide the file list drawer |
| `Click` file, below 60 columns | open the file in the reader |
| `Click` a footer hint, below 80 columns | press the hint's key (`^q` has no tap) |
| `Click` `Copy`, below 80 columns | copy the selection, as `y` does |

The `«` and `»` marks are drawn at 80 columns and wider only; below 80 the `≡` and `✕` boxes in the headers do their work. A click on the file list while another pane has the focus only moves the focus there; the next click selects the row. At 60 columns and wider a click selects a file but does not open it: Enter does. Below 60 columns the file list is a full-width drawer over the reader, and a focus that reaches the reader (Enter, Tab, or a tap on a file) closes it. While the drawer covers the reader, moving through the list does not preview a file; Enter or a tap on a file opens it. On the welcome screen below 60 columns the text shows full width and the list starts hidden: the `≡` box (or a tap on the reader's header row) opens the list as the drawer, and the `✕` box on the list's header hides it. While the list is hidden the welcome says `Tap ≡ to browse files.`. The `✕` box shows beside a document between 60 and 79 columns too, so the list can be hidden by touch there.

Below 80 columns the footer keeps the keys that leave the current state first (`Copy` while text is selected, `Esc`, `^/`, `q`), then `?`, `/`, `d`, `Enter`, and `Tab`, and a row that is too short drops the rest from its end. At 80 columns and wider the footer keeps Scouter's order.

Some keys work but are not in the list: `J` and `K` move five rows, Page Up and Page Down move a screen, Home and End go to the top and bottom, `[` and `]` step through hunks, and `Shift+n` goes to the previous hunk on the changes screen. Dragging a scrollbar's thumb scrolls its pane, a click on the track jumps to that row, and the wheel over a bar scrolls one row.
