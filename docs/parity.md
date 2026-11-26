# Differences from Scouter

fmc starts as a port of [Scouter](https://github.com/indrazm/scouter). It aims to look and behave the same, except for the differences on this page. Each one was chosen on purpose.

Ported from: Scouter v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

## Name

- fmc shows its own name and version where Scouter shows `scouter`: the welcome screen, `--help`, `--version`, and error messages.

## Keys

- `Ctrl+q` quits. Plain `q` does not quit; it closes the open document, so the welcome screen returns, or the key list. On the changes screen, `q` goes back to browsing, as `d` does. `Ctrl+c` still quits.
- Opening a file from the tree keeps the focus in the tree, so you can keep browsing with `j` and `k`. Press `Tab` or click the reader to move there.

## Changes screen

- After folding a file with `c`, the footer and the scrollbar show the new position at once. Scouter shows the old values until you scroll.
- After switching between one file and all files (`a`) or changing the context (`x`), the selected file stays at the top. Scouter can land elsewhere.

## Copying text

- Dragging the mouse across text selects it and keeps the selection on screen. Nothing is copied until you press `y`. Press `Esc` to clear the selection. With no selection, `y` copies the path, as in Scouter.

## File tree

- The tree has no root row. The folder's own files and folders start at the top, with no indent, and its name stays in the sidebar title. Scouter draws the folder as the first row, which collapses like any folder.
- Opening a file inside a collapsed folder expands every folder down to the file, so the file's row is visible. Scouter can leave the file's own folder collapsed.
- When fmc starts on a file it cannot show (not UTF-8 text), the tree still lists the file's folder. Scouter leaves the tree empty.

## Help

- `?` shows the key list as tables in the reader, with the mouse actions in their own table. `--help` describes the same key list, so the two always agree. `?` or `Esc` brings the document back where it was. The key list is Markdown, so `/` is refused there, as Scouter refuses it in a rendered Markdown file: press `m` to see the source, then `/` to search the list.
