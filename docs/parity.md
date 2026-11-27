# Differences from Scouter

fmc starts as a port of [Scouter](https://github.com/indrazm/scouter). It aims to look and behave the same, except for the differences on this page. Each one was chosen on purpose.

Ported from: Scouter v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

## Name

- fmc shows its own name and version where Scouter shows `scouter`: the welcome screen, `--help`, `--version`, and error messages.
- There is no `--upgrade` option. Update fmc by installing a new build.

## Keys

- `Ctrl+q` quits. Plain `q` does not quit; it closes the open document, so the welcome screen returns, or the key list. On the changes screen, `q` goes back to browsing, as `d` does. `Ctrl+c` still quits.
- `:q` then Enter quits too, as in Neovim (`:q!`, `:qa`, and `:quit` work as well). `Esc` cancels the `:` prompt.
- Because `^q quit` is one column wider than Scouter's `q quit`, the footer at 80 columns can drop one more hint (on the changes screen, `a one file`). The key list (`?`) always shows every key.
- Moving through the file list previews each file in the reader while the list keeps the focus (auto-preview). Press `p` in the list to turn it off or on, or set `auto_preview` in the settings file. Scouter shows a file only when you open it.
- Only Enter opens a file, and it moves the focus to the reader, as in Scouter. `l` and `→` only expand folders; in Scouter they also open files.
- At 80 columns and wider, while another pane has the focus, a click on the file list only moves the focus there; the next click selects. Below 80 columns the first tap on a row acts. Showing the hidden sidebar with `Ctrl+/` or the `»` mark also focuses it.
- `]` and `[` move the list selection by two rows and scroll the reader by five lines (both set in the settings file). Scouter uses them only on the changes screen.
- A click anywhere in a pane (the file list, the reader, the changes list, or the diff) moves the focus there. In Scouter only a click on a row does something.
- A row in the file list, the finder, or the changes list acts when you release the click, not when you press it, so a drag or swipe over a row does nothing. Scouter acts on the press. Recorded in `architecture.md`, "three layout tiers", record 8 (plan-00005).
- `Ctrl+/` shows or hides the sidebar. `Ctrl+.` shows or hides dotfiles in the tree and the finder, and `.` does the same for terminals that cannot send `Ctrl+.` (Terminal.app). Dotfiles are shown by default, as in Scouter. Scouter has none of these keys.

## Layout

- Below 80 columns the sidebar hides once a file is open, and the reader takes the full width. The welcome screen keeps the sidebar from 60 columns up. Below 60 columns there is no welcome screen and no sidebar: the list is the first page (see the next bullets). At 80 columns or more the layout is Scouter's. Scouter always shows the sidebar.
- At 80 columns and wider a mark on the border shows or hides the sidebar with a click: `«` on the sidebar's border while it is shown, `»` on the reader's left border while it is hidden. Below 80 columns the marks are not drawn; the `‹` and `›` chevrons at the start of the header titles take their place (60 to 79 columns), and a tap on the title beside a chevron does the same. A manual choice holds until the width crosses 80 again.
- Nothing about the sidebar or the dotfiles is saved across runs.
- Below 60 columns (a phone) fmc becomes a two-page browser: the file list fills the screen, opening a file shows it full screen, and the header's `‹` (or `q`, `Esc`) goes back. A bottom bar has large buttons with words (`Find`, `Changes`, `Keys`, `Back`, `Search`). The panel border and the two-row header are drawn as at 60 to 79 columns, the same as in the medium tier (plan-00007, `architecture.md`); there is no welcome card. From 60 to 74 columns the welcome keeps its wordmark, tagline, and prompt but drops its card, since the reader beside the list is narrower than the card. From 60 to 79 columns the list sits beside the reader and hides once a file is open; `‹` and `›` in the titles hide and show it. A tap acts when the finger lifts without moving, so swiping never opens a file. At 80 columns and wider the layout is Scouter's.

## Changes screen

- A diff file folds when you release the click or tap on its header, so a swipe that starts on a header does not fold it. Scouter folds on the press.
- A click on a changed file in the list selects it and shows its diff, and the list keeps the focus, as the file tree does. Enter moves the focus to the diff. Scouter moves the focus on the click. On a phone a tap still opens the diff page.
- The changes screen opens showing one file's diff. Press `a` to see all files stacked, as Scouter shows them at first.
- After a change that moves the diff (folding with `c`, reloading with `r`), the footer and the scrollbar show the new position at once. Scouter can show the old values until you scroll.
- After switching between one file and all files (`a`) or changing the context (`x`), the selected file stays at the top. Scouter can land elsewhere.

## Reader

- A file's highlighting, and whether `m` renders it as Markdown, follow `file_associations` in the settings file: a pattern can name the filetype (`*.rs.bak` as Rust, `.env.*` as an env file, `docs/**/*.txt` as Markdown), and `plaintext` shows plain text. Scouter reads the extension only (`scouter:src/reader/code-view.ts:31-34`). A folder opened through a symlink matches path patterns in the reader but by name only on the changes screen. Recorded in `architecture.md`, "Approved deviation: `file_associations` in `settings.json` picks a file's filetype".
- Line numbers stay in place when you scroll sideways. In Scouter they scroll away with the text.
- In a file inside a git repository, the reader shows a one-column strip left of the line numbers: `▎` green for an added line, `▎` yellow for a changed line, and `▁` red on the line above a deletion (`▔` on line 1 for a deletion above it). The strip keeps its column in a file without changes. `git_gutter: false` in the settings file turns it off, and outside a repository nothing changes. Scouter's code view has no signs (`src/reader/code-view.ts:121-131`). Recorded in `architecture.md`, "Approved deviation: the reader shows git change signs in a strip left of the line numbers".
- A heading in the rendered view keeps its `#` marker (`# `, `## `, and so on) in the heading's colour, so the levels read without a larger font. Scouter conceals the marker (`scouter:src/reader/code-view.ts:132-145`). Emphasis, code, and link markers stay concealed. A setext heading (`Title` over `===` or `---`) shows the marker of its level, and its underline row is not drawn. Recorded in `architecture.md`, "Approved deviation: rendered Markdown headings keep their `#` marker".
- `/` searches the rendered view. Each occurrence is marked in the text the view draws (`**Harry** Potter` matches `Harry Potter`), the current one in its own colour, and `n`, `N`, and `p` step through them; the footer counts them as `k of N`. A fence in a language fmc does not highlight is counted and reached but not marked. Scouter refuses `/` in a rendered Markdown file (`scouter:src/app/app.ts:999-1005`). Recorded in `architecture.md`, "Approved deviation: search runs in the rendered Markdown view" (Scouter v0.3.4, `434d000f`; owner, 2026-11-27).
- In a rendered Markdown file inside a git repository, each block with an added or changed line gets a bar in the sign column at the left edge, beside the block's rows (green for added, yellow for changed); a removed line shows a `▁` (or `▔` above the first line) under the block above it. Scouter's rendered view has no signs. Recorded in `architecture.md`, "Approved deviation: the rendered view shows git change signs beside its blocks". `git_gutter` controls it.
- A ```` ```mermaid ```` fence in the rendered view is drawn as a box diagram at the reader's width. A diagram too wide for the reader, or one fmc cannot draw, shows its source in a frame with the reason. Scouter shows the fence as plain code. The source view (`m`) keeps the fence as text, and edge labels take the text colour, not the muted one. Recorded in `architecture.md`, "Approved deviation: a Mermaid fence in rendered Markdown is drawn as a diagram".

## Copying text

- Dragging the mouse across text selects it and keeps the selection on screen. Nothing is copied until you press `y`. Press `Esc` to clear the selection. With no selection, `y` copies the path, as in Scouter.

## File tree

- The tree has no root row. The folder's own files and folders start at the top, with no indent, and its name stays in the sidebar title. Scouter draws the folder as the first row, which collapses like any folder.
- Opening a file inside a collapsed folder expands every folder down to the file, so the file's row is visible. Scouter can leave the file's own folder collapsed.
- When fmc starts on a file it cannot show (not UTF-8 text), the tree still lists the file's folder. Scouter leaves the tree empty.

## Help

- `?` shows the key list as tables in the reader, with the mouse actions in their own table. `--help` describes the same key list, so the two always agree. `?` or `Esc` brings the document back where it was. The key list is Markdown, so `/` searches it in the rendered view, as in any rendered Markdown file (see Reader).

## Footer

- A binary file's footer shows only its size and `binary`. Scouter can keep the position of the document opened before it.
