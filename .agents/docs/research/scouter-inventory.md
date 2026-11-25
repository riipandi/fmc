> Point-in-time research, 2026-11-26. Source: agent research for plan-00001 (Scouter at commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`). `scouter:<path>` is relative to the Scouter repository at that commit. Other scratch paths (`<scratch>/…`) were local working copies and probes that are not kept. The tree and the locked crates win over this file.

# Scouter 0.3.4: feature and visual inventory

Target: https://github.com/indrazm/scouter, commit `434d000` ("Release 0.3.4"), cloned at `scouter:`.
Purpose: a 1:1 reference for a Rust port.

Conventions:
- Source citations are `path:line`, relative to `scouter:`.
- "Observed" means seen in a live Rex capture. Screen captures (117 Rex text and HTML captures) were taken during the run and are not kept in the repo; section 19 indexes what they showed. Section 19 indexes them.
- "Not verified" means the claim comes from code or docs only.

---

## 0. Key answers

| Question | Answer |
| --- | --- |
| Mermaid | **Not supported.** No Mermaid code exists in the repo, the lockfile, or `@opentui/core`. A ```` ```mermaid ```` fence in rendered Markdown is drawn as an ordinary code block with its body as plain text, with no highlighting and no diagram (observed, screens 04, 05). Details in section 7. |
| Markdown | Only `.md` (exactly the `markdown` filetype) opens **rendered** by default, using OpenTUI's native `MarkdownRenderable`. `m` toggles to source with line numbers. Headings, emphasis, links, code fences, tables, task lists, quotes and rules are drawn with OpenTUI's own styles. Details in section 6. |
| Syntax highlighting | Tree-sitter via OpenTUI's `CodeRenderable` and `TreeSitterClient` (a Worker running `web-tree-sitter` WASM). Grammars: 5 bundled with OpenTUI (JS, TS, TSX, Markdown(+inline), Zig) and 11 vendored in `grammars/` (JSON, CSS, HTML, Go, Rust, Python, Bash, YAML, TOML, C, Java). Colours come from one `SyntaxStyle` built from the palette. Observed in captures 19b and 20 as truecolour HTML. |
| Diffs | Git-backed. One `git diff` for the whole tree, split into per-file patches, each drawn by one OpenTUI `DiffRenderable` (one renderable per file, in a stacked scroll view). Unified and split views. Hunk headers (`@@`) are not drawn. Details in section 11. |
| OpenTUI features used | `CodeRenderable`, `LineNumberRenderable`, `MarkdownRenderable`, `DiffRenderable`, `ImageRenderable`, `TextRenderable`, `TextBufferRenderable`, `BoxRenderable`, `ScrollBoxRenderable`, `SyntaxStyle`, `TreeSitterClient`, `pathToFiletype`, `imageInfo`, `StyledText`, `RGBA`, `MouseButton`, plus renderer APIs (clipboard OSC 52, mouse pointer, selection, `theme_mode` event, palette query). Section 16. |
| Runtime deps | Two, pinned exactly: `@opentui/core` 0.9.0 and `web-tree-sitter` 0.25.10 (peer of OpenTUI; Scouter never imports it). Plus Bun 1.4.4 and git on the host. |

Port-relevant notes:
- Everything is drawn into a retained tree of flex-layout renderables. Most "modes" are visibility toggles on shared panes, not separate screens.
- The file tree, change list, and diff stack all reuse row/block objects across refreshes and only repaint on changes.
- The reader, the diff, and the preview each own their own tree-sitter buffer. Re-highlighting happens on content change only.

---

## 1. CLI

Entry: `src/index.ts` (installed as the `scouter` launcher written by `install.sh:124-153`).

| Invocation | Behaviour | Source |
| --- | --- | --- |
| `scouter` | Opens the current directory (`process.cwd()`, resolved). | `src/index.ts:41` |
| `scouter DIR` | Opens DIR as the tree root. | `src/index.ts:41-42` |
| `scouter FILE` | Opens FILE in the reader. The tree root becomes FILE's parent folder (observed, screen 107). | `src/app/app.ts:246-256`, `src/app/app.ts:349`, `src/explorer/file-tree.ts:69-70` |
| `scouter -h` / `--help` | Prints usage and the key list (including a `Mouse` section), then exits 0. Matches if the flag appears anywhere in argv. | `src/index.ts:7-16`, `:21-24`; `src/core/keymap.ts:57-62` |
| `scouter -v` / `--version` | Prints `scouter 0.3.4`. | `src/index.ts:32-35` (version from `package.json`) |
| `scouter --upgrade` | Handled by the installed launcher: re-runs `install.sh` with `SCOUTER_HOME` and `SCOUTER_BIN_DIR` set. From a source checkout it errors out. | `install.sh:139-146`; `src/index.ts:26-30` |
| More than one argument | Error: `Expected one file or directory. Use --help for usage.` (exit 1). | `src/index.ts:37` |
| Not a TTY on stdin or stdout | Error: `Run Scouter in an interactive terminal.` (exit 1). This check runs before the path is stat'd, so a missing path also reports this when not on a TTY. | `src/index.ts:39` |
| Path is neither file nor directory (e.g. socket) | Error: `Only files and directories can be opened.` | `src/index.ts:44` |
| Any startup or open error | Prints `scouter: <message>` to stderr and exits 1. | `src/index.ts:65-68` |

Renderer options (`src/index.ts:48-54`): `exitOnCtrlC: true`, `screenMode: "alternate-screen"`, `backgroundColor: RGBA.defaultBackground()`, `consoleMode: "disabled"`, `useMouse: true`. Teardown calls `renderer.destroy()` and waits for `renderer.closed` (`:55-62`).

Launcher (installed only, `install.sh:136-153`): runs `bun --config=/dev/null --no-env-file <app>/src/index.ts "$@"`, so a viewed folder's `bunfig.toml` or `.env` cannot run code. Documented in `docs/decision/0008-the-launcher-ignores-the-viewed-folders-config.md`.

Installer (`install.sh`):
- `--help`, `--uninstall`. Usage is at `install.sh:30-47`.
- Install location `${SCOUTER_HOME:-$HOME/.scouter}` (`:15`). Launcher `${SCOUTER_BIN_DIR:-$HOME/.local/bin}/scouter` (`:16`).
- Repo `${SCOUTER_REPO:-https://github.com/indrazm/scouter.git}` (`:12`). Ref `${SCOUTER_REF}`, which defaults to the newest `v*` tag via `git ls-remote`, or `main` (`:14`, `:73-76`, `:86-92`).
- Minimum Bun `1.4.4` (`:17`, `:53-58`).
- Downloads with `git init` + `fetch --depth 1` + `reset --hard` (`:97-111`). Dependencies with `bun install --frozen-lockfile --ignore-scripts` (`:114-122`).
- Never edits shell config. Prints a PATH hint instead (`:148`, `hint_path`).
- Only touches what it created, recognised by the `.scouter-installer` marker and the `# scouter-launcher` marker (`:20-22`, `:127-129`, `:185-196`).
- Checkout mode: run from a checkout, it links to that checkout instead (`:60-71`, `:169-180`).

Not run during this study: installing would write to `~/.scouter` and `~/.local/bin`. The live run used `bun --config=/dev/null --no-env-file src/index.ts` against the clone, which is the same command the launcher execs.

---

## 2. Layout and chrome

Source: `src/app/app.ts:86-197`.

Overall: a column with a `panes` row (flex 1) and a 2-row footer (`src/app/app.ts:86-187`).

```
┌─ sidebar (browser panel, rounded border) ─┐┌─ reader (viewer panel, rounded border) ───┐
│ header 2 rows (title, badge, hairline)    ││ header 2 rows (parent dim + bold title)   │
│ find bar (2 rows, only in find mode)      ││                                           │
│ tree / change list (scroll box)           ││ file view / markdown / image / diff stack │
│                                           ││                                           │
└───────────────────────────────────────────┘└───────────────────────────────────────────┘
 [FILES]  /path/to/file ...right facts...
 hint hint hint ... q quit
```

Geometry:
- Sidebar width: `min(40, max(28, floor(termWidth × 0.25)))` (`src/explorer/sidebar-resizer.ts:49`). A width the user has set is kept for the session (`src/explorer/sidebar-resizer.ts:55-58`, `:69-72`; `DEVELOPMENT.md:49` says the same).
- Constraints: sidebar ≥ 24 columns, reader ≥ 28 columns (`src/explorer/sidebar-resizer.ts:4-5`, `:64-67`). Observed: sidebar 24 when shrunk (screen 78).
- Resize handle: a 1-column absolute strip on the sidebar's right edge with `col-resize` pointer. Keyboard `Ctrl+←/→` moves by 2 columns (`:6`, `:55-58`).
- Footer: `height: 2` (`src/ui/footer.ts:41-47`). Row 1 is the status band, with a pill, location, and right facts. Row 2 is the hint row.
- Header: `height: 2`, bottom hairline border (`src/ui/header.ts:16-27`).
- Find bar: `height: 2`, bottom border (`src/find/file-finder.ts:85-94`). It replaces the sidebar header while active (`src/app/app.ts:378-384`).
- Panel borders: rounded (`src/app/app.ts:96-103`, `:107-116`). Focused pane: `theme.accent` (indigo). Unfocused: `theme.border` (`src/app/app.ts:708-713`; `docs/decision/0021-the-active-pane-has-an-indigo-border.md`).

Header content:
- Sidebar title: root folder name in browse mode (`src/app/chrome.ts:29-32`), or `Changes` plus a count in diff mode (`:24-27`, count gets `+` when truncated). Badge `files` (quiet) in browse mode.
- Reader title: `Reader` when empty, else the file's path split into a dim parent (`dir/`) and a bold title (`src/app/chrome.ts:7-18`). Focused title is bold and `theme.text`. Unfocused is `theme.muted` (`src/ui/header.ts:81-84`).
- Reader badges (`src/app/chrome.ts:61-99`, rendered in `src/ui/header.ts:78-102`): `read-only` (quiet, browse mode), `rendered` (quiet, Markdown preview), `image` (quiet), `binary` (quiet), `first 1 MiB` (pill, truncated), `wrap` (pill, not shown for rendered, image, or binary). In diff mode: change kind (`modified`, `added`, `untracked`, ...), `all files` / `one file` (quiet), `split` (pill), `wide context` (pill). `preview` (pill) while a text-search hit is previewed (`src/app/app.ts:461-463`).
- Pill style: `theme.accentInk` text on `theme.accent` background, bold (`src/ui/header.ts:95-100`). Quiet style: `theme.dim` text.
- Error notice: an error from the last action replaces the reader header's badges in `theme.error`, bold, until the next key (`src/app/app.ts:531-534`, `src/app/app.ts:734`; `src/ui/header.ts:86-88`; `docs/decision/0015-...`).

Footer status row (`src/ui/footer.ts:198-228`):
- Left: a pill then the location or a message. Pill text (`src/ui/footer-hints.ts:7`, `:32-35`):
  - browse/files: `FILES`; browse/reader: `READER`
  - diff/list: `CHANGES`; diff/reader: `DIFF`
  - prompt: `FIND` (file search), `TEXT` (text search), `SEARCH` (in-file search)
- Location is `~`-tildified (`src/ui/footer.ts:16-19`, `:121-129`). Messages replace the location and are shown in `theme.accentSoft`, or `theme.error` when an error (`:217-220`).
- Long locations are cut from the left with `…` (`src/ui/styled.ts:67-75`). Messages are cut from the right (`:77-85`).
- Right facts (`src/ui/footer.ts:230-311`), in drop order `encoding`, `language`, `files`, `lines`, `size`, `hunks`, `counts`, `position`, `count` (`:292`):
  - text: `N lines` (with `+` when truncated), language name, `UTF-8`, position
  - image: `W×H`, format, position
  - binary: size, `binary`
  - diff: `N files` (`+` if truncated), `k/M hunks` (or `M hunks`), `+A −R` (green/red), position
  - pending count (`40`) while typing a count prefix (`src/ui/footer.ts:280`)
- Position strings (`src/app/chrome.ts:111-120`): `All` (no scroll range), `Top`, `Bot`, or `NN%`.
- Facts are separated by ` · ` in `theme.ignored` (`src/ui/footer.ts:305`).

Hint row (`src/ui/footer-hints.ts:9-127`), chosen per pane and per state, dropped from the end to fit (`src/ui/footer.ts:313-336`, `fitHintList` `:129-143`). `q` is kept last. Exact hint lists are in section 4.

---

## 3. Screens and modes

| # | Screen / state | What it looks like | Source |
| --- | --- | --- | --- |
| 1 | Welcome (empty reader, startup with a folder) | Centred 3-row block wordmark `SCOUTER` (`█▀▀ █▀▀ █▀█ …`), tagline `a quiet, read-only file viewer  vX.Y.Z`, `Select a file and press Enter.`, a rounded hint card (`j k ↑ ↓ move`, `Enter open file or folder`, `/ find a file`, `d changes in a git repository`, `q quit`), and `Press ? for every key.  ·  No editing. Just reading.` | `src/ui/welcome.ts:7-87` (observed, screens 01, 65) |
| 2 | Browse (file tree + reader) | Tree on the left, reader on the right. Selected row has `▎` in accent and a row surface. | `src/explorer/explorer.ts:179-294`, `src/app/app.ts:372-376` |
| 3 | Reader: text | Gutter with line numbers (dim), code in tree-sitter colours, last line not shown if empty. | `src/reader/code-view.ts:265-294` |
| 4 | Reader: Markdown (rendered) | Default for `.md`. See section 6. | `src/reader/code-view.ts:230-263`, `docs/decision/0012-...` |
| 5 | Reader: Markdown source | `m`. Gutter and syntax highlight, `w` and pan apply. | `src/reader/code-view.ts:238-243` |
| 6 | Reader: image | PNG, JPG, JPEG, GIF, WEBP as pictures, scaled to fit. Footer shows `W×H · format`. | `src/reader/image-view.ts:14-56`, `src/core/files.ts:9`, `docs/decision/0013-...` |
| 7 | Reader: binary notice | Centred: `Binary files are not supported.` (bold), `name  ·  size` (accent, ignored dot, muted), `Scouter only displays UTF-8 text and images.` (dim). Focus stays in the sidebar. | `src/reader/binary-view.ts:6-70`, `src/app/app.ts:411-419`, `docs/decision/0036-...` (observed, screen 25) |
| 8 | Reader: error | Error appears in the reader header (red) and the footer. The previous file stays open. Examples: `Only UTF-8 text files are supported.` (screen 92), `Cannot expand a symbolic-link loop.` (93), `Not a git repository.` (95), `owl.txt is no longer on disk.` (102). | `src/app/app.ts:531-534`, `src/core/files.ts:183`, `src/diff/git.ts:114-120`, `src/explorer/file-tree.ts:152` |
| 9 | Find file (`/` in sidebar) | Find bar replaces the sidebar header: `/ <query>▎  N matches`, or `indexing…`, `no matches`, `type to find a file, Tab to search inside files`. Results are a flat list with paths. | `src/find/file-finder.ts:49-56`, `:207-273`, `src/app/app.ts:974-981` (observed 08, 09, 12) |
| 10 | Find text (`Tab` inside find) | Prompt `≡ <query>▎`. Rows are `file:line  snippet  folder/`, with the match highlighted. Status `searching N/M`, `N matches`, `first 200 matches`, `N+`, `no matches`, `type 2+ characters`. Highlighted hit previews in the reader. | `src/find/file-finder.ts:62-73`, `:248-273`, `src/find/preview.ts` (observed 10, 80) |
| 11 | In-file search (`/` in reader) | Footer `SEARCH` pill, `/query  N of M` or `no matches`. Matches are amber chips. | `src/reader/document-search.ts:172-185`, `src/reader/match-style.ts` (observed 29-33, 34 html) |
| 12 | Changes (diff) | Sidebar: change list (`Changes N`). Reader: stacked diff of every file, each with a band header. Footer `CHANGES`/`DIFF`. | `src/diff/diff-screen.ts`, `src/diff/diff-blocks.ts`, `src/diff/changes.ts` (observed 35-45) |
| 13 | Changes, split | Two columns per diff block. Needs a reader ≥ 60 columns. | `src/diff/diff-blocks.ts:20`, `:254-260`, `src/app/app.ts:536-540` (observed 37, 75, 77) |
| 14 | Changes, empty | `No changes` with `The working tree matches HEAD.` (or `an empty repository`). | `src/diff/diff-screen.ts:395-402`, `:407-409` |
| 15 | Help overlay (`?`) | Centred rounded card titled ` Keyboard Shortcuts `, sections Browse, Reader, Changes, Everywhere. Mouse section is not in the card. Any key closes it. | `src/ui/help.ts:15-70`, `src/app/app.ts:746-750` (observed 26) |
| 16 | Empty / no-document reader | Welcome as in row 1 (sidebar focus). | `src/app/reader-pane.ts:50`, `:156`, `:225` |

Modal behaviours:
- Find bar and in-file search are prompts. While open, typed keys go to the prompt, not to commands (`src/app/app.ts:733-736`). Observed: `d` typed into the in-file search became part of the query (`/letd`). `q` searches instead of quitting.
- Help overlay consumes the next key (`src/app/app.ts:746-750`). `q` and `Ctrl+c` still quit first (`:740-744`).
- Loading never blocks movement or scrolling. Only starting another open or diff read is refused, with `Still loading. Try again when it finishes.` Pressing `d` during a diff load cancels it (`src/app/app.ts:291-315`, `:495-504`; `docs/decision/0020-...`).

---

## 4. Keybindings

Dispatch order (`src/app/app.ts:733-768`):
1. Find or in-file search prompt (`src/find/file-finder.ts:140-155`, `src/reader/document-search.ts:38-53`).
2. Quit: `q` (plain) or `Ctrl+c` (`src/core/keys.ts:7-9`).
3. Help overlay: any key closes it.
4. Count digits in the reader (`src/reader/reader-motion.ts:20-31`).
5. Global commands: `Tab`, `Esc`, `?`, `Ctrl+←/→`, `Ctrl+o`, `Alt+i`, `w`, `m`, `d`, `y` (`src/app/app.ts:770-788`).
6. Changes-screen keys, in any focus (`src/app/app.ts:869-884`).
7. Focused list or reader.

Source of truth for the `?` list and `--help`: `src/core/keymap.ts:3-55`. The README keys table is tested against it (`tests/keymap.test.ts`). Footer hints are hand-written per context (`src/ui/footer-hints.ts`).

### 4.1 Global

| Key | Action | Source |
| --- | --- | --- |
| `q`, `Ctrl+c` | Quit (`renderer.destroy()`) | `src/core/keys.ts:7-9`, `src/app/app.ts:740-744` |
| `?` | Help overlay; any key closes | `src/app/app.ts:775`, `:746-750` |
| `Tab` | Switch pane: list ↔ reader (or list ↔ diff body in diff mode) | `src/app/app.ts:771`, `:862-867` |
| `Esc` | Clear mouse selection, else dismiss a kept search (reader), else focus the list | `src/app/app.ts:790-799`, `src/reader/document-search.ts:118-124` |
| `Ctrl+←` / `Ctrl+→` | Sidebar narrower / wider by 2 columns | `src/app/app.ts:777-779`, `src/explorer/sidebar-resizer.ts:55-58` |
| `Ctrl+o` | Jump back through opened places (100-entry history) | `src/app/app.ts:783`, `src/reader/jump-history.ts`, `docs/decision/0030-...` |
| `Alt+i` (sent as `ESC i`) | Jump forward | `src/app/app.ts:785`, `docs/decision/0030-...` |
| `w` | Toggle wrap (global: persists across files). Not for rendered Markdown or images. | `src/app/app.ts:658-665`, `src/app/app.ts:76` |
| `m` | Toggle Markdown rendered/source (reader, browse mode) | `src/app/app.ts:430-441`, `:77` |
| `d` | Enter / leave the changes screen (`Loading changes…` shown; a second `d` while loading cancels) | `src/app/app.ts:492-510`, `:78` |
| `y` | Copy the active selection, else the current path (`Copied path.`). Writes to the host clipboard. | `src/app/app.ts:814-830`, `:79` |

### 4.2 Sidebar: file tree (focus on list, browse mode)

| Key | Action | Source |
| --- | --- | --- |
| `j` `k` `↓` `↑` | Move 1 row | `src/core/keys.ts:28-34` |
| `J` `K` (Shift) | Move 5 rows | `src/core/keys.ts:32` |
| `PageDown` `PageUp` | Move viewport−1 rows | `src/core/keys.ts:36-38` |
| `g` `gg` `Home` | First row | `src/core/keys.ts:17-21`, `src/app/app.ts:937-942` |
| `G` `End` | Last row | `src/core/keys.ts:17-21` |
| `Enter` | Folder: expand/collapse. File: open in reader. | `src/app/app.ts:913-916`, `:667-685` |
| `l` `→` | Collapsed folder: open (expand). Expanded folder: move to first child. File: open. | `src/app/app.ts:913-916`, `:667-685` |
| `h` `←` `Backspace` | Expanded folder: collapse. Otherwise move to parent row. | `src/app/app.ts:919-923`, `:687-699` |
| `-` | Make the parent folder the root (`Already at the top of the file system.` at `/`) | `src/app/app.ts:925-929`, `:613-629` |
| `/` | Find bar (file mode) | `src/app/app.ts:931-935`, `:974-981` |

Mouse on a row: selects and opens or toggles (see section 4.6).

### 4.3 Reader (focus on reader, browse mode)

| Key | Action | Source |
| --- | --- | --- |
| `j` `k` `↓` `↑` | Scroll 1 line | `src/core/keys.ts:51-56` |
| `h` `l` `←` `→` | Pan 4 columns (only visible when wrap is off) | `src/core/keys.ts:44-50` |
| `Space` `PageDown` | Scroll 0.9 page | `src/core/keys.ts:57-62` |
| `b` `PageUp` | Scroll back 0.9 page | `src/core/keys.ts:57-62` |
| `Ctrl+d` `Ctrl+u` | Scroll ½ page | `src/core/keys.ts:67-72` |
| `gg` `Home` | Top | `src/app/app.ts:954-961` |
| `G` `End` | Bottom. With a count (`120G`), goes to line 120. | `src/app/app.ts:954-961`, `src/reader/reader-motion.ts:20-31` |
| `0`–`9` | Build a count (only before `G`). Shown in footer. Ignored for rendered Markdown and in the file list. | `src/reader/reader-motion.ts:20-31`, `docs/decision/0029-...` |
| `{` `}` | Previous / next blank line (paragraph) into the top row. Refused for rendered Markdown. | `src/reader/reader-motion.ts:44-66`, `:98-112` |
| `z` `z` | Centre the current match, or the middle line when there is none | `src/reader/reader-motion.ts:44-53`, `:114-117` |
| `/` | In-file search prompt (refused in rendered Markdown: `Press m to search this Markdown file's source.`) | `src/app/app.ts:964-972`, `:999-1008` |
| `n` | Next match (wraps) | `src/app/app.ts:964-972`, `src/reader/document-search.ts:63-76` |
| `N` `p` | Previous match (wraps) | `src/app/app.ts:964-972` |
| `m`, `w`, `y`, `d`, `Tab`, `Esc` | As global | section 4.1 |

Keys while the in-file search prompt is open (`src/reader/document-search.ts:38-53`):
- Typing and `Backspace` edit the query. Typing jumps to the first match at or below the top line (`:126-142`).
- `Enter` or `↓`: next match. `Shift+Enter` or `↑`: previous match. Wraps.
- `Esc`: closes the prompt and keeps the matches (`n`, `N` still work). An empty query is cleared at once. The next `Esc` clears the kept search and a further `Esc` returns focus to the list (`docs/decision/0026-...`).

### 4.4 Changes screen (diff mode)

Keys handled in any focus (`src/app/app.ts:869-884`):

| Key | Action | Source |
| --- | --- | --- |
| `s` | Split ↔ unified. Refused for unified with reader < 60 columns (`Split needs 60+ columns.`). | `src/app/app.ts:872`, `:536-540`, `src/diff/diff-blocks.ts:254-260` |
| `a` | All files (stacked) ↔ one file | `src/app/app.ts:873`, `src/diff/diff-screen.ts:289-304` |
| `c` | Fold / unfold the file under the cursor | `src/app/app.ts:874`, `src/diff/diff-screen.ts:268-275` |
| `x` | Normal (-U3) ↔ wide (-U12) context | `src/app/app.ts:875`, `src/diff/diff-screen.ts:251-266`, `src/diff/diff-screen.ts:23` |
| `r` | Reload now (`Loading changes…`) | `src/app/app.ts:876` |
| `o` | Open the file at the current hunk in the reader. Refused for deleted files (`This file was deleted, so there is nothing to open.`). | `src/app/app.ts:877`, `:1015-1025`, `:1027-1044` |
| `n` `]` | Next hunk (across all files). Messages `Last hunk.` / `First hunk.`. | `src/app/app.ts:878`, `:886-891` |
| `N` `p` `[` | Previous hunk | `src/app/app.ts:879` |
| `d` | Back to browse (focus returns to where it was) | section 4.1, `src/app/app.ts:559-577` |

Changes list (focus on list):

| Key | Action | Source |
| --- | --- | --- |
| `j` `k` `J` `K` `PageUp` `PageDown` | Move selection (as file tree) | `src/core/keys.ts:28-42`, `src/diff/changes.ts:164-166` |
| `g` `G` `Home` `End` | First / last change | `src/app/app.ts:902-907` |
| `Enter` `l` `→` | Focus the diff body (or the list if no blocks yet) | `src/app/app.ts:896-899`, `src/diff/diff-screen.ts:318-322` |
| `Tab` | Switch list ↔ diff body | `src/app/app.ts:862-867` |

Diff body (focus on diff reader):
- `j` `k` `↓` `↑` scroll; `h` `l` pan when not wrapping; `gg` `G`; `Ctrl+d` `Ctrl+u`; `Space` `b`; `Tab`; `w` wrap; `c` fold; mouse click on a header to fold (`src/diff/diff-blocks.ts:468-473`).
- Footer hints for the diff reader: `j k scroll`, `n N hunk`, `o open file`, `s split`, `a one file`, `Tab files`, `c fold`, `x context`, `y copy`, `w wrap`, `h l pan` (no wrap), `gg G top/bottom`, `^d ^u page`, `d back`, `? help`, `q quit` (`src/ui/footer-hints.ts:47-79`).

### 4.5 Prompts

File finder (`/` in sidebar), `src/find/file-finder.ts:140-155` and `:289-295`:

| Key | Action |
| --- | --- |
| Typed text | Query. Name mode narrows as you type. Text mode scans 150 ms after typing stops, at 2+ characters. |
| `Backspace` | Delete one character |
| `Tab` | Switch name ↔ text search. The query is kept. |
| `↑` `↓` | Move highlight. In text mode the reader previews the highlighted hit. |
| `Enter` | Open the highlighted entry. File with a text hit: opens at the line. Folder with a typed query: reveal and expand it. Empty query: open or toggle the entry. |
| `Esc` | Cancel. Tree returns to its previous state. |

In-file search prompt: see section 4.3.

### 4.6 Mouse

Mouse is enabled (`src/index.ts:54` `useMouse: true`). These are the behaviours in code. **None of them were triggered live**, because `rex send` / `send-key` cannot inject mouse events.

| Gesture | Behaviour | Source |
| --- | --- | --- |
| Left click on a tree row | Select the row and open or toggle it | `src/explorer/explorer.ts:190-196`, `src/app/app.ts:720-731` |
| Left click on a change row | Select and focus the diff | `src/diff/changes.ts:188-194`, `src/diff/diff-screen.ts:318-322` |
| Left click on a diff file header | Fold or unfold that file | `src/diff/diff-blocks.ts:468-473` |
| Left drag on the sidebar edge | Resize the sidebar; cursor becomes `col-resize`, edge is lit in `theme.accentSoft` on hover | `src/explorer/sidebar-resizer.ts:28-102` |
| Left drag across text (reader, diff, rendered Markdown, preview) | Selects. On release the text is copied (OSC 52 plus the local clipboard) and the selection cleared. Shows `Copied N characters.` / `Copied N lines.` | `src/app/app.ts:801-812`, `src/core/clipboard.ts:39-52`, `docs/decision/0033-...` |
| Wheel | Scroll the pane under the pointer. No handler in Scouter. Assumed from OpenTUI's `ScrollBoxRenderable` default (not verified). | `src/reader/reader-pane.ts:31-48` |
| Scrollbar | Thumb is drawn in half-block glyphs (`▐ ▝ ▗`) in `theme.muted` on the right edge, and updates scroll position | `src/ui/styled.ts:28-51`, `src/reader/reader-pane.ts:43` |

Focus rule: clicking a row never moves focus between panes (`AGENTS.md`, `docs/decision/0003-...`). Only the drag-to-copy path changes the selection.

### 4.7 Keys in the help card vs the real key handling

The `?` card lists, among others, `Ctrl+o Alt+i`, `Space b ^d ^u` and `n N p`. It does not list `J`, `K`, `PageUp`, `PageDown`, `[`, `]`, `Home`, `End`, `Backspace` in the reader, `-`, or `N` for changes. These work (see sections 4.2-4.4) but are not documented in the card (`src/core/keymap.ts`).

---

## 5. Rendering per file type

Dispatch (`src/app/app.ts:317-342`):
1. Directory: list it.
2. Image extension (`.png .jpg .jpeg .gif .webp`, case-insensitive; `src/core/files.ts:9`, `:13-15`): read ≤ 10 MiB (`src/core/files.ts:137-150`), decode header with `imageInfo`, else `This image could not be read.` (`src/reader/image-view.ts:5-12`).
3. Otherwise read as a text document (`src/core/files.ts:90-113`). Limits and classification:
   - Reads up to 1 MiB (`src/core/files.ts:6`). Larger files show `first 1 MiB` and `+` on the line count. The cut is at a UTF-8 boundary (`:115-119`).
   - Any NUL byte in the first 1 MiB → binary (`src/core/files.ts:172`, `:59-63`).
   - Not valid UTF-8 → `Only UTF-8 text files are supported.` (`:179-185`).
   - Not a regular file (fifo, device, socket) → `Only regular text files can be viewed.` (`:128`).
   - Symlinks are followed on open (`open()` plus `isFile()`), and displayed by name (see section 9).
4. Binary → binary notice (section 3 row 7).

Text rules (`src/core/files.ts:17-24`, `:100`):
- CRLF and lone CR → LF.
- Control characters (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F, U+007F–U+009F) and bidi overrides (U+202A–U+202E, U+2066–U+2069) → `�`.
- Tabs → 4 spaces.
- Empty file: `(empty file)` in the gutter-less code view (`src/reader/code-view.ts:288`).
- Trailing newline removed for display (`:288`). Line count from `countLogicalLines` (`src/core/files.ts:73-80`).

Syntax highlighting is on for languages that `languageFor()` recognises, otherwise plain (see section 8). The footer language is the detected name, or `plain text` (`src/app/reader-pane.ts:129-136`).

Search over unhighlighted text uses the placeholder filetype `scouter-plain` so the match highlight still runs (`src/reader/match-style.ts:9`, `src/reader/code-view.ts:225-226`, `docs/decision/0025-...`).

Images (`src/reader/image-view.ts`, `docs/decision/0013-...`):
- `ImageRenderable` with `protocol: "auto"` (Kitty graphics, Sixel, or character blocks), `fit: "fit"`, `width: 100%`, height = viewport. Protocol is overridable only by `ScouterAppOptions.imageProtocol` (`src/app/types.ts:6`), which the CLI does not set.
- Footer: `W×H · format`. Header badge: `image`.
- No zoom, scroll, crop, or animation. SVG is refused as binary.
- Observed: the PNG reader area is blank in the Rex text capture and the HTML capture has no `<img>` or block cells (screens 24, 24b). Actual rendering is **not verified** (see section 20).

---

## 6. Markdown

Only `.md` files are Markdown (`src/reader/code-view.ts:230-232`, `languageFor` from `pathToFiletype`: `md` and `markdown` map to `markdown`, `mdx` does not; `node_modules/@opentui/core/chunk-bun-rgb7yx3r.js:5539-5540`). The rendered view is the default on open (`src/reader/code-view.ts:235-243`, `:248`). The choice resets when another file opens (`:248`). It is not persisted (`docs/decision/0012-...`).

Engine: `MarkdownRenderable` (`src/reader/code-view.ts:132-151`), an OpenTUI renderable backed by a native document (`MarkdownDocument`, `node_modules/@opentui/core/index.bun.js:10513`, `:10645`). It is given:
- `syntaxStyle`: the same `SyntaxStyle` as code.
- `treeSitterClient`: the same client, so fenced code blocks use the same grammars.
- `fg`: `theme.text`. `width: 100%`, `paddingLeft: 2`, `paddingRight: 2`.
- `conceal`: default `true` (`index.bun.js:10673`), so markup markers are hidden.
- `renderBlock`: calls the default renderer, then retints selections (`src/reader/code-view.ts:142-150`).

Rendering model (from the OpenTUI bundle, observed in captures):
- Headings: no `#` markers. Bold, `markup.heading` style → `syntax.function` colour (`src/reader/code-view.ts:60-63`). Observed `#82aaff`, bold (screen 20 html).
- Paragraphs: word-wrapped to reader width. Always wraps, so `w` and `h l` have no effect (`docs/decision/0012-...`; the `wrap` badge is suppressed for rendered, `src/app/chrome.ts:96`).
- Emphasis: `markup.bold` / `markup.strong` → bold (`:64-65`). `markup.italic` → italic (`:66`).
- Inline code: `markup.raw` → `syntax.string` colour (`:67`). Observed `cargo build` in `#a7c7a1`.
- Links: `markup.link` → `syntax.function` colour, underlined (`:68`). The text is shown; the URL is hidden. Each link is emitted as a terminal hyperlink (OSC 8), observed as `<a href>` in the HTML capture (screen 21). Reference-style links work (`[Da Vinci notes][davinci]`, screen 21).
- Unordered lists: `- ` marker kept. Ordered lists: `1. ` kept (observed, screens 04, 05).
- Task lists: `- [x]` → `☑`, `- [ ]` → `☐` (observed, screen 04).
- Blockquotes: left bar `│` (quote border, `node_modules/@opentui/core/index.bun.js:11227-11234`). Observed, screens 04 and 05.
- Thematic break: full-width `─` line (`index.bun.js:11208-11216`). Observed, screen 05.
- Tables: drawn with box characters (`┌ ┬ ┐ │ ├ ┼ ┤ └ ┴ ┘`) and column widths, filling the reader width (`TextTableRenderable`, `index.bun.js`). Observed, screen 04.
- Fenced code blocks: no fence markers. The body is a `CodeRenderable`, with filetype taken from the info string (`infoStringToFiletype`, `node_modules/@opentui/core/chunk-bun-rgb7yx3r.js:5659-5671`). Highlighted when the filetype has a registered grammar. Observed in HTML capture 20: `rust` (`fn` keyword `#c792ea` bold, `main`/`println!` `#82aaff`, strings `#a7c7a1`) and `ts` (`import`, `const`, `from` keyword; `openVault` function).
- Math blocks: no filetype (`index.bun.js:11266`), not tested.
- HTML blocks: plain text. Not tested.
- Images inside Markdown: not tested.
- Front matter: not tested.
- Line numbers: not shown in rendered view (`docs/decision/0012-...`). Shown in source (`m`).
- Preview appears after a short parse, not in the same frame (`docs/decision/0012-...`).
- Selection colour: `theme.selectionBg` via `applySelectionTint` (`src/reader/code-view.ts:73-80`).

Behaviour differences with source mode:
- Search: refused in rendered mode with `Press m to search this Markdown file's source.` (`src/app/app.ts:999-1005`).
- Paragraph motion: refused with `Press m to move by paragraph in this Markdown file's source.` (`src/reader/reader-motion.ts:103-106`).
- Count prefix: digits ignored (`src/reader/reader-motion.ts:25`).
- `m` toggles and resets scroll to the top (`src/app/app.ts:430-441`, `src/reader/reader-pane.ts:138-146`).
- Preview in the finder (text mode) is always source, not rendered (`src/find/preview.ts:106-113`; observed screen 10).
- Opening a text hit with Enter switches a rendered file to source and seeds the search (`src/app/app.ts:1027-1044`; observed screen 81).
- Diff screen shows `.md` changes as text diffs (`docs/decision/0012-...`).

---

## 7. Mermaid

**Mermaid is not implemented.** Evidence:
- `grep -ri mermaid` over the repo, excluding `.git`, returns nothing (no source, docs, tests, or fixtures; the README is free of it).
- `bun.lock` has no Mermaid package.
- `node_modules/@opentui/core` (`index.bun.js` and both `chunk-bun-*.js`) contains no `mermaid` string.
- Observed, screens 04 and 05: a ```` ```mermaid ```` fence in rendered Markdown shows the fence markers hidden and the body verbatim. For example, `flowchart TD`, `A[Owl post arrives] --> B{Vault key valid?}`, `sequenceDiagram`, `Harry->>Ledger: openVault(...)`. It is ordinary default-coloured text (`#d4d8de`, observed html 21), with no diagram rendering.
- Source mode (`m`) shows the fence and body as text (screen 06).

Mechanism, if a future version wanted Mermaid:
- The info string `mermaid` maps to filetype `mermaid` (`infoStringToFiletype` returns the token when no extension mapping exists, `node_modules/@opentui/core/chunk-bun-rgb7yx3r.js:5659-5671`).
- OpenTUI then creates a `CodeRenderable` with `filetype: "mermaid"` (`index.bun.js:11266`). No parser is registered for `mermaid`, so it is drawn as plain text.
- To render diagrams would need an external layout engine. None is present. This is a port decision, not a 1:1 feature.

---

## 8. Syntax highlighting

### 8.1 Engine

- Scouter creates one `TreeSitterClient` with `dataPath` = `${XDG_CACHE_HOME || ~/.cache}/scouter` (`src/reader/code-view.ts:104-106`). It registers the vendored grammars (`src/reader/grammars.ts:16-18`).
- The client runs parsing in a Worker (`parser.worker.js`, `node_modules/@opentui/core/chunk-bun-rgb7yx3r.js:4552-4553`, `:4621`).
- Queries are `highlights.scm` files (plus `injections.scm` for Markdown). Node types map to style names through `SyntaxStyle` (`src/reader/code-view.ts:38-71`).
- `CodeRenderable` with `drawUnstyledText: true`: plain text draws immediately, colours arrive after the parse (`src/reader/code-view.ts:109-120`).
- Highlight rewrites happen on content change only (`src/reader/code-view.ts:265-294`; `previewSynced` / `codeSynced` flags).
- Search marks are added with the `onHighlight` hook as the group `search.match`, not by writing to the buffer (`src/reader/match-style.ts:30-37`, `docs/decision/0025-...`). At most 2000 marks per file (`:11`). Skipped when lower-casing changes length (`:17`).

### 8.2 Languages (`languageFor`, `src/reader/code-view.ts:20-34`)

Filetype comes from OpenTUI `pathToFiletype` (extension or basename). It is then checked against `supportedLanguages`.

| Language | Filetype(s) | Source of grammar | Extensions seen |
| --- | --- | --- | --- |
| JavaScript | `javascript`, `javascriptreact` | bundled OpenTUI (`chunk-bun-rgb7yx3r.js:4410-4415`) | `.js .mjs .cjs` (`javascript`), `.jsx` (`javascriptreact`) (`chunk-bun-rgb7yx3r.js:5502`, `:5529`, `:5530`, `:5543`) |
| TypeScript | `typescript`, `typescriptreact` | bundled OpenTUI (`:4416-4425`) | `.ts .mts` (`typescript`), `.tsx` (`typescriptreact`, TSX grammar) (`:5546`, `:5569`, `:5570`) |
| Markdown | `markdown`, `markdown_inline` | bundled, with injections for code fences (`:4426-4453`) | `.md`, `.markdown` (`:5539-5540`) |
| Zig | `zig` | bundled (`:4454-4458`) | `.zig` |
| JSON | `json` | vendored `grammars/json` | `.json` |
| CSS | `css` | vendored | `.css` |
| HTML | `html` | vendored | `.html` |
| Go | `go` | vendored | `.go` |
| Rust | `rust` | vendored | `.rs` |
| Python | `python` | vendored | `.py` |
| Bash | `bash` | vendored | `.sh`, `.bash`, `.zsh` (`chunk-bun-rgb7yx3r.js:5499`, `:5565`, `:5581`) |
| YAML | `yaml` | vendored | `.yaml`, `.yml` |
| TOML | `toml` | vendored | `.toml` |
| C | `c` | vendored | `.c`, `.h` |
| Java | `java` | vendored | `.java` |

Vendored grammars (`grammars/manifest.json`, `grammars/<name>/`), each with `LICENSE` (MIT) and `highlights.scm`:

| filetype | npm package | version |
| --- | --- | --- |
| json | tree-sitter-json | 0.24.8 |
| css | tree-sitter-css | 0.25.0 |
| html | tree-sitter-html | 0.23.2 |
| go | tree-sitter-go | 0.25.0 |
| rust | tree-sitter-rust | 0.24.0 |
| python | tree-sitter-python | 0.25.0 |
| bash | tree-sitter-bash | 0.25.1 |
| yaml | @tree-sitter-grammars/tree-sitter-yaml | 0.7.1 |
| toml | @tree-sitter-grammars/tree-sitter-toml | 0.7.0 |
| c | tree-sitter-c | 0.24.1 |
| java | tree-sitter-java | 0.23.5 |

SHA-256 of each `.wasm` and `highlights.scm` is in the manifest. A test checks them (`docs/decision/0014-vendor-more-tree-sitter-grammars.md`, `tests/languages.test.ts`). C++ (3.3 MB) and Ruby (2.1 MB) are excluded for size (`docs/decision/0014-...`).

Not highlighted (plain text): anything else, including `Makefile`, `Dockerfile`, `.mdx` (no mapping), `.txt`, `.log`, `.lock`, `.svg`, and symlinks whose name has no extension.

### 8.3 Colours

`syntaxStyles()` (`src/reader/code-view.ts:38-71`) maps tree-sitter capture names to the palette. Dark values, with light in parentheses:

| Capture(s) | Style | Dark (light) |
| --- | --- | --- |
| `default` | fg text | `#d4d8de` (`#1f2328`) |
| `keyword` | fg `syntax.keyword`, bold | `#c792ea` (`#8250df`) |
| `operator`, `tag` | fg `syntax.keyword` | `#c792ea` (`#8250df`) |
| `string`, `markup.raw` | fg `syntax.string` | `#a7c7a1` (`#116329`) |
| `comment` | fg `syntax.comment`, italic | `#8b929c` (`#646e78`) |
| `number`, `boolean`, `constant`, `type`, `constructor`, `label`, `namespace`, `escape` | fg `syntax.constant` | `#e5b880` (`#953800`) |
| `function`, `property`, `attribute`, `markup.heading*`, `markup.link` (underline) | fg `syntax.function` | `#82aaff` (`#0550ae`) |
| `punctuation` | fg `syntax.punctuation` | `#b2b9c3` (`#57606a`) |
| `variable` | fg text | `#d4d8de` (`#1f2328`) |
| `markup.bold`, `markup.strong` | bold | (inherits) |
| `markup.italic` | italic | (inherits) |
| `search.match` | fg `matchFg`, bg `matchBg`, bold | `#0f1117` on `#e5b880` (`#1f2328` on `#ffdf5d`) |

Observed (html captures 19b, 20, 34): `use`, `struct`, `fn`, `let`, `mut` → `rgb(199,146,234)` bold; `HashMap`, `Vault`, `u32`, `String` → `rgb(229,184,128)`; `key`, `balance`, `open` → `rgb(130,170,255)`; `"Harry Potter"` → `rgb(167,199,161)`; `::`, `;`, `{` → `rgb(178,185,195)`; `1` (line number) → `rgb(102,109,120)` (`#666d78`); search match → fg `rgb(15,17,23)` on `rgb(229,184,128)`, bold.

---

## 9. File tree

Source: `src/explorer/explorer.ts`, `src/explorer/file-tree.ts`, `src/core/files.ts`.

### 9.1 What is listed

- Every entry in the folder except the fixed noise list (`src/core/files.ts:34`): `.git`, `.hg`, `.svn`, `.DS_Store`, `.AppleDouble`, `Thumbs.db`, `desktop.ini`. Not configurable (`docs/decision/0004-...`).
- Dotfiles are shown (`.gitignore`, `.oxlintrc.json`, observed).
- Git-ignored entries are shown, dimmed with `theme.ignored`, not hidden. Ignored status is from `git ls-files --others --ignored --exclude-standard --directory -z` (`src/diff/git.ts:248-250`, observed: `target/` and `node_modules/` dimmed in screens 01-03).
- Ignored folders are still expandable. The finder index skips them (`src/find/file-finder.ts:130`, `src/find/search.ts:15-47`, via the `skip` callback).
- Symlinks: kind `link` (`src/core/files.ts:44`), link icon ``, colour `theme.accentSoft`. A link to a directory expands, after a loop check (`src/explorer/file-tree.ts:98`, `:146-153`). Observed: `loop -> ..` gives `Cannot expand a symbolic-link loop.` (screen 93); `readme-link -> ../../README.md` opens as plain text, since the name has no extension (screen 94).
- Other kinds (sockets, fifos): kind `other`, shown with the default icon.

### 9.2 Sorting

- Folders before files, then names by `Intl.Collator()` (default locale, `src/core/files.ts:11`, `:50-56`). Observed order: `assets, node_modules, notes, scripts, src, target, tests, .gitignore, app.log, README.md` (screen 01).
- Finder index (name search) sorts by UTF-16 code-unit comparison of names, folders first (`src/find/search.ts:10-13`, `:15-47`).
- Finder results: a match whose name contains the first word ranks first. Then shorter relative paths first (`src/find/search.ts:49-54`, `:56-88`).

### 9.3 Icons and colours

- Nerd Font glyphs, not emoji. Codepoints are in the Private Use Area (`src/explorer/explorer.ts:32-54`), so a Nerd Font is needed. Not verified visually (text capture shows blanks in some glyphs).
- Map (`fileIcons`, `src/explorer/explorer.ts:32-54`), as Private Use Area codepoints: `.js .mjs .cjs` U+E74E; `.ts .mts .cts` U+E628; `.jsx .tsx` U+E7BA; `.json` U+E60B; `.md .mdx` U+F48A; `.py` U+E73C; `.go` U+E627; `.rs` U+E7A8; `.zig` U+E6A9; `.html` U+E736; `.css` U+E749; `.sh` U+F489; `.yaml .yml .toml` U+F013. Default file U+F016 (`:270`). Link U+F0C1 (`:269`). Folder open U+F07C, closed U+F07B (`:264-267`).
- Icon colours (`iconColor`, `src/explorer/explorer.ts:56-91`): folder `theme.folder`; link `accentSoft`; `.ts .tsx .jsx .go .css` `syntax.function`; `.js .json .yaml .toml .zig` `syntax.constant`; `.md` `accentSoft`; `.py .sh` `syntax.string`; `.rs .html` `syntax.keyword`; default `theme.muted`.

### 9.4 Row anatomy

From `paintRow` (`src/explorer/explorer.ts:258-293`):
- Column 0: `▎` in `theme.accent` on the selected row, otherwise a space.
- Depth guides: `│ ` repeated per depth level in `theme.ignored` (`:280`).
- Folder marker `▾ ` (open) or `▸ ` (closed) in `theme.dim`, accent when selected (`:263`, `:283`). Files get two spaces.
- Icon and a space (`:284`).
- Name, plus `/` for folders. Bold and `theme.accent` when selected. Name colour `theme.text`, or `theme.ignored` for ignored (`:272-273`, `:285`).
- Right side: a git status letter in its colour, bold (`row.mark`, `:290-292`). Folders with changes inside get `•` in `theme.modified` (`src/diff/marks.ts:84`).
- Selected row background: `theme.rowActiveBg` (`:271`). Others transparent.
- Finder rows show the matched line instead: `▎ icon name:line  before **match** after  folder/` (`:93-111`).

Status letters for tracked changes (`kinds`, `src/diff/git.ts:303-310`; colours `src/diff/changes.ts:32-48`):

| Letter | Kind | Colour |
| --- | --- | --- |
| M | modified | `modified` `#e5b880` |
| A | added | `added` `#8fd3a8` |
| D | deleted | `removed` `#f2a1a1` |
| R | renamed | `accentSoft` `#c7d2fe` |
| T | typechange | `muted` `#8b929c` |
| U | conflict | `removed` |
| ? | untracked | `muted` |

Git marks in the tree are computed for the browsed folder's repository only (`src/diff/marks.ts:18-25`, `:34-48`). If the browsed folder is not in a git repository, no marks appear, even when a nested folder is a repository (observed, screen 59: `demo-repo` is a repo, but under the non-repo root `fmc-ref` it has no `M` marks).

### 9.5 Reveal and expansion

- `reveal(path)` expands ancestors (`src/explorer/file-tree.ts:111-118`), and is used for the finder, change-list `o`, and startup file arguments.
- **Observed bug (screens 15, 16, 41, 42, 107):** `expandTo` loops over `rest.split(sep).slice(0, -1)` and expands each *ancestor of the immediate parent*, not the immediate parent (`src/explorer/file-tree.ts:76-81`). The result: opening `src/ledger/chamber13.ts` expands `src` but not `ledger`, so the file has no row. The selection then falls back to row 0, which is the root (`src/explorer/explorer.ts:158-163`). The reader does show the file. The `Enter` on a folder match in the finder hits the same bug (screen 13). Not fixed in 0.3.4 as far as the code shows; the `docs/decision` set does not cover it. Reproduced live, cause identified from code.
- Startup with a file argument: the tree root is the file's parent (`src/app/app.ts:349-353`, `src/explorer/file-tree.ts:70`, observed screen 107).
- `-`: make the parent folder the root, and keep the previous root expanded (observed screen 59).

### 9.6 Rules for keys in the tree

- Selection survives re-render: the same path is kept if it still exists (`src/explorer/explorer.ts:158-163`).
- Re-listing (`refresh`): children are reconciled, keeping expanded state and object identity for unchanged paths (`src/explorer/file-tree.ts:31-67`).

---

## 10. Find and search

### 10.1 File finder (`/` in sidebar)

Files: `src/find/file-finder.ts`, `src/find/search.ts`, `src/find/types.ts`.

- Index: breadth-first walk of the browsed root, 32 directories at a time (`src/find/search.ts:8`, `:15-47`). Capped at 20,000 entries; the footer then says `first 20,000 files` (`src/find/search.ts:6`, `src/find/file-finder.ts:245`). Git-ignored paths are skipped (`src/find/file-finder.ts:130`). Entries include folders, so folder names can be matched (observed, screen 12).
- Name matching: every whitespace-separated word must occur in the lower-cased relative path. Name-first ordering (`src/find/search.ts:56-88`). Result cap 200 (`:7`). Footer shows `N matches` or `no matches`.
- Index is walked on demand and discarded on close (`docs/decision/0016-...`).

Text search (Tab in finder; `src/find/search.ts:90-237`, `src/find/file-finder.ts:181-205`):
- Starts 150 ms after typing stops (`src/find/file-finder.ts:13`, `:189`). Needs ≥ 2 characters (`:12`).
- Reads only indexed regular files (not links, not ignored, not images; `src/find/search.ts:190-202`), through `readDocument` (1 MiB cap, UTF-8 only; `src/core/files.ts:90`).
- 16 files per batch (`src/find/search.ts:93`, `:198-202`). Status reports every 120 ms (`:94`, `:221-233`).
- Case-insensitive substring match per line (`src/find/search.ts:144-182`). Snippet: up to 40 chars lead-in, 300 chars total (`:95-101`).
- Limits: 10 hits per file (`:91`), 200 hits in total (`:90`), 64 MiB of text scanned (`:92`). Footer: `first 200 matches` or `N+`.
- Typing again drops the running scan (`src/find/file-finder.ts:181-191`).
- Highlighted hit previews in the reader (`src/app/app.ts:995-997`, `src/app/reader-pane.ts:203-215`, `src/find/preview.ts`). Preview is source, scrolled so the line sits about a third down (`src/find/preview.ts:115-120`), tinted (`:9-11`, `:95`), with `preview` badge (`src/app/app.ts:461-463`). Hidden, not closed, when the finder ends (`src/app/reader-pane.ts:217-229`).
- Enter on a text hit opens the file at that line in source view, and seeds a kept search for the text (`src/app/app.ts:1027-1044`, observed screen 81).

### 10.2 In-file search (`/` in reader)

- Prompt `SEARCH` in footer (`src/ui/footer-hints.ts:7`, `:24-29`). Shows `/query  N of M` or `no matches` (`src/reader/document-search.ts:172-185`).
- Matches are whole-line hits by lower-cased `includes` over the document's source text (`src/reader/document-search.ts:151-170`). `n`/`N` step through matching lines (not individual occurrences), with wrap.
- Highlights every occurrence on the line (`src/reader/match-style.ts:13-28`, observed screens 29, 34).
- Kept search follows the file when it is saved or another opens (`src/reader/document-search.ts:83-91`, `docs/decision/0022-...`). Observed: after opening `target/debug/big.log` the footer still showed `/main  no matches` (screen 85).
- Esc behaviour: section 4.3.

---

## 11. Git integration and diff viewer

### 11.1 Repository access

All git runs in `src/diff/git.ts`.

- Runner: `Bun.spawn(["git", "-C", dir, "--no-pager", "--literal-pathspecs", "-c", "core.fsmonitor=false", "-c", "core.quotepath=false", ...args])` (`src/diff/git.ts:47-77`, `:24-31`).
- Environment (`src/diff/git.ts:34-45`): inherits `process.env`; deletes `GIT_EXTERNAL_DIFF`; sets `GIT_OPTIONAL_LOCKS=0`, `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`; sets `GIT_INDEX_FILE` to a private copy.
- Diff options: `--no-ext-diff --no-textconv --no-color -M` (`src/diff/git.ts:32`).
- Timeout 15 s (`:9`, `:59`). Output cap 8 MiB for listings (`:8`, `:79-107`).
- Repository discovery: `git rev-parse --show-toplevel` (`src/diff/git.ts:112`). Non-repository error: `Not a git repository.` (`:114-120`).
- Base: `HEAD` if it exists, else the empty tree (`git hash-object -t tree /dev/null`, `:129-139`). This makes a repository with no commits show everything as added.
- Git index is copied to a temp dir (`mkdtemp`, `:124`). The copy keeps the original's mtime (`utimes`, `:183`). It is refreshed only when the source's inode, size, mtime, or ctime changes. A 2-second racy window forces a re-copy (`:10`, `:156-198`). Copy removed on close and on `process.exit` (`:372-375`). Rationale: `docs/decision/0007-...`.
- One queue per repository (`exclusive`, `src/diff/git.ts:141-148`).

Git commands (exact):

| Purpose | Command | Source |
| --- | --- | --- |
| Repo root | `rev-parse --show-toplevel` | `src/diff/git.ts:112` |
| HEAD | `rev-parse --verify --quiet HEAD` | `:130` |
| Empty tree | `hash-object -t tree /dev/null` | `:137` |
| Index path | `rev-parse --path-format=absolute --git-path index` | `:123` |
| Status letters | `diff <base> --name-status -z --no-ext-diff --no-textconv --no-color -M` | `:211`, `:246` |
| Line counts | `diff <base> --numstat -z <opts>` | `:212` |
| Untracked | `ls-files --others --exclude-standard -z` | `:213`, `:247` |
| Staged names | `diff --cached <base> --name-only -z <opts>` | `:214` |
| Unstaged names | `diff --name-only -z <opts>` | `:215` |
| Ignored (marks) | `ls-files --others --ignored --exclude-standard --directory -z` | `:248-250` |
| Whole-tree patch | `diff -U<n> <base> <opts>` (partial, 4 MiB cap) | `src/diff/patch.ts:30-37` |
| Per-file patch | `diff -U<n> <base> <opts> -- <path(s)>` (1 MiB cap) | `src/diff/patch.ts:144-156` |

Git is never asked for `status`, and no write command is run.

### 11.2 Change list

- Built from `--name-status` and `--numstat`, plus untracked files, then sorted by path using code-unit `<` (`src/diff/git.ts:204-236`). Capped at 1000 (`:12`, `:234-235`). Note: `Only the first N changes are listed.` (`src/diff/changes.ts:104`).
- Status letter and kind as in section 9.4 table.
- Renames: `R` with `← old-name` (`src/diff/changes.ts:231`, `src/diff/diff-blocks.ts:444`), detected with `-M`.
- Staged and unstaged marks (`src/diff/changes.ts:256-264`): `●` staged, `○` unstaged, `◐` both. Untracked files have no mark. Derived from two extra name-only diffs (`src/diff/git.ts:214-215`). Observed: `vault.ts ◐`, `symbology.py ●`, `README.md ○`, deleted `lumos.md ●` (screen 35).
- Right side: `+N` (green), `−N` (red, U+2212), `bin` for binary, `new` when counts are unknown (untracked). `+N −N` separated by a space (`src/diff/changes.ts:266-278`).
- Row layout: 5-column status (`▎` when selected, letter, space, mark), name bold when selected, `← old`, dim `folder/`, counts (`src/diff/changes.ts:226-253`, `:196-199`).

### 11.3 Stacked diff (all files, default scope)

- One block per changed file (`src/diff/diff-blocks.ts:391-423`), separated by a 1-row margin (`:401`).
- Header (height 1, or 2 with a top hairline for blocks after the first; `:455-479`): background is the band tint (`theme.fallbackBackground`-derived, `:237-240`, `src/app/app.ts:204-205`). Content (`headerTitleContent`, `:425-449`): `▎` (active) or two spaces, fold chevron `▾`/`▸` dim, status letter bold coloured, index mark, bold name (`accentSoft` when active), `← old` for renames, dim `folder/`. Right side: counts and a 5-cell proportional stat bar (`statBarSegments`, `:72-96`): `▪` green for additions and red for removals, at least one cell each when both exist.
- Body: an OpenTUI `DiffRenderable` per file (`src/diff/diff-blocks.ts:504-521`), given:
  - `diff`: the file's patch text
  - `view`: `unified` or `split`
  - `wrapMode`: `none` or `word` (wrap)
  - `filetype`: `languageFor(path)`, so changed code is highlighted
  - `syntaxStyle`, `treeSitterClient`: same as reader
  - `gutter: { minDigits, paddingLeft: 1, paddingRight: 1 }`, where `minDigits` is shared across all blocks so the code columns line up (`src/diff/diff-blocks.ts:55-70`, `docs/decision/0035-...`)
  - colours: `addedBg #12281c`, `removedBg #2b1518`, `contextBg` transparent, `addedLineNumberBg #173a27`, `removedLineNumberBg #3b1b20`, sign colours `added #8fd3a8` / `removed #f2a1a1`, line number fg `dim` (`src/diff/diff-blocks.ts:118-128`, `src/ui/theme.ts:58-61`, `:64-65`, `:68`)
- Only one file per `DiffRenderable` (`docs/decision/0005-...`).
- Hunk header lines (`@@ -a,b +c,d @@`) are **not drawn**. Confirmed: zero `@@` in any capture. The hunk index is still computed from them (`src/diff/diff-blocks.ts:47-53`, `:552-591`).
- Folded file: body hidden (`content.visible = false`, `:242-252`).
- Notice body (no diff text): single line, `theme.muted`, left padding 2 (`src/diff/diff-blocks.ts:523-532`).
- Scroll follows selection in all-files scope (`src/diff/diff-blocks.ts:373-389`, `src/diff/diff-screen.ts:552-565`).

Notice texts (`src/diff/patch.ts:9-11`, `:167-169`, `:188-194`; `src/diff/diff-blocks.ts:523`):
- `Binary file: no preview.` (binary, or `Binary files … differ`).
- `This diff is too large to display (limit: 1 MiB).` (per-file patch > 1 MiB).
- `Too much to show at once. Press a to view this file alone.` (stacked total > 4 MiB).
- `No line changes (permissions or file type only).`
- `Renamed from <old> with no content changes.`
- `Unmerged: resolve the conflict to see a diff.`
- `New file: Binary files are not supported.` (untracked binary; observed for `seal.bin`).
- `New file: File is too large (limit: 1 MiB).` (untracked).
- `New empty file.`

### 11.4 Patch synthesis

Source: `src/diff/patch.ts`.
- Whole-tree patch: one `git diff -U<n> <base>` call under the limit of 4 MiB (partial), then split into per-file chunks at `diff --git` boundaries (`:30-40`, `:108-117`). The last chunk is dropped if cut (`:109`).
- Chunk path resolution uses `rename to`, then `+++ b/`, then `--- a/`, then the `diff --git` header (`:119-142`).
- Per-file: `git diff -U<n> <base> <opts> -- <paths>` (old and new paths for renames), 1 MiB cap (`:144-156`).
- Untracked: synthesised patch with `new file mode 100644`, `--- /dev/null`, `+++ b/<path>`, `@@ -0,0 +1,N @@`, all lines prefixed `+` (`:158-186`). Text read via `readDocument` (1 MiB; `:161`).
- Hunk counting: `@@` lines, `+`/`-` lines after the first hunk (`:196-206`).
- Patch text is passed through `displayText` and tabs become 4 spaces (`:78`).
- Binary detection: `/^Binary files .* differ$/m` (`:80`), or `change.binary` from numstat (`:14`).
- Context: `-U3` normal, `-U12` wide (`src/diff/diff-screen.ts:22-23`, `docs/decision/0035-...`).

### 11.5 Split view

- `DiffRenderable` `view: "split"` (`src/diff/diff-blocks.ts:254-260`, `:507`).
- Needs the reader ≥ 60 columns (`MIN_SPLIT_COLUMNS = 60`, `src/diff/diff-blocks.ts:20`). Refused otherwise with `Split needs 60+ columns.` (`src/app/app.ts:536-540`, observed screen 77). Toggling from split to unified is always allowed.
- Each file's split is independent, but all share the same `minDigits`.
- Observed: untracked file in split has an empty left column (screen 75).
- Not switched back automatically if the window narrows later (`docs/decision/0015-...`).

### 11.6 Hunks, folding, scope

- Hunk navigation: `n` / `]` next, `N` / `p` / `[` previous, across files (`src/diff/diff-blocks.ts:345-358`). Footer `k/M hunks` (`src/ui/footer.ts:240-249`). Messages `First hunk.` / `Last hunk.` at the ends (`src/app/app.ts:886-891`).
- `o` opens the file at the active hunk's new-file line (`src/diff/diff-blocks.ts:360-371`, `src/diff/diff-screen.ts:310-316`). Leaves the changes screen and opens in source view (`src/app/app.ts:1015-1044`; observed screen 43). Jump history records the reader's previous file and scroll position (`docs/decision/0030-...`).
- Scope `a`: all files (stacked) or only the selected file. In one-file scope the stacked view is replaced (`src/diff/diff-screen.ts:289-304`, observed screen 38).
- Fold `c`: collapses one file body. Header keeps its chevron (`src/diff/diff-blocks.ts:242-252`, observed screen 39).
- Wide context `x` (observed screen 40 shows the `wide context` pill).
- Wrap `w`: applies to diff blocks (`src/diff/diff-blocks.ts:534-546`).
- Footer diff facts (`src/app/chrome.ts:101-109`, `src/ui/footer.ts:233-261`): `N files`, `k/M hunks`, `+A −R`, position.

### 11.7 Refresh

- Polled every 2 s while the screen is open (`src/app/app.ts:43`, `src/diff/diff-screen.ts:214-220`). Back-off: next delay = clamp(3 × last duration, 2 s, 30 s) (`src/diff/diff-screen.ts:24`, `:337-350`).
- Each poll: list changes (`listChanges`, `src/diff/diff-screen.ts:352-377`). If the list signature is unchanged, a stamp per changed file (`changesStamp`, `src/diff/git.ts:271-289`) decides whether to re-read patches. Unchanged → nothing drawn.
- `DiffRenderable`s are reused when the same files are in the same order; only changed patch content is rebuilt (`src/diff/diff-blocks.ts:280-326`, `patchEqual`).
- Scroll position is held during background refresh (`src/diff/diff-screen.ts:387-390`, `:391`).
- `r` reloads immediately (observed screen 73).
- Live refresh observed: an on-disk edit to `README.md` and a new file `notes/hagrid.md` appeared in the list within 3 s (screens 67, 69).

---

## 12. Watching for file changes

No filesystem watchers (`docs/decision/0006-...`, `docs/decision/0017-...`). Polling only.

Browse (`src/explorer/browse-watcher.ts`, `src/app/app.ts:170-174`, `:579-611`):
- Every 2 s (`src/app/app.ts:44`), back-off as above (`src/explorer/browse-watcher.ts:3`, `:22-42`).
- Each tick: re-list expanded folders (`src/app/app.ts:587-589`), refresh git marks (`:590`), then re-read the open text document if its `mtime:size` stamp changed (`src/app/reader-pane.ts:173-201`, `src/core/files.ts:82-88`).
- A missing open file keeps its content and shows `<name> is no longer on disk.` in the header (`src/app/app.ts:593-595`, `src/app/reader-pane.ts:178-186`; observed screen 102).
- Polling pauses while the changes screen is open (`src/app/app.ts:579-582`).
- Refreshes queue so they never overlap (`src/explorer/browse-watcher.ts:17-20`).
- Observed: on-disk edit to an open file re-rendered in place, footer line count updated (`80 ## Live edit` appeared in screen 67).

---

## 13. Status bar, header, help overlay

- Footer and header: sections 2 and 3.
- Help overlay: `src/ui/help.ts`. Rounded card, width from content, title ` Keyboard Shortcuts `, title colour `accentSoft`, border `accent`. Vertically centred (`:43-45`). Section titles `muted`, bold; key column width 15 (`src/core/keymap.ts:64`); keys `accentSoft`, actions `text`. The overlay sits 2 rows above the footer (`:22`). Observed in screen 26.
- Help overlay omits the `Mouse` section that `--help` shows (`src/ui/help.ts:55` iterates `keySections` only; `src/core/keymap.ts:57-62`).

Footer hint sets (`src/ui/footer-hints.ts:37-127`):
- Browse, file list: `j k move`, `Enter open`, `/ find`, `- up`, `h back`, `Tab reader`, `d changes`, `w wrap`, `^←→ resize`, `^o alt+i jump`, `? help`, `q quit`.
- Browse, reader: `j k scroll`, `h l pan` (no wrap), `gg G top/bottom`, `{ } paragraph` (no Markdown), `z z centre`, `^d ^u page`, `Tab files`, `m preview|source` (Markdown), `/ search` (no Markdown), `y copy`, `d changes`, `^o alt+i jump`, `w wrap` (no Markdown), `? help`, `q quit`.
- Browse, image: `Tab files`, `d changes`, `q quit`.
- Diff, list and reader: section 4.4.
- Prompts: `FIND`: `type filter`, `↑ ↓ move`, `Enter open`, `Tab text`, `Esc cancel`. `TEXT`: `type search`, `↑ ↓ move`, `Enter open`, `Tab files`, `Esc cancel`. `SEARCH`: `type search`, `Enter next`, `↑ ↓ prev / next`, `Esc close` (`src/ui/footer-hints.ts:9-30`).
- Keys are drawn `accentSoft`, actions `dim`, separated by three spaces (`src/ui/footer-hints.ts:129-143`, `HINT_GAP` `:5`).

Welcome card and wordmark: section 3 row 1.

---

## 14. Theme and colours

Source: `src/ui/theme.ts`, `src/ui/system-theme.ts`, `src/ui/types.ts`. Two complete palettes, dark and light (`src/ui/theme.ts:41-113`). `theme` is a live proxy to the palette in use (`:119-121`). Any colour not in the palette is a bug (`docs/decision/0027-...`).

### 14.1 Palette values

| Token | Dark (`src/ui/theme.ts:41-76`) | Light (`:78-113`) | Used for |
| --- | --- | --- | --- |
| `text` | `#d4d8de` | `#1f2328` | body text, names |
| `muted` | `#8b929c` | `#59636e` | secondary text, scrollbar thumb, unfocused title |
| `dim` | `#666d78` | `#7d8590` | hints, gutters, paths |
| `ignored` | `#4a505a` | `#b1b8c0` | git-ignored rows, separators (` · `, `│`) |
| `border` | `#2a2f3a` | `#d0d7de` | unfocused panel border, header hairline |
| `rowActiveBg` | `#222633` | `#eaeef2` | selected row surface |
| `folder` | `#8fa3bf` | `#4c5c75` | folder icons |
| `selectionBg` | `#2c365e` | `#d8e0ff` | mouse selection background |
| `accent` | `#818cf8` | `#4f46e5` | focused border, pill background, prompt marks, `▎` |
| `accentSoft` | `#c7d2fe` | `#4338ca` | key names in footer, active headers, links, `←` |
| `accentInk` | `#0f1117` | `#ffffff` | text on accent pills |
| `error` | `#f2a1a1` | `#cf222e` | error header and footer |
| `hover` | `#343a42` | `#d0d7de` | defined in the palette; no reference in `src/` |
| `added` | `#8fd3a8` | `#1a7f37` | `+N`, `●`, diff sign, stat bar, `A` |
| `removed` | `#f2a1a1` | `#cf222e` | `−N`, `D`, `U`, diff sign |
| `modified` | `#e5b880` | `#9a6700` | `M`, `◐`, folder `•` |
| `addedBg` | `#12281c` | `#dafbe1` | added diff line |
| `removedBg` | `#2b1518` | `#ffebe9` | removed diff line |
| `addedGutter` | `#173a27` | `#aceebb` | added line number cell |
| `removedGutter` | `#3b1b20` | `#ffcecb` | removed line number cell |
| `matchFg` | `#0f1117` | `#1f2328` | search match text |
| `matchBg` | `#e5b880` | `#ffdf5d` | search match chip |
| `targetBg` | `#262b46` | `#e0e7ff` | target line (preview and search hit) |
| `targetGutter` | `#343b66` | `#c7d2fe` | target line gutter |
| `wordmark[0..2]` | `#a5b4fc`, `#818cf8`, `#6366f1` | `#6366f1`, `#4f46e5`, `#3730a3` | welcome wordmark rows |
| `fallbackBackground` | `#131519` | `#f6f8fa` | diff band / footer tint when the terminal background is unknown |
| `syntax.keyword` | `#c792ea` | `#8250df` | keywords, operators, tags |
| `syntax.string` | `#a7c7a1` | `#116329` | strings, inline code |
| `syntax.comment` | `#8b929c` | `#646e78` | comments (italic) |
| `syntax.constant` | `#e5b880` | `#953800` | numbers, booleans, types, labels |
| `syntax.function` | `#82aaff` | `#0550ae` | functions, properties, headings, links |
| `syntax.punctuation` | `#b2b9c3` | `#57606a` | punctuation |

`selectionFg` is not a palette member. `docs/decision/0033-...` says it is; the code only has `selectionBg` (`src/ui/theme.ts:12`, `src/reader/code-view.ts:73-80`).

Literal colours outside the palette: `transparent` = RGBA(0,0,0,0) (`src/ui/theme.ts:154`). OpenTUI's Markdown renderer falls back to `#888888` for rules (`node_modules/@opentui/core/index.bun.js:11216`) and `#FFFFFF` for quotes (`:11233`) when no style foreground is found.

### 14.2 Mode and tint

- Mode is `dark` or `light`. Default `dark` (`src/ui/theme.ts:117`).
- Brightness: `0.299 r + 0.587 g + 0.114 b`. `< 0.5` = dark, else light (`src/ui/theme.ts:134-141`).
- Tint: the terminal background mixed 2.5% toward white (dark) or black (light). Fallback to the palette's `fallbackBackground` if unknown (`src/ui/theme.ts:143-152`, `TINT = 0.025`). Used for the footer band (`src/app/app.ts:204`) and diff file headers (`src/diff/diff-blocks.ts:237-240`). The sidebar is not tinted (`docs/decision/0034-...`).
- Scrollbar: track transparent, thumb `theme.muted` (`src/ui/styled.ts:24-26`, `:14-22`).

### 14.3 Detection and switching

Source: `src/ui/system-theme.ts`, `src/app/app.ts:194-218`.
- On start: query the terminal palette (`getPalette({ size: 1, timeout: 500 })`) for `defaultBackground` (`src/ui/system-theme.ts:73-82`).
- On announce: OpenTUI `theme_mode` event triggers a fresh check (`:31-35`).
- Polling: every 3 s (`:4`), only if the terminal answered the first time and has not announced a change (`:37-46`). Late answers older than the latest request are dropped (`:53`, `:76`).
- Mode: from the background brightness, else the hint from the renderer, else `dark` (`:55`).
- A change triggers `applyTheme`: updates every pane, rebuilds help, keeps focus and scroll (`src/app/app.ts:199-218`, `docs/decision/0027-...`).
- Test checks contrast of each palette (`tests/theme.test.ts`, `docs/decision/0027-...`, `0028-...`, `0034-...`).

---

## 15. Configuration, environment, files written

Configuration: none (`docs/decision/0001-...`). No config file, no flags beyond those in section 1.

Environment variables read by Scouter:

| Variable | Read in | Effect |
| --- | --- | --- |
| `XDG_CACHE_HOME` | `src/reader/code-view.ts:105` | Tree-sitter data path: `${XDG_CACHE_HOME}/scouter` (fallback `~/.cache/scouter`) |
| `HOME` (via `homedir()`) | `src/reader/code-view.ts:105`, `src/ui/footer.ts:17` | Cache fallback; `~` in footer path |
| `WAYLAND_DISPLAY` | `src/core/clipboard.ts:11` | Prefers `wl-copy` on Linux |
| `SSH_TTY`, `SSH_CONNECTION` | `src/core/clipboard.ts:21` | Disables host clipboard helpers (OSC 52 still used) |
| all of `process.env` | `src/diff/git.ts:36` | Passed to every git call, with the overrides in section 11.1 |
| `GIT_EXTERNAL_DIFF` | `src/diff/git.ts:37` | Removed |

Installer variables (install.sh): `SCOUTER_REF`, `SCOUTER_HOME`, `SCOUTER_BIN_DIR`, `SCOUTER_REPO` (section 1).

Files written by Scouter at runtime:
- Tree-sitter cache under `${XDG_CACHE_HOME:-~/.cache}/scouter` (`src/reader/code-view.ts:105`).
- Temporary git index copies `scouter-index-*` under `os.tmpdir()`, removed on close and on exit (`src/diff/git.ts:124`, `:150-154`, `:372-375`).
- Nothing in the working tree or `.git`. Read-only by design (`docs/decision/0001-...`, `docs/decision/0007-...`).

Files written by the installer: `~/.scouter/` (clone, with marker `.scouter-installer`), `~/.local/bin/scouter` (launcher, marker `# scouter-launcher`). Nothing in shell config.

Clipboard (`src/core/clipboard.ts`): OSC 52 through `renderer.copyToClipboardOSC52` plus host command: `pbcopy` on macOS (`:6`), `wl-copy` / `xclip -selection clipboard` / `xsel --clipboard --input` on Linux (`:8-15`). Host write skipped over SSH (`:21`). Result is `true` if either worked (`:39-43`). Notice `Copied N characters.` / `Copied N lines.` / `Copied path.`, cleared after 2 s (`src/app/app.ts:846-860`, `:829`, `src/core/clipboard.ts:45-52`).

---

## 16. Dependencies

Runtime, from `package.json` (lines 17-19) and `bun.lock`:

| Package | Version | Used for in Scouter | Notes |
| --- | --- | --- | --- |
| `@opentui/core` | **0.9.0** (exact pin) | Everything visual and terminal: renderer, renderables, input, mouse, selection, clipboard, syntax, Markdown, diff, images | Zig native library via `@opentui/core-<platform>` optional deps (`bun.lock:21-37`). Transitives: `bun-ffi-structs` 0.3.1, `diff` 9.0.0 (not imported in the bundle; `parsePatch` is local), `string-width` 7.2.0. Engines: Bun ≥ 1.3.14. Upgrade reason: `docs/decision/0032-...` (tree-sitter leak fix). |
| `@opentui/core-darwin-arm64` (and other platforms) | 0.9.0 | Native `libopentui.dylib` / `.so` / `.dll` | Ships libwebp, wuffs, stb, lcms2, zlib, ghostty, and ffmpeg licences (`node_modules/@opentui/core-darwin-arm64/`). |
| `web-tree-sitter` | **0.25.10** (exact pin) | Peer dependency of OpenTUI; WASM tree-sitter runtime | Scouter does not import it (`docs/decision/0032-...`). |

Vendored grammars (`grammars/`): 11 WASM files with `highlights.scm` and MIT `LICENSE`. Versions in section 8.2. Total about 4.5 MB (`docs/decision/0014-...`).

Dev-only (`package.json:21-25`): `@types/bun` 1.4.4, `oxfmt` 0.76.0, `oxlint` 1.91.0, `tsrs` 0.21.0-ts7.1.0-beta (typecheck), `tooling/oxlint/` custom rules (`.oxlintrc.json`).

Installed on the host: Bun ≥ 1.4.4 (`package.json:27-30`, `packageManager`), git. Optional: `pbcopy` (macOS), `wl-copy`/`xclip`/`xsel` (Linux), a Nerd Font for icons (inferred from PUA glyphs; not verified).

Install sizes noted by the project: OpenTUI 4.7 MB, grammars ~4.5 MB (`docs/decision/0014-...`).

Installed-in-this-study `node_modules` (`bun install --frozen-lockfile --ignore-scripts`): 22 packages.

---

## 17. Performance tricks

1. Renderable reuse: explorer rows (`src/explorer/explorer.ts:136-168`), change rows (`src/diff/changes.ts:74-112`), diff blocks (`src/diff/diff-blocks.ts:280-326`) are reused and only restyled or re-painted.
2. Signature and stamp gating: browse refresh repaints marks only when they changed (`src/app/app.ts:604-609`); diff refresh skips when list signature and per-file stamps match (`src/diff/diff-screen.ts:352-377`); document refresh compares `mtime:size` (`src/app/reader-pane.ts:173-201`, `src/core/files.ts:82-84`).
3. Single git call for patches: `git diff` once for the whole tree, split in process (`src/diff/patch.ts:22-40`).
4. Private index copy with cached stamp (`src/diff/git.ts:156-198`).
5. Serialised git per repository (`src/diff/git.ts:141-148`), so no overlapping git calls.
6. Adaptive polling back-off (`src/explorer/browse-watcher.ts:22-42`, `src/diff/diff-screen.ts:337-350`).
7. Tree-sitter on a Worker (`node_modules/@opentui/core/chunk-bun-rgb7yx3r.js:4552-4553`). Highlight only on content change (`src/reader/code-view.ts:265-294`).
8. One `SyntaxStyle` per theme, rebuilt only on restyle (`src/reader/code-view.ts:38-71`, `:154-167`).
9. Search marks through `onHighlight`, not buffer writes (`src/reader/match-style.ts`), capped at 2000 (`:11`).
10. Lower-cased lines cache in in-file search, keyed by text identity (`src/reader/document-search.ts:164-170`). Blank-line index cache (`src/reader/reader-motion.ts:68-96`).
11. Bounded I/O: 1 MiB read cap, 8 KiB probe buffer (`src/core/files.ts:6`, `:8`, `:152-177`); 8 MiB listing cap, 1 MiB per-file patch, 4 MiB stacked (`src/diff/git.ts:8`, `src/diff/patch.ts:5-6`); 10 MiB image cap (`src/core/files.ts:7`).
12. Batching: 32 directories per index batch (`src/find/search.ts:8`), 16 files per text-search batch (`:93`), 16 untracked patches per batch (`src/diff/patch.ts:7`).
13. Text-search throttling: 150 ms debounce (`src/find/file-finder.ts:13`), 120 ms progress report (`src/find/search.ts:94`), hard stops at 200 hits or 64 MiB (`src/find/search.ts:90-92`).
- One `Intl.Collator` and a cached scrollbar colour (`src/core/files.ts:11`, `src/ui/styled.ts:14-22`). `Bun.stringWidth` for display width (`src/ui/styled.ts:63-85`).
15. Git runs with `GIT_OPTIONAL_LOCKS=0` and a private index, so polling never takes the repository's index lock (`src/diff/git.ts:38`, `docs/decision/0007-...`).

---

## 18. Edge cases

Observed in live run (screen numbers in section 19):
- Welcome hint line is cut at 80 columns: `…No editing. Just readin` (screen 01).
- `/private/tmp/…` paths on macOS because `process.cwd()` resolves symlinks (every footer).
- Footer position stuck at `All` after an image is opened and a binary file is selected: binary does not call `updatePosition`, and `setDocument` does not clear the browse position (screen 25). Minor.
- `Esc` from a file prompt returns the tree to its previous path. In the text finder, `↓` moves and previews (screen 80).
- Wrap state is global: opening another file keeps `wrap` on (screens 22, 23).
- Kept in-file search carries over to other files (screen 85).
- `Ctrl+o` with no history: `No earlier place to go back to.` (screen 115).
- Pan (`h`, `l`) does nothing when wrap is on (screen 90 vs 91).
- Deleted file in change list, `o`: `This file was deleted, so there is nothing to open.` (screen 41).
- Non-UTF-8 text: `Only UTF-8 text files are supported.`, previous file stays (screen 92).
- Symlink loop: `Cannot expand a symbolic-link loop.` (screen 93).
- Outside a repo: `Not a git repository.` (screen 95).
- Deleted open file: notice, content kept (screen 102).
- Sidebar at minimum 24 columns (screen 78). Split refused in narrow reader (screen 77).
- Tree reveal off-by-one for collapsed nested folders (section 9.5; screens 15, 16, 41, 42, 107).
- Finder `Enter` on a nested folder match: selection falls to the root (screen 13).

From source, not triggered:
- Image over 10 MiB: `Image is too large (limit: 10 MiB).` (`src/core/files.ts:140`).
- Header decode failure: `This image could not be read.` (`src/reader/image-view.ts:10`).
- File over 1 MiB text: `first 1 MiB` and `+` (observed, screen 85).
- Changes list over 1000 files: `Only the first N changes are listed.` (`src/diff/changes.ts:104`, `src/diff/git.ts:12`).
- Stacked diff over 4 MiB: `Too much to show at once…` (`src/diff/patch.ts:11`).
- Per-file patch over 1 MiB (`src/diff/patch.ts:10`).
- Finder over 20,000 entries (`src/find/search.ts:6`).
- Text search over 200 hits (`src/find/search.ts:7`, observed text in `src/find/file-finder.ts:72`).
- Repository with no commits: base is the empty tree, everything staged is `added` (`src/diff/git.ts:129-139`, `docs/decision/0005-diff-screen-shows-all-files-stacked.md`). Not triggered.
- Git timeout 15 s (`src/diff/git.ts:9`). Not triggered.
- Git not installed: `Git is not installed or could not be started.` (`src/diff/git.ts:61-63`). Not triggered.
- Clipboard failure: `Could not copy to clipboard.` (`src/app/app.ts:852`). Not triggered.

---

## 19. Live repro

### 19.1 Method

- Scouter run from source with the launcher's flags: `bun --config=/dev/null --no-env-file scouter:src/index.ts .`, cwd `<scratch>/demo-repo`, inside new Rex tabs (`rex window new … --focus=false --keep-open --shell none`) in the current session. The Control Plane pane (`block:01a2144a-042f-…`, `$REX_BLOCK`) was not touched.
- Driver: `<scratch>/out/drive.sh` (`rex send-key` / `rex send` / `rex capture --trim`, plus `--format=html`). Each capture is one file in `<scratch>/out/screens/`.
- Tab geometry: fresh tabs are 80×24. A zoom on/off cycle gave 187×60, which is what most of the later captures show. Widths are in each capture's header line.
- Demo repo: `<scratch>/demo-repo` (committed: `README.md` with headings, lists, tables, code fences, links, blockquotes, task list, `mermaid` flowchart and sequence diagram, a rule, reference link; `src/main.rs`, `src/ledger/vault.ts`, `scripts/symbology.py`, `docs/guide/lumos.md`, `tests/notes.txt`, `assets/marauder-map.png`, `.gitignore`). Uncommitted: README and main.rs modified, `vault.ts` staged then modified again (◐), `symbology.py` staged (●), `lumos.md` deleted and staged (`D ●`), untracked `chamber13.ts`, `notes/owl-post.md`, `assets/seal.bin` (4 KB random binary), `notes/hagrid.md` (later), ignored `target/`, `node_modules/`, `app.log`. Names: Harry Potter, Dan Brown, Hermione Granger, Bill Weasley, Robert Langdon.
- Deviation: the `lumos.md` deletion is staged (`D ●`) rather than unstaged (`○`). A later `git reset` was denied by the permission layer, so I could not turn it into an unstaged deletion.
- Side effects on the host: `y` wrote paths to the macOS clipboard (`pbcopy`). The clipboard now contains a path from the demo repo.
- Demo repo edits made during the run (for live-refresh checks): README append, `notes/hagrid.md` created, `target/debug/owl.txt` created then deleted, `target/debug/` edge-case files (`big.log` 1.5 MB, `wide.txt`, `latin1.txt`, `loop` and `readme-link` symlinks).
- Tabs closed at the end (`rex window close`). `Code Editor` and `Control Plane` windows left alone.

### 19.2 Screen index (selected)

All in `<scratch>/out/screens/`. `.txt` = Rex text capture (trimmed); `.html` = `--format=html` capture with colours.

| File | State |
| --- | --- |
| `00-cli-help.txt`, `00-cli-version.txt` | `scouter --help` and `--version` |
| `01-browse-welcome.txt` | Welcome at 80×24 |
| `02`–`03` | Tree expansion |
| `04-readme-rendered.txt` | README rendered (top) |
| `04-readme-rendered-open-probe-k:shift+g.txt` | **Stray.** Misnamed capture from a bad driver call; content is a README view. Ignore. |
| `05-readme-rendered-bottom.txt` | README rendered, bottom: mermaid bodies as text |
| `06-readme-source-m.txt` | README source (`m`) |
| `07-readme-source-wrap.txt` | Source with wrap |
| `08`–`13` | File finder: empty, name, text, names for `ledger`, folder Enter, folder reveal |
| `14`, `16` | Probes for the reveal bug |
| `15-finder-open-nested-file.txt` | Opening `src/ledger/chamber13.ts`: tree bug |
| `17-rust-main-source.txt`, `17b-…html` | **Mislabelled.** Captured the chamber13 reader with a `/main.rs` search prompt, because the keys went to the reader. Use 18/19 instead. |
| `18-finder-main-rs-open.txt` | `src/main.rs` opened via finder |
| `19-rust-main-source.txt`, `19b-rust-main-source.html` | `main.rs` source and colours |
| `20-readme-rendered-html.html`, `21-readme-rendered-bottom-html.html` | Rendered README colours and links |
| `22-ts-vault-source.txt`, `23-python-symbology-source.txt` | TypeScript and Python reader |
| `24-image-png-open.txt`, `24b-image-png-html.html` | PNG: footer `64×32 · png`; no visible image in capture |
| `25-binary-seal-open.txt` | Binary notice |
| `26-help-overlay.txt` | `?` overlay |
| `27` | Help dismissed by `x` |
| `29`–`33` | In-file search: typed, Enter, Esc keeps, `N`, cleared |
| `34-reader-search-typing-html.html` | Search match colours |
| `35`–`43` | Changes: all, next hunk, split, one file, fold, wide context, `o` on deleted, `o` on vault, `o` on README |
| `44`–`45` | Ctrl+o and Alt+i |
| `46`–`56` | Source nowrap, `}` `{`, `40`→pending, `40G`, `zz`, Space, `b`, Ctrl+d, `gg` |
| `57`–`58`, `78`, `77` | Sidebar resize, minimum, split refused |
| `59`, `95` | Go up, changes in non-repo |
| `60`–`63` | Copy path |
| `64` | Quit (`q`); pane empty afterwards |
| `65` | Fresh tab at demo repo |
| `66`–`69` | Live edit refresh (reader, tree, changes) |
| `70`–`76` | Changes focus, hunk steps, reload, wrap, split on vault |
| `79`–`83` | Finder cancel, text down-preview, open text hit, kept search n and Shift+Enter |
| `84`–`86` | `target/debug` tree, big.log first-megabyte, Esc clears kept search |
| `87`–`91` | Wide line, wrap, pan |
| `92`–`94` | latin1 error, symlink loop, readme symlink |
| `96`–`98` | Tree gg, h, l, G |
| `99`–`102` | Created file appears, opened, deleted on disk |
| `103`–`106` | J, PageDown, PageUp, K |
| `107` | Startup with a file argument |
| `108`–`115` | Changes in that tab: `]`, `p`, Ctrl+d, `G`, Space, `r`, Ctrl+o without history |
| `116` | Diff state before quit |

### 19.3 Not triggered live

- All mouse behaviour (section 4.6). Rex has no mouse injection in this path.
- Image rendering (section 5). Capture shows no image.
- `Shift+Enter` in the open-prompt in-file search. Only `↑` and `↓` were used with the prompt open, and `Shift+Enter` was tested only with the prompt closed (no-op).
- `h`/`l` pan in the diff view with wrap off.
- `Esc` clearing a mouse selection (`renderer.clearSelection`).
- Empty repository, git timeout, git missing, clipboard failure, image size limit, bad image header, changes over 1000 files, stacked diff over 4 MiB, untruncated files over 1 MiB (shows `first 1 MiB` only via `big.log`, observed).
- Rendered-Markdown math, HTML, and images.
- Installer (not run, to keep `~/.scouter` and `~/.local/bin` untouched).

---

## 20. Open questions

1. Image rendering: is the PNG drawn with Kitty graphics, Sixel, or nothing in Rex? The text capture shows the area blank and the HTML capture has no image cells. Needs a terminal check outside Rex or Rex's graphics support statement.
2. Mouse: do click, drag-copy, wheel, and edge drag work exactly as coded in Rex? Needs a mouse-capable run.
3. `y` in the file list: the `Copied path.` footer notice did not appear in capture 60 (`pbpaste` did show the path). It did appear in capture 62/63 from the reader. Cause not found.
4. Tree reveal bug (section 9.5): is there an upstream fix? Not in `docs/decision/`. Looks like an off-by-one in `expandTo`. Any Rust port should decide whether to keep the bug.
5. `Shift+Enter` vs `Enter`: Rex may not send distinct keys. Scouter documents `↑` as the dependable way back (`docs/decision/0026-...`).
6. Nerd Font: icons are PUA codepoints. The glyphs appear blank in text capture, so their presence in a non-Nerd terminal (tofu) is unverified.
7. Diff syntax colours: source says `DiffRenderable` gets `filetype` and `treeSitterClient`, so changed code should be highlighted. Not colour-verified in a diff capture.
8. Hunk header `@@` lines are not drawn. Is that an OpenTUI default or a Scouter choice? Source does not set it; it is OpenTUI behaviour.
9. Markdown preview: `m` toggles, `w` ignored. Does the rendered view ever break table widths when a cell exceeds the reader?
10. Whether the wheel is handled by OpenTUI scroll boxes in a way that needs porting detail.

---

## 21. Documentation and code drift

- `docs/decision/0003-no-frames-tinted-sidebar-dimmed-inactive-pane.md`: dims inactive panes and has no frames. Superseded by `0021` (indigo border, no dim) and `0034` (hairline borders, no sidebar tint). Code follows 0021 and 0034.
- `docs/decision/0033-copy-selected-text-and-paths.md`: says the palettes include `selectionFg` and `selectionBg`. Code has only `selectionBg` (`src/ui/theme.ts:12`).
- `docs/decision/0027-…md`: says a test checks WCAG contrast of palettes. `tests/theme.test.ts` exists. Not re-checked here.
- `src/ui/help.ts` omits the Mouse section, which `--help` includes (section 3).
- `docs/decision/0034-…md`: says the sidebar has no tint and footer/diff headers are tinted. Code agrees (section 14.2).
- `README.md` keys table: all keys in the table are in `src/core/keymap.ts` (tested). The table does not mention `J K PageUp PageDown Home End [ ]` or `N` in the changes list; these work.
- `docs/decision/0014-vendor-more-tree-sitter-grammars.md`: lists C and Java; manifest matches (`grammars/manifest.json`).

---

## 22. Port checklist (from the findings)

- Retained renderable tree with flex layout, two panes, footer, overlays (help, find bar, in-file prompt).
- Tree: lazy folder listing, Intl-like sort (folders first, then case-insensitive natural), noise list, ignored dimmed, Nerd Font icons, git letters, folder dots, row reuse, reveal logic (reproduce the bug only if required).
- Reader: text (1 MiB cap, UTF-8, binary NUL check, tabs 4 spaces, control chars to `�`), line-number gutter, syntax (tree-sitter with the 16 listed grammars), search marks, count prefix, paragraph and centring motions, jump history of 100.
- Markdown: rendered (headings, emphasis, links with OSC 8, inline code, fenced code with tree-sitter by info string, lists, task boxes, quotes, rules, tables), toggle source. Mermaid: no rendering, show as code text.
- Images: decode header, scale to fit, protocol auto. Binary notice screen.
- Git: `rev-parse`, `diff --name-status/numstat/name-only`, `ls-files` variants, private index copy, racy window 2 s, per-repo queue, 1000 change cap, 1 MiB per-file / 4 MiB stacked, untracked synthesis.
- Diff view: stacked blocks with band headers, stat bars, unified and split (≥ 60 columns), fold, context 3/12, hunk navigation, one-file scope, polling 2 s with back-off, stamp-based refresh.
- Finder: 20k index, name matching with AND words, text search (2+ chars, 150 ms debounce, 200 hits, 10 per file, 64 MiB), preview with third-line scroll.
- Theme: two palettes (exact values in section 14), brightness mode, tint 2.5%, live terminal-background detection with 3 s polling.
- Clipboard: OSC 52 plus `pbcopy` / `wl-copy` / `xclip` / `xsel`, skip host helper over SSH.
- CLI: `-h`, `-v`, one optional path, TTY check, exit 1 on error, `--upgrade` only via launcher.
