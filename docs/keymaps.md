# Keys

Press `?` inside fmc to see this list as tables in the reader, or run `fmc --help`.

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

With the list focused, the file it selects shows in the reader after a short pause, without moving the focus (auto-preview). A folder selection leaves the reader as it is. `p` turns auto-preview off or on for this session, and `auto_preview` in the settings file sets the start (`configuration.md`). `]` and `[` move the list by `sidebar_jump` rows, and scroll the reader by `reader_jump` lines; on the changes screen they step through the hunks.

## Find a file

`/` in the file list opens the finder. Its matches take the place of the tree until it closes.

| Keys | Action |
| --- | --- |
| type | filter by name: every word must appear in the path (or, when fuzzy is on, the whole query in order); in text search, the lines that contain the text |
| `Ctrl+F` | flip name matching between substring and fuzzy for this session; the footer says `fuzzy` when it is on |
| `Backspace` | delete a character |
| `Tab` | switch between file names and text in files |
| `↑` `↓` | move the highlight; on a text hit the reader shows its line in a preview |
| `Enter` | open the file, at the line for a text hit; reveal the folder the query found; with no text, open or toggle the row |
| `Esc` | close the finder; the tree is back where it was |

Text search starts at two characters and lists at most 200 hits, ten from one file.

Fuzzy matching (`find_mode` in the settings file, `configuration.md`) takes the query as one needle, spaces included, and matches its letters in order anywhere in the path, ignoring case; the best matches come first, and the matched letters are drawn in the match colour.

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
| `Click` row (file list, finder, changes) | open or select, on release; a drag or swipe over the row does nothing |
| `Click` the `›` or the title beside it, at the reader header, at 60 to 79 columns | show the hidden file list and focus it |
| `Click` the `‹` or the title beside it, at the list header, at 60 to 79 columns | hide the file list |
| `Click` file, below 60 columns | open the file: the reader page takes the screen |
| `q` or `Esc`, on a page below 60 columns | go back a page (Reader, Keys, Changes, Find) |
| `Click` the `‹` or the title beside it, below 60 columns, on a page other than Files | go back a page, as `q` does |
| `Click` a bottom-bar word, below 60 columns | press its key: `Find` is `/`, `Changes` is `d`, `Keys` is `?`, `Back` is `q`, `Search` is `/`, `All` or `One` is `a`, `Split` is `s`, `Copy` is `y`, and `Cancel` is `Esc` |
| `Click` a footer hint, below 80 columns | press the hint's key (`^q` has no tap) |
| `Click` `Copy`, below 80 columns | copy the selection, as `y` does |

The `«` and `»` marks are drawn at 80 columns and wider only; below 80 the `‹` and `›` chevrons at the start of the header titles do their work, and the title beside each does the same. Below 60 columns the `‹` at the start of a page's header (every page but Files) goes back, and the title beside it does the same. At 80 columns and wider, a click on the file list while another pane has the focus only moves the focus there; the next click selects the row. Below 80 columns the first tap on a row acts, since a phone shows no focus to aim with. At 60 columns and wider a click selects a file but does not open it: Enter does. Below 60 columns the list and the reader are pages: the list fills the screen, a file opens the reader page, and `q`, `Esc`, or Tab goes back to the list. There is no welcome screen and no preview below 60 columns. Between 60 and 79 columns the `‹` chevron hides the list beside a document, so it can be hidden by touch there.

From 60 to 79 columns the footer keeps the keys that leave the current state first (`Copy` while text is selected, `Esc`, `^/`, `q`), then `?`, `/`, `d`, `Enter`, and `Tab`, and a row that is too short drops the rest from its end. At 80 columns and wider the footer keeps Scouter's order. Below 60 columns the footer is one row of words, and each word is a tap; an open prompt or a notice takes that row.

Some keys work but are not in the list: `J` and `K` move five rows, Page Up and Page Down move a screen, Home and End go to the top and bottom, `[` and `]` step through hunks, and `Shift+n` goes to the previous hunk on the changes screen. Dragging a scrollbar's thumb scrolls its pane, a click on the track jumps to that row, and the wheel over a bar scrolls one row.
