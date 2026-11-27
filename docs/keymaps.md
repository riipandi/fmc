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
| `a` | new file; a name ending in / makes a folder |
| `A` | new folder |
| `R` | duplicate the row under a new name |
| `r` | rename |
| `D` `Delete` | move the marked rows, or the row, to the Trash |
| `c` | copy the marked rows, or the row |
| `x` | cut the marked rows, or the row; a paste moves them |
| `v` | paste into the folder, or beside the file |
| `s` | mark the row and move down |
| `S` | clear the marks |

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

## Name field

The name field of a dialog (a new file or folder, a rename) holds one line. Its caret moves and its text is edited by word and by line. These keys are not in the `?` list.

| Keys | Action |
| --- | --- |
| type | insert at the caret |
| `Backspace` | delete the character before the caret |
| `Option+←` `Option+→` `Option+B` `Option+F` `Ctrl+←` `Ctrl+→` | move the caret by word |
| `Option+Backspace` `Ctrl+W` | delete the word before the caret |
| `Option+Delete` `Option+D` | delete the word after the caret |
| `Cmd+←` `Cmd+→` `Ctrl+A` `Ctrl+E` | move the caret to the start or the end of the line |
| `Cmd+Backspace` `Ctrl+U` | delete to the start of the line |
| `Ctrl+K` | delete to the end of the line |
| `←` `→` `Tab` | move the focus between the field and the buttons |
| `Enter` `Esc` | confirm; cancel |

The Option keys need the terminal to send Option as Alt. The Cmd keys need the terminal to pass Cmd on to fmc; a terminal that does neither sends what its own keymap maps the chord to, such as `Ctrl+A` for `Cmd+←`. The bare arrows move the focus between the field and the buttons, not the caret, so the caret moves with the keys above.

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
| `Esc` | clear selection or search, or back to the file list; in the list, drop a copy or cut, then the marks |
| `q` | close the open document or key list; back from the changes |
| `?` | this key list; ? or Esc closes it |
| `Ctrl+/` | show / hide the sidebar |
| `Ctrl+.` `.` | show / hide dotfiles in the tree and the finder |
| `Ctrl+,` | the Settings page; each change saves to settings.json |
| `:` `q` | quit |
| `^q` `^c` | quit |

