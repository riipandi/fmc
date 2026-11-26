# Differences from Scouter

fmc starts as a port of [Scouter](https://github.com/indrazm/scouter). It aims to look and behave the same, except for the differences on this page. Each one was chosen on purpose.

Ported from: Scouter v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`.

## Name

- fmc shows its own name and version where Scouter shows `scouter`: the welcome screen, `--help`, `--version`, and error messages.

## Keys

- `Ctrl+q` quits. Plain `q` does not quit. `Ctrl+c` still quits.
- Opening a file from the tree keeps the focus in the tree, so you can keep browsing with `j` and `k`. Press `Tab` or click the reader to move there.

## Copying text

- Dragging the mouse across text selects it and keeps the selection on screen. Nothing is copied until you press `y`. Press `Esc` to clear the selection. With no selection, `y` copies the path, as in Scouter.

## File tree

- The tree has no root row. The folder's own files and folders start at the top, with no indent, and its name stays in the sidebar title. Scouter draws the folder as the first row, which collapses like any folder.
- Opening a file inside a collapsed folder expands every folder down to the file, so the file's row is visible. Scouter can leave the file's own folder collapsed.
- When fmc starts on a file it cannot show (not UTF-8 text), the tree still lists the file's folder. Scouter leaves the tree empty.

## Help

- The `?` card lists the mouse actions, like `--help` does.
