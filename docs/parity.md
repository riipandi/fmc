# Differences from Scouter

fmc is a port of [Scouter](https://github.com/indrazm/scouter), a read-only terminal file browser. It aims to look and behave the same, except for the differences on this page. Each one was chosen on purpose. "Scouter" in this page means the baseline: v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

## Name and command line

- fmc shows its own name and version where Scouter shows `scouter`: the welcome screen, `--help`, `--version`, and error messages.
- The welcome screen's tagline says `a quiet terminal file manager`, and its closing line says `Space opens the menu.`. Scouter says `a quiet, read-only file viewer` and `No editing. Just reading.`, because fmc now writes files (`architecture.md`, "the welcome screen names fmc a file manager").
- There is no `--upgrade`. To update fmc, install a new build (see [Install](install.md)).

## Keys

- `Ctrl+q` quits. A plain `q` closes the open document, so the welcome screen returns, or it closes the key list. On the changes screen `q` goes back to browsing, as `d` does. `Ctrl+c` still quits.
- `:q` then Enter quits too, as in Neovim. `:q!`, `:qa`, and `:quit` work as well, and `Esc` cancels the `:` prompt.
- Only Enter opens a file, and it moves the focus to the reader. `l` and `→` only expand folders; in Scouter they also open a file.
- `]` and `[` move the list selection by two rows and scroll the reader by five lines. Both amounts are set in the [settings](configuration.md). Scouter uses them only on the changes screen.
- `Ctrl+,` opens the Settings page, which fmc adds. Scouter has no settings page, and its decision record 0001 keeps it read-only and without configuration (`scouter:docs/decision/0001-read-only-and-minimal.md:7`). Only a terminal with the kitty keyboard protocol can send `Ctrl+,`, so a terminal without it cannot open the page.
- `Ctrl+/` shows or hides the sidebar. `Ctrl+.` shows or hides dotfiles in the tree and the finder, and `.` does the same for terminals that cannot send `Ctrl+.`, such as Terminal.app. Dotfiles are shown by default, as in Scouter. Scouter has none of these keys.
- `Ctrl+F` in the finder switches name matching to fuzzy (see Finder).
- The footer at 80 columns can drop one more hint than Scouter's does, on the changes screen `a one file`, because `^q quit` is one column wider than `q quit`. The key list always shows every key.
- The keys can be changed in `keymap.json`, which is read at start (plan-00015). Scouter's keys are fixed (`scouter:src/core/keymap.ts:3-55`). The defaults are Scouter's keys with the differences above, and `architecture.md` records the file ("Approved deviation: the keys can be changed in keymap.json"). The file is described in [Keys](keymaps.md#changing-the-keys).

## Mouse and focus

- A click anywhere in a pane (the file list, the reader, the changes list, or the diff) moves the focus there. In Scouter only a click on a row does that.
- A row in the file list, the finder, or the changes list acts when you release the click, not when you press it. A drag or swipe over a row does nothing. Scouter acts on the press.
- A diff file folds when you release the click on its header, so a swipe that starts on a header does not fold it. Scouter folds on the press.
- A click on a changed file in the changes list selects it and shows its diff, and the list keeps the focus, as the file tree does. Enter moves the focus to the diff. Scouter moves the focus on the click.
- At 80 columns and wider, while another pane has the focus, the first click on the file list only moves the focus there, and the next click selects the row. Below 80 columns the first tap on a row acts.
- Dragging across text selects it and keeps the selection on screen. Nothing is copied until you press `y`, and `Esc` clears the selection. Scouter copies as soon as the mouse is released. With no selection, `y` copies the path, as in Scouter.

## Layout

- Below 80 columns the sidebar hides once a file is open, and the reader takes the full width. The welcome screen keeps the sidebar from 60 columns up. Scouter always shows the sidebar.
- At 80 columns and wider, a mark on the border shows or hides the sidebar with a click: `«` on the sidebar's border while it is shown, and `»` on the reader's left border while it is hidden. Below 80 columns those marks are not drawn. The `‹` and `›` chevrons at the start of the header titles take their place from 60 to 79 columns, and a tap on the title beside a chevron does the same. A manual choice holds until the width crosses 80 again.
- The sidebar's visibility is not saved across runs. The sidebar's width, wrap, the changes screen's options, the dotfile choice, and the finder's matching are kept in `meta/state.json` ([Configuration](configuration.md#remembered-state)); Scouter keeps them for the session only.
- The footer's left side shows the browsed root, with `~` for the home folder, and the git branch as `⎇ name` (a detached `HEAD` shows `⎇` and its short hash). It shows on the files, reader, Settings, and changes screens from 60 columns up, and the root is cut from the left before the branch. Scouter shows the selected file's or folder's path there, which the reader's header already names (`architecture.md`, "Approved deviation: the footer shows the browsed root and the git branch"). Below 60 columns the bottom bar keeps its words, and neither the root nor the branch is shown there. The root's last name and the branch are drawn in a muted colour, where Scouter draws the path's last name in the normal text colour.
- Below 60 columns (a phone), fmc is a two-page browser. The file list fills the screen, opening a file shows it full screen, and the header's `‹` (or `q` or `Esc`) goes back. The bottom bar has large buttons with words: `Find`, `Changes`, and `Keys` on the list; `Back`, `Search`, and `Keys` on the reader. There is no sidebar and no welcome card. From 60 to 74 columns the welcome keeps its wordmark, tagline, and prompt, and drops its card. A tap acts when the finger lifts without moving, so a swipe never opens a file.
- From 60 to 79 columns the list sits beside the reader and hides once a file is open. `‹` and `›` in the titles hide and show it.

## Settings

- fmc reads a settings file and writes it from the Settings page (see [Configuration](configuration.md)). Each change saves at once. Scouter has no settings file, and the values fmc reads from it are fixed in Scouter.
- The state fmc keeps across runs is in `meta/state.json`, which the settings file does not hold. Scouter writes no state at all.
- fmc writes a JSON Schema for the settings file to `meta/config.schema.json` in its config folder at each start, so an editor can complete and check `settings.json` ([Configuration](configuration.md), "Editor completion"). Scouter has no configuration file and no schema.
- Scouter has one fixed palette pair, dark and light, and the terminal picks between them. fmc offers a catalog of 16 themes and a theme mode, chosen in the settings file; the default is Scouter's own pair ([Configuration](configuration.md), "Themes").

## Reader

- A Markdown file that starts with YAML front matter (`---`) or TOML front matter (`+++`) shows it as a highlighted code block in the rendered view. Scouter draws the `---` as a rule and the fields as a paragraph, and the closing `---` as a heading. The source view (`m`) shows the file as it is.
- Moving through the file list previews each file in the reader while the list keeps the focus (auto-preview). Press `p` in the list to turn it off or on, or set `auto_preview` in the settings file. Scouter shows a file only when you open it.
- Highlighting follows `file_associations` in the settings file. A pattern can name the filetype a file is shown as: `*.rs.bak` as Rust, `.env.*` as an env file, or `docs/**/*.txt` as Markdown. `plaintext` shows a file with no highlighting. Scouter reads the extension only.
- Languages beyond Scouter's are coloured: astro, diff, elixir, heex, hcl, jsdoc, json5, just, lua, query, regex, sql, svelte, vim, vimdoc, and xml. `.heex`, `.eex`, and `.leex` (Elixir templates) are read as heex, and `.hcl`, `.tf`, and `.tfvars` (Terraform) as hcl; `.json5` as json5; `.scm` (a tree-sitter query) as query; and `justfile` as just. The pattern argument of `find`, `match`, `gmatch`, and `gsub` in a Lua file is coloured as a Lua pattern. Scouter shows these strings plain. Scouter shows these files plain. The code inside some of them is coloured as its own language: a Svelte or Astro `<script>` and `<style>` block, an Astro frontmatter, the markup of a HEEx template, and the HTML of an Elixir `~H` sigil. Scouter shows these plain.
- HTML's `<script>` and `<style>` blocks are coloured as JavaScript and CSS. Scouter shows them plain.
- In JavaScript, TypeScript, and Python, a regex is coloured as a pattern: a regex literal, and the string of Python's `re` functions such as `re.compile(r"...")`. A `/** */` comment is coloured by its JSDoc tags, so `@param` reads as a keyword. A `//` comment and an ordinary string are unchanged. Scouter shows these plain. A fenced block in one of these languages in Markdown is coloured the same way.
- Line numbers stay in place when you scroll sideways. In Scouter they scroll away with the text.
- In a file inside a git repository, the reader shows a one-column strip left of the line numbers: a green bar for an added line, a yellow bar for a changed line, and a red `▁` on the line above a deletion (`▔` on line 1 for a deletion above it). The strip keeps its column in a file with no changes, so the numbers do not move. `git_gutter` in the settings file turns it off. Scouter's code view has no change signs.
- A heading in the rendered Markdown view keeps its `#` marker (`#`, `##`, and so on) in the heading's colour, so the levels read without a larger font. Scouter hides the marker. Emphasis, code, and link markers stay hidden. A setext heading (a title underlined with `===` or `---`) shows the marker of its level, and its underline is not drawn.
- In a rendered Markdown file inside a git repository, each block with an added or changed line gets a bar at its left edge: green for added, yellow for changed. A removed line shows a `▁` under the block above it, or a `▔` above the first line. `git_gutter` controls these as well. Scouter's rendered view has no change signs.
- `/` searches the rendered Markdown view. Each occurrence is marked in the text the view draws, so `**Harry** Potter` matches `Harry Potter`. The current match has its own colour, and `n`, `N`, and `p` step through them. The footer counts them as `k of N`. A fence in a language fmc does not highlight is counted and reached, but not marked. Scouter refuses `/` in a rendered Markdown file.
- A `mermaid` fence in the rendered view is drawn as a box diagram at the reader's width. A diagram too wide for the reader, or one fmc cannot draw, shows its source in a frame with the reason. Scouter shows the fence as plain code. The diagram sits on a panel tone, `fallback-background`, across the reader's width. The source view (`m`) keeps the fence as text, and edge labels take the text colour rather than the muted one.
- The footer of a binary file shows only its size and `binary`. Scouter can keep the position of the document opened before it.

## Changes screen

- The changes screen opens showing one file's diff. Press `a` to stack all files, as Scouter shows them at first.
- After a change that moves the diff (folding with `c`, or reloading with `r`), the footer and the scrollbar show the new position at once. Scouter can show the old values until you scroll.
- After switching between one file and all files (`a`), or changing the context (`x`), the selected file stays at the top. Scouter can land elsewhere.
- A diff whose lines fit the pane does not pan: its header's counts and stat bar sit at the right edge, and `h` and `l` do nothing. Scouter's diff is wider than the pane by its gutter, so it pans and hides the bar until you pan. A longer line still pans, by its own overflow.

## Finder

- `Ctrl+F` in the finder switches the name matching between substring and fuzzy, and the choice is kept across runs in `meta/state.json`; `find_mode` in the settings file sets the start until the state holds one. The default, `substring`, is Scouter's match. With fuzzy on, the query is one pattern matched in order anywhere in the path, the rows rank by how well they match, and the matched letters are highlighted. Scouter has no fuzzy match.

## File tree

- The tree has no root row. The folder's own files and folders start at the top, with no indent, and the folder's name stays in the sidebar's title. Scouter draws the folder as the first row, which collapses like any other folder.
- Opening a file inside a collapsed folder expands every folder down to the file, so the file's row is visible. Scouter can leave the file's own folder collapsed.
- When fmc starts on a file it cannot show (one that is not UTF-8 text), the tree still lists the file's folder. Scouter leaves the tree empty.

## File operations

- fmc creates, renames, copies, moves, and deletes files from the file list, and it has a context menu for them (`Space`, or a right-click). Scouter is read-only and has none of this. Writes stay inside the folder fmc opened, links are never followed, and a delete goes to the Trash, except a link, which Finder will not trash, so fmc refuses it. Marks (`s`) and the bulk copy, cut, and delete over them are fmc's own (`architecture.md`, "The file-manager operations are the one write path" and "Marks are a set of paths in the browser").

## Opening in other apps

- fmc opens a file or folder in another app. `o` uses the app macOS picks for it, `O` lists the apps that open it, `e` opens the folder in the editor set in the settings, and `E` shows the row in Finder. Scouter has none of these: it spawns only `pbcopy` and `git`. Every launch goes through `/usr/bin/open` (see [Usage](usage.md#opening-in-other-apps) and `architecture.md`, "Approved deviation: files and folders open in other apps").
- The `O` picker's rows, and its Finder row, which is always last, are fmc's own (`architecture.md`, "Approved deviation: `open_with` in settings.json and the Open With picker").

## Help

- `?` shows the key list as tables in the reader, with the mouse actions in a table of their own. `--help` prints the usage and the options and points to `?` for the keys; Scouter's `--help` lists every key. `?` or `Esc` brings the document back where it was. Scouter shows its key list as a card over the panes.
- The key list is Markdown, so `/` searches it in the rendered view, as it does any rendered Markdown file.