`Ctrl+,` opens the [Settings page](usage.md#settings). Only a terminal that supports the kitty keyboard protocol can send `Ctrl+,`, and a terminal without it cannot open the page; the other keys still work. There is no other key for the page.

`:` opens a command line in the footer, as in Neovim. `:q`, `:q!`, `:qa`, `:qa!`, and `:quit` quit. `Esc`, or Enter on an empty line, cancels; any other command shows `Not a command`.

## Mouse

| Keys | Action |
| --- | --- |
| `Left` `click` | select file, or toggle folder |
| `Right` `click` | context menu for the row |
| `Hover` | select a context-menu row; highlight a dialog button |
| `Click` | focus the pane under the pointer |
| `Drag` `text` | select; y copies it |
| `Wheel` | scroll the pane under the pointer |
| `Drag` `edge` | resize the list |
| `Click` `«»` | show / hide the sidebar, from its mark |

Clicks and taps in the narrow layouts, the header chevrons, and the bottom bar's words are described in [Usage](usage.md#layouts-by-width). A click on a diff file's header folds it, and a row in the file list, the finder, or the changes list acts when the button is released, so a drag over a row does nothing. Dragging a scrollbar's thumb scrolls its pane, a click on the track jumps there, and the wheel over a bar scrolls one row.

Some keys work but are not in the lists above: `J` and `K` move five rows, Page Up and Page Down move a screen, Home and End go to the top and bottom, and `Shift+n` goes to the previous hunk on the changes screen.

## Changing the keys

fmc reads `keymap.json` when it starts, from `$XDG_CONFIG_HOME/fmc/keymap.json` or `~/.config/fmc/keymap.json` when `XDG_CONFIG_HOME` is unset, empty, or relative. fmc only reads the file, and a change applies the next time fmc starts. A missing file keeps the keys above. The `?` list and `fmc --help` show the keys the file sets.

The file is one JSON object. Each entry names a context and a key, then an action name, or `null` to unbind the default key in that context:

```json
{
  "everywhere.w": null,
  "reader.x": "toggle_wrap",
  "browse.g x": "open_settings"
}
```

Here `w` no longer wraps anywhere, `x` wraps in the reader, and `g x` in the tree opens the Settings page. Keys are separated by spaces for a sequence. A longer sequence that starts with a key makes that key wait for the next one; a key that cannot follow cancels the wait and is read on its own.

### Contexts

| Name | Where it applies |
| --- | --- |
| `everywhere` | every screen and focus; a prompt keeps the keys it reads |
| `browse` | the file tree, with the focus in the list |
| `reader` | the open file, with the focus in the reader |
| `changes` | the changes screen, in either focus |
| `changes_list` | the changes screen's list |
| `changes_reader` | the changes screen's diff |

### Key names

- A letter or a digit is itself: `g`, `7`. An uppercase letter is Shift with the letter: `G` is Shift+g.
- A symbol is itself: `?`, `:`, `/`, `.`, `[`, `{`.
- `ctrl-` and `alt-` add Ctrl or Alt: `ctrl-q`, `alt-i`, `ctrl-alt-q`. A Ctrl or Alt key ignores Shift.
- `shift-` marks Shift on a letter or a named key: `shift-down`.
- Named keys: `enter`, `escape`, `space`, `tab`, `backspace`, `delete`, `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`.
- `*-` before a key matches it with any modifiers, as the defaults do for `tab` and `escape`.

A key name fmc does not know, a context it does not have, or an action it does not have skips that entry with a notice.

### Notices

fmc shows each notice in the footer at start, and `fmc --help` prints them to the terminal's error stream. Each names the file and the line of its entry:

- `keymap.json line 3: "reader.ctrl-zz": "ctrl-zz" is not a key name.`
- `keymap.json line 2: "nowhere.j": names no context "nowhere".`
- `keymap.json line 4: "reader.x": names no action "scroll_sideways".`

A file that is not a JSON object, or is not JSON, keeps every key and gives one notice. A file over 1 MiB is not read.

### Action names

The names are fixed; a name that changes is a breaking change to the file. A default key listed below is the key the action has before a file changes it.

| Name | Default keys (file syntax) |
| --- | --- |
| `quit` | `ctrl-q` `ctrl-c` `ctrl-alt-q` `ctrl-alt-c` |
| `help` | `?` |
| `close` | `q` |
| `toggle_sidebar` | `ctrl-/` `ctrl-_` |
| `toggle_dotfiles` | `ctrl-.` `.` |
| `switch_pane` | `*-tab` |
| `escape` | `*-escape` |
| `resize_narrower` | `ctrl-left` |
| `resize_wider` | `ctrl-right` |
| `jump_back` | `ctrl-o` |
| `jump_forward` | `alt-i` |
| `toggle_wrap` | `w` |
| `toggle_markdown` | `m` |
| `toggle_changes` | `d` |
| `copy` | `y` |
| `move_down` | `j` `down` |
| `move_up` | `k` `up` |
| `move_down_five` | `shift-down` `J` |
| `move_up_five` | `shift-up` `K` |
| `move_page_down` | `pagedown` |
| `move_page_up` | `pageup` |
| `top` | `g` `home` |
| `bottom` | `G` `end` |
| `toggle` | `enter` |
| `expand` | `l` `L` `right` |
| `collapse` | `h` `H` `left` `backspace` |
| `go_up` | `-` |
| `toggle_preview` | `p` `P` |
| `jump_next` | `]` |
| `jump_previous` | `[` |
| `open_command` | `:` |
| `open_settings` | `ctrl-,` |
| `find` | `/` |
| `context_menu` | `space` |
| `new_file` | `a` |
| `new_folder` | `A` |
| `duplicate` | `R` |
| `rename` | `r` |
| `delete` | `D` `delete` |
| `copy_file` | `c` |
| `cut_file` | `x` |
| `paste_file` | `v` |
| `toggle_mark` | `s` |
| `clear_marks` | `S` |
| `toggle_fuzzy` | `ctrl-f` |
| `focus_diff` | `enter` `l` `L` `right` |
| `scroll_down` | `J` `j` `down` |
| `scroll_up` | `K` `k` `up` |
| `pan_left` | `H` `h` `left` |
| `pan_right` | `L` `l` `right` |
| `page_down` | `space` `pagedown` |
| `page_up` | `B` `b` `pageup` |
| `half_page_down` | `ctrl-d` |
| `half_page_up` | `ctrl-u` |
| `paragraph_previous` | `{` |
| `paragraph_next` | `}` |
| `centre` | none |
| `search` | `/` |
| `next_match` | `n` |
| `previous_match` | `N` `p` `P` |
| `toggle_split` | `s` `S` |
| `toggle_scope` | `a` `A` |
| `toggle_fold` | `c` `C` |
| `toggle_context` | `x` `X` |
| `reload` | `r` `R` |
| `open_changed` | `o` `O` |
| `next_hunk` | `]` `n` |
| `previous_hunk` | `[` `N` `p` `P` |

Count digits (`120G`) and `zz` are not in the file: the reader keeps them.
